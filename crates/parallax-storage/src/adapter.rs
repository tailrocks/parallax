//! The storage adapter boundary. Everything engine-specific lives behind
//! `TelemetryStore`; product code never sees an engine.

use crate::model::*;
use parallax_proto::semconv;
use std::collections::HashMap;
use std::ops::RangeInclusive;

pub const MAX_ROWS: usize = 500;
pub const ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT: usize = 24;
pub const ATTRIBUTE_COMPARE_TOP_N_CAP: usize = 50;
pub const FIELD_KEYS_CAP: usize = 200;
pub const FIELD_TOP_VALUES_CAP: usize = 10;
pub const SERVICE_MAP_TRACE_CAP: usize = 100;

/// A run id observed in telemetry (spans/logs carrying `parallax.run.id`),
/// whether or not the run was registered through the CLI wrapper. This is
/// how externally-instrumented tools (e.g. jackin') appear in the runs UI.
#[derive(Debug, Clone)]
pub struct ObservedRun {
    pub run_id: String,
    pub first_nanos: u128,
    pub last_nanos: u128,
    pub span_count: u64,
    pub log_count: u64,
    /// One service name seen under this run (display hint).
    pub service: String,
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

pub const RUNTIME_METRIC_PREFIXES: &[&str] = &[
    "process.",
    "system.",
    "jvm.",
    "tokio.runtime.",
    "container.",
    "db.client.connection.",
];

pub fn runtime_metric_family(name: &str) -> Option<&'static str> {
    RUNTIME_METRIC_PREFIXES
        .iter()
        .find(|prefix| name.starts_with(**prefix))
        .map(|prefix| prefix.trim_end_matches('.'))
}

pub fn runtime_metric_unit(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let unit = if lower.ends_with("_bytes")
        || lower.ends_with(".bytes")
        || lower.contains(".memory.")
        || lower.contains("_memory_")
    {
        "bytes"
    } else if lower.ends_with("_ms") || lower.ends_with(".ms") {
        "ms"
    } else if lower.contains("cpu.utilization") || lower.contains("cpu_usage") {
        "ratio"
    } else {
        return None;
    };
    Some(unit.to_string())
}

pub fn metric_group_label_allowed(label: &str) -> bool {
    let lower = label.trim().to_ascii_lowercase();
    if lower.is_empty() || lower.len() > 128 {
        return false;
    }
    let compact = lower.replace(['.', '-'], "_");
    let leaf = lower.rsplit('.').next().unwrap_or(lower.as_str());
    let leaf_compact = leaf.replace('-', "_");
    !matches!(
        lower.as_str(),
        "trace.id" | "run.id" | "user.id" | "session.id"
    ) && !matches!(
        compact.as_str(),
        "trace_id" | "run_id" | "user_id" | "session_id"
    ) && !matches!(
        leaf_compact.as_str(),
        "trace_id" | "run_id" | "user_id" | "session_id"
    )
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
        || lower == "graphql.document"
        || lower == "url.full"
        || lower == "url.query"
        || lower == "process.command_args"
        || lower == "resource.process.command_args"
        || lower == "shell.command"
        || lower.ends_with(".body")
        || lower.ends_with("_body")
        || lower.ends_with(".message")
        || lower.ends_with("_message"))
}

pub fn field_key_namespace(key: &str) -> String {
    let logical = key.strip_prefix("resource.").unwrap_or(key);
    logical
        .split('.')
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or("custom")
        .to_string()
}

