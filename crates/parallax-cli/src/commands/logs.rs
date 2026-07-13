//! Historical log browsing command.

use super::filters::{LogsFilter, parse_since, severity_min};
use super::forwarding::{now_nanos, relative};
use crate::client::{Client, gql_str};

/// `parallax logs [--trace|--run] [--service] [--level] [--grep] [--since] [--limit]`.
pub(crate) async fn logs(client: &Client, filter: LogsFilter<'_>) -> anyhow::Result<()> {
    let mut args: Vec<String> = Vec::new();
    if let Some(trace_id) = filter.trace {
        args.push(format!(r#"traceId: "{}""#, gql_str(trace_id)));
    }
    if let Some(run_id) = filter.run {
        args.push(format!(r#"runId: "{}""#, gql_str(run_id)));
    }
    if let Some(service) = filter.service {
        args.push(format!(r#"service: "{}""#, gql_str(service)));
    }
    if let Some(level) = filter.level {
        args.push(format!("severityMin: {}", severity_min(level)?));
    }
    if let Some(needle) = filter.grep {
        args.push(format!(r#"query: "{}""#, gql_str(needle)));
    }
    if filter.trace.is_none() && filter.run.is_none() {
        let from = now_nanos().saturating_sub(parse_since(filter.since)?);
        args.push(format!(r#"fromNanos: "{from}""#));
    }
    args.push(format!("limit: {}", filter.limit));
    let response = client
        .graphql(&format!(
            r#"{{ logs({}) {{ tsNanos service severityText body }} }}"#,
            args.join(", ")
        ))
        .await?;
    let logs = response
        .pointer("/data/logs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if logs.is_empty() {
        println!("no matching logs");
        return Ok(());
    }
    for log in &logs {
        println!(
            "{:<10} [{}] {} {}",
            relative(log["tsNanos"].as_str().unwrap_or("0")),
            log["service"].as_str().unwrap_or("-"),
            log["severityText"].as_str().unwrap_or("-"),
            log["body"].as_str().unwrap_or(""),
        );
    }
    Ok(())
}
