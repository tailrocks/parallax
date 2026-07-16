//! SSE follow commands and combined run watch.

use super::filters::{LogsFilter, TracesFilter, parse_duration_ms, parse_since, severity_min};
use super::forwarding::relative;
use crate::client::Client;

/// Tail an SSE endpoint, printing each row via `print`; `for_window`
/// (e.g. 30s) stops after that long and reports the match count — the
/// agent-verification mode ("watch whether it still appears").
async fn tail_sse(
    client: &Client,
    path_and_query: &str,
    for_window: Option<&str>,
    label: &str,
    print: impl Fn(&serde_json::Value),
) -> anyhow::Result<()> {
    use tokio_stream::StreamExt as _;
    let deadline = for_window
        .map(|window| {
            parse_since(window).and_then(|nanos| {
                u64::try_from(nanos / 1_000_000)
                    .map_err(|_| anyhow::anyhow!("follow window is too large"))
            })
        })
        .transpose()?
        .map(|millis| tokio::time::Instant::now() + std::time::Duration::from_millis(millis));
    let response = client.sse(path_and_query).await?;
    let mut stream = response.bytes_stream();
    let mut pending = String::new();
    let mut matched: u64 = 0;
    loop {
        let chunk = match deadline {
            Some(deadline) => {
                match tokio::time::timeout_at(deadline, stream.next()).await {
                    Ok(chunk) => chunk,
                    Err(_) => break, // window elapsed
                }
            }
            None => stream.next().await,
        };
        let Some(chunk) = chunk else { break };
        pending.push_str(&String::from_utf8_lossy(&chunk?));
        // SSE frames: "data: <json>\n"; keep-alives and partial lines skipped.
        while let Some(newline) = pending.find('\n') {
            let line = pending[..newline].trim().to_string();
            pending.drain(..=newline);
            let Some(payload) = line.strip_prefix("data: ") else {
                continue;
            };
            matched += print_payload(payload, &print);
        }
    }
    if let Some(window) = for_window {
        println!("-- watched {window}: {matched} matching {label}(s)");
    }
    Ok(())
}

fn print_payload(payload: &str, print: &impl Fn(&serde_json::Value)) -> u64 {
    let Ok(serde_json::Value::Array(rows)) = serde_json::from_str(payload) else {
        return 0;
    };
    for row in &rows {
        print(row);
    }
    u64::try_from(rows.len()).unwrap_or(u64::MAX)
}

/// `parallax logs --follow` — kubectl-style live tail over SSE.
pub(crate) async fn logs_follow(
    client: &Client,
    filter: LogsFilter<'_>,
    for_window: Option<&str>,
) -> anyhow::Result<()> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(service) = filter.service {
        params.push(("service", service.into()));
    }
    if let Some(level) = filter.level {
        params.push(("severity_min", severity_min(level)?.to_string()));
    }
    if let Some(needle) = filter.grep {
        params.push(("q", needle.into()));
    }
    if let Some(trace_id) = filter.trace {
        params.push(("trace_id", trace_id.into()));
    }
    if let Some(invocation_id) = filter.run {
        params.push(("invocation_id", invocation_id.into()));
    }
    let query = encode_query(&params);
    tail_sse(
        client,
        &format!("/v1/logs/stream{query}"),
        for_window,
        "log event",
        |log| {
            println!(
                "{:<10} [{}] {} {}",
                relative(log["tsNanos"].as_str().unwrap_or("0")),
                log["service"].as_str().unwrap_or("-"),
                log["severityText"].as_str().unwrap_or("-"),
                log["body"].as_str().unwrap_or(""),
            );
        },
    )
    .await
}

/// `parallax traces --follow` — live finished-span feed over SSE.
pub(crate) async fn traces_follow(
    client: &Client,
    filter: TracesFilter<'_>,
    for_window: Option<&str>,
) -> anyhow::Result<()> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(service) = filter.service {
        params.push(("service", service.into()));
    }
    if let Some(min) = filter.min_duration {
        params.push(("min_duration_ms", parse_duration_ms(min)?.to_string()));
    }
    if filter.errors_only {
        params.push(("errors_only", "true".into()));
    }
    if let Some(needle) = filter.grep {
        params.push(("q", needle.into()));
    }
    if let Some(invocation_id) = filter.run {
        params.push(("invocation_id", invocation_id.into()));
    }
    let query = encode_query(&params);
    tail_sse(
        client,
        &format!("/v1/traces/stream{query}"),
        for_window,
        "span",
        print_span_line,
    )
    .await
}

fn print_span_line(span: &serde_json::Value) {
    let millis = span["durationNs"]
        .as_str()
        .and_then(|d| d.parse::<f64>().ok())
        .unwrap_or(0.0)
        / 1e6;
    println!(
        "{:<10} {} [{}] {} — {millis:.1}ms {}",
        relative(span["tsNanos"].as_str().unwrap_or("0")),
        span["traceId"].as_str().unwrap_or("-"),
        span["service"].as_str().unwrap_or("-"),
        span["name"].as_str().unwrap_or("-"),
        span["statusCode"]
            .as_str()
            .map(|s| s.trim_start_matches("STATUS_CODE_"))
            .unwrap_or("-"),
    );
}

/// `parallax run watch <invocation_id>` — the run-scoped combined live tail: new
/// log records and finished spans for one run id, interleaved as they
/// arrive (the CLI mirror of the run page's Live mode). `--for 30s` watches
/// a fixed window and reports per-stream match counts — the agent
/// verification loop for a specific run.
pub(crate) async fn run_watch(
    client: &Client,
    invocation_id: &str,
    level: Option<&str>,
    grep: Option<&str>,
    for_window: Option<&str>,
) -> anyhow::Result<()> {
    println!(
        "watching run {invocation_id} — live logs + spans{}",
        for_window
            .map(|w| format!(" for {w}"))
            .unwrap_or_else(|| " (Ctrl-C to stop)".into())
    );
    let mut log_params: Vec<(&str, String)> = vec![("invocation_id", invocation_id.into())];
    if let Some(level) = level {
        log_params.push(("severity_min", severity_min(level)?.to_string()));
    }
    if let Some(needle) = grep {
        log_params.push(("q", needle.into()));
    }
    let span_params: Vec<(&str, String)> = vec![("invocation_id", invocation_id.into())];
    let logs_path = format!("/v1/logs/stream{}", encode_query(&log_params));
    let spans_path = format!("/v1/traces/stream{}", encode_query(&span_params));
    let logs = tail_sse(client, &logs_path, for_window, "log event", |log| {
        println!(
            "[log]  {:<10} {} {}",
            relative(log["tsNanos"].as_str().unwrap_or("0")),
            log["severityText"].as_str().unwrap_or("-"),
            log["body"].as_str().unwrap_or(""),
        );
    });
    let spans = tail_sse(client, &spans_path, for_window, "span", |span| {
        print!("[span] ");
        print_span_line(span);
    });
    let (logs, spans) = tokio::join!(logs, spans);
    logs?;
    spans?;
    Ok(())
}

fn encode_query(params: &[(&str, String)]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let encoded: Vec<String> = params
        .iter()
        .map(|(key, value)| format!("{key}={}", urlencoding::encode(value)))
        .collect();
    format!("?{}", encoded.join("&"))
}
