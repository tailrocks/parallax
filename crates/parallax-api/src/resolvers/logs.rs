//! GraphQL logs domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;

use crate::{ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string};

use crate::resolvers::common::Point;

pub struct LogRecord(pub(crate) model::LogRow);

#[graphql_object(context = ApiContext)]
impl LogRecord {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn event_name(&self) -> &str {
        &self.0.event_name
    }
    fn observed_ts_nanos(&self) -> String {
        nanos_string(self.0.observed_ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn severity_num(&self) -> i32 {
        self.0.severity_num
    }
    fn severity_text(&self) -> &str {
        &self.0.severity_text
    }
    fn body(&self) -> &str {
        &self.0.body
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn run_id(&self) -> Option<&str> {
        self.0.run_id.as_deref()
    }
    fn scope_name(&self) -> &str {
        &self.0.scope_name
    }
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
    }
    fn resource(&self) -> String {
        self.0.resource.to_string()
    }
}

pub(crate) async fn logs_by_trace(
    context: &ApiContext,
    trace_id: String,
) -> FieldResult<Vec<LogRecord>> {
    let logs = context.logs_for(&trace_id).await?;
    Ok(logs.iter().cloned().map(LogRecord).collect())
}

pub(crate) async fn logs_by_run(
    context: &ApiContext,
    run_id: String,
    limit: Option<i32>,
) -> FieldResult<Vec<LogRecord>> {
    let logs = context
        .store
        .logs_by_run(&run_id, clamp_limit(limit, 500))
        .await
        .map_err(field_err)?;
    Ok(logs.into_iter().map(LogRecord).collect())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn logs(
    context: &ApiContext,
    trace_id: Option<String>,
    run_id: Option<String>,
    service: Option<String>,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    query: Option<String>,
    limit: Option<i32>,
) -> FieldResult<Vec<LogRecord>> {
    let from: u128 = match from_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid fromNanos"))?,
        None => 0,
    };
    let to: u128 = match to_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid toNanos"))?,
        None => u128::MAX,
    };
    let limit = clamp_limit(limit, 500);
    let mut logs = match (&trace_id, &run_id) {
        (Some(trace_id), _) => context
            .store
            .logs_by_trace(trace_id)
            .await
            .map_err(field_err)?,
        (None, Some(run_id)) => context
            .store
            .logs_by_run(run_id, MAX_ROWS)
            .await
            .map_err(field_err)?,
        (None, None) => {
            let logs = context
                .store
                .logs_search(
                    service.as_deref(),
                    from..=to,
                    severity_min,
                    severity_max,
                    query.as_deref(),
                    limit,
                )
                .await
                .map_err(field_err)?;
            return Ok(logs.into_iter().map(LogRecord).collect());
        }
    };
    // Anchored reads come back ascending and unfiltered: apply the
    // remaining filters here, newest first.
    logs.retain(|l| {
        l.ts_nanos >= from
            && l.ts_nanos <= to
            && service.as_deref().is_none_or(|svc| l.service == svc)
            && severity_min.is_none_or(|min| l.severity_num >= min)
            && severity_max.is_none_or(|max| l.severity_num <= max)
            && query
                .as_deref()
                .is_none_or(|needle| l.body.contains(needle))
    });
    logs.sort_by_key(|l| std::cmp::Reverse(l.ts_nanos));
    logs.truncate(limit);
    Ok(logs.into_iter().map(LogRecord).collect())
}

pub(crate) async fn logs_around(
    context: &ApiContext,
    anchor_nanos: String,
    window_seconds: Option<i32>,
    service: Option<String>,
    trace_id: Option<String>,
    limit: Option<i32>,
) -> FieldResult<Vec<LogRecord>> {
    let anchor: u128 = anchor_nanos
        .parse()
        .map_err(|_| field_err("invalid anchorNanos"))?;
    let window =
        u128::try_from(window_seconds.unwrap_or(30).clamp(1, 600)).unwrap_or(30) * 1_000_000_000;
    let from = anchor.saturating_sub(window);
    let to = anchor.saturating_add(window);
    let limit = clamp_limit(limit, 200);
    let mut logs =
        if let Some(trace_id) = trace_id.as_deref().filter(|trace_id| !trace_id.is_empty()) {
            context
                .logs_for(trace_id)
                .await?
                .iter()
                .filter(|log| {
                    log.ts_nanos >= from
                        && log.ts_nanos <= to
                        && service.as_deref().is_none_or(|svc| log.service == svc)
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            context
                .store
                .logs_search(service.as_deref(), from..=to, None, None, None, limit)
                .await
                .map_err(field_err)?
        };
    logs.sort_by_key(|log| log.ts_nanos);
    logs.truncate(limit);
    Ok(logs.into_iter().map(LogRecord).collect())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn log_count_series(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
    service: Option<String>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    query: Option<String>,
    step_seconds: Option<i32>,
) -> FieldResult<Vec<Point>> {
    let from: u128 = from_nanos
        .parse()
        .map_err(|_| field_err("invalid fromNanos"))?;
    let to: u128 = to_nanos.parse().map_err(|_| field_err("invalid toNanos"))?;
    let step =
        u128::try_from(step_seconds.unwrap_or(60).clamp(1, 86_400)).unwrap_or(60) * 1_000_000_000;
    let series = context
        .store
        .log_count_series(
            service.as_deref(),
            from..=to,
            severity_min,
            severity_max,
            query.as_deref(),
            step,
        )
        .await
        .map_err(field_err)?;
    Ok(series.into_iter().map(Point).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;

    use std::sync::Arc;

    #[tokio::test]
    async fn logs_around_returns_windowed_ascending_rows() {
        let store = Arc::new(MemoryStore::new());
        let anchor = 100_000_000_000;
        let mut anchor_log = log_row("api", "trace-a", anchor, "anchor");
        anchor_log.event_name = "checkout.completed".into();
        anchor_log.observed_ts_nanos = anchor + 2_000_000_000;
        store
            .ingest_logs(
                vec![
                    log_row("api", "trace-a", anchor - 60_000_000_000, "too-old"),
                    log_row("api", "trace-a", anchor - 10_000_000_000, "before"),
                    anchor_log,
                    log_row("api", "trace-a", anchor + 10_000_000_000, "after"),
                    log_row("api", "trace-a", anchor + 60_000_000_000, "too-new"),
                ],
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            format!(
                r#"{{
                  logsAround(anchorNanos: "{anchor}", windowSeconds: 30, service: "api") {{
                    tsNanos body eventName observedTsNanos
                  }}
                }}"#
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "logsAround query: {json}");
        let rows = json
            .pointer("/data/logsAround")
            .and_then(|value| value.as_array())
            .unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| row["body"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["before", "anchor", "after"]
        );
        assert_eq!(
            rows[1].get("eventName"),
            Some(&serde_json::json!("checkout.completed"))
        );
        assert_eq!(
            rows[1].get("observedTsNanos"),
            Some(&serde_json::json!("102000000000"))
        );
    }

    #[tokio::test]
    async fn logs_around_can_scope_to_trace_inside_window() {
        let store = Arc::new(MemoryStore::new());
        let anchor = 100_000_000_000;
        store
            .ingest_logs(
                vec![
                    log_row("api", "trace-a", anchor - 1_000_000_000, "trace-a-before"),
                    log_row("api", "trace-b", anchor, "trace-b-anchor"),
                    log_row("api", "trace-a", anchor + 1_000_000_000, "trace-a-after"),
                ],
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            format!(
                r#"{{
                  logsAround(anchorNanos: "{anchor}", windowSeconds: 30, traceId: "trace-a") {{
                    body traceId
                  }}
                }}"#
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "logsAround trace: {json}");
        assert_eq!(
            json.pointer("/data/logsAround")
                .and_then(|value| value.as_array())
                .unwrap()
                .iter()
                .map(|row| row["body"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["trace-a-before", "trace-a-after"]
        );
    }

    #[tokio::test]
    async fn logs_around_clamps_window_and_limit() {
        let store = Arc::new(MemoryStore::new());
        let anchor = 1_000_000_000_000;
        let mut rows = (0..550)
            .map(|index| {
                log_row(
                    "api",
                    "trace-a",
                    anchor + index * 1_000_000,
                    &format!("near-{index}"),
                )
            })
            .collect::<Vec<_>>();
        rows.push(log_row(
            "api",
            "trace-a",
            anchor + 700_000_000_000,
            "beyond-clamped-window",
        ));
        store.ingest_logs(rows, Default::default()).await.unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            format!(
                r#"{{
                  logsAround(anchorNanos: "{anchor}", windowSeconds: 9999, limit: 9999) {{
                    body
                  }}
                }}"#
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "logsAround clamp: {json}");
        let rows = json
            .pointer("/data/logsAround")
            .and_then(|value| value.as_array())
            .unwrap();
        assert_eq!(rows.len(), MAX_ROWS);
        assert!(
            rows.iter()
                .all(|row| row["body"] != "beyond-clamped-window")
        );
    }
}
