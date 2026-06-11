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

    /// Recent occurrences of this issue, newest first.
    async fn events(
        &self,
        context: &ApiContext,
        limit: Option<i32>,
    ) -> FieldResult<Vec<ErrorEvent>> {
        let events = context
            .store
            .error_events_by_fingerprint(&self.0.fingerprint, 0..=u128::MAX, clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(events.into_iter().map(ErrorEvent).collect())
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
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
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

pub struct Run(model::RunRecord);

#[graphql_object(context = ApiContext)]
impl Run {
    fn run_id(&self) -> &str {
        &self.0.run_id
    }
    fn command(&self) -> Option<&str> {
        self.0.command.as_deref()
    }
    fn started_at_nanos(&self) -> String {
        nanos_string(self.0.started_at_nanos)
    }
    fn ended_at_nanos(&self) -> Option<String> {
        self.0.ended_at_nanos.map(nanos_string)
    }
    fn exit_code(&self) -> Option<i32> {
        self.0.exit_code
    }
    fn status(&self) -> &str {
        &self.0.status
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

    /// Grouped errors, newest activity first.
    async fn issues(
        context: &ApiContext,
        limit: Option<i32>,
        status: Option<String>,
    ) -> FieldResult<Vec<Issue>> {
        let issues = context
            .metadata
            .issues(clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(issues
            .into_iter()
            .filter(|i| status.as_deref().is_none_or(|s| i.status == s))
            .map(Issue)
            .collect())
    }

    async fn issue(context: &ApiContext, fingerprint: String) -> FieldResult<Option<Issue>> {
        let issues = context.metadata.issues(MAX_ROWS).await.map_err(field_err)?;
        Ok(issues
            .into_iter()
            .find(|i| i.fingerprint == fingerprint)
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

    /// Traces produced by one run (grouped from its run-tagged spans).
    async fn traces_by_run(
        context: &ApiContext,
        run_id: String,
        limit: Option<i32>,
    ) -> FieldResult<Vec<Trace>> {
        let spans = context
            .store
            .spans_by_run(&run_id, clamp_limit(limit, 200))
            .await
            .map_err(field_err)?;
        let mut by_trace: Vec<(String, Vec<model::SpanRow>)> = Vec::new();
        for span in spans {
            match by_trace.iter_mut().find(|(t, _)| *t == span.trace_id) {
                Some((_, group)) => group.push(span),
                None => by_trace.push((span.trace_id.clone(), vec![span])),
            }
        }
        Ok(by_trace
            .into_iter()
            .map(|(trace_id, spans)| Trace { trace_id, spans })
            .collect())
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

    /// The bounded, redacted, hypothesis-ranked evidence bundle for an issue
    /// — the agent handoff artifact.
    async fn bundle(
        context: &ApiContext,
        fingerprint: String,
        max_tokens: Option<i32>,
    ) -> FieldResult<Option<BundleOut>> {
        let issues = context.metadata.issues(MAX_ROWS).await.map_err(field_err)?;
        let Some(issue) = issues.into_iter().find(|i| i.fingerprint == fingerprint) else {
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
        let max_tokens = usize::try_from(max_tokens.unwrap_or(10_000).max(500)).unwrap_or(10_000);
        let bundle = parallax_core::bundle::assemble(
            parallax_core::bundle::BundleInputs {
                issue,
                events,
                trace_spans,
                trace_logs,
            },
            max_tokens,
        );
        let markdown = parallax_core::bundle::to_markdown(&bundle);
        let canonical_hash = bundle.canonical_hash.clone().unwrap_or_default();
        let json = serde_json::to_string_pretty(&bundle).map_err(field_err)?;
        Ok(Some(BundleOut {
            json,
            markdown,
            canonical_hash,
        }))
    }

    /// Distinct metric names seen by the store (drives the dashboard builder).
    async fn metric_names(context: &ApiContext) -> FieldResult<Vec<String>> {
        context.store.metric_names().await.map_err(field_err)
    }

    /// Distinct service names (drives the service-overview selector).
    async fn services(context: &ApiContext) -> FieldResult<Vec<String>> {
        context.store.service_names().await.map_err(field_err)
    }

    /// Aggregated series for a point metric (gauge/sum); agg one of
    /// avg|min|max|sum|rate.
    async fn metric_series(
        context: &ApiContext,
        name: String,
        from_nanos: String,
        to_nanos: String,
        service: Option<String>,
        step_seconds: Option<i32>,
        agg: Option<String>,
    ) -> FieldResult<Vec<Point>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let agg = MetricAgg::parse(agg.as_deref().unwrap_or("avg"))
            .ok_or_else(|| field_err("agg must be avg|min|max|sum|rate"))?;
        let series = context
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
        Ok(series.into_iter().map(Point).collect())
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
        Ok(runs.into_iter().map(Run).collect())
    }
}

pub struct Mutation;

#[graphql_object(context = ApiContext)]
impl Mutation {
    /// Set an issue's workflow status (open | resolved).
    async fn issue_set_status(
        context: &ApiContext,
        fingerprint: String,
        status: String,
    ) -> FieldResult<bool> {
        if !matches!(status.as_str(), "open" | "resolved") {
            return Err(field_err("status must be open or resolved"));
        }
        context
            .metadata
            .set_issue_status(&fingerprint, &status)
            .await
            .map_err(field_err)?;
        Ok(true)
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

    /// Create or update a user dashboard; returns its id.
    async fn dashboard_save(
        context: &ApiContext,
        name: String,
        layout: String,
        id: Option<String>,
    ) -> FieldResult<String> {
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
        Ok(id)
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
