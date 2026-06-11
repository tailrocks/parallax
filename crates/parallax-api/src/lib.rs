//! Parallax GraphQL API — the V1 read surface from the implementation spec §8,
//! resolved against the storage adapters. Every client (CLI, UI, agents) goes
//! through this schema; none touch storage directly.

use async_graphql::{ComplexObject, Context, EmptySubscription, Object, Schema, SimpleObject};
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model;
use std::sync::Arc;

/// Nanosecond timestamps cross the API as strings (JSON numbers lose
/// precision past 2^53).
fn nanos_string(nanos: u128) -> String {
    nanos.to_string()
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Issue {
    pub fingerprint: String,
    pub title: String,
    pub error_type: String,
    pub culprit: Option<String>,
    pub service: String,
    pub status: String,
    pub first_seen_nanos: String,
    pub last_seen_nanos: String,
    pub event_count: u64,
    pub last_trace_id: Option<String>,
}

#[ComplexObject]
impl Issue {
    /// Recent occurrences of this issue, newest first.
    async fn events(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 50)] limit: usize,
    ) -> async_graphql::Result<Vec<ErrorEvent>> {
        let store = ctx.data::<Arc<dyn TelemetryStore>>()?;
        let events = store
            .error_events_by_fingerprint(&self.fingerprint, 0..=u128::MAX, limit.min(500))
            .await?;
        Ok(events.into_iter().map(ErrorEvent::from).collect())
    }
}

impl From<model::Issue> for Issue {
    fn from(issue: model::Issue) -> Self {
        Self {
            fingerprint: issue.fingerprint,
            title: issue.title,
            error_type: issue.error_type,
            culprit: issue.culprit,
            service: issue.service,
            status: issue.status,
            first_seen_nanos: nanos_string(issue.first_seen_nanos),
            last_seen_nanos: nanos_string(issue.last_seen_nanos),
            event_count: issue.event_count,
            last_trace_id: issue.last_trace_id,
        }
    }
}

#[derive(SimpleObject)]
pub struct ErrorEvent {
    pub ts_nanos: String,
    pub service: String,
    pub fingerprint: String,
    pub error_type: String,
    pub message: String,
    pub stacktrace: Option<String>,
    pub source: String,
    pub trace_id: String,
    pub span_id: String,
    pub attributes: String,
}

impl From<model::ErrorEventRow> for ErrorEvent {
    fn from(row: model::ErrorEventRow) -> Self {
        Self {
            ts_nanos: nanos_string(row.ts_nanos),
            service: row.service,
            fingerprint: row.fingerprint,
            error_type: row.error_type,
            message: row.message,
            stacktrace: row.stacktrace,
            source: serde_json::to_string(&row.source)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            trace_id: row.trace_id,
            span_id: row.span_id,
            attributes: row.attributes.to_string(),
        }
    }
}

#[derive(SimpleObject)]
pub struct Span {
    pub ts_nanos: String,
    pub service: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub status_code: String,
    pub status_message: String,
    pub duration_ns: String,
    pub scope_name: String,
    pub attributes: String,
    pub resource: String,
}

impl From<model::SpanRow> for Span {
    fn from(row: model::SpanRow) -> Self {
        Self {
            ts_nanos: nanos_string(row.ts_nanos),
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            parent_span_id: row.parent_span_id,
            name: row.name,
            kind: row.kind,
            status_code: row.status_code,
            status_message: row.status_message,
            duration_ns: row.duration_ns.to_string(),
            scope_name: row.scope_name,
            attributes: row.attributes.to_string(),
            resource: row.resource.to_string(),
        }
    }
}

#[derive(SimpleObject)]
pub struct LogRecord {
    pub ts_nanos: String,
    pub service: String,
    pub severity_num: i32,
    pub severity_text: String,
    pub body: String,
    pub trace_id: String,
    pub span_id: String,
    pub attributes: String,
}

impl From<model::LogRow> for LogRecord {
    fn from(row: model::LogRow) -> Self {
        Self {
            ts_nanos: nanos_string(row.ts_nanos),
            service: row.service,
            severity_num: row.severity_num,
            severity_text: row.severity_text,
            body: row.body,
            trace_id: row.trace_id,
            span_id: row.span_id,
            attributes: row.attributes.to_string(),
        }
    }
}

