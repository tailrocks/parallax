//! CLI commands over the API: runs (with wrapper mode), issues, traces, logs.

use crate::client::{Client, gql_str};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn new_run_id() -> String {
    // Time-based id is enough for a single-user local tool.
    format!("run_{:x}", now_nanos())
}

fn relative(nanos_str: &str) -> String {
    let nanos: u128 = nanos_str.parse().unwrap_or(0);
    let now = now_nanos();
    let secs = now.saturating_sub(nanos) / 1_000_000_000;
    match secs {
        0..=59 => format!("{secs}s ago"),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=86_399 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86_400),
    }
}

/// `parallax run start [-- <command…>]`
pub async fn run_start(client: &Client, command: Vec<String>) -> anyhow::Result<i32> {
    let run_id = new_run_id();
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
        println!("export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317");
        println!("export OTEL_RESOURCE_ATTRIBUTES=parallax.run_id={run_id}");
        println!("# run id: {run_id}  (finish with: parallax run finish {run_id} <exit-code>)");
        return Ok(0);
    }

    // Wrapper mode: inject env, run the child, capture the exit code.
    println!("run {run_id}: {}", command.join(" "));
    let status = tokio::process::Command::new(&command[0])
        .args(&command[1..])
        .env("OTEL_EXPORTER_OTLP_ENDPOINT", "http://127.0.0.1:4317")
        .env(
            "OTEL_RESOURCE_ATTRIBUTES",
            format!("parallax.run_id={run_id}"),
        )
        .status()
        .await?;
    let exit_code = status.code().unwrap_or(-1);

    client
        .graphql(&format!(
            r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {exit_code}) }}"#,
            gql_str(&run_id),
            now_nanos()
        ))
        .await?;
    println!("run {run_id} finished with exit code {exit_code}");
    println!("inspect: parallax run inspect {run_id}   issues: parallax issue list");
    Ok(exit_code)
}

pub async fn run_finish(client: &Client, run_id: &str, exit_code: i32) -> anyhow::Result<()> {
    client
        .graphql(&format!(
            r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {exit_code}) }}"#,
            gql_str(run_id),
            now_nanos()
        ))
        .await?;
    println!("run {run_id} finished ({exit_code})");
    Ok(())
}

pub async fn run_list(client: &Client) -> anyhow::Result<()> {
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

/// `parallax run inspect <run_id>` — the run's record (run-scoped telemetry
/// joins land with the adapter's run-filtered reads in the next slice).
pub async fn run_inspect(client: &Client, run_id: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(
            r#"{ runs(limit: 500) { runId command status exitCode startedAtNanos endedAtNanos } }"#,
        )
        .await?;
    let runs = response
        .pointer("/data/runs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let Some(run) = runs.iter().find(|r| r["runId"] == run_id) else {
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
    println!("issues from this period: parallax issue list");
    Ok(())
}

pub async fn issue_list(client: &Client, status: Option<&str>) -> anyhow::Result<()> {
    let filter = status
        .map(|s| format!(r#"(status: "{}")"#, gql_str(s)))
        .unwrap_or_default();
    let response = client
        .graphql(&format!(
            r#"{{ issues{filter} {{ fingerprint title service status eventCount lastSeenNanos }} }}"#
        ))
        .await?;
    let issues = response
        .pointer("/data/issues")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if issues.is_empty() {
        println!("no issues — either your code is perfect or nothing is sending telemetry yet");
        return Ok(());
    }
    println!(
        "{:<18} {:<8} {:>6}  {:<10} {:<12} title",
        "FINGERPRINT", "STATUS", "EVENTS", "LAST SEEN", "SERVICE"
    );
    for issue in issues {
        println!(
            "{:<18} {:<8} {:>6}  {:<10} {:<12} {}",
            issue["fingerprint"].as_str().unwrap_or("-"),
            issue["status"].as_str().unwrap_or("-"),
            issue["eventCount"].as_u64().unwrap_or(0),
            relative(issue["lastSeenNanos"].as_str().unwrap_or("0")),
            issue["service"].as_str().unwrap_or("-"),
            issue["title"].as_str().unwrap_or("-"),
        );
    }
    Ok(())
}

/// `parallax issue context <fingerprint>` — the agent handoff: the bounded,
/// redacted, hypothesis-ranked evidence bundle, rendered by the server.
pub async fn issue_context(client: &Client, fingerprint: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ bundle(fingerprint: "{}") {{ markdown canonicalHash }} }}"#,
            gql_str(fingerprint)
        ))
        .await?;
    let Some(bundle) = response.pointer("/data/bundle").filter(|v| !v.is_null()) else {
        anyhow::bail!("issue {fingerprint} not found");
    };
    println!("{}", bundle["markdown"].as_str().unwrap_or(""));
    if let Some(hash) = bundle["canonicalHash"].as_str() {
        println!("\n---\nbundle: {hash}");
    }
    Ok(())
}

pub async fn trace_inspect(client: &Client, trace_id: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ trace(traceId: "{0}") {{ spans {{ name service kind statusCode durationNs spanId parentSpanId }} }}
                 logsByTrace(traceId: "{0}") {{ severityText body }} }}"#,
            gql_str(trace_id)
        ))
        .await?;
    let Some(spans) = response
        .pointer("/data/trace/spans")
        .and_then(|v| v.as_array())
    else {
        anyhow::bail!("trace {trace_id} not found");
    };
    println!("trace {trace_id} — {} span(s)", spans.len());
    for span in spans {
        let micros = span["durationNs"]
            .as_str()
            .and_then(|d| d.parse::<u128>().ok())
            .unwrap_or(0)
            / 1_000;
        println!(
            "  [{}] {} — {} {} ({micros}µs)",
            span["service"].as_str().unwrap_or("-"),
            span["name"].as_str().unwrap_or("-"),
            span["kind"].as_str().unwrap_or("-"),
            span["statusCode"].as_str().unwrap_or("-"),
        );
    }
    if let Some(logs) = response
        .pointer("/data/logsByTrace")
        .and_then(|v| v.as_array())
        && !logs.is_empty()
    {
        println!("logs:");
        for log in logs {
            println!(
                "  {} {}",
                log["severityText"].as_str().unwrap_or("-"),
                log["body"].as_str().unwrap_or(""),
            );
        }
    }
    Ok(())
}
