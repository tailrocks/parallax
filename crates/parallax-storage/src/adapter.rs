//! The storage adapter boundary. Everything engine-specific lives behind
//! `TelemetryStore`; product code never sees an engine.

use crate::model::*;
use std::ops::RangeInclusive;

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
    pub error_only: bool,
    /// Substring of the representative span name.
    pub name_contains: Option<String>,
    pub limit: usize,
}

#[async_trait::async_trait]
pub trait TelemetryStore: Send + Sync {
    /// Ingest a traces batch: forward the raw OTLP bytes to GreptimeDB's native
    /// `/v1/otlp/v1/traces` endpoint (auto-creates `opentelemetry_traces`). The
    /// decoded `spans` are the tee — kept for the in-memory adapter and unused
    /// by the native forward.
    async fn ingest_traces(&self, spans: Vec<SpanRow>, raw: bytes::Bytes) -> anyhow::Result<()>;
    /// Ingest a logs batch: forward the raw OTLP bytes to the native
    /// `/v1/otlp/v1/logs` endpoint (auto-creates `opentelemetry_logs`).
    async fn ingest_logs(&self, logs: Vec<LogRow>, raw: bytes::Bytes) -> anyhow::Result<()>;
    /// Ingest a metrics batch: forward the raw OTLP bytes to the native
    /// `/v1/otlp/v1/metrics` endpoint (per-metric metric-engine tables), then
    /// persist the run-scoped subset of `points` into `run_metric_points`.
    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        histograms: Vec<HistogramRow>,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()>;
    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> anyhow::Result<()>;

    /// Anchored read: every span of one trace, start-time ascending.
    async fn spans_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<SpanRow>>;
    /// Run-scoped read: every span tagged with one `parallax.run.id`.
    async fn spans_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<SpanRow>>;
    /// Run-scoped read: every log tagged with one `parallax.run.id`.
    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>>;
    /// Anchored read: every log of one trace, time ascending.
    async fn logs_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<LogRow>>;
    /// Distinct metric names (both point and histogram metrics).
    async fn metric_names(&self) -> anyhow::Result<Vec<String>>;
    /// Distinct service names seen in metrics.
    async fn service_names(&self) -> anyhow::Result<Vec<String>>;
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
    /// Error events for a fingerprint within a time range, newest first.
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>>;
    /// Distinct run ids seen in telemetry, most recent activity first.
    async fn observed_runs(&self, limit: usize) -> anyhow::Result<Vec<ObservedRun>>;
    /// Recent traces (root spans + aggregates), newest first.
    async fn recent_traces(&self, limit: usize) -> anyhow::Result<Vec<TraceSummary>> {
        self.traces_search(&TraceQuery {
            limit,
            ..TraceQuery::default()
        })
        .await
    }
    /// Filtered trace browse (root spans + aggregates), newest first.
    async fn traces_search(&self, query: &TraceQuery) -> anyhow::Result<Vec<TraceSummary>>;
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
        body_contains: Option<&str>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>>;
    /// Raw read-only SQL against the engine (SELECT-shaped statements only —
    /// callers enforce the read-only guard). The in-memory store has no SQL
    /// surface and returns an error.
    async fn raw_sql(&self, query: &str) -> anyhow::Result<SqlResult>;
}
