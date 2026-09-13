//! Invocation lifecycle, inspection, bundle, and agent-session commands.

use super::capture::{CapturedStream, redact_command_line, tee_bounded};
use super::forwarding::*;
use super::output::*;
use crate::OutputFormat;
use crate::client::{Client, gql_str};
use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::trace::{IdGenerator as _, RandomIdGenerator, SdkTracerProvider};

struct InvocationSessionSpan {
    provider: SdkTracerProvider,
    span: opentelemetry_sdk::trace::Span,
}

impl InvocationSessionSpan {
    fn start(
        endpoint: &str,
        protocol: &str,
        invocation_id: &str,
        command: Option<&str>,
    ) -> anyhow::Result<Self> {
        let exporter = if protocol == OTLP_HTTP_PROTOCOL {
            let endpoint = if endpoint.trim_end_matches('/').ends_with("/v1/traces") {
                endpoint.to_string()
            } else {
                format!("{}/v1/traces", endpoint.trim_end_matches('/'))
            };
            opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_endpoint(endpoint)
                .build()?
        } else {
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()?
        };
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        let tracer = provider.tracer("parallax-cli");
        let mut span = tracer.start(parallax_semconv::CLI_COMMAND_SPAN_NAME);
        span.set_attribute(KeyValue::new(
            parallax_semconv::CLI_INVOCATION_ID,
            invocation_id.to_string(),
        ));
        if let Some(command) = command {
            span.set_attribute(KeyValue::new("process.command", command.to_string()));
        }
        span.set_attribute(KeyValue::new(
            parallax_semconv::APP_MODE,
            parallax_semconv::APP_MODE_ONE_SHOT,
        ));
        Ok(Self { provider, span })
    }

    fn traceparent(&self) -> String {
        traceparent(self.span.span_context())
    }

    fn finish(mut self, exit_code: i32) {
        self.span.set_attribute(KeyValue::new(
            parallax_semconv::PROCESS_EXIT_CODE,
            i64::from(exit_code),
        ));
        self.span.set_attribute(KeyValue::new(
            parallax_semconv::OUTCOME,
            if exit_code == 0 {
                parallax_semconv::OUTCOME_SUCCESS
            } else {
                parallax_semconv::OUTCOME_FAILURE
            },
        ));
        if exit_code != 0 {
            self.span
                .set_status(Status::error(format!("child exited with {exit_code}")));
        }
        self.span.end();
        if let Err(error) = self.provider.shutdown() {
            tracing::warn!(%error, "failed to flush run-session span");
        }
    }
}

fn traceparent(context: &opentelemetry::trace::SpanContext) -> String {
    format!("00-{}-{}-01", context.trace_id(), context.span_id())
}

pub(super) fn generated_traceparent() -> String {
    let generator = RandomIdGenerator::default();
    format!(
        "00-{}-{}-01",
        generator.new_trace_id(),
        generator.new_span_id()
    )
}

/// `parallax invocation start [--otlp-forward <target>] [--print-env] [-- <command…>]`
///
/// Default: child telemetry → Parallax's own receiver. Compare mode (forward set
/// via flag or `PARALLAX_OTLP_FORWARD`): child telemetry → the collector (Rotel),
/// which fans it out to every backend incl. Parallax for side-by-side comparison.
pub(crate) async fn invocation_start(
    client: &Client,
    command: Vec<String>,
    forward: Option<String>,
    print_env: bool,
) -> anyhow::Result<i32> {
    let invocation_id = new_invocation_id();
    let parallax_endpoints = parallax_endpoints_from_server(client).await?;
    let fwd = resolve_forward(forward.as_deref(), &parallax_endpoints.grpc)?;
    let attrs = forward_resource_attrs(&invocation_id, fwd.compare);
    let mut pairs = otel_env_pairs(&fwd.endpoint, fwd.protocol, &attrs);
    pairs.push(("CLI_INVOCATION_ID", invocation_id.clone()));
    pairs.push((
        "PARALLAX_OTLP_HTTP_TRACES_ENDPOINT",
        http_traces_endpoint(&fwd, &parallax_endpoints.http_traces),
    ));

    // Dry-run: print the env we *would* inject, run nothing, record nothing.
    if print_env && !command.is_empty() {
        pairs.push(("TRACEPARENT", generated_traceparent()));
        for (key, value) in &pairs {
            println!("export {key}={value}");
        }
        return Ok(0);
    }

    // Secrets are redacted in the stored command line at capture: the span
    // attribute and the invocation record below both use this copy.
    let command_str = (!command.is_empty()).then(|| redact_command_line(&command));
    let session = InvocationSessionSpan::start(
        &fwd.endpoint,
        fwd.protocol,
        &invocation_id,
        command_str.as_deref(),
    )?;
    if let Err(error) = client
        .graphql(&format!(
            r#"mutation {{ invocationStart(invocationId: "{}", command: {}, appMode: "one_shot", startedAtNanos: "{}") }}"#,
            gql_str(&invocation_id),
            command_str
                .as_deref()
                .map(|c| format!("\"{}\"", gql_str(c)))
                .unwrap_or_else(|| "null".to_string()),
            now_nanos()
        ))
        .await
    {
        session.finish(-1);
        return Err(error);
    }
    pairs.push(("TRACEPARENT", session.traceparent()));

    if command.is_empty() {
        // Bare mode: print exports for the developer to source.
        for (key, value) in &pairs {
            println!("export {key}={value}");
        }
        println!(
            "# invocation id: {invocation_id}  (finish with: parallax invocation finish {invocation_id} <exit-code>)"
        );
        session.finish(0);
        return Ok(0);
    }

    execute_child(client, &command, &pairs, &fwd, session, &invocation_id).await
}

