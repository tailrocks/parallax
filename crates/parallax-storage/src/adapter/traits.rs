use super::*;

#[async_trait::async_trait]
pub trait IngestStore: Send + Sync {
    /// Ingest a traces batch: forward the raw OTLP bytes to GreptimeDB's native
    /// `/v1/otlp/v1/traces` endpoint (auto-creates `opentelemetry_traces`). The
    /// decoded `spans` are the tee — used by test adapters and ignored by the
    /// native production forward.
    async fn ingest_traces(
        &self,
        request: &parallax_proto::collector_trace::ExportTraceServiceRequest,
        raw: bytes::Bytes,
    ) -> StorageResult<()>;
    /// Ingest a logs batch: forward the raw OTLP bytes to the native
    /// `/v1/otlp/v1/logs` endpoint (auto-creates `opentelemetry_logs`).
    async fn ingest_logs(
        &self,
        request: &parallax_proto::collector_logs::ExportLogsServiceRequest,
        raw: bytes::Bytes,
    ) -> StorageResult<()>;
    /// Ingest a metrics batch: forward the raw OTLP bytes to the native
    /// `/v1/otlp/v1/metrics` endpoint (per-metric metric-engine tables), then
    /// persist the run-scoped subset of `points` into `run_metric_points`.
    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
        raw: bytes::Bytes,
    ) -> StorageResult<()>;
    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> StorageResult<()>;
}

