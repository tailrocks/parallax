//! Live telemetry tail: the ingest worker broadcasts every normalized log
//! and span batch; `/v1/logs/stream` and `/v1/traces/stream` serve them as
//! Server-Sent Events with the same filter vocabulary as the `logs` and
//! `traces` GraphQL queries. SSE over WebSocket deliberately: a tail is
//! one-way, EventSource reconnects on its own, and the local profile has no
//! proxy buffering to fear.
//!
//! Live filters are per-row predicates only (service, severity/duration
//! floor, substring, ids) — no aggregation, no time travel. History and
//! aggregates belong to the polling queries; this is the kubectl-logs view.

use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_core::Stream;
use parallax_storage::model::{LogRow, SpanRow};
use std::sync::Arc;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

/// Broadcast capacity: a lagging browser drops old batches rather than
/// stalling ingest (broadcast semantics), which is exactly tail behavior.
const CHANNEL_CAPACITY: usize = 256;

pub type LogSender = tokio::sync::broadcast::Sender<Arc<[LogRow]>>;
pub type SpanSender = tokio::sync::broadcast::Sender<Arc<[SpanRow]>>;

/// Both live fan-out channels, cloned into the worker and the routes.
#[derive(Clone)]
pub struct LiveChannels {
    pub logs: LogSender,
    pub spans: SpanSender,
}

pub fn channels() -> LiveChannels {
    LiveChannels {
        logs: tokio::sync::broadcast::channel(CHANNEL_CAPACITY).0,
        spans: tokio::sync::broadcast::channel(CHANNEL_CAPACITY).0,
    }
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
    State(channels): State<LiveChannels>,
    Query(filter): Query<StreamFilter>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let receiver = channels.logs.subscribe();
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

/// Span tail filters — per-row predicates, mirroring the `traces` GraphQL
/// vocabulary where it applies to a single finished span.
#[derive(Debug, serde::Deserialize)]
pub struct SpanStreamFilter {
    pub service: Option<String>,
    /// Only spans at least this long (the live "show me slow ones" knob).
    pub min_duration_ms: Option<f64>,
    /// Only spans with STATUS_CODE_ERROR.
    pub errors_only: Option<bool>,
    /// Substring of the span name.
    pub q: Option<String>,
    pub trace_id: Option<String>,
    pub run_id: Option<String>,
}

impl SpanStreamFilter {
    fn matches(&self, span: &SpanRow) -> bool {
        self.service.as_deref().is_none_or(|s| span.service == s)
            && self
                .min_duration_ms
                .is_none_or(|ms| span.duration_ns as f64 >= ms * 1e6)
            && (!self.errors_only.unwrap_or(false) || span.status_code == "STATUS_CODE_ERROR")
            && self.q.as_deref().is_none_or(|n| span.name.contains(n))
            && self.trace_id.as_deref().is_none_or(|t| span.trace_id == t)
            && self
                .run_id
                .as_deref()
                .is_none_or(|r| span.run_id.as_deref() == Some(r))
    }
}

fn span_event(span: &SpanRow) -> serde_json::Value {
    serde_json::json!({
        "tsNanos": span.ts_nanos.to_string(),
        "service": span.service,
        "traceId": span.trace_id,
        "spanId": span.span_id,
        "parentSpanId": span.parent_span_id,
        "name": span.name,
        "kind": span.kind,
        "statusCode": span.status_code,
        "durationNs": span.duration_ns.to_string(),
        "runId": span.run_id,
    })
}

pub async fn stream_traces(
    State(channels): State<LiveChannels>,
    Query(filter): Query<SpanStreamFilter>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let receiver = channels.spans.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(move |batch| {
        let batch = batch.ok()?;
        let matching: Vec<serde_json::Value> = batch
            .iter()
            .filter(|span| filter.matches(span))
            .map(span_event)
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