#[derive(SimpleObject)]
pub struct Trace {
    pub trace_id: String,
    pub spans: Vec<Span>,
}

#[derive(SimpleObject)]
pub struct Run {
    pub run_id: String,
    pub command: Option<String>,
    pub started_at_nanos: String,
    pub ended_at_nanos: Option<String>,
    pub exit_code: Option<i32>,
    pub status: String,
}

impl From<model::RunRecord> for Run {
    fn from(run: model::RunRecord) -> Self {
        Self {
            run_id: run.run_id,
            command: run.command,
            started_at_nanos: nanos_string(run.started_at_nanos),
            ended_at_nanos: run.ended_at_nanos.map(nanos_string),
            exit_code: run.exit_code,
            status: run.status,
        }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &'static str {
        "ok"
    }

    async fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Grouped errors, newest activity first.
    async fn issues(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 50)] limit: usize,
        status: Option<String>,
    ) -> async_graphql::Result<Vec<Issue>> {
        let metadata = ctx.data::<Arc<MetadataStore>>()?;
        let issues = metadata.issues(limit.min(500)).await?;
        Ok(issues
            .into_iter()
            .filter(|i| status.as_deref().is_none_or(|s| i.status == s))
            .map(Issue::from)
            .collect())
    }

    async fn issue(
        &self,
        ctx: &Context<'_>,
        fingerprint: String,
    ) -> async_graphql::Result<Option<Issue>> {
        let metadata = ctx.data::<Arc<MetadataStore>>()?;
        Ok(metadata
            .issues(500)
            .await?
            .into_iter()
            .find(|i| i.fingerprint == fingerprint)
            .map(Issue::from))
    }

    /// Every span of one trace, start-time ascending (cross-service).
    async fn trace(
        &self,
        ctx: &Context<'_>,
        trace_id: String,
    ) -> async_graphql::Result<Option<Trace>> {
        let store = ctx.data::<Arc<dyn TelemetryStore>>()?;
        let spans = store.spans_by_trace(&trace_id).await?;
        if spans.is_empty() {
            return Ok(None);
        }
        Ok(Some(Trace {
            trace_id,
            spans: spans.into_iter().map(Span::from).collect(),
        }))
    }

    /// Logs correlated to one trace, time ascending.
    async fn logs_by_trace(
        &self,
        ctx: &Context<'_>,
        trace_id: String,
    ) -> async_graphql::Result<Vec<LogRecord>> {
        let store = ctx.data::<Arc<dyn TelemetryStore>>()?;
        Ok(store
            .logs_by_trace(&trace_id)
            .await?
            .into_iter()
            .map(LogRecord::from)
            .collect())
    }

    async fn runs(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 50)] limit: usize,
    ) -> async_graphql::Result<Vec<Run>> {
        let metadata = ctx.data::<Arc<MetadataStore>>()?;
        Ok(metadata
            .runs(limit.min(500))
            .await?
            .into_iter()
            .map(Run::from)
            .collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Set an issue's workflow status (open | resolved).
    async fn issue_set_status(
        &self,
        ctx: &Context<'_>,
        fingerprint: String,
        status: String,
    ) -> async_graphql::Result<bool> {
        if !matches!(status.as_str(), "open" | "resolved") {
            return Err("status must be open or resolved".into());
        }
        let metadata = ctx.data::<Arc<MetadataStore>>()?;
        metadata.set_issue_status(&fingerprint, &status).await?;
        Ok(true)
    }
}

pub type ParallaxSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Build the schema with the storage adapters injected and the spec §4 cost
/// limits enforced.
pub fn build_schema(
    store: Arc<dyn TelemetryStore>,
    metadata: Arc<MetadataStore>,
    max_depth: usize,
    max_complexity: usize,
) -> ParallaxSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(store)
        .data(metadata)
        .limit_depth(max_depth)
        .limit_complexity(max_complexity)
        .finish()
}
