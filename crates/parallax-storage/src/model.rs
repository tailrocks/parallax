//! Normalized telemetry rows — the shapes the storage adapters persist,
//! mirroring the `GreptimeDB` DDL in the implementation spec §5.

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
    /// Raw `OTel` span events JSON (`[{name, time_unix_nano, attributes}]`) when
    /// the backing source projects it; absent sources default to no events.
    pub events: Option<String>,
    /// `OTel` span links — `[{traceId, spanId, attributes}]` — the standard
    /// cross-trace correlation (batch/async "sub-operation" references).
    pub links: serde_json::Value,
    pub attributes: serde_json::Value,
    pub resource: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRow {
    pub ts_nanos: u128,
    /// `OTel` `EventName`, set for typed log events and empty for plain logs.
    pub event_name: String,
    /// `OTel` `ObservedTimestamp` in ns. Zero means the source did not set it.
    pub observed_ts_nanos: u128,
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
    pub run_id: Option<String>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricExemplarRow {
    pub ts_nanos: u128,
    pub service: String,
    pub name: String,
    pub value: f64,
    pub trace_id: String,
    pub span_id: String,
    pub run_id: Option<String>,
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
    /// Bounded top-tag-values cache as JSON: `{key: {value: count}}`.
    #[serde(default = "default_tags")]
    pub tags: String,
}

fn default_tags() -> String {
    "{}".to_string()
}

/// How `issues` lists are ordered (spec §8 `IssueSort`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSortKey {
    LastSeen,
    FirstSeen,
    Events,
    /// Last-24h occurrence sum from the minute rollups.
    Trend,
}

/// Filter + page window for `issues` (spec §8; flat-args dialect).
#[derive(Debug, Clone, Default)]
pub struct IssueQuery {
    pub service: Option<String>,
    pub status: Option<String>,
    /// Substring match against title, `error_type`, and fingerprint.
    pub query: Option<String>,
    /// Window on `last_seen`.
    pub from_nanos: Option<u128>,
    pub to_nanos: Option<u128>,
    pub tag_key: Option<String>,
    pub tag_value: Option<String>,
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

/// One aggregated point of a metric series.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SeriesPoint {
    pub ts_nanos: u128,
    pub value: f64,
}

/// Aggregations for metric series (RATE applies to monotonic sums: per-second
/// delta of the bucketed sum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricAgg {
    Avg,
    Min,
    Max,
    Sum,
    Rate,
}

impl MetricAgg {
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "avg" => Self::Avg,
            "min" => Self::Min,
            "max" => Self::Max,
            "sum" => Self::Sum,
            "rate" => Self::Rate,
            _ => return None,
        })
    }
}

/// Saved user dashboard (metadata store).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub layout: String,
    pub created_at_nanos: u128,
    pub updated_at_nanos: u128,
}

/// Saved investigation/case state (metadata store).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investigation {
    pub id: String,
    pub name: String,
    pub state: String,
    pub created_at_nanos: u128,
    pub updated_at_nanos: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvestigationState {
    pub version: u8,
    #[serde(default)]
    pub window: InvestigationWindow,
    #[serde(default)]
    pub pins: Vec<InvestigationPin>,
    #[serde(default)]
    pub notes: String,
}

impl Default for InvestigationState {
    fn default() -> Self {
        Self {
            version: 1,
            window: InvestigationWindow::default(),
            pins: Vec::new(),
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvestigationWindow {
    pub range: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

impl Default for InvestigationWindow {
    fn default() -> Self {
        Self {
            range: "24h".to_string(),
            from: None,
            to: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvestigationPin {
    pub kind: InvestigationPinKind,
    #[serde(rename = "ref")]
    pub ref_: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InvestigationPinKind {
    Trace,
    Issue,
    Run,
    Log,
    View,
}

/// Named saved URL-state view (metadata store).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedView {
    pub id: String,
    pub name: String,
    pub page: String,
    pub state: String,
    pub created_at_nanos: u128,
    pub updated_at_nanos: u128,
}

/// One bucket of an issue's occurrence trend (metadata store rollup).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub ts_nanos: u128,
    pub count: u64,
}

/// One derived error occurrence, ready for metadata issue upsert.
#[derive(Debug)]
pub struct IssueOccurrence<'a> {
    pub fingerprint: &'a str,
    pub title: String,
    pub error_type: &'a str,
    pub culprit: Option<String>,
    pub service: &'a str,
    pub ts_nanos: u128,
    pub trace_id: Option<&'a str>,
    /// The event's attributes — merged into the issue's bounded tag cache.
    pub attributes: &'a serde_json::Value,
}
