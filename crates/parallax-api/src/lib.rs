//! Parallax GraphQL API — the V1 surface from the implementation spec §8,
//! served by **Juniper** (operator instruction, 2026-06-12: the library he
//! uses in his own services). Every client (CLI, UI, agents) goes through
//! this schema; none touch storage directly.
//!
//! Juniper notes (per the spec's dependency table): GraphQL `Int` is i32 —
//! counts saturate; nanosecond timestamps cross as strings; field names are
//! auto-camelCased; cost limits are resolver-level caps in V1.

use juniper::{EmptySubscription, FieldError, FieldResult, RootNode, graphql_object};
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model;
use parallax_storage::model::{MetricAgg, SeriesPoint};
use std::sync::Arc;

/// Request context: the storage adapters.
#[derive(Clone)]
pub struct ApiContext {
    pub store: Arc<dyn TelemetryStore>,
    pub metadata: Arc<MetadataStore>,
}

impl juniper::Context for ApiContext {}

fn field_err(e: impl std::fmt::Display) -> FieldError {
    FieldError::from(e.to_string())
}

fn nanos_string(nanos: u128) -> String {
    nanos.to_string()
}

fn saturate_i32(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Resolver-level row cap (the spec's Juniper note: cost limits are
/// resolver-level in V1; query-cost middleware is M5 hardening).
const MAX_ROWS: usize = 500;

fn clamp_limit(limit: Option<i32>, default: usize) -> usize {
    limit
        .map_or(default, |l| usize::try_from(l.max(0)).unwrap_or(default))
        .min(MAX_ROWS)
}

pub struct Issue(model::Issue);

#[graphql_object(context = ApiContext)]
impl Issue {
    fn fingerprint(&self) -> &str {
        &self.0.fingerprint
    }
    fn title(&self) -> &str {
        &self.0.title
    }
    fn error_type(&self) -> &str {
        &self.0.error_type
    }
    fn culprit(&self) -> Option<&str> {
        self.0.culprit.as_deref()
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn status(&self) -> &str {
        &self.0.status
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.0.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn event_count(&self) -> i32 {
        saturate_i32(self.0.event_count)
    }
    fn last_trace_id(&self) -> Option<&str> {
        self.0.last_trace_id.as_deref()
    }
    /// Bounded top-tag-values cache as JSON: `{key: {value: count}}`.
    fn tags(&self) -> &str {
        &self.0.tags
    }

    /// The last-24h occurrence sparkline (hourly buckets), oldest first.
    async fn trend(&self, context: &ApiContext) -> FieldResult<Vec<TrendPoint>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(field_err)?
            .as_nanos();
        let since = now.saturating_sub(24 * 3_600_000_000_000);
        let points = context
            .metadata
            .issue_trend(&self.0.fingerprint, since, 3600)
            .await
            .map_err(field_err)?;
        Ok(points.into_iter().map(TrendPoint).collect())
    }

    /// The most recent stored occurrence.
    async fn latest_event(&self, context: &ApiContext) -> FieldResult<Option<ErrorEvent>> {
        let events = context
            .store
            .error_events_by_fingerprint(&self.0.fingerprint, 0..=u128::MAX, 1)
            .await
            .map_err(field_err)?;
        Ok(events.into_iter().next().map(ErrorEvent))
    }

    /// Recent occurrences of this issue, newest first, optionally
    /// range-bounded (`fromNanos`/`toNanos`).
    async fn events(
        &self,
        context: &ApiContext,
        limit: Option<i32>,
        from_nanos: Option<String>,
        to_nanos: Option<String>,
    ) -> FieldResult<Vec<ErrorEvent>> {
        let from = match from_nanos {
            Some(s) => s.parse().map_err(|_| field_err("invalid fromNanos"))?,
            None => 0,
        };
        let to = match to_nanos {
            Some(s) => s.parse().map_err(|_| field_err("invalid toNanos"))?,
            None => u128::MAX,
        };
        let events = context
            .store
            .error_events_by_fingerprint(&self.0.fingerprint, from..=to, clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(events.into_iter().map(ErrorEvent).collect())
    }
}

/// Page of issues plus the (scan-capped) total for pagination.
pub struct IssueList {
    items: Vec<model::Issue>,
    total: usize,
}

#[graphql_object(context = ApiContext)]
impl IssueList {
    fn items(&self) -> Vec<Issue> {
        self.items.iter().cloned().map(Issue).collect()
    }
    /// Matching issues before paging — exact up to the 1000-row scan window.
    fn total(&self) -> i32 {
        i32::try_from(self.total).unwrap_or(i32::MAX)
    }
}

/// How `issues` lists are ordered. TREND = last-24h occurrence sum.
#[derive(juniper::GraphQLEnum, Clone, Copy)]
pub enum IssueSort {
    LastSeen,
    FirstSeen,
    Events,
    Trend,
}

impl IssueSort {
    fn key(self) -> model::IssueSortKey {
        match self {
            Self::LastSeen => model::IssueSortKey::LastSeen,
            Self::FirstSeen => model::IssueSortKey::FirstSeen,
            Self::Events => model::IssueSortKey::Events,
            Self::Trend => model::IssueSortKey::Trend,
        }
    }
}

pub struct ErrorEvent(model::ErrorEventRow);

#[graphql_object(context = ApiContext)]
impl ErrorEvent {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn fingerprint(&self) -> &str {
        &self.0.fingerprint
    }
    fn error_type(&self) -> &str {
        &self.0.error_type
    }
    fn message(&self) -> &str {
        &self.0.message
    }
    fn stacktrace(&self) -> Option<&str> {
        self.0.stacktrace.as_deref()
    }
    fn source(&self) -> String {
        serde_json::to_string(&self.0.source)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string()
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
    }
}

pub struct Span(model::SpanRow);

#[graphql_object(context = ApiContext)]
impl Span {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn parent_span_id(&self) -> Option<&str> {
        self.0.parent_span_id.as_deref()
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn kind(&self) -> &str {
        &self.0.kind
    }
    fn status_code(&self) -> &str {
        &self.0.status_code
    }
    fn status_message(&self) -> &str {
        &self.0.status_message
    }
    fn duration_ns(&self) -> String {
        self.0.duration_ns.to_string()
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

pub struct LogRecord(model::LogRow);

#[graphql_object(context = ApiContext)]
impl LogRecord {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
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

pub struct Trace {
    trace_id: String,
    spans: Vec<model::SpanRow>,
}

#[graphql_object(context = ApiContext)]
impl Trace {
    fn trace_id(&self) -> &str {
        &self.trace_id
    }
    fn spans(&self) -> Vec<Span> {
        self.spans.iter().cloned().map(Span).collect()
    }
}

pub struct SqlResultOut(parallax_storage::adapter::SqlResult);

#[graphql_object(context = ApiContext)]
impl SqlResultOut {
    fn columns(&self) -> &[String] {
        &self.0.columns
    }
    /// Each row as a JSON array string (heterogeneous cell types).
    fn rows(&self) -> Vec<String> {
        self.0
            .rows
            .iter()
            .map(|row| serde_json::Value::Array(row.clone()).to_string())
            .collect()
    }
    fn row_count(&self) -> i32 {
        i32::try_from(self.0.rows.len()).unwrap_or(i32::MAX)
    }
}

pub struct ObservedRun(parallax_storage::adapter::ObservedRun);

#[graphql_object(context = ApiContext)]
impl ObservedRun {
    fn run_id(&self) -> &str {
        &self.0.run_id
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn first_nanos(&self) -> String {
        nanos_string(self.0.first_nanos)
    }
    fn last_nanos(&self) -> String {
        nanos_string(self.0.last_nanos)
    }
    fn span_count(&self) -> i32 {
        i32::try_from(self.0.span_count).unwrap_or(i32::MAX)
    }
    fn log_count(&self) -> i32 {
        i32::try_from(self.0.log_count).unwrap_or(i32::MAX)
    }
}

pub struct TraceSummary(parallax_storage::adapter::TraceSummary);

#[graphql_object(context = ApiContext)]
impl TraceSummary {
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn root_name(&self) -> &str {
        &self.0.root_name
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn start_nanos(&self) -> String {
        nanos_string(self.0.start_nanos)
    }
    fn duration_ns(&self) -> String {
        nanos_string(self.0.duration_ns)
    }
    fn span_count(&self) -> i32 {
        i32::try_from(self.0.span_count).unwrap_or(i32::MAX)
    }
    fn has_error(&self) -> bool {
        self.0.has_error
    }
}

pub struct Run {
    record: model::RunRecord,
    /// Trace ids + error events of this run, fetched once however many of
    /// the derived fields a query selects.
    stats: tokio::sync::OnceCell<RunStats>,
}

struct RunStats {
    trace_ids: Vec<String>,
    events: Vec<model::ErrorEventRow>,
}

impl Run {
    fn new(record: model::RunRecord) -> Self {
        Self {
            record,
            stats: tokio::sync::OnceCell::new(),
        }
    }

    async fn stats(&self, context: &ApiContext) -> FieldResult<&RunStats> {
        self.stats
            .get_or_try_init(|| async {
                let spans = context
                    .store
                    .spans_by_run(&self.record.run_id, MAX_ROWS)
                    .await
                    .map_err(field_err)?;
                let mut trace_ids: Vec<String> = Vec::new();
                for span in &spans {
                    if !trace_ids.contains(&span.trace_id) {
                        trace_ids.push(span.trace_id.clone());
                    }
                }
                let events = context
                    .store
                    .error_events_by_traces(&trace_ids, MAX_ROWS)
                    .await
                    .map_err(field_err)?;
                Ok(RunStats { trace_ids, events })
            })
            .await
    }
}

#[graphql_object(context = ApiContext)]
impl Run {
    fn run_id(&self) -> &str {
        &self.record.run_id
    }
    fn command(&self) -> Option<&str> {
        self.record.command.as_deref()
    }
    fn started_at_nanos(&self) -> String {
        nanos_string(self.record.started_at_nanos)
    }
    fn ended_at_nanos(&self) -> Option<String> {
        self.record.ended_at_nanos.map(nanos_string)
    }
    fn exit_code(&self) -> Option<i32> {
        self.record.exit_code
    }
    /// running | finished | external (auto-registered from telemetry).
    fn status(&self) -> &str {
        &self.record.status
    }
    /// Error events derived inside this run's traces.
    async fn error_count(&self, context: &ApiContext) -> FieldResult<i32> {
        Ok(saturate_i32(self.stats(context).await?.events.len() as u64))
    }
    /// Distinct traces this run produced.
    async fn trace_count(&self, context: &ApiContext) -> FieldResult<i32> {
        Ok(saturate_i32(
            self.stats(context).await?.trace_ids.len() as u64
        ))
    }
    /// Grouped issues whose events fell inside this run's traces.
    async fn issues(&self, context: &ApiContext) -> FieldResult<Vec<Issue>> {
        let stats = self.stats(context).await?;
        let mut fingerprints: Vec<String> = Vec::new();
        for event in &stats.events {
            if !fingerprints.contains(&event.fingerprint) {
                fingerprints.push(event.fingerprint.clone());
            }
        }
        let issues = context
            .metadata
            .issues_by_fingerprints(&fingerprints)
            .await
            .map_err(field_err)?;
        Ok(issues.into_iter().map(Issue).collect())
    }
}

pub struct Point(SeriesPoint);

#[graphql_object(context = ApiContext)]
impl Point {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn value(&self) -> f64 {
        self.0.value
    }
}

/// One series of a (possibly grouped) metric query; `groupValue` is null for
/// ungrouped queries.
pub struct Series {
    group_value: Option<String>,
    points: Vec<SeriesPoint>,
}

#[graphql_object(context = ApiContext)]
impl Series {
    fn group_value(&self) -> Option<&str> {
        self.group_value.as_deref()
    }
    fn points(&self) -> Vec<Point> {
        self.points.iter().copied().map(Point).collect()
    }
}

/// The predefined per-service overview (spec §8): well-known metric names,
/// graceful absence — a missing instrument yields an empty series.
pub struct ServiceOverview {
    service: String,
    from: u128,
    to: u128,
    step: u128,
}

impl ServiceOverview {
    async fn first_nonempty_points(
        &self,
        context: &ApiContext,
        candidates: &[&str],
    ) -> FieldResult<Vec<SeriesPoint>> {
        for name in candidates {
            let series = context
                .store
                .metric_series(
                    name,
                    Some(&self.service),
                    self.from..=self.to,
                    self.step,
                    MetricAgg::Avg,
                )
                .await
                .map_err(field_err)?;
            if !series.is_empty() {
                return Ok(series);
            }
        }
        Ok(Vec::new())
    }

    async fn duration_quantile(
        &self,
        context: &ApiContext,
        q: f64,
    ) -> FieldResult<Vec<SeriesPoint>> {
        for name in REQUEST_DURATION_METRICS {
            let series = context
                .store
                .histogram_quantile(name, Some(&self.service), self.from..=self.to, self.step, q)
                .await
                .map_err(field_err)?;
            if !series.is_empty() {
                return Ok(series);
            }
        }
        Ok(Vec::new())
    }
}

/// Well-known request-duration histograms, preferred order (OTel semconv).
const REQUEST_DURATION_METRICS: &[&str] = &["http.server.request.duration", "rpc.server.duration"];

#[graphql_object(context = ApiContext)]
impl ServiceOverview {
    /// Process/system CPU, averaged per step.
    async fn cpu(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .first_nonempty_points(
                context,
                &[
                    "process.cpu.utilization",
                    "process.cpu.usage",
                    "system.cpu.utilization",
                ],
            )
            .await?
            .into_iter()
            .map(Point)
            .collect())
    }

    /// Process memory, averaged per step.
    async fn memory(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .first_nonempty_points(
                context,
                &[
                    "process.memory.usage",
                    "process.memory.virtual",
                    "system.memory.usage",
                ],
            )
            .await?
            .into_iter()
            .map(Point)
            .collect())
    }

    /// Requests per second from the request-duration histogram's sample
    /// counts.
    async fn request_rate(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        let step_secs = (self.step / 1_000_000_000).max(1) as f64;
        for name in REQUEST_DURATION_METRICS {
            let counts = context
                .store
                .histogram_count_series(name, Some(&self.service), self.from..=self.to, self.step)
                .await
                .map_err(field_err)?;
            if !counts.is_empty() {
                return Ok(counts
                    .into_iter()
                    .map(|p| {
                        Point(SeriesPoint {
                            ts_nanos: p.ts_nanos,
                            value: p.value / step_secs,
                        })
                    })
                    .collect());
            }
        }
        Ok(Vec::new())
    }

    async fn latency_p50(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .duration_quantile(context, 0.50)
            .await?
            .into_iter()
            .map(Point)
            .collect())
    }
    async fn latency_p95(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .duration_quantile(context, 0.95)
            .await?
            .into_iter()
            .map(Point)
            .collect())
    }
    async fn latency_p99(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .duration_quantile(context, 0.99)
            .await?
            .into_iter()
            .map(Point)
            .collect())
    }

    /// Derived error events per second for this service.
    async fn error_rate(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        let step_secs = (self.step / 1_000_000_000).max(1) as f64;
        let counts = context
            .store
            .error_count_series(&self.service, self.from..=self.to, self.step)
            .await
            .map_err(field_err)?;
        Ok(counts
            .into_iter()
            .map(|p| {
                Point(SeriesPoint {
                    ts_nanos: p.ts_nanos,
                    value: p.value / step_secs,
                })
            })
            .collect())
    }
}

pub struct TrendPoint(model::TrendPoint);

#[graphql_object(context = ApiContext)]
impl TrendPoint {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn count(&self) -> i32 {
        i32::try_from(self.0.count).unwrap_or(i32::MAX)
    }
}

pub struct Dashboard(model::Dashboard);

#[graphql_object(context = ApiContext)]
impl Dashboard {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    /// Widget layout as a JSON string:
    /// [{metric, agg, chart, title, quantile?}].
    fn layout(&self) -> &str {
        &self.0.layout
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

fn parse_range(from_nanos: &str, to_nanos: &str) -> juniper::FieldResult<(u128, u128)> {
    let from: u128 = from_nanos
        .parse()
        .map_err(|_| field_err("invalid fromNanos"))?;
    let to: u128 = to_nanos.parse().map_err(|_| field_err("invalid toNanos"))?;
    if from > to {
        return Err(field_err("fromNanos must be <= toNanos"));
    }
    Ok((from, to))
}

fn step_nanos(step_seconds: Option<i32>) -> u128 {
    u128::try_from(step_seconds.unwrap_or(60).max(1)).unwrap_or(60) * 1_000_000_000
}

pub struct BundleOut {
    json: String,
    markdown: String,
    canonical_hash: String,
}

#[graphql_object(context = ApiContext)]
impl BundleOut {
    /// The bundle as canonical JSON.
    fn json(&self) -> &str {
        &self.json
    }
    /// The agent-facing Markdown projection.
    fn markdown(&self) -> &str {
        &self.markdown
    }
    fn canonical_hash(&self) -> &str {
        &self.canonical_hash
    }
}

pub struct Query;

#[graphql_object(context = ApiContext)]
impl Query {
    fn health() -> &'static str {
        "ok"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Grouped errors: filtered, sorted, paged (spec §8 `issues`). The
    /// `query` argument substring-matches title, error type, and fingerprint;
    /// `fromNanos`/`toNanos` window on last-seen; `tagKey`+`tagValue` filter
    /// on the cached tags.
    #[allow(clippy::too_many_arguments)]
    async fn issues(
        context: &ApiContext,
        service: Option<String>,
        status: Option<String>,
        query: Option<String>,
        from_nanos: Option<String>,
        to_nanos: Option<String>,
        tag_key: Option<String>,
        tag_value: Option<String>,
        sort: Option<IssueSort>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> FieldResult<IssueList> {
        if let Some(status) = status.as_deref()
            && !matches!(status, "open" | "resolved")
        {
            return Err(field_err("status must be open or resolved"));
        }
        let filter = model::IssueQuery {
            service,
            status,
            query,
            from_nanos: match from_nanos {
                Some(s) => Some(s.parse().map_err(|_| field_err("invalid fromNanos"))?),
                None => None,
            },
            to_nanos: match to_nanos {
                Some(s) => Some(s.parse().map_err(|_| field_err("invalid toNanos"))?),
                None => None,
            },
            tag_key,
            tag_value,
        };
        let offset = usize::try_from(offset.unwrap_or(0).max(0)).unwrap_or(0);
        let (items, total) = context
            .metadata
            .issues_filtered(
                &filter,
                sort.unwrap_or(IssueSort::LastSeen).key(),
                clamp_limit(limit, 50),
                offset,
            )
            .await
            .map_err(field_err)?;
        Ok(IssueList { items, total })
    }

    async fn issue(context: &ApiContext, fingerprint: String) -> FieldResult<Option<Issue>> {
        Ok(context
            .metadata
            .issue(&fingerprint)
            .await
            .map_err(field_err)?
            .map(Issue))
    }

    /// Occurrence counts per bucket for one issue's sparkline, oldest
    /// first. Defaults: the last 24 hours in one-hour buckets.
    async fn issue_trend(
        context: &ApiContext,
        fingerprint: String,
        hours: Option<i32>,
        step_seconds: Option<i32>,
    ) -> FieldResult<Vec<TrendPoint>> {
        let hours = u64::try_from(hours.unwrap_or(24).clamp(1, 24 * 30)).unwrap_or(24);
        let step = u32::try_from(step_seconds.unwrap_or(3600).clamp(60, 86_400)).unwrap_or(3600);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(field_err)?
            .as_nanos();
        let since = now.saturating_sub(u128::from(hours) * 3_600_000_000_000);
        let points = context
            .metadata
            .issue_trend(&fingerprint, since, step)
            .await
            .map_err(field_err)?;
        Ok(points.into_iter().map(TrendPoint).collect())
    }

    /// Every span of one trace, start-time ascending (cross-service).
    async fn trace(context: &ApiContext, trace_id: String) -> FieldResult<Option<Trace>> {
        let spans = context
            .store
            .spans_by_trace(&trace_id)
            .await
            .map_err(field_err)?;
        if spans.is_empty() {
            return Ok(None);
        }
        Ok(Some(Trace { trace_id, spans }))
    }

    /// Logs correlated to one trace, time ascending.
    async fn logs_by_trace(context: &ApiContext, trace_id: String) -> FieldResult<Vec<LogRecord>> {
        let logs = context
            .store
            .logs_by_trace(&trace_id)
            .await
            .map_err(field_err)?;
        Ok(logs.into_iter().map(LogRecord).collect())
    }

    /// Traces produced by one run, summarized (root span + aggregates),
    /// newest first. Open one via `trace(traceId:)`.
    async fn traces_by_run(
        context: &ApiContext,
        run_id: String,
        limit: Option<i32>,
    ) -> FieldResult<Vec<TraceSummary>> {
        let spans = context
            .store
            .spans_by_run(&run_id, MAX_ROWS)
            .await
            .map_err(field_err)?;
        let mut by_trace: Vec<(String, Vec<model::SpanRow>)> = Vec::new();
        for span in spans {
            match by_trace.iter_mut().find(|(t, _)| *t == span.trace_id) {
                Some((_, group)) => group.push(span),
                None => by_trace.push((span.trace_id.clone(), vec![span])),
            }
        }
        let mut summaries: Vec<parallax_storage::adapter::TraceSummary> = by_trace
            .into_iter()
            .map(|(trace_id, spans)| {
                let root = spans
                    .iter()
                    .find(|s| s.parent_span_id.as_deref().is_none_or(str::is_empty))
                    .unwrap_or(&spans[0]);
                let start = spans.iter().map(|s| s.ts_nanos).min().unwrap_or(0);
                let end = spans
                    .iter()
                    .map(|s| s.ts_nanos + s.duration_ns)
                    .max()
                    .unwrap_or(start);
                parallax_storage::adapter::TraceSummary {
                    trace_id,
                    root_name: root.name.clone(),
                    service: root.service.clone(),
                    start_nanos: start,
                    duration_ns: end.saturating_sub(start),
                    span_count: spans.len() as u64,
                    has_error: spans.iter().any(|s| s.status_code == "STATUS_CODE_ERROR"),
                }
            })
            .collect();
        summaries.sort_by_key(|s| std::cmp::Reverse(s.start_nanos));
        summaries.truncate(clamp_limit(limit, 200));
        Ok(summaries.into_iter().map(TraceSummary).collect())
    }

    /// Logs produced by one run.
    async fn logs_by_run(
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

    /// Unified log browse (spec §8 `logs`): every filter optional, newest
    /// first. `query` substring-matches the body; trace/run scoping
    /// composes with the other filters.
    #[allow(clippy::too_many_arguments)]
    async fn logs(
        context: &ApiContext,
        trace_id: Option<String>,
        run_id: Option<String>,
        service: Option<String>,
        from_nanos: Option<String>,
        to_nanos: Option<String>,
        severity_min: Option<i32>,
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
                && query
                    .as_deref()
                    .is_none_or(|needle| l.body.contains(needle))
        });
        logs.sort_by_key(|l| std::cmp::Reverse(l.ts_nanos));
        logs.truncate(limit);
        Ok(logs.into_iter().map(LogRecord).collect())
    }

    /// Raw read-only SQL against the telemetry engine (GreptimeDB) — the
    /// engine's full query power over logs, traces, and metrics tables.
    /// SELECT-shaped single statements only.
    async fn sql(context: &ApiContext, query: String) -> FieldResult<SqlResultOut> {
        let trimmed = query.trim();
        let lowered = trimmed.to_ascii_lowercase();
        let read_only = [
            "select", "with", "show", "describe", "desc", "explain", "tql",
        ]
        .iter()
        .any(|prefix| lowered.starts_with(prefix));
        if !read_only {
            return Err(field_err(
                "only read-only statements are allowed (SELECT/WITH/SHOW/DESCRIBE/EXPLAIN/TQL)",
            ));
        }
        if trimmed.trim_end_matches(';').contains(';') {
            return Err(field_err("multiple statements are not allowed"));
        }
        let result = context
            .store
            .raw_sql(trimmed.trim_end_matches(';'))
            .await
            .map_err(field_err)?;
        Ok(SqlResultOut(result))
    }

    /// Log counts per time bucket under the same filters as `logs` — the
    /// Discover-style histogram above the log table.
    async fn log_count_series(
        context: &ApiContext,
        from_nanos: String,
        to_nanos: String,
        service: Option<String>,
        severity_min: Option<i32>,
        query: Option<String>,
        step_seconds: Option<i32>,
    ) -> FieldResult<Vec<Point>> {
        let from: u128 = from_nanos
            .parse()
            .map_err(|_| field_err("invalid fromNanos"))?;
        let to: u128 = to_nanos.parse().map_err(|_| field_err("invalid toNanos"))?;
        let step = u128::try_from(step_seconds.unwrap_or(60).clamp(1, 86_400)).unwrap_or(60)
            * 1_000_000_000;
        let series = context
            .store
            .log_count_series(
                service.as_deref(),
                from..=to,
                severity_min,
                query.as_deref(),
                step,
            )
            .await
            .map_err(field_err)?;
        Ok(series.into_iter().map(Point).collect())
    }

    /// One run by id (wrapper-registered or auto-registered external).
    async fn run(context: &ApiContext, run_id: String) -> FieldResult<Option<Run>> {
        Ok(context
            .metadata
            .run(&run_id)
            .await
            .map_err(field_err)?
            .map(Run::new))
    }

    /// One saved dashboard by id.
    async fn dashboard(context: &ApiContext, id: String) -> FieldResult<Option<Dashboard>> {
        Ok(context
            .metadata
            .dashboard(&id)
            .await
            .map_err(field_err)?
            .map(Dashboard))
    }

    /// The predefined service overview (spec §8): CPU, memory, request rate,
    /// latency percentiles, error rate from well-known metric names, with
    /// graceful absence.
    async fn service_overview(
        context: &ApiContext,
        service: String,
        from_nanos: String,
        to_nanos: String,
        step_seconds: Option<i32>,
    ) -> FieldResult<ServiceOverview> {
        let _ = context;
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        Ok(ServiceOverview {
            service,
            from,
            to,
            step: step_nanos(step_seconds),
        })
    }

    /// Run ids observed in telemetry (any tool exporting `parallax.run_id`
    /// — e.g. jackin'), newest activity first. Independent of wrapper
    /// registration: this is how external runs appear in the UI.
    async fn observed_runs(
        context: &ApiContext,
        limit: Option<i32>,
    ) -> FieldResult<Vec<ObservedRun>> {
        let runs = context
            .store
            .observed_runs(clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(runs.into_iter().map(ObservedRun).collect())
    }

    /// Recent traces (root span + aggregates), newest first.
    async fn recent_traces(
        context: &ApiContext,
        limit: Option<i32>,
    ) -> FieldResult<Vec<TraceSummary>> {
        let traces = context
            .store
            .recent_traces(clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(traces.into_iter().map(TraceSummary).collect())
    }

    /// Filtered trace browse (UI Traces page / `parallax traces`): every
    /// filter optional; filters hit the root span except `errorOnly`,
    /// which looks at the whole trace.
    #[allow(clippy::too_many_arguments)]
    async fn traces(
        context: &ApiContext,
        service: Option<String>,
        from_nanos: Option<String>,
        to_nanos: Option<String>,
        min_duration_ms: Option<f64>,
        error_only: Option<bool>,
        query: Option<String>,
        limit: Option<i32>,
    ) -> FieldResult<Vec<TraceSummary>> {
        let parse = |bound: Option<String>, label: &str| -> FieldResult<Option<u128>> {
            bound
                .map(|s| {
                    s.parse::<u128>()
                        .map_err(|_| field_err(format!("invalid {label}")))
                })
                .transpose()
        };
        let trace_query = parallax_storage::adapter::TraceQuery {
            service: service.filter(|s| !s.is_empty()),
            from_nanos: parse(from_nanos, "fromNanos")?,
            to_nanos: parse(to_nanos, "toNanos")?,
            min_duration_ns: min_duration_ms
                .filter(|ms| *ms > 0.0)
                .map(|ms| (ms * 1e6) as u128),
            error_only: error_only.unwrap_or(false),
            name_contains: query.filter(|q| !q.trim().is_empty()),
            limit: clamp_limit(limit, 50),
        };
        let traces = context
            .store
            .traces_search(&trace_query)
            .await
            .map_err(field_err)?;
        Ok(traces.into_iter().map(TraceSummary).collect())
    }

    /// The bounded, redacted, hypothesis-ranked evidence bundle — the agent
    /// handoff artifact. Exactly one anchor: `fingerprint` (issue), `runId`,
    /// or `traceId` (spec §8). Null when the anchor does not exist.
    async fn bundle(
        context: &ApiContext,
        fingerprint: Option<String>,
        run_id: Option<String>,
        trace_id: Option<String>,
        max_tokens: Option<i32>,
    ) -> FieldResult<Option<BundleOut>> {
        use parallax_core::bundle::{BundleAnchor, BundleInputs};
        let max_tokens = usize::try_from(max_tokens.unwrap_or(10_000).max(500)).unwrap_or(10_000);
        let anchors = [fingerprint.is_some(), run_id.is_some(), trace_id.is_some()];
        if anchors.iter().filter(|present| **present).count() != 1 {
            return Err(field_err(
                "bundle takes exactly one anchor: fingerprint, runId, or traceId",
            ));
        }

        let inputs = if let Some(fingerprint) = fingerprint {
            let Some(issue) = context
                .metadata
                .issue(&fingerprint)
                .await
                .map_err(field_err)?
            else {
                return Ok(None);
            };
            let events = context
                .store
                .error_events_by_fingerprint(&fingerprint, 0..=u128::MAX, 5)
                .await
                .map_err(field_err)?;
            let (trace_spans, trace_logs) = match issue.last_trace_id.as_deref() {
                Some(trace_id) => (
                    context
                        .store
                        .spans_by_trace(trace_id)
                        .await
                        .map_err(field_err)?,
                    context
                        .store
                        .logs_by_trace(trace_id)
                        .await
                        .map_err(field_err)?,
                ),
                None => (Vec::new(), Vec::new()),
            };
            BundleInputs {
                anchor: BundleAnchor::Issue(Box::new(issue)),
                events,
                trace_spans,
                trace_logs,
            }
        } else if let Some(run_id) = run_id {
            let Some(run) = context.metadata.run(&run_id).await.map_err(field_err)? else {
                return Ok(None);
            };
            let spans = context
                .store
                .spans_by_run(&run_id, MAX_ROWS)
                .await
                .map_err(field_err)?;
            let mut trace_ids: Vec<String> = Vec::new();
            for span in &spans {
                if !trace_ids.contains(&span.trace_id) {
                    trace_ids.push(span.trace_id.clone());
                }
            }
            let events = context
                .store
                .error_events_by_traces(&trace_ids, 50)
                .await
                .map_err(field_err)?;
            let mut fingerprints: Vec<String> = Vec::new();
            for event in &events {
                if !fingerprints.contains(&event.fingerprint) {
                    fingerprints.push(event.fingerprint.clone());
                }
            }
            let issues = context
                .metadata
                .issues_by_fingerprints(&fingerprints)
                .await
                .map_err(field_err)?;
            // The trace behind the newest error carries the evidence; the
            // run's logs are the log section.
            let evidence_trace = events.first().map(|e| e.trace_id.clone());
            let trace_spans = match &evidence_trace {
                Some(trace_id) if !trace_id.is_empty() => spans
                    .iter()
                    .filter(|s| s.trace_id == *trace_id)
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            };
            let trace_logs = context
                .store
                .logs_by_run(&run_id, 200)
                .await
                .map_err(field_err)?;
            BundleInputs {
                anchor: BundleAnchor::Run {
                    run: Box::new(run),
                    issues,
                },
                events,
                trace_spans,
                trace_logs,
            }
        } else {
            let trace_id = trace_id.unwrap_or_default();
            let trace_spans = context
                .store
                .spans_by_trace(&trace_id)
                .await
                .map_err(field_err)?;
            if trace_spans.is_empty() {
                return Ok(None);
            }
            let events = context
                .store
                .error_events_by_traces(std::slice::from_ref(&trace_id), 50)
                .await
                .map_err(field_err)?;
            let mut fingerprints: Vec<String> = Vec::new();
            for event in &events {
                if !fingerprints.contains(&event.fingerprint) {
                    fingerprints.push(event.fingerprint.clone());
                }
            }
            let issues = context
                .metadata
                .issues_by_fingerprints(&fingerprints)
                .await
                .map_err(field_err)?;
            let trace_logs = context
                .store
                .logs_by_trace(&trace_id)
                .await
                .map_err(field_err)?;
            BundleInputs {
                anchor: BundleAnchor::Trace { trace_id, issues },
                events,
                trace_spans,
                trace_logs,
            }
        };

        let bundle = parallax_core::bundle::assemble(inputs, max_tokens);
        let markdown = parallax_core::bundle::to_markdown(&bundle);
        let canonical_hash = bundle.canonical_hash.clone().unwrap_or_default();
        let json = serde_json::to_string_pretty(&bundle).map_err(field_err)?;
        Ok(Some(BundleOut {
            json,
            markdown,
            canonical_hash,
        }))
    }

    /// Distinct metric names seen by the store (drives the dashboard
    /// builder), optionally prefix-filtered.
    async fn metric_names(
        context: &ApiContext,
        prefix: Option<String>,
    ) -> FieldResult<Vec<String>> {
        let mut names = context.store.metric_names().await.map_err(field_err)?;
        if let Some(prefix) = prefix {
            names.retain(|n| n.starts_with(&prefix));
        }
        Ok(names)
    }

    /// Distinct service names (drives the service-overview selector).
    async fn services(context: &ApiContext) -> FieldResult<Vec<String>> {
        context.store.service_names().await.map_err(field_err)
    }

    /// Aggregated series for a point metric (gauge/sum); agg one of
    /// avg|min|max|sum|rate. With `groupBy` (an attribute key) one series
    /// per value; without it a single series with a null `groupValue`
    /// (spec §8 `metricSeries`).
    #[allow(clippy::too_many_arguments)]
    async fn metric_series(
        context: &ApiContext,
        name: String,
        from_nanos: String,
        to_nanos: String,
        service: Option<String>,
        group_by: Option<String>,
        step_seconds: Option<i32>,
        agg: Option<String>,
    ) -> FieldResult<Vec<Series>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let agg = MetricAgg::parse(agg.as_deref().unwrap_or("avg"))
            .ok_or_else(|| field_err("agg must be avg|min|max|sum|rate"))?;
        match group_by {
            Some(group_by) => {
                let groups = context
                    .store
                    .metric_series_grouped(
                        &name,
                        service.as_deref(),
                        &group_by,
                        from..=to,
                        step_nanos(step_seconds),
                        agg,
                    )
                    .await
                    .map_err(field_err)?;
                Ok(groups
                    .into_iter()
                    .map(|(group_value, points)| Series {
                        group_value: Some(group_value),
                        points,
                    })
                    .collect())
            }
            None => {
                let points = context
                    .store
                    .metric_series(
                        &name,
                        service.as_deref(),
                        from..=to,
                        step_nanos(step_seconds),
                        agg,
                    )
                    .await
                    .map_err(field_err)?;
                Ok(vec![Series {
                    group_value: None,
                    points,
                }])
            }
        }
    }

    /// Approximate quantile series from a histogram metric (q in 0..=1).
    async fn histogram_quantile(
        context: &ApiContext,
        name: String,
        from_nanos: String,
        to_nanos: String,
        q: f64,
        service: Option<String>,
        step_seconds: Option<i32>,
    ) -> FieldResult<Vec<Point>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let series = context
            .store
            .histogram_quantile(
                &name,
                service.as_deref(),
                from..=to,
                step_nanos(step_seconds),
                q,
            )
            .await
            .map_err(field_err)?;
        Ok(series.into_iter().map(Point).collect())
    }

    /// Saved user dashboards, most recently updated first.
    async fn dashboards(context: &ApiContext) -> FieldResult<Vec<Dashboard>> {
        let dashboards = context.metadata.dashboards().await.map_err(field_err)?;
        Ok(dashboards.into_iter().map(Dashboard).collect())
    }

    async fn runs(context: &ApiContext, limit: Option<i32>) -> FieldResult<Vec<Run>> {
        let runs = context
            .metadata
            .runs(clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(runs.into_iter().map(Run::new).collect())
    }
}

pub struct Mutation;

#[graphql_object(context = ApiContext)]
impl Mutation {
    /// Set an issue's workflow status (open | resolved); returns the updated
    /// issue (spec §8: `Issue!`).
    async fn issue_set_status(
        context: &ApiContext,
        fingerprint: String,
        status: String,
    ) -> FieldResult<Issue> {
        if !matches!(status.as_str(), "open" | "resolved") {
            return Err(field_err("status must be open or resolved"));
        }
        context
            .metadata
            .set_issue_status(&fingerprint, &status)
            .await
            .map_err(field_err)?;
        context
            .metadata
            .issue(&fingerprint)
            .await
            .map_err(field_err)?
            .map(Issue)
            .ok_or_else(|| field_err(format!("issue {fingerprint} not found")))
    }

    /// Register a run (the CLI wrapper calls this before launching).
    async fn run_start(
        context: &ApiContext,
        run_id: String,
        command: Option<String>,
        started_at_nanos: String,
    ) -> FieldResult<bool> {
        let nanos: u128 = started_at_nanos
            .parse()
            .map_err(|_| field_err("invalid nanos"))?;
        context
            .metadata
            .start_run(&run_id, command.as_deref(), nanos)
            .await
            .map_err(field_err)?;
        Ok(true)
    }

    /// Create or update a user dashboard; returns the saved dashboard
    /// (spec §8: `Dashboard!`).
    async fn dashboard_save(
        context: &ApiContext,
        name: String,
        layout: String,
        id: Option<String>,
    ) -> FieldResult<Dashboard> {
        // Layout must at least be valid JSON; widget semantics are the UI's.
        if serde_json::from_str::<serde_json::Value>(&layout).is_err() {
            return Err(field_err("layout must be valid JSON"));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let id = id.unwrap_or_else(|| format!("dash_{now:x}"));
        context
            .metadata
            .dashboard_save(&id, &name, &layout, now)
            .await
            .map_err(field_err)?;
        context
            .metadata
            .dashboard(&id)
            .await
            .map_err(field_err)?
            .map(Dashboard)
            .ok_or_else(|| field_err("dashboard save did not persist"))
    }

    /// Delete a user dashboard.
    async fn dashboard_delete(context: &ApiContext, id: String) -> FieldResult<bool> {
        context
            .metadata
            .dashboard_delete(&id)
            .await
            .map_err(field_err)
    }

    /// Close a run with the wrapped command's exit code.
    async fn run_finish(
        context: &ApiContext,
        run_id: String,
        ended_at_nanos: String,
        exit_code: i32,
    ) -> FieldResult<bool> {
        let nanos: u128 = ended_at_nanos
            .parse()
            .map_err(|_| field_err("invalid nanos"))?;
        context
            .metadata
            .finish_run(&run_id, nanos, exit_code)
            .await
            .map_err(field_err)?;
        Ok(true)
    }
}

pub type Schema = RootNode<Query, Mutation, EmptySubscription<ApiContext>>;

pub fn build_schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}

/// Execute one GraphQL request against the schema — the whole integration
/// layer (the server's axum handler wraps this in ~10 lines).
pub async fn execute(
    schema: &Schema,
    context: &ApiContext,
    request: juniper::http::GraphQLRequest,
) -> juniper::http::GraphQLResponse {
    request.execute(schema, context).await
}
