//! Normalized telemetry rows — the shapes the storage adapters persist,
//! mirroring the GreptimeDB DDL in the implementation spec §5.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanRow {
    pub ts_nanos: u128,
    pub service: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub status_code: String,
    pub status_message: String,
    pub duration_ns: u128,
    pub run_id: Option<String>,
    pub scope_name: String,
    pub attributes: serde_json::Value,
    pub resource: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRow {
    pub ts_nanos: u128,
    pub service: String,
    pub severity_num: i32,
    pub severity_text: String,
    pub body: String,
    pub trace_id: String,
    pub span_id: String,
    pub run_id: Option<String>,
    pub scope_name: String,
    pub attributes: serde_json::Value,
    pub resource: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPointRow {
    pub ts_nanos: u128,
    pub service: String,
    pub name: String,
    pub value: f64,
    pub is_monotonic: bool,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramRow {
    pub ts_nanos: u128,
    pub service: String,
    pub name: String,
    pub count: u64,
    pub sum: f64,
    pub bucket_counts: Vec<u64>,
    pub bounds: Vec<f64>,
    pub attributes: serde_json::Value,
}

/// Where an error event was derived from (both exception encodings + status).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSource {
    SpanException,
    SpanStatus,
    LogRecord,
    LogException,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEventRow {
    pub ts_nanos: u128,
    pub service: String,
    pub fingerprint: String,
    pub error_type: String,
    pub message: String,
    pub stacktrace: Option<String>,
    pub source: ErrorSource,
    pub trace_id: String,
    pub span_id: String,
    pub attributes: serde_json::Value,
}

/// Mutable issue state (metadata store; spec §6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub fingerprint: String,
    pub title: String,
    pub error_type: String,
    pub culprit: Option<String>,
    pub service: String,
    pub status: String,
    pub first_seen_nanos: u128,
    pub last_seen_nanos: u128,
    pub event_count: u64,
    pub last_trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub command: Option<String>,
    pub started_at_nanos: u128,
    pub ended_at_nanos: Option<u128>,
    pub exit_code: Option<i32>,
    pub status: String,
}
