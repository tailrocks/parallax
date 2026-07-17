//! Invocation-scoped metric snapshot (plan 105: `parallax metrics --invocation`).

use super::filters::parse_since;
use super::forwarding::{now_nanos, relative};
use crate::client::{Client, gql_str};

/// `parallax metrics --invocation <id> [--since] [--json]` — the bounded
/// typed projection over `invocation_metric_points` (canonical native-family
/// names, finite samples only). An unknown invocation errors; a known
/// invocation with no metric points reports that explicitly.
pub(crate) async fn metrics_invocation(
    client: &Client,
    invocation_id: &str,
    since: &str,
    json: bool,
) -> anyhow::Result<()> {
    let to = now_nanos();
    let from = to.saturating_sub(parse_since(since)?);
    let response = client
        .graphql(&format!(
            r#"{{
              invocation(invocationId: "{id}") {{ invocationId status }}
              invocationMetrics(invocationId: "{id}", fromNanos: "{from}", toNanos: "{to}") {{
                name pointCount lastValue lastTsNanos
              }}
            }}"#,
            id = gql_str(invocation_id),
        ))
        .await?;
    if response
        .pointer("/data/invocation")
        .is_none_or(|v| v.is_null())
    {
        anyhow::bail!("unknown invocation {invocation_id:?} — not registered on this server");
    }
    let rows = response
        .pointer("/data/invocationMetrics")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if json {
        println!(
            "{}",
            serde_json::json!({
                "invocationId": invocation_id,
                "fromNanos": from.to_string(),
                "toNanos": to.to_string(),
                "metrics": rows,
            })
        );
        return Ok(());
    }
    if rows.is_empty() {
        println!("invocation {invocation_id} recorded no metric points in the last {since}");
        return Ok(());
    }
    println!(
        "{:<48} {:>10} {:>16}  last seen",
        "metric", "points", "last value"
    );
    for row in &rows {
        println!(
            "{:<48} {:>10} {:>16}  {}",
            row["name"].as_str().unwrap_or("-"),
            row["pointCount"].as_str().unwrap_or("0"),
            row["lastValue"].as_f64().unwrap_or(0.0),
            relative(row["lastTsNanos"].as_str().unwrap_or("0")),
        );
    }
    Ok(())
}
