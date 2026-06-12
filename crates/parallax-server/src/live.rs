//! Live log tail: the ingest worker broadcasts every normalized log batch;
//! `/v1/logs/stream` serves them as Server-Sent Events with the same filter
//! vocabulary as the `logs` GraphQL query. SSE over WebSocket deliberately:
//! a log tail is one-way, EventSource reconnects on its own, and the local
//! profile has no proxy buffering to fear.

use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_core::Stream;
use parallax_storage::model::LogRow;
use std::sync::Arc;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

/// Broadcast capacity: a lagging browser drops old batches rather than
/// stalling ingest (broadcast semantics), which is exactly tail behavior.
const CHANNEL_CAPACITY: usize = 256;

pub type LogSender = tokio::sync::broadcast::Sender<Arc<[LogRow]>>;

pub fn channel() -> LogSender {
    tokio::sync::broadcast::channel(CHANNEL_CAPACITY).0
}

#[derive(Debug, serde::Deserialize)]
pub struct StreamFilter {
    pub service: Option<String>,
    pub severity_min: Option<i32>,
    pub q: Option<String>,
    pub trace_id: Option<String>,
    pub run_id: Option<String>,
}

impl StreamFilter {
    fn matches(&self, log: &LogRow) -> bool {
        self.service.as_deref().is_none_or(|s| log.service == s)
            && self.severity_min.is_none_or(|m| log.severity_num >= m)
            && self.q.as_deref().is_none_or(|n| log.body.contains(n))
            && self.trace_id.as_deref().is_none_or(|t| log.trace_id == t)
            && self
                .run_id
                .as_deref()
                .is_none_or(|r| log.run_id.as_deref() == Some(r))
    }
}

fn log_event(log: &LogRow) -> serde_json::Value {
    serde_json::json!({
        "tsNanos": log.ts_nanos.to_string(),
        "service": log.service,
        "severityNum": log.severity_num,
        "severityText": log.severity_text,
        "body": log.body,
        "traceId": log.trace_id,
        "spanId": log.span_id,
        "runId": log.run_id,
        "scopeName": log.scope_name,
        "attributes": log.attributes.to_string(),
        "resource": log.resource.to_string(),
    })
}

pub async fn stream_logs(
    State(sender): State<LogSender>,
    Query(filter): Query<StreamFilter>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let receiver = sender.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(move |batch| {
        // Lagged receivers skip dropped batches and keep tailing.
        let batch = batch.ok()?;
        let matching: Vec<serde_json::Value> = batch
            .iter()
            .filter(|log| filter.matches(log))
            .map(log_event)
            .collect();
        if matching.is_empty() {
            return None;
        }
        Some(
            Event::default()
                .json_data(serde_json::Value::Array(matching))
                .map_err(axum::Error::new),
        )
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
