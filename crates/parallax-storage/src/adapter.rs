//! The storage adapter boundary. Everything engine-specific lives behind
//! `TelemetryStore`; product code never sees an engine.

use crate::model::*;
use parallax_semconv as semconv;
use std::collections::HashMap;
use std::ops::RangeInclusive;

mod error;
pub use error::{StorageError, StorageErrorKind, StorageResult};

pub use crate::adapter_math::{attribute_compare_score, rate_from_buckets};
pub use crate::adapter_rules::{
    field_key_identifier_like, field_key_namespace, metric_group_label_allowed,
    runtime_metric_family, runtime_metric_unit,
};

pub const MAX_ROWS: usize = 500;
pub const ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT: usize = 24;
pub const ATTRIBUTE_COMPARE_TOP_N_CAP: usize = 50;
pub const FIELD_KEYS_CAP: usize = 200;
pub const FIELD_TOP_VALUES_CAP: usize = 10;
pub const SERVICE_MAP_TRACE_CAP: usize = 100;

/// An invocation id observed in telemetry (spans/logs carrying
/// `cli.invocation.id`), whether or not it was registered through the CLI
/// wrapper. This is how externally-instrumented tools (e.g. jackin') appear
/// in the invocations UI.
#[derive(Debug, Clone)]
pub struct ObservedInvocation {
    pub invocation_id: String,
    pub first_nanos: u128,
    pub last_nanos: u128,
    pub span_count: u64,
    pub log_count: u64,
    /// One service name seen under this invocation (display hint).
    pub service: String,
    /// Latest `cli.command.name` seen on this invocation's root spans.
    pub last_command: Option<String>,
    /// `app.mode` seen on this invocation's root spans.
    pub app_mode: Option<String>,
}

/// One interactive session inside an invocation, paired from
/// `session.start` / `session.end` log events. An open session has no end.
#[derive(Debug, Clone)]
pub struct InvocationSession {
    pub session_id: String,
    pub previous_session_id: Option<String>,
    pub start_nanos: u128,
    pub end_nanos: Option<u128>,
}

/// One screen visit paired from `ui.screen.entered` / `ui.screen.exited`
/// events by `ui.screen.visit.id`.
#[derive(Debug, Clone)]
pub struct ScreenVisit {
    pub screen_id: String,
    pub visit_id: String,
    pub session_id: Option<String>,
    pub navigation_sequence: Option<i64>,
    pub transition_reason: Option<String>,
    pub entered_nanos: u128,
    pub exited_nanos: Option<u128>,
}

/// One bounded user action (`ui.action` root span).
#[derive(Debug, Clone)]
pub struct UiAction {
    pub name: String,
    pub screen_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: String,
    pub start_nanos: u128,
    pub duration_ns: u128,
    pub outcome: Option<String>,
    pub has_error: bool,
}

/// Aggregate of one `background.cycle.name` family of periodic daemon spans.
#[derive(Debug, Clone)]
pub struct BackgroundCycleSummary {
    pub name: String,
    pub count: u64,
    pub error_count: u64,
    pub p50_ns: Option<f64>,
    pub p95_ns: Option<f64>,
    pub last_nanos: u128,
    pub last_trace_id: String,
}

/// One consumer attempt of a detached job.
#[derive(Debug, Clone)]
pub struct JobAttempt {
    pub start_nanos: u128,
    pub duration_ns: u128,
    pub outcome: Option<String>,
    pub has_error: bool,
    pub trace_id: String,
}

/// One detached job: producer span plus consumer attempts sharing `job.id`.
#[derive(Debug, Clone)]
pub struct JobSummary {
    pub job_id: String,
    pub job_type: Option<String>,
    pub produced_nanos: Option<u128>,
    pub attempts: Vec<JobAttempt>,
    pub last_trace_id: String,
}

/// One agent conversation (`gen_ai.conversation.id`) summary.
#[derive(Debug, Clone)]
pub struct ConversationSummary {
    pub conversation_id: String,
    pub agent_name: Option<String>,
    pub provider_name: Option<String>,
    pub first_nanos: u128,
    pub last_nanos: u128,
    pub span_count: u64,
    pub input_tokens: Option<f64>,
    pub output_tokens: Option<f64>,
}