pub fn field_key_identifier_like(key: &str) -> bool {
    let logical = key.strip_prefix("resource.").unwrap_or(key);
    let lower = logical.to_ascii_lowercase();
    let compact = lower.replace(['.', '-'], "_");
    let leaf = lower.rsplit('.').next().unwrap_or(lower.as_str());
    let leaf_compact = leaf.replace('-', "_");

    matches!(
        leaf_compact.as_str(),
        "id" | "trace_id" | "span_id" | "run_id" | "user_id" | "session_id" | "enduser_id"
    ) || lower.ends_with(".id")
        || lower.ends_with("_id")
        || compact.contains("trace_id")
        || compact.contains("span_id")
        || compact.contains("run_id")
        || compact.contains("user_id")
        || compact.contains("session_id")
        || lower.contains("uuid")
        || lower.contains("guid")
        || lower.contains("fingerprint")
        || lower.contains("hash")
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
        "trace_id" | "span_id" | "run_id" | "user_id" | "session_id" | "enduser_id"
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
        || compact.contains("run_id")
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

pub fn attribute_compare_score(
    selected_count: u64,
    selected_total: u64,
    baseline_count: u64,
    baseline_total: u64,
) -> f64 {
    let selected_share = if selected_total == 0 {
        0.0
    } else {
        selected_count as f64 / selected_total as f64
    };
    let baseline_share = if baseline_total == 0 {
        0.0
    } else {
        baseline_count as f64 / baseline_total as f64
    };
    (selected_share - baseline_share).clamp(0.0, 1.0)
}

#[async_trait::async_trait]
pub trait TelemetryStore: Send + Sync {
    /// Ingest a traces batch: forward the raw OTLP bytes to GreptimeDB's native
    /// `/v1/otlp/v1/traces` endpoint (auto-creates `opentelemetry_traces`). The
    /// decoded `spans` are the tee — kept for the in-memory adapter and unused
    /// by the native forward.
    async fn ingest_traces(
        &self,
        request: &parallax_proto::collector_trace::ExportTraceServiceRequest,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()>;
    /// Ingest a logs batch: forward the raw OTLP bytes to the native
    /// `/v1/otlp/v1/logs` endpoint (auto-creates `opentelemetry_logs`).
    async fn ingest_logs(
        &self,
        request: &parallax_proto::collector_logs::ExportLogsServiceRequest,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()>;
    /// Ingest a metrics batch: forward the raw OTLP bytes to the native
    /// `/v1/otlp/v1/metrics` endpoint (per-metric metric-engine tables), then
    /// persist the run-scoped subset of `points` into `run_metric_points`.
    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()>;
    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> anyhow::Result<()>;

    /// Anchored read: every span of one trace, start-time ascending.
    async fn spans_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<SpanRow>>;
    /// Resolve summaries for span-link target trace ids.
    /// Returns at most one summary per id, preserving input order where possible.
    async fn traces_by_ids(&self, trace_ids: &[String]) -> anyhow::Result<Vec<TraceSummary>>;
    /// Run-scoped read: every span tagged with one `parallax.run.id`.
    /// `range` bounds the logs-table fallback scan (plan 085).
    async fn spans_by_run(
        &self,
        run_id: &str,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<SpanRow>>;
    /// Batched run-scoped span read: up to `limit_per_run` newest spans per
    /// run id (then returned start-time ascending within each run). Default
    /// loops `spans_by_run`; Greptime overrides with one windowed query.
    async fn spans_by_runs(
        &self,
        run_ids: &[String],
        limit_per_run: usize,
    ) -> anyhow::Result<HashMap<String, Vec<SpanRow>>> {
        let mut out = HashMap::with_capacity(run_ids.len());
        let range = 0..=u128::MAX;
        for run_id in run_ids {
            out.insert(
                run_id.clone(),
                self.spans_by_run(run_id, limit_per_run, range.clone())
                    .await?,
            );
        }
        Ok(out)
    }
    /// Run-scoped read: every log tagged with one `parallax.run.id`.
    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>>;
    /// Anchored read: every log of one trace, time ascending.
    async fn logs_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<LogRow>>;
    /// Distinct metric names inside `range` (plan 085 windows extension scan).
    async fn metric_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>>;
    /// Discover groupable metric label/tag keys for one metric.
    async fn metric_labels(&self, name: &str) -> anyhow::Result<Vec<String>>;
    /// Distinct scalar values for one metric label inside an inclusive window.
    async fn metric_label_values(
        &self,
        name: &str,
        label: &str,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<String>>;
    /// Distinct service names across signals inside `range` (plan 085).
    async fn service_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>>;
    /// Whole-system overview counters for an inclusive time window.
    async fn overview_totals(&self, range: RangeInclusive<u128>) -> anyhow::Result<OverviewTotals>;
    /// Signal volume per bucket for overview trend charts.
    async fn signal_count_series(
        &self,
        kind: SignalKind,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
    /// Service summary rows for the services index.
    async fn service_summaries(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceSummary>>;
    /// Per-version activity windows for one service, ordered by first sighting.
    async fn release_windows(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ReleaseWindow>>;
    /// Resource-identity catalog rows, one per service in the window.
    async fn service_catalog(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceCatalogRow>>;
    /// Trace-derived RED series; works even when a service emits no metrics.
    async fn span_red_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<SpanRed>;
    /// Aggregated series for a point metric, bucketed by `step_nanos`.
    /// `run_id` scopes to points whose resource carried `parallax.run.id`
    /// (run-anchored cross-analytics: CPU/memory beside a run's traces).
    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
        run_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
    /// Approximate quantile series from a histogram metric's buckets.
    async fn histogram_quantile(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        q: f64,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
    /// Multiple histogram quantiles from one logical scan. The default loops
    /// [`Self::histogram_quantile`]; Greptime overrides it with one
    /// multi-quantile SQL query. Return order matches `quantiles`.
    async fn histogram_quantiles(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        quantiles: &[f64],
    ) -> anyhow::Result<Vec<Vec<SeriesPoint>>> {
        let mut out = Vec::with_capacity(quantiles.len());
        for q in quantiles {
            out.push(
                self.histogram_quantile(name, service, range.clone(), step_nanos, *q)
                    .await?,
            );
        }
        Ok(out)
    }
    /// Trace-linked metric exemplars, time-bounded and newest first.
    async fn metric_exemplars(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<MetricExemplarRow>>;
    /// Error events for a fingerprint within a time range, newest first.
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>>;
    /// Distinct run ids inside `range`, most recent activity first (plan 085).
    async fn observed_runs(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ObservedRun>>;
    /// Recent traces (root spans + aggregates), newest first.
    async fn recent_traces(&self, limit: usize) -> anyhow::Result<Vec<TraceSummary>> {
        Ok(self
            .traces_search(&TraceQuery {
                limit,
                ..TraceQuery::default()
            })
            .await?
            .items)
    }
    /// Filtered trace browse (root spans + aggregates).
    async fn traces_search(&self, query: &TraceQuery) -> anyhow::Result<TraceList>;
    /// Overrepresented span-attribute values in a selected cohort compared
    /// with a baseline cohort. Candidate-key discovery is bounded and denies
    /// identifier, raw text, stacktrace, and secret-shaped attributes.
    async fn attribute_compare(
        &self,
        selected: RangeInclusive<u128>,
        baseline: RangeInclusive<u128>,
        service: Option<&str>,
        error_only: bool,
        keys: &[String],
        top_n: usize,
    ) -> anyhow::Result<Vec<AttributeCompareRow>>;
    /// Discover scalar span/resource attribute keys in a window. Resource
    /// attributes are exposed as `resource.<key>` so the API key is unambiguous.
    async fn span_field_keys(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<FieldKey>>;
    /// Bounded stats for one discovered span/resource attribute key.
    async fn span_field_stats(
        &self,
        key: &str,
        range: RangeInclusive<u128>,
        service: Option<&str>,
    ) -> anyhow::Result<FieldStats>;
    /// Trace-path service edges derived from child SERVER spans paired with
    /// their parent span inside bounded traces from the requested window.
    async fn service_map(
        &self,
        range: RangeInclusive<u128>,
        max_traces: usize,
    ) -> anyhow::Result<Vec<ServiceEdge>>;
    /// Error events across a set of traces, newest first (run-anchored reads).
    async fn error_events_by_traces(
        &self,
        trace_ids: &[String],
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>>;
    /// Unified log browse: every filter optional, newest first.
    async fn logs_search(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<LogRow>>;
    /// Aggregated series split by one attribute key's value (spec §8
    /// `metricSeries(groupBy:)`); rows missing the key group under "(none)".
    async fn metric_series_grouped(
        &self,
        name: &str,
        service: Option<&str>,
        group_by: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<Vec<(String, Vec<SeriesPoint>)>>;
    /// Runtime metric lanes across known runtime families for service/run scope.
    async fn runtime_snapshot(
        &self,
        service: Option<&str>,
        run_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<RuntimeMetricSeries>>;
    /// Histogram sample counts summed per bucket (request-rate numerator).
    async fn histogram_count_series(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
    /// Error events per bucket for one service (overview error rate).
    async fn error_count_series(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
    /// Log counts per bucket under the same filters as `logs_search` — the
    /// Discover-style histogram must reflect the active query.
    async fn log_count_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
    /// Raw read-only SQL against the engine (SELECT-shaped statements only).
    /// The API enforces the user-facing guard; adapters repeat a defensive
    /// shape check before execution. The in-memory store has no SQL surface
    /// and returns an error.
    async fn raw_sql(&self, query: &str) -> anyhow::Result<SqlResult>;
}