#[async_trait::async_trait]
pub trait TraceStore: Send + Sync {
    /// Anchored read: every span of one trace, start-time ascending.
    async fn spans_by_trace(&self, trace_id: &str) -> StorageResult<Vec<SpanRow>>;
    /// Resolve summaries for span-link target trace ids.
    /// Returns at most one summary per id, preserving input order where possible.
    async fn traces_by_ids(&self, trace_ids: &[String]) -> StorageResult<Vec<TraceSummary>>;
    /// Run-scoped read: every span tagged with one `parallax.run.id`.
    /// `range` bounds the logs-table fallback scan (plan 085).
    async fn spans_by_run(
        &self,
        run_id: &str,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<SpanRow>>;
    /// Batched run-scoped span read: up to `limit_per_run` newest spans per
    /// run id (then returned start-time ascending within each run). Default
    /// loops `spans_by_run`; Greptime overrides with one windowed query.
    async fn spans_by_runs(
        &self,
        run_ids: &[String],
        limit_per_run: usize,
    ) -> StorageResult<HashMap<String, Vec<SpanRow>>> {
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
}

#[async_trait::async_trait]
pub trait LogStore: Send + Sync {
    /// Run-scoped read: every log tagged with one `parallax.run.id`.
    async fn logs_by_run(&self, run_id: &str, limit: usize) -> StorageResult<Vec<LogRow>>;
    /// Anchored read: every log of one trace, time ascending.
    async fn logs_by_trace(&self, trace_id: &str) -> StorageResult<Vec<LogRow>>;
}

#[async_trait::async_trait]
pub trait MetricStore: Send + Sync {
    /// Distinct metric names inside `range` (plan 085 windows extension scan).
    async fn metric_names(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<String>>;
    /// Discover groupable metric label/tag keys for one metric.
    async fn metric_labels(&self, name: &str) -> StorageResult<Vec<String>>;
    /// Distinct scalar values for one metric label inside an inclusive window.
    async fn metric_label_values(
        &self,
        name: &str,
        label: &str,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<String>>;
}

#[async_trait::async_trait]
pub trait ServiceAnalyticsStore: Send + Sync {
    /// Distinct service names across signals inside `range` (plan 085).
    async fn service_names(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<String>>;
    /// Whole-system overview counters for an inclusive time window.
    async fn overview_totals(&self, range: RangeInclusive<u128>) -> StorageResult<OverviewTotals>;
    /// Signal volume per bucket for overview trend charts.
    async fn signal_count_series(
        &self,
        kind: SignalKind,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>>;
    /// Service summary rows for the services index.
    async fn service_summaries(
        &self,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<ServiceSummary>>;
    /// Per-version activity windows for one service, ordered by first sighting.
    async fn release_windows(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<ReleaseWindow>>;
    /// Resource-identity catalog rows, one per service in the window.
    async fn service_catalog(
        &self,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<ServiceCatalogRow>>;
    /// Trace-derived RED series; works even when a service emits no metrics.
    async fn span_red_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<SpanRed>;
}

#[async_trait::async_trait]
pub trait MetricAnalyticsStore: Send + Sync {
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
    ) -> StorageResult<Vec<SeriesPoint>>;
    /// Approximate quantile series from a histogram metric's buckets.
    async fn histogram_quantile(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        q: f64,
    ) -> StorageResult<Vec<SeriesPoint>>;
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
    ) -> StorageResult<Vec<Vec<SeriesPoint>>> {
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
    ) -> StorageResult<Vec<MetricExemplarRow>>;
}

#[async_trait::async_trait]
pub trait RunStore: Send + Sync {
    /// Error events for a fingerprint within a time range, newest first.
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<ErrorEventRow>>;
    /// Error events for multiple fingerprints, newest first per fingerprint.
    /// Adapters override this with one physical query; the default preserves
    /// compatibility for capability implementations while migration completes.
    async fn error_events_by_fingerprints(
        &self,
        fingerprints: &[String],
        range: RangeInclusive<u128>,
        limit_per_fingerprint: usize,
    ) -> StorageResult<HashMap<String, Vec<ErrorEventRow>>> {
        let mut events = HashMap::with_capacity(fingerprints.len());
        for fingerprint in fingerprints {
            events.insert(
                fingerprint.clone(),
                self.error_events_by_fingerprint(fingerprint, range.clone(), limit_per_fingerprint)
                    .await?,
            );
        }
        Ok(events)
    }
    /// Distinct run ids inside `range`, most recent activity first (plan 085).
    async fn observed_runs(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<ObservedRun>>;
}

#[async_trait::async_trait]
pub trait TraceAnalyticsStore: Send + Sync {
    /// Recent traces (root spans + aggregates), newest first.
    async fn recent_traces(&self, limit: usize) -> StorageResult<Vec<TraceSummary>> {
        Ok(self
            .traces_search(&TraceQuery {
                limit,
                ..TraceQuery::default()
            })
            .await?
            .items)
    }
    /// Filtered trace browse (root spans + aggregates).
    async fn traces_search(&self, query: &TraceQuery) -> StorageResult<TraceList>;
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
    ) -> StorageResult<Vec<AttributeCompareRow>>;
    /// Discover scalar span/resource attribute keys in a window. Resource
    /// attributes are exposed as `resource.<key>` so the API key is unambiguous.
    async fn span_field_keys(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<FieldKey>>;
    /// Bounded stats for one discovered span/resource attribute key.
    async fn span_field_stats(
        &self,
        key: &str,
        range: RangeInclusive<u128>,
        service: Option<&str>,
    ) -> StorageResult<FieldStats>;
    /// Trace-path service edges derived from child SERVER spans paired with
    /// their parent span inside bounded traces from the requested window.
    async fn service_map(
        &self,
        range: RangeInclusive<u128>,
        max_traces: usize,
    ) -> StorageResult<Vec<ServiceEdge>>;
    /// Error events across a set of traces, newest first (run-anchored reads).
    async fn error_events_by_traces(
        &self,
        trace_ids: &[String],
        limit: usize,
    ) -> StorageResult<Vec<ErrorEventRow>>;
}

#[async_trait::async_trait]
pub trait LogAnalyticsStore: Send + Sync {
    /// Unified log browse: every filter optional, newest first.
    async fn logs_search(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        limit: usize,
    ) -> StorageResult<Vec<LogRow>>;
}

#[async_trait::async_trait]
pub trait RuntimeMetricStore: Send + Sync {
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
    ) -> StorageResult<Vec<(String, Vec<SeriesPoint>)>>;
    /// Runtime metric lanes across known runtime families for service/run scope.
    async fn runtime_snapshot(
        &self,
        service: Option<&str>,
        run_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<RuntimeMetricSeries>>;
    /// Histogram sample counts summed per bucket (request-rate numerator).
    async fn histogram_count_series(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>>;
}

#[async_trait::async_trait]
pub trait ErrorAnalyticsStore: Send + Sync {
    /// Error events per bucket for one service (overview error rate).
    async fn error_count_series(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>>;
}

#[async_trait::async_trait]
pub trait LogCountStore: Send + Sync {
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
    ) -> StorageResult<Vec<SeriesPoint>>;
}

#[async_trait::async_trait]
pub trait RawSqlStore: Send + Sync {
    /// Raw read-only SQL against the engine (SELECT-shaped statements only).
    /// The API enforces the user-facing guard; adapters repeat a defensive
    /// shape check before execution. The test store has no SQL surface
    /// and returns an error.
    async fn raw_sql(&self, query: &str) -> StorageResult<SqlResult>;
}

pub trait TelemetryStore:
    IngestStore
    + TraceStore
    + LogStore
    + MetricStore
    + ServiceAnalyticsStore
    + MetricAnalyticsStore
    + RunStore
    + TraceAnalyticsStore
    + LogAnalyticsStore
    + RuntimeMetricStore
    + ErrorAnalyticsStore
    + LogCountStore
    + RawSqlStore
{
}

impl<T> TelemetryStore for T where
    T: IngestStore
        + TraceStore
        + LogStore
        + MetricStore
        + ServiceAnalyticsStore
        + MetricAnalyticsStore
        + RunStore
        + TraceAnalyticsStore
        + LogAnalyticsStore
        + RuntimeMetricStore
        + ErrorAnalyticsStore
        + LogCountStore
        + RawSqlStore
{
}