async fn execute_child(
    client: &Client,
    command: &[String],
    pairs: &[(&str, String)],
    fwd: &Forward,
    session: InvocationSessionSpan,
    invocation_id: &str,
) -> anyhow::Result<i32> {
    // Wrapper mode: inject env, run the child, capture the exit code.
    println!("Parallax invocation id: {invocation_id}");
    println!("command: {}", command.join(" "));
    if fwd.compare {
        println!(
            "telemetry → Rotel (fan-out) {}   [COMPARE MODE]",
            fwd.endpoint
        );
        println!("   ↳ parallax · maple · signoz · openobserve · sentry");
        preflight_warn(&fwd.endpoint).await;
    } else {
        println!("telemetry → Parallax {}", fwd.endpoint);
    }
    println!("live: parallax invocation watch {invocation_id}");
    let mut cmd = tokio::process::Command::new(&command[0]);
    cmd.args(&command[1..]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    for (key, value) in pairs {
        cmd.env(key, value);
    }
    // Always attempt invocationFinish even when the child fails to spawn, so
    // the invocation does not stay stuck in `running` forever.
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            // Side effect first (close the invocation), then report the
            // spawn failure — same precedence as the old `status?; finish?`.
            if let Err(finish_error) = finish_invocation(client, invocation_id, -1, None).await {
                tracing::warn!(%finish_error, "invocationFinish failed after spawn error");
            }
            session.finish(-1);
            return Err(error.into());
        }
    };
    // The child's streams flow to the terminal live while bounded heads are
    // retained for telemetry (flat memory: cap + one chunk per stream).
    let stdout_task = tokio::spawn(capture_pipe(child.stdout.take(), tokio::io::stdout()));
    let stderr_task = tokio::spawn(capture_pipe(child.stderr.take(), tokio::io::stderr()));
    let status = child.wait().await;
    let exit_code = match &status {
        Ok(status) => status.code().unwrap_or(-1),
        Err(_) => -1,
    };
    let (stdout_cap, stderr_cap) = tokio::join!(stdout_task, stderr_task);
    let output = status.is_ok().then(|| {
        (
            settle_capture("stdout", stdout_cap),
            settle_capture("stderr", stderr_cap),
        )
    });

    let finish = finish_invocation(
        client,
        invocation_id,
        exit_code,
        output.as_ref().map(|(stdout, stderr)| (stdout, stderr)),
    )
    .await;

    session.finish(exit_code);

    status?; // propagate spawn/wait error AFTER finishing the invocation
    finish?;
    println!("Parallax invocation {invocation_id} finished with exit code {exit_code}");
    println!("inspect: parallax invocation inspect {invocation_id}   issues: parallax issue list");
    Ok(exit_code)
}

/// Tee one child pipe to its terminal counterpart, retaining the bounded
/// head. A missing pipe (never happens after an explicit `piped()` setup)
/// yields an empty capture.
async fn capture_pipe<R, W>(pipe: Option<R>, writer: W) -> std::io::Result<CapturedStream>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    match pipe {
        Some(pipe) => tee_bounded(pipe, writer).await,
        None => Ok(CapturedStream::empty()),
    }
}

