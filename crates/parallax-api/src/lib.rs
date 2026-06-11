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