/// Result of a raw read-only SQL query against the engine (the GreptimeDB
/// power feature surfaced through API/CLI/UI).
#[derive(Debug, Clone)]
pub struct SqlResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// One trace summarized for list views: the root span plus aggregates.
#[derive(Debug, Clone)]
pub struct TraceSummary {
    pub trace_id: String,
    pub root_name: String,
    pub service: String,
    pub start_nanos: u128,
    pub duration_ns: u128,
    pub span_count: u64,
    pub has_error: bool,
}

/// Paged trace-list result. `total` is exact for in-memory; Greptime counts
/// the same filtered representative-span result set used for the page.
#[derive(Debug, Clone)]
pub struct TraceList {
    pub items: Vec<TraceSummary>,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TraceSort {
    #[default]
    StartDesc,
    DurationDesc,
    DurationAsc,
    SpanCountDesc,
}

/// Whole-system overview counters for one inclusive time window.
#[derive(Debug, Clone)]
pub struct OverviewTotals {
    pub span_count: u64,
    pub trace_count: u64,
    pub log_count: u64,
    pub metric_point_count: u64,
    pub error_count: u64,
    pub error_rate: f64,
    pub active_services: u64,
}

/// Per-service summary row for the services index.
#[derive(Debug, Clone)]
pub struct ServiceSummary {
    pub name: String,
    pub last_seen_nanos: u128,
    pub span_count: u64,
    pub error_count: u64,
    pub p95_ms: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseWindow {
    pub version: String,
    pub first_seen_nanos: u128,
    pub last_seen_nanos: u128,
    pub span_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCatalogRow {
    pub name: String,
    pub service_version: Option<String>,
    pub service_namespace: Option<String>,
    pub deployment_environment: Option<String>,
    pub telemetry_sdk_language: Option<String>,
    pub telemetry_sdk_name: Option<String>,
    pub telemetry_sdk_version: Option<String>,
    pub last_seen_nanos: u128,
    pub instance_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
    Spans,
    Traces,
    Logs,
    Errors,
    MetricPoints,
}

/// Trace-derived RED series. Rate is spans/second per bucket; latency values
/// are milliseconds.
#[derive(Debug, Clone, Default)]
pub struct SpanRed {
    pub rate: Vec<SeriesPoint>,
    pub error_rate: Vec<SeriesPoint>,
    pub p50: Vec<SeriesPoint>,
    pub p95: Vec<SeriesPoint>,
    pub p99: Vec<SeriesPoint>,
}

/// One runtime metric lane returned by `runtimeSnapshot`.
#[derive(Debug, Clone)]
pub struct RuntimeMetricSeries {
    pub family: String,
    pub metric: String,
    pub unit: Option<String>,
    pub points: Vec<SeriesPoint>,
}

/// Filtered trace browse (UI Traces page / CLI `parallax traces` / GraphQL
/// `traces`): every filter optional. `service` matches any trace the service
/// **participates in** (a span of that service anywhere in the trace, not only
/// the root) — so a cross-service trace rooted at `checkout` still surfaces
/// under `--service catalog`. `error_only` looks at the whole trace. The other
/// filters apply to the trace's **representative span**: its root (no parent),
/// or — when no root span was stored (e.g. all-`INTERNAL` traces) — the
/// earliest span, so such traces still list instead of vanishing.
#[derive(Debug, Clone, Default)]
pub struct TraceQuery {
    pub service: Option<String>,
    pub from_nanos: Option<u128>,
    pub to_nanos: Option<u128>,
    pub min_duration_ns: Option<u128>,
    pub max_duration_ns: Option<u128>,
    pub error_only: bool,
    /// Substring of the representative span name.
    pub name_contains: Option<String>,
    pub limit: usize,
    pub offset: usize,
    pub sort: TraceSort,
}

/// One overrepresented span-attribute value in a selected window compared
/// with a baseline window.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeCompareRow {
    pub key: String,
    pub value: String,
    pub selected_count: u64,
    pub selected_total: u64,
    pub baseline_count: u64,
    pub baseline_total: u64,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldSource {
    Span,
    Resource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldKey {
    pub key: String,
    pub namespace: String,
    pub source: FieldSource,
    pub row_count: u64,
    pub non_null_count: u64,
    pub coverage: f64,
    pub is_identifier: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValueCount {
    pub value: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldStats {
    pub key: String,
    pub namespace: String,
    pub source: FieldSource,
    pub row_count: u64,
    pub non_null_count: u64,
    pub distinct_count: u64,
    pub coverage: f64,
    pub capped: bool,
    pub is_identifier: bool,
    pub top_values: Vec<FieldValueCount>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceEdge {
    pub source: String,
    pub target: String,
    pub call_count: u64,
    pub error_count: u64,
    pub p50_ms: f64,
    pub p95_ms: f64,
}

pub fn span_field_key_allowed(key: &str) -> bool {
    let trimmed = key.trim();
    if trimmed.is_empty() || trimmed.len() > 160 {
        return false;
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
    {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    !(lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("authorization")
        || lower.contains("cookie")
        || lower.contains("stacktrace")
        || lower == "db.statement"
        || lower == "db.query.text"
        || lower == semconv::GRAPHQL_DOCUMENT
        || lower == "url.full"
        || lower == "url.query"
        || lower == "process.command_args"
        || lower == "resource.process.command_args"
        || lower == semconv::SHELL_COMMAND
        || lower.ends_with(".body")
        || lower.ends_with("_body")
        || lower.ends_with(".message")
        || lower.ends_with("_message"))
}

pub fn field_value_allowed(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && !trimmed.chars().any(char::is_control)
}

pub fn field_value_display(value: &str) -> Option<String> {
    if !field_value_allowed(value) {
        return None;
    }
    let trimmed = value.trim();
    if trimmed.chars().count() <= 256 {
        return Some(trimmed.to_string());
    }
    Some(format!(
        "{}...",
        trimmed.chars().take(256).collect::<String>()
    ))
}

pub fn attribute_compare_key_allowed(key: &str) -> bool {
    let trimmed = key.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let compact = lower.replace(['.', '-'], "_");
    let leaf = lower.rsplit('.').next().unwrap_or(lower.as_str());

    if lower.starts_with("enduser.")
        || matches!(
            lower.as_str(),
            "trace.id"
                | "span.id"
                | "run.id"
                | "user.id"
                | "session.id"
                | "enduser.id"
                | semconv::EXCEPTION_MESSAGE
                | semconv::EXCEPTION_STACKTRACE
                | semconv::EXCEPTION_ESCAPED
                | "db.statement"
                | "http.request.body"
                | "http.response.body"
                | "url.full"
        )
    {
        return false;
    }

    if matches!(
        compact.as_str(),
        "trace_id" | "span_id" | "invocation_id" | "user_id" | "session_id" | "enduser_id"
    ) {
        return false;
    }

    if matches!(
        leaf,
        "id" | "token" | "password" | "secret" | "authorization"
    ) || lower.ends_with(".id")
        || lower.ends_with("_id")
        || compact.contains("trace_id")
        || compact.contains("span_id")
        || compact.contains("invocation_id")
        || compact.contains("user_id")
        || compact.contains("session_id")
        || lower.contains("uuid")
        || lower.contains("guid")
        || lower.contains("fingerprint")
        || lower.contains("hash")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("authorization")
        || lower.contains("stacktrace")
        || lower.contains("message")
        || lower.contains("body")
    {
        return false;
    }

    true
}

pub fn attribute_compare_value_allowed(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 128 || trimmed.chars().any(char::is_control) {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let compact = lower.replace('-', "");
    if lower.contains("-----begin")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("authorization")
        || lower.contains("bearer ")
    {
        return false;
    }
    if compact.len() >= 24 && compact.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    true
}

mod traits;
pub use traits::*;
