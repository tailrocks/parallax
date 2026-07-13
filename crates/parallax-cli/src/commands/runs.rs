//! Run lifecycle, inspection, bundle, and agent-session commands.

use super::forwarding::*;
use super::output::*;
use crate::OutputFormat;
use crate::client::{Client, gql_str};

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
    let default_parallax_endpoint = parallax_endpoint_from_server(client).await?;
    let fwd = resolve_forward(forward.as_deref(), &default_parallax_endpoint)?;
    let attrs = forward_resource_attrs(&run_id, fwd.compare);
    let pairs = otel_env_pairs(&fwd.endpoint, fwd.protocol, &attrs);

    // Dry-run: print the env we *would* inject, run nothing, record nothing.
    if print_env && !command.is_empty() {
        for (key, value) in &pairs {
            println!("export {key}={value}");
        }
        return Ok(0);
    }

    let command_str = (!command.is_empty()).then(|| command.join(" "));
    client
        .graphql(&format!(
            r#"mutation {{ runStart(runId: "{}", command: {}, startedAtNanos: "{}") }}"#,
            gql_str(&run_id),
            command_str
                .as_deref()
                .map(|c| format!("\"{}\"", gql_str(c)))
                .unwrap_or_else(|| "null".to_string()),
            now_nanos()
        ))
        .await?;

    if command.is_empty() {
        // Bare mode: print exports for the developer to source.
        for (key, value) in &pairs {
            println!("export {key}={value}");
        }
        println!("# run id: {run_id}  (finish with: parallax run finish {run_id} <exit-code>)");
        return Ok(0);
    }

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
    for (key, value) in &pairs {
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
            gql_str(&run_id),
            now_nanos()
        ))
        .await;

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
