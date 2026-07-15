//! Run lifecycle, inspection, bundle, and agent-session commands.

use super::forwarding::*;
use super::output::*;
use crate::OutputFormat;
use crate::client::{Client, gql_str};
use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::trace::{IdGenerator as _, RandomIdGenerator, SdkTracerProvider};

struct RunSessionSpan {
    provider: SdkTracerProvider,
    span: opentelemetry_sdk::trace::Span,
}

impl RunSessionSpan {
    fn start(
        endpoint: &str,
        protocol: &str,
        run_id: &str,
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
        let mut span = tracer.start("parallax.run.session");
        span.set_attribute(KeyValue::new(
            parallax_semconv::PARALLAX_RUN_ID,
            run_id.to_string(),
        ));
        if let Some(command) = command {
            span.set_attribute(KeyValue::new("process.command", command.to_string()));
        }
        Ok(Self { provider, span })
    }

    fn traceparent(&self) -> String {
        traceparent(self.span.span_context())
    }

    fn finish(mut self, exit_code: i32) {
        self.span
            .set_attribute(KeyValue::new("process.exit.code", i64::from(exit_code)));
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

/// `parallax run start [--otlp-forward <target>] [--print-env] [-- <command…>]`
///
/// Default: child telemetry → Parallax's own receiver. Compare mode (forward set
/// via flag or `PARALLAX_OTLP_FORWARD`): child telemetry → the collector (Rotel),
/// which fans it out to every backend incl. Parallax for side-by-side comparison.
pub(crate) async fn run_start(
    client: &Client,
    command: Vec<String>,
    forward: Option<String>,
    print_env: bool,
) -> anyhow::Result<i32> {
    let run_id = new_run_id();
    let parallax_endpoints = parallax_endpoints_from_server(client).await?;
    let fwd = resolve_forward(forward.as_deref(), &parallax_endpoints.grpc)?;
    let attrs = forward_resource_attrs(&run_id, fwd.compare);
    let mut pairs = otel_env_pairs(&fwd.endpoint, fwd.protocol, &attrs);
    pairs.push(("PARALLAX_RUN_ID", run_id.clone()));
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

    let command_str = (!command.is_empty()).then(|| command.join(" "));
    let session =
        RunSessionSpan::start(&fwd.endpoint, fwd.protocol, &run_id, command_str.as_deref())?;
    if let Err(error) = client
        .graphql(&format!(
            r#"mutation {{ runStart(runId: "{}", command: {}, startedAtNanos: "{}") }}"#,
            gql_str(&run_id),
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
        println!("# run id: {run_id}  (finish with: parallax run finish {run_id} <exit-code>)");
        session.finish(0);
        return Ok(0);
    }

    execute_child(client, &command, &pairs, &fwd, session, &run_id).await
}

async fn execute_child(
    client: &Client,
    command: &[String],
    pairs: &[(&str, String)],
    fwd: &Forward,
    session: RunSessionSpan,
    run_id: &str,
) -> anyhow::Result<i32> {
    // Wrapper mode: inject env, run the child, capture the exit code.
    println!("Parallax run id: {run_id}");
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
    println!("live: parallax run watch {run_id}");
    let mut cmd = tokio::process::Command::new(&command[0]);
    cmd.args(&command[1..]);
    for (key, value) in pairs {
        cmd.env(key, value);
    }
    // Always attempt runFinish even when the child fails to spawn, so the run
    // does not stay stuck in `running` forever.
    let status = cmd.status().await;
    let exit_code = match &status {
        Ok(status) => status.code().unwrap_or(-1),
        Err(_) => -1,
    };

    let finish = client
        .graphql(&format!(
            r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {exit_code}) }}"#,
            gql_str(run_id),
            now_nanos()
        ))
        .await;

    session.finish(exit_code);

    status?; // propagate spawn error AFTER finishing the run
    finish?;
    println!("Parallax run {run_id} finished with exit code {exit_code}");
    println!("inspect: parallax run inspect {run_id}   issues: parallax issue list");
    Ok(exit_code)
}