/// A failed capture must not lose the invocation record: warn and store an
/// empty head. The exit code stays the source of truth for the outcome.
fn settle_capture(
    stream: &str,
    joined: Result<std::io::Result<CapturedStream>, tokio::task::JoinError>,
) -> CapturedStream {
    match joined {
        Ok(Ok(captured)) => captured,
        Ok(Err(error)) => {
            tracing::warn!(%error, stream, "child output capture failed; storing empty head");
            CapturedStream::empty()
        }
        Err(error) => {
            tracing::warn!(%error, stream, "capture task failed; storing empty head");
            CapturedStream::empty()
        }
    }
}

/// Build the `invocationFinish` mutation. `output` carries the bounded
/// child-output heads (wrapper mode only); bare finishes pass `None` and
/// leave any stored output untouched.
fn finish_mutation(
    invocation_id: &str,
    exit_code: i32,
    output: Option<(&CapturedStream, &CapturedStream)>,
) -> String {
    let outcome = if exit_code == 0 { "success" } else { "failure" };
    let output_args = match output {
        Some((stdout, stderr)) => format!(
            r#", stdoutText: "{}", stdoutTruncatedBytes: {}, stderrText: "{}", stderrTruncatedBytes: {}"#,
            gql_str(&stdout.text),
            stdout.truncated_i32(),
            gql_str(&stderr.text),
            stderr.truncated_i32(),
        ),
        None => String::new(),
    };
    format!(
        r#"mutation {{ invocationFinish(invocationId: "{}", endedAtNanos: "{}", exitCode: {exit_code}, outcome: "{outcome}"{output_args}) }}"#,
        gql_str(invocation_id),
        now_nanos()
    )
}

async fn finish_invocation(
    client: &Client,
    invocation_id: &str,
    exit_code: i32,
    output: Option<(&CapturedStream, &CapturedStream)>,
) -> anyhow::Result<()> {
    client
        .graphql(&finish_mutation(invocation_id, exit_code, output))
        .await?;
    Ok(())
}

pub(crate) async fn invocation_finish(c: &Client, id: &str, code: i32) -> anyhow::Result<()> {
    finish_invocation(c, id, code, None).await?;
    println!("invocation {id} finished ({code})");
    Ok(())
}

pub(crate) async fn invocation_list(client: &Client) -> anyhow::Result<()> {
    let response = client
        .graphql(r#"{ invocations { invocationId command status exitCode startedAtNanos } }"#)
        .await?;
    let runs = response
        .pointer("/data/invocations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if runs.is_empty() {
        println!("no invocations yet — start one with: parallax invocation start -- <command>");
        return Ok(());
    }
    println!(
        "{:<24} {:<10} {:>5}  {:<10} command",
        "INVOCATION", "STATUS", "EXIT", "STARTED"
    );
    for run in runs {
        println!(
            "{:<24} {:<10} {:>5}  {:<10} {}",
            run["invocationId"].as_str().unwrap_or("-"),
            run["status"].as_str().unwrap_or("-"),
            run["exitCode"]
                .as_i64()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".into()),
            relative(run["startedAtNanos"].as_str().unwrap_or("0")),
            run["command"].as_str().unwrap_or("-"),
        );
    }
    Ok(())
}

/// `parallax invocation inspect <invocation_id>` — the invocation record plus its
/// derived counts and grouped issues.
pub(crate) async fn invocation_inspect(client: &Client, invocation_id: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ invocation(invocationId: "{}") {{ invocationId command status exitCode startedAtNanos endedAtNanos
                 stdoutText stdoutTruncatedBytes stderrText stderrTruncatedBytes
                 errorCount traceCount issues {{ fingerprint title }} }} }}"#,
            gql_str(invocation_id)
        ))
        .await?;
    let Some(run) = response
        .pointer("/data/invocation")
        .filter(|v| !v.is_null())
    else {
        anyhow::bail!("invocation {invocation_id} not found");
    };
    println!("invocation {invocation_id}");
    println!("  status:  {}", run["status"].as_str().unwrap_or("-"));
    println!("  command: {}", run["command"].as_str().unwrap_or("-"));
    println!(
        "  started: {}",
        relative(run["startedAtNanos"].as_str().unwrap_or("0"))
    );
    if let Some(code) = run["exitCode"].as_i64() {
        println!("  exit:    {code}");
    }
    if let Some(stdout) = run["stdoutText"].as_str() {
        println!(
            "  stdout:  {} bytes stored (+{} truncated)",
            stdout.len(),
            run["stdoutTruncatedBytes"].as_i64().unwrap_or(0)
        );
    }
    if let Some(stderr) = run["stderrText"].as_str() {
        println!(
            "  stderr:  {} bytes stored (+{} truncated)",
            stderr.len(),
            run["stderrTruncatedBytes"].as_i64().unwrap_or(0)
        );
    }
    println!("  traces:  {}", run["traceCount"].as_i64().unwrap_or(0));
    println!("  errors:  {}", run["errorCount"].as_i64().unwrap_or(0));
    if let Some(issues) = run["issues"].as_array()
        && !issues.is_empty()
    {
        println!("issues in this invocation:");
        for issue in issues {
            println!(
                "  {}  {}",
                issue["fingerprint"].as_str().unwrap_or("-"),
                issue["title"].as_str().unwrap_or("-"),
            );
        }
        println!("context: parallax issue context <fingerprint>");
    }
    println!(
        "bundle: parallax invocation bundle {invocation_id}   traces: parallax trace inspect <trace_id>"
    );
    Ok(())
}

