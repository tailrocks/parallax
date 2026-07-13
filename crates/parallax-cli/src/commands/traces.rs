//! Historical trace browsing and trace inspection commands.

use super::filters::{TracesFilter, parse_duration_ms, parse_since};
use super::forwarding::{now_nanos, relative};
use crate::client::{Client, gql_str};

/// `parallax traces [--run] [--service] [--min-duration] [--errors] [--grep] [--since] [--limit]`.
pub(crate) async fn traces(client: &Client, filter: TracesFilter<'_>) -> anyhow::Result<()> {
    // --run anchors on the run's traces (tracesByRun); other filters are
    // the browse query.
    let (pointer, query) = match filter.run {
        Some(run_id) => (
            "/data/tracesByRun",
            format!(
                r#"{{ tracesByRun(runId: "{}", limit: {}) {{ traceId rootName service startNanos durationNs spanCount hasError }} }}"#,
                gql_str(run_id),
                filter.limit
            ),
        ),
        None => {
            let mut args: Vec<String> = Vec::new();
            if let Some(service) = filter.service {
                args.push(format!(r#"service: "{}""#, gql_str(service)));
            }
            if let Some(min) = filter.min_duration {
                args.push(format!("minDurationMs: {}", parse_duration_ms(min)?));
            }
            if filter.errors_only {
                args.push("errorOnly: true".into());
            }
            if let Some(needle) = filter.grep {
                args.push(format!(r#"query: "{}""#, gql_str(needle)));
            }
            let from = now_nanos().saturating_sub(parse_since(filter.since)?);
            args.push(format!(r#"fromNanos: "{from}""#));
            args.push(format!("limit: {}", filter.limit));
            (
                "/data/traces",
                format!(
                    r#"{{ traces({}) {{ traceId rootName service startNanos durationNs spanCount hasError }} }}"#,
                    args.join(", ")
                ),
            )
        }
    };
    let response = client.graphql(&query).await?;
    let traces = response
        .pointer(pointer)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if traces.is_empty() {
        println!("no matching traces");
        return Ok(());
    }
    for trace in &traces {
        let millis = trace["durationNs"]
            .as_str()
            .and_then(|d| d.parse::<f64>().ok())
            .unwrap_or(0.0)
            / 1e6;
        println!(
            "{:<10} {} [{}] {} — {} span(s), {millis:.1}ms{}",
            relative(trace["startNanos"].as_str().unwrap_or("0")),
            trace["traceId"].as_str().unwrap_or("-"),
            trace["service"].as_str().unwrap_or("-"),
            trace["rootName"].as_str().unwrap_or("-"),
            trace["spanCount"].as_i64().unwrap_or(0),
            if trace["hasError"].as_bool().unwrap_or(false) {
                ", ERROR"
            } else {
                ""
            },
        );
    }
    Ok(())
}

pub(crate) async fn trace_inspect(client: &Client, trace_id: &str) -> anyhow::Result<()> {
    let trace_id = trace_id.parse::<parallax_model::TraceId>()?;
    let trace_id = trace_id.as_str();
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