pub(crate) async fn run_finish(c: &Client, id: &str, code: i32) -> anyhow::Result<()> {
    c.graphql(&format!(
        r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {code}) }}"#,
        gql_str(id),
        now_nanos()
    ))
    .await?;
    println!("run {id} finished ({code})");
    Ok(())
}

pub(crate) async fn run_list(client: &Client) -> anyhow::Result<()> {
    let response = client
        .graphql(r#"{ runs { runId command status exitCode startedAtNanos } }"#)
        .await?;
    let runs = response
        .pointer("/data/runs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if runs.is_empty() {
        println!("no runs yet — start one with: parallax run start -- <command>");
        return Ok(());
    }
    println!(
        "{:<24} {:<10} {:>5}  {:<10} command",
        "RUN", "STATUS", "EXIT", "STARTED"
    );
    for run in runs {
        println!(
            "{:<24} {:<10} {:>5}  {:<10} {}",
            run["runId"].as_str().unwrap_or("-"),
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

/// `parallax run inspect <run_id>` — the run's record plus its derived
/// counts and grouped issues.
pub(crate) async fn run_inspect(client: &Client, run_id: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ run(runId: "{}") {{ runId command status exitCode startedAtNanos endedAtNanos
                 errorCount traceCount issues {{ fingerprint title }} }} }}"#,
            gql_str(run_id)
        ))
        .await?;
    let Some(run) = response.pointer("/data/run").filter(|v| !v.is_null()) else {
        anyhow::bail!("run {run_id} not found");
    };
    println!("run {run_id}");
    println!("  status:  {}", run["status"].as_str().unwrap_or("-"));
    println!("  command: {}", run["command"].as_str().unwrap_or("-"));
    println!(
        "  started: {}",
        relative(run["startedAtNanos"].as_str().unwrap_or("0"))
    );
    if let Some(code) = run["exitCode"].as_i64() {
        println!("  exit:    {code}");
    }
    println!("  traces:  {}", run["traceCount"].as_i64().unwrap_or(0));
    println!("  errors:  {}", run["errorCount"].as_i64().unwrap_or(0));
    if let Some(issues) = run["issues"].as_array()
        && !issues.is_empty()
    {
        println!("issues in this run:");
        for issue in issues {
            println!(
                "  {}  {}",
                issue["fingerprint"].as_str().unwrap_or("-"),
                issue["title"].as_str().unwrap_or("-"),
            );
        }
        println!("context: parallax issue context <fingerprint>");
    }
    println!("bundle: parallax run bundle {run_id}   traces: parallax trace inspect <trace_id>");
    Ok(())
}

/// `parallax run bundle <run_id>` — the run-anchored evidence bundle
/// (scope §2.4: the run model's bundle).
pub(crate) async fn run_bundle(c: &Client, id: &str, fmt: OutputFormat) -> anyhow::Result<()> {
    let query = match fmt {
        OutputFormat::Markdown => format!(
            r#"{{ bundle(runId: "{}") {{ markdown canonicalHash }} }}"#,
            gql_str(id)
        ),
        OutputFormat::Json => format!(
            r#"{{ bundle(runId: "{}") {{ json canonicalHash }} }}"#,
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

/// `parallax run agent <run_id>` — run-scoped agent-session projection
/// (tool steps, token totals). Null when no agent spans were detected.
pub(crate) async fn run_agent_session(
    client: &Client,
    run_id: &str,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ agentSession(runId: "{}") {{
                rootSpanId totalInputTokens totalOutputTokens errorCount truncated
                steps {{
                  spanId traceId kind name startNanos durationNs isError
                  genAiOperation inputTokens outputTokens
                }}
            }} }}"#,
            gql_str(run_id)
        ))
        .await?;
    let Some(session) = response
        .pointer("/data/agentSession")
        .filter(|v| !v.is_null())
    else {
        anyhow::bail!("no agent session detected for run {run_id}");
    };
    let (stdout, stderr) = render_agent_session(format, run_id, session);
    print!("{stdout}");
    eprint!("{stderr}");
    Ok(())
}
