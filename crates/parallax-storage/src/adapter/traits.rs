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
    /// persist the run-scoped subset of `points` into `invocation_metric_points`.
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
    /// Run-scoped read: every span tagged with one `cli.invocation.id`.
    /// `range` bounds the logs-table fallback scan (plan 085).
    async fn spans_by_invocation(
        &self,
        invocation_id: &str,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<SpanRow>>;
    /// Batched run-scoped span read: up to `limit_per_invocation` newest spans per
    /// run id (then returned start-time ascending within each run). Default
    /// loops `spans_by_invocation`; Greptime overrides with one windowed query.
    async fn spans_by_invocations(
        &self,
        invocation_ids: &[String],
        limit_per_invocation: usize,
    ) -> StorageResult<HashMap<String, Vec<SpanRow>>> {
        let mut out = HashMap::with_capacity(invocation_ids.len());
        let range = 0..=u128::MAX;
        for invocation_id in invocation_ids {
            out.insert(
                invocation_id.clone(),
                self.spans_by_invocation(invocation_id, limit_per_invocation, range.clone())
                    .await?,
            );
        }
        Ok(out)
    }
}

#[async_trait::async_trait]
pub trait LogStore: Send + Sync {
    /// Run-scoped read: every log tagged with one `cli.invocation.id`.
    async fn logs_by_invocation(
        &self,
        invocation_id: &str,
        limit: usize,
    ) -> StorageResult<Vec<LogRow>>;
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
    /// Window-scoped explorer catalog (plan 168): canonical names with kind,
    /// unit, emitting services, last datapoint, and finite-sample counts.
    /// Bounded and batched — one logical scan, never per-metric fan-out.
    /// `q` is a case-insensitive substring filter on the canonical name.
    async fn metric_catalog(
        &self,
        range: RangeInclusive<u128>,
        q: Option<&str>,
        kind: Option<MetricKind>,
        limit: usize,
    ) -> StorageResult<Vec<MetricCatalogEntry>>;
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
    /// `invocation_id` scopes to points whose resource carried `cli.invocation.id`
    /// (run-anchored cross-analytics: CPU/memory beside a run's traces).
    /// `attribute_filters` narrow on metric label values (plan 168 where).
    #[expect(clippy::too_many_arguments, reason = "stable metric read contract")]
    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
        invocation_id: Option<&str>,
        attribute_filters: &[AttributeFilter],
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> StorageResult<Vec<SeriesPoint>>;
    /// Approximate quantile series from a histogram metric's buckets.
    #[expect(clippy::too_many_arguments, reason = "stable metric read contract")]
    async fn histogram_quantile(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        range: RangeInclusive<u128>,
        step_nanos: u128,
        q: f64,
    ) -> StorageResult<Vec<SeriesPoint>>;
    /// Multiple histogram quantiles from one logical scan. The default loops
    /// [`Self::histogram_quantile`]; Greptime overrides it with one
    /// multi-quantile SQL query. Return order matches `quantiles`.
    #[expect(clippy::too_many_arguments, reason = "stable metric read contract")]
    async fn histogram_quantiles(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        range: RangeInclusive<u128>,
        step_nanos: u128,
        quantiles: &[f64],
    ) -> StorageResult<Vec<Vec<SeriesPoint>>> {
        let mut out = Vec::with_capacity(quantiles.len());
        for q in quantiles {
            out.push(
                self.histogram_quantile(
                    name,
                    service,
                    attribute_filters,
                    range.clone(),
                    step_nanos,
                    *q,
                )
                .await?,
            );
        }
        Ok(out)
    }
    /// Histogram average per bucket (Δ`_sum`/Δ`_count` over cumulative
    /// exports, reset-clamped; zero-growth buckets skipped).
    async fn histogram_avg(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>>;
    /// Bounded invocation-scoped metric family summaries (plan 105): one row
    /// per canonical native-family identity, name ascending, finite samples
    /// only, at most `limit` rows. Never scans the native catalog.
    async fn invocation_metric_summaries(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<InvocationMetricSummary>>;
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
pub trait InvocationStore: Send + Sync {
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
    /// Bounded facet dimensions (`INVOCATION_FACET_DIMENSIONS`) with
    /// per-value DISTINCT-invocation counts inside `range` (plan 164
    /// facet sidebar). Empty-valued rows are not counted.
    async fn invocation_facets(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<Facet>>;
    /// Distinct invocation ids inside `range`, most recent activity first.
    async fn observed_invocations(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<ObservedInvocation>>;
    /// Sessions inside one invocation from `session.start`/`session.end` log
    /// events, oldest first. An open session has `end_nanos = None`.
    async fn sessions_by_invocation(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<InvocationSession>>;
    /// Screen visits paired by `ui.screen.visit.id` for an invocation and/or
    /// session scope, navigation order ascending.
    async fn screen_visits(
        &self,
        invocation_id: Option<&str>,
        session_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<ScreenVisit>>;
    /// `ui.action` root spans for one invocation, newest first.
    async fn ui_actions(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<UiAction>>;
    /// `background.cycle` spans grouped by `background.cycle.name`.
    async fn background_cycles(
        &self,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<BackgroundCycleSummary>>;
    /// Detached jobs: spans carrying `job.id`, grouped into producer time and
    /// consumer attempts, newest first.
    async fn jobs(
        &self,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<JobSummary>>;
    /// Agent conversations: spans carrying `gen_ai.conversation.id`.
    async fn conversations(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<ConversationSummary>>;
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
    /// Duration percentiles of the representative spans matching `query`
    /// (plan 164 preset chips). The query's own duration bounds are ignored
    /// so `> p50` / `> p95` presets never feed back into themselves.
    async fn trace_duration_stats(&self, query: &TraceQuery) -> StorageResult<DurationStats>;
    /// Bounded facet dimensions (`TRACE_FACET_DIMENSIONS`) with per-value
    /// DISTINCT-trace counts inside the query's window/participation
    /// (plan 164 facet sidebar). Empty-valued rows are not counted.
    async fn trace_facets(&self, query: &TraceQuery) -> StorageResult<Vec<Facet>>;
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
    /// Uninstrumented dependency edges (plan 166): CLIENT/PRODUCER spans with
    /// no same-trace SERVER/CONSUMER child in another instrumented service,
    /// grouped by (service, dependency identity). Identity ladder over generic
    /// attributes only: `db.system.name`/`db.system` → database,
    /// `messaging.system` → queue, else `server.address` → external HTTP.
    async fn external_dependency_edges(
        &self,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<ExternalDependencyEdge>>;
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
        attribute_filters: &[AttributeFilter],
        limit: usize,
    ) -> StorageResult<Vec<LogRow>>;
    /// Bounded facet dimensions (`LOG_FACET_DIMENSIONS`) with per-value log
    /// counts under the same filters as `logs_search` (plan 164 sidebar).
    #[expect(clippy::too_many_arguments, reason = "mirrors logs_search filters")]
    async fn log_facets(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        attribute_filters: &[AttributeFilter],
    ) -> StorageResult<Vec<Facet>>;
}

#[async_trait::async_trait]
pub trait RuntimeMetricStore: Send + Sync {
    /// Aggregated series split by one attribute key's value (spec §8
    /// `metricSeries(groupBy:)`); rows missing the key group under "(none)".
    #[expect(clippy::too_many_arguments, reason = "stable metric read contract")]
    async fn metric_series_grouped(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        group_by: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> StorageResult<Vec<(String, Vec<SeriesPoint>)>>;
    /// Runtime metric lanes across known runtime families for service/run scope.
    async fn runtime_snapshot(
        &self,
        service: Option<&str>,
        invocation_id: Option<&str>,
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
        attribute_filters: &[AttributeFilter],
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
    + InvocationStore
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
        + InvocationStore
        + TraceAnalyticsStore
        + LogAnalyticsStore
        + RuntimeMetricStore
        + ErrorAnalyticsStore
        + LogCountStore
        + RawSqlStore
{
}