/// `parallax invocation bundle <invocation_id>` — the run-anchored evidence bundle
/// (scope §2.4: the run model's bundle).
pub(crate) async fn invocation_bundle(
    c: &Client,
    id: &str,
    fmt: OutputFormat,
    max_tokens: Option<u32>,
) -> anyhow::Result<()> {
    let tokens = max_tokens
        .map(|n| format!(", maxTokens: {n}"))
        .unwrap_or_default();
    let query = match fmt {
        OutputFormat::Markdown => format!(
            r#"{{ bundle(invocationId: "{}"{tokens}) {{ markdown canonicalHash }} }}"#,
            gql_str(id)
        ),
        OutputFormat::Json => format!(
            r#"{{ bundle(invocationId: "{}"{tokens}) {{ json canonicalHash }} }}"#,
            gql_str(id)
        ),
    };
    let response = c.graphql(&query).await?;
    let Some(bundle) = response.pointer("/data/bundle").filter(|v| !v.is_null()) else {
        anyhow::bail!("run {id} not found");
    };
    let (stdout, stderr) = render_bundle(fmt, bundle);
    print!("{stdout}");
    eprint!("{stderr}");
    Ok(())
}

/// `parallax invocation agent <invocation_id>` — invocation-scoped agent-session projection
/// (tool steps, token totals). Null when no agent spans were detected.
pub(crate) async fn invocation_agent_session(
    client: &Client,
    invocation_id: &str,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ agentSession(invocationId: "{}") {{
                rootSpanId totalInputTokens totalOutputTokens errorCount truncated
                steps {{
                  spanId traceId kind name startNanos durationNs isError
                  genAiOperation inputTokens outputTokens
                }}
            }} }}"#,
            gql_str(invocation_id)
        ))
        .await?;
    let Some(session) = response
        .pointer("/data/agentSession")
        .filter(|v| !v.is_null())
    else {
        anyhow::bail!("no agent session detected for run {invocation_id}");
    };
    let (stdout, stderr) = render_agent_session(format, invocation_id, session);
    print!("{stdout}");
    eprint!("{stderr}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finish_mutation_escapes_multiline_output() {
        let stdout = CapturedStream {
            text: "a\nb\"c\\d".into(),
            truncated_bytes: 7,
        };
        let stderr = CapturedStream::empty();
        let mutation = finish_mutation("run-1", 1, Some((&stdout, &stderr)));
        assert!(
            mutation.contains(r#"stdoutText: "a\nb\"c\\d""#),
            "{mutation}"
        );
        assert!(mutation.contains("stdoutTruncatedBytes: 7"), "{mutation}");
        assert!(mutation.contains(r#"stderrText: """#), "{mutation}");
        assert!(mutation.contains(r#"outcome: "failure""#), "{mutation}");
    }

    #[test]
    fn bare_finish_omits_output_args() {
        let mutation = finish_mutation("run-1", 0, None);
        assert!(!mutation.contains("stdoutText"), "{mutation}");
        assert!(!mutation.contains("stderrText"), "{mutation}");
        assert!(mutation.contains(r#"outcome: "success""#), "{mutation}");
    }
}
