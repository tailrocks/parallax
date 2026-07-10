//! Parallax GraphQL API — the V1 surface from the implementation spec §8,
//! served by **Juniper** (operator instruction, 2026-06-12: the library he
//! uses in his own services). Every client (CLI, UI, agents) goes through
//! this schema; none touch storage directly.
//!
//! Juniper notes (per the spec's dependency table): GraphQL `Int` is i32 —
//! counts saturate; nanosecond timestamps cross as strings; field names are
//! auto-camelCased; cost limits are resolver-level caps in V1.

use juniper::{EmptySubscription, FieldError, FieldResult, RootNode, graphql_object};
use parallax_core::{agent_session, gaps, semconv, span_events, story, trace_analysis};
use parallax_storage::adapter::{
    ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow as StorageAttributeCompareRow,
    FieldKey as StorageFieldKey, FieldSource, FieldStats as StorageFieldStats,
    FieldValueCount as StorageFieldValueCount, OverviewTotals,
    ReleaseWindow as StorageReleaseWindow, RuntimeMetricSeries as StorageRuntimeMetricSeries,
    SERVICE_MAP_TRACE_CAP, ServiceCatalogRow as StorageServiceCatalogRow,
    ServiceEdge as StorageServiceEdge, ServiceSummary as StorageServiceSummary,
    SpanRed as StorageSpanRed, TelemetryStore, metric_group_label_allowed,
};
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model;
use parallax_storage::model::{MetricAgg, SeriesPoint};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

/// Request context: the storage adapters.
#[derive(Clone)]
pub struct ApiContext {
    pub store: Arc<dyn TelemetryStore>,
    pub metadata: Arc<MetadataStore>,
    pub otlp_grpc_port: u16,
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
/// Raw SQL is the power surface, so it gets a larger cap than typed resolvers
/// while still bounding GraphQL response size.
const SQL_MAX_ROWS: usize = 2_000;
const SAVED_VIEW_NAME_MAX: usize = 120;
const SAVED_VIEWS_PER_PAGE: usize = 100;
const INVESTIGATION_NAME_MAX: usize = 120;
const INVESTIGATION_PIN_CAP: usize = 100;
const INVESTIGATION_NOTES_MAX_BYTES: usize = 64 * 1024;

fn clamp_limit(limit: Option<i32>, default: usize) -> usize {
    limit
        .map_or(default, |l| usize::try_from(l.max(0)).unwrap_or(default))
        .min(MAX_ROWS)
}

fn validate_saved_view_name(name: &str) -> FieldResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(field_err("saved view name is required"));
    }
    if name.chars().count() > SAVED_VIEW_NAME_MAX {
        return Err(field_err("saved view name is too long"));
    }
    Ok(name.to_string())
}

fn validate_saved_view_page(page: &str) -> FieldResult<()> {
    if page.is_empty() || page.len() > 128 || !page.starts_with('/') {
        return Err(field_err("saved view page must be a route path"));
    }
    Ok(())
}

fn validate_investigation_name(name: &str) -> FieldResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(field_err("investigation name is required"));
    }
    if name.chars().count() > INVESTIGATION_NAME_MAX {
        return Err(field_err("investigation name is too long"));
    }
    Ok(name.to_string())
}

fn validate_investigation_state(state: &str) -> FieldResult<()> {
    let parsed: model::InvestigationState =
        serde_json::from_str(state).map_err(|_| field_err("state must be valid JSON"))?;
    if parsed.version != 1 {
        return Err(field_err("investigation state version must be 1"));
    }
    if parsed.pins.len() > INVESTIGATION_PIN_CAP {
        return Err(field_err("investigation pin cap exceeded"));
    }
    if parsed.notes.len() > INVESTIGATION_NOTES_MAX_BYTES {
        return Err(field_err("investigation notes are too long"));
    }
    Ok(())
}

/// Metric names flow into storage identifiers; keep them inside the OTel metric-name grammar.
fn validate_metric_name(name: &str) -> FieldResult<()> {
    let ok = !name.is_empty()
        && name.len() <= 255
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'));
    if ok {
        Ok(())
    } else {
        Err(field_err("invalid metric name"))
    }
}

fn validate_metric_group_label(label: &str) -> FieldResult<()> {
    let ok = label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
        && metric_group_label_allowed(label);
    if ok {
        Ok(())
    } else {
        Err(field_err(
            "high-cardinality identifier - filter, don't group",
        ))
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanLink {
    trace_id: String,
    span_id: String,
    attributes: String,
}

#[graphql_object(context = ApiContext)]
impl SpanLink {
    fn trace_id(&self) -> &str {
        &self.trace_id
    }
    fn span_id(&self) -> &str {
        &self.span_id
    }
    fn attributes(&self) -> &str {
        &self.attributes
    }
}

fn span_links_from_value(links: &serde_json::Value) -> Vec<SpanLink> {
    links
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|link| {
            let trace_id = link
                .get("traceId")
                .or_else(|| link.get("trace_id"))?
                .as_str()?
                .to_string();
            if trace_id.is_empty() {
                return None;
            }
            let span_id = link
                .get("spanId")
                .or_else(|| link.get("span_id"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let attributes = link
                .get("attributes")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
                .to_string();
            Some(SpanLink {
                trace_id,
                span_id,
                attributes,
            })
        })
        .collect()
}

fn linked_trace_ids(spans: &[model::SpanRow], anchor_trace_id: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    for span in spans {
        for link in span_links_from_value(&span.links) {
            if link.trace_id == anchor_trace_id || !seen.insert(link.trace_id.clone()) {
                continue;
            }
            ids.push(link.trace_id);
            if ids.len() >= MAX_ROWS {
                return ids;
            }
        }
    }
    ids
}

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
    /// OTel span links as JSON — spans in other traces this span causally
    /// references (batch/async sub-operations).
    fn links(&self) -> String {
        self.0.links.to_string()
    }
    /// Typed span-link targets, beside the raw JSON string for compatibility.
    fn typed_links(&self) -> Vec<SpanLink> {
        span_links_from_value(&self.0.links)
    }
    /// OTel span events as JSON: `[{name, timeUnixNano, attributes}]`.
    fn events(&self) -> String {
        self.0.events.clone().unwrap_or_else(|| "[]".to_string())
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
    fn event_name(&self) -> &str {
        &self.0.event_name
    }
    fn observed_ts_nanos(&self) -> String {
        nanos_string(self.0.observed_ts_nanos)
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

pub struct TraceEvent(span_events::TraceEvent);

#[graphql_object(context = ApiContext)]
impl TraceEvent {
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn span_name(&self) -> &str {
        &self.0.span_name
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn time_unix_nano(&self) -> String {
        nanos_string(self.0.time_unix_nano)
    }
    fn attributes(&self) -> String {
        let attributes: BTreeMap<_, _> = self.0.attributes.iter().cloned().collect();
        serde_json::to_string(&attributes).unwrap_or_else(|_| "{}".to_string())
    }
}

pub struct TraceEventsOut(span_events::TraceEvents);

#[graphql_object(context = ApiContext)]
impl TraceEventsOut {
    fn events(&self) -> Vec<TraceEvent> {
        self.0.events.iter().cloned().map(TraceEvent).collect()
    }
    fn truncated(&self) -> bool {
        self.0.truncated()
    }
    fn skipped_spans(&self) -> i32 {
        saturate_i32(self.0.skipped_spans as u64)
    }
}

pub struct SqlResultOut {
    result: parallax_storage::adapter::SqlResult,
    truncated: bool,
}

fn cap_sql_result(
    mut result: parallax_storage::adapter::SqlResult,
    max_rows: usize,
) -> SqlResultOut {
    let truncated = result.rows.len() > max_rows;
    if truncated {
        result.rows.truncate(max_rows);
    }
    SqlResultOut { result, truncated }
}

#[graphql_object(context = ApiContext)]
impl SqlResultOut {
    fn columns(&self) -> &[String] {
        &self.result.columns
    }
    /// Each row as a JSON array string (heterogeneous cell types).
    fn rows(&self) -> Vec<String> {
        self.result
            .rows
            .iter()
            .map(|row| serde_json::Value::Array(row.clone()).to_string())
            .collect()
    }
    fn row_count(&self) -> i32 {
        i32::try_from(self.result.rows.len()).unwrap_or(i32::MAX)
    }
    fn truncated(&self) -> bool {
        self.truncated
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

pub struct TraceList(parallax_storage::adapter::TraceList);

#[graphql_object(context = ApiContext)]
impl TraceList {
    fn items(&self) -> Vec<TraceSummary> {
        self.0.items.iter().cloned().map(TraceSummary).collect()
    }
    /// Matching traces before paging. String avoids GraphQL Int saturation.
    fn total(&self) -> String {
        self.0.total.to_string()
    }
}

pub struct CriticalHop(trace_analysis::CriticalHop);

#[graphql_object(context = ApiContext)]
impl CriticalHop {
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn self_time_ns(&self) -> String {
        nanos_string(self.0.self_time_ns)
    }
    fn gated_by_child(&self) -> Option<&str> {
        self.0.gated_by_child.as_deref()
    }
    fn clock_suspect(&self) -> bool {
        self.0.clock_suspect
    }
}

pub struct CriticalPath(trace_analysis::CriticalPath);

#[graphql_object(context = ApiContext)]
impl CriticalPath {
    fn hops(&self) -> Vec<CriticalHop> {
        self.0.hops.iter().cloned().map(CriticalHop).collect()
    }
    fn total_gated_ns(&self) -> String {
        nanos_string(self.0.total_gated_ns)
    }
    fn unattached(&self) -> Vec<String> {
        self.0.unattached.clone()
    }
}

pub struct DiffSpan(trace_analysis::DiffSpan);

#[graphql_object(context = ApiContext)]
impl DiffSpan {
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn service(&self) -> &str {
        &self.0.service
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
    fn duration_ns(&self) -> String {
        nanos_string(self.0.duration_ns)
    }
    fn depth(&self) -> i32 {
        i32::try_from(self.0.depth).unwrap_or(i32::MAX)
    }
    fn match_key(&self) -> &str {
        &self.0.match_key
    }
}

pub struct ChangedSpan(trace_analysis::ChangedSpan);

#[graphql_object(context = ApiContext)]
impl ChangedSpan {
    fn before(&self) -> DiffSpan {
        DiffSpan(self.0.before.clone())
    }
    fn after(&self) -> DiffSpan {
        DiffSpan(self.0.after.clone())
    }
    fn duration_delta_ns(&self) -> String {
        self.0.duration_delta_ns.to_string()
    }
    fn duration_delta_pct(&self) -> f64 {
        self.0.duration_delta_pct
    }
    fn status_changed(&self) -> bool {
        self.0.status_changed
    }
}

pub struct TraceDiff(trace_analysis::TraceDiff);

#[graphql_object(context = ApiContext)]
impl TraceDiff {
    fn added(&self) -> Vec<DiffSpan> {
        self.0.added.iter().cloned().map(DiffSpan).collect()
    }
    fn removed(&self) -> Vec<DiffSpan> {
        self.0.removed.iter().cloned().map(DiffSpan).collect()
    }
    fn changed(&self) -> Vec<ChangedSpan> {
        self.0.changed.iter().cloned().map(ChangedSpan).collect()
    }
}

pub struct StoryBeat(story::StoryBeat);

#[graphql_object(context = ApiContext)]
impl StoryBeat {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn lane(&self) -> &str {
        &self.0.lane
    }
    fn kind(&self) -> &str {
        &self.0.kind
    }
    fn title(&self) -> &str {
        &self.0.title
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> Option<&str> {
        self.0.span_id.as_deref()
    }
    fn severity(&self) -> Option<&str> {
        self.0.severity.as_deref()
    }
    fn duration_ns(&self) -> Option<String> {
        self.0.duration_ns.map(nanos_string)
    }
}

pub struct AgentSessionOut {
    session: agent_session::AgentSession,
    truncated: bool,
}

pub struct AgentStepOut(agent_session::AgentStep);

fn agent_step_kind_name(kind: agent_session::AgentStepKind) -> &'static str {
    match kind {
        agent_session::AgentStepKind::InvokeAgent => "INVOKE_AGENT",
        agent_session::AgentStepKind::ExecuteTool => "EXECUTE_TOOL",
        agent_session::AgentStepKind::Shell => "SHELL",
        agent_session::AgentStepKind::Other => "OTHER",
    }
}

#[graphql_object(context = ApiContext)]
impl AgentSessionOut {
    fn root_span_id(&self) -> Option<&str> {
        self.session.root_span_id.as_deref()
    }
    fn steps(&self) -> Vec<AgentStepOut> {
        self.session
            .steps
            .iter()
            .cloned()
            .map(AgentStepOut)
            .collect()
    }
    fn total_input_tokens(&self) -> String {
        self.session.total_input_tokens.to_string()
    }
    fn total_output_tokens(&self) -> String {
        self.session.total_output_tokens.to_string()
    }
    fn error_count(&self) -> i32 {
        i32::try_from(self.session.error_count).unwrap_or(i32::MAX)
    }
    fn truncated(&self) -> bool {
        self.truncated
    }
}

#[graphql_object(context = ApiContext)]
impl AgentStepOut {
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn kind(&self) -> &str {
        agent_step_kind_name(self.0.kind)
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn start_nanos(&self) -> String {
        nanos_string(self.0.start_nanos)
    }
    fn duration_ns(&self) -> String {
        nanos_string(self.0.duration_ns)
    }
    fn is_error(&self) -> bool {
        self.0.is_error
    }
    fn gen_ai_operation(&self) -> Option<&str> {
        self.0.gen_ai_operation.as_deref()
    }
    fn input_tokens(&self) -> Option<String> {
        self.0.input_tokens.map(|tokens| tokens.to_string())
    }
    fn output_tokens(&self) -> Option<String> {
        self.0.output_tokens.map(|tokens| tokens.to_string())
    }
}

pub struct AttributeCompareRow(StorageAttributeCompareRow);

#[graphql_object(context = ApiContext)]
impl AttributeCompareRow {
    fn key(&self) -> &str {
        &self.0.key
    }
    fn value(&self) -> &str {
        &self.0.value
    }
    fn selected_count(&self) -> String {
        self.0.selected_count.to_string()
    }
    fn selected_total(&self) -> String {
        self.0.selected_total.to_string()
    }
    fn baseline_count(&self) -> String {
        self.0.baseline_count.to_string()
    }
    fn baseline_total(&self) -> String {
        self.0.baseline_total.to_string()
    }
    fn score(&self) -> f64 {
        self.0.score
    }
}

fn field_source_name(source: FieldSource) -> &'static str {
    match source {
        FieldSource::Span => "SPAN",
        FieldSource::Resource => "RESOURCE",
    }
}

pub struct FieldKey(StorageFieldKey);

#[graphql_object(context = ApiContext)]
impl FieldKey {
    fn key(&self) -> &str {
        &self.0.key
    }
    fn namespace(&self) -> &str {
        &self.0.namespace
    }
    fn source(&self) -> &str {
        field_source_name(self.0.source)
    }
    fn row_count(&self) -> String {
        self.0.row_count.to_string()
    }
    fn non_null_count(&self) -> String {
        self.0.non_null_count.to_string()
    }
    fn coverage(&self) -> f64 {
        self.0.coverage
    }
    fn is_identifier(&self) -> bool {
        self.0.is_identifier
    }
}

pub struct FieldValueCount(StorageFieldValueCount);

#[graphql_object(context = ApiContext)]
impl FieldValueCount {
    fn value(&self) -> &str {
        &self.0.value
    }
    fn count(&self) -> String {
        self.0.count.to_string()
    }
}

pub struct FieldStats(StorageFieldStats);

#[graphql_object(context = ApiContext)]
impl FieldStats {
    fn key(&self) -> &str {
        &self.0.key
    }
    fn namespace(&self) -> &str {
        &self.0.namespace
    }
    fn source(&self) -> &str {
        field_source_name(self.0.source)
    }
    fn row_count(&self) -> String {
        self.0.row_count.to_string()
    }
    fn non_null_count(&self) -> String {
        self.0.non_null_count.to_string()
    }
    fn distinct_count(&self) -> String {
        self.0.distinct_count.to_string()
    }
    fn coverage(&self) -> f64 {
        self.0.coverage
    }
    fn capped(&self) -> bool {
        self.0.capped
    }
    fn is_identifier(&self) -> bool {
        self.0.is_identifier
    }
    fn top_values(&self) -> Vec<FieldValueCount> {
        self.0
            .top_values
            .iter()
            .cloned()
            .map(FieldValueCount)
            .collect()
    }
}

pub struct EvidenceGap(gaps::EvidenceGap);

#[graphql_object(context = ApiContext)]
impl EvidenceGap {
    fn kind(&self) -> &str {
        &self.0.kind
    }
    fn subject(&self) -> &str {
        &self.0.subject
    }
    fn detail(&self) -> &str {
        &self.0.detail
    }
}

#[derive(juniper::GraphQLEnum, Clone, Copy)]
pub enum TraceSort {
    StartDesc,
    DurationDesc,
    DurationAsc,
    SpanCountDesc,
}

impl From<TraceSort> for parallax_storage::adapter::TraceSort {
    fn from(value: TraceSort) -> Self {
        match value {
            TraceSort::StartDesc => Self::StartDesc,
            TraceSort::DurationDesc => Self::DurationDesc,
            TraceSort::DurationAsc => Self::DurationAsc,
            TraceSort::SpanCountDesc => Self::SpanCountDesc,
        }
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
                let mut seen_trace_ids = HashSet::new();
                for span in &spans {
                    let trace_id = span.trace_id.clone();
                    if seen_trace_ids.insert(trace_id.clone()) {
                        trace_ids.push(trace_id);
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
        let mut seen_fingerprints = HashSet::new();
        for event in &stats.events {
            let fingerprint = event.fingerprint.clone();
            if seen_fingerprints.insert(fingerprint.clone()) {
                fingerprints.push(fingerprint);
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

pub struct RuntimeMetric(StorageRuntimeMetricSeries);

#[graphql_object(context = ApiContext)]
impl RuntimeMetric {
    fn family(&self) -> &str {
        &self.0.family
    }
    fn metric(&self) -> &str {
        &self.0.metric
    }
    fn unit(&self) -> Option<&str> {
        self.0.unit.as_deref()
    }
    fn points(&self) -> Vec<Point> {
        self.0.points.iter().copied().map(Point).collect()
    }
}

pub struct MetricExemplar(model::MetricExemplarRow);

#[graphql_object(context = ApiContext)]
impl MetricExemplar {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn value(&self) -> f64 {
        self.0.value
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

pub struct Overview(OverviewTotals);

#[graphql_object(context = ApiContext)]
impl Overview {
    fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
    fn trace_count(&self) -> String {
        self.0.trace_count.to_string()
    }
    fn log_count(&self) -> String {
        self.0.log_count.to_string()
    }
    fn metric_point_count(&self) -> String {
        self.0.metric_point_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn error_rate(&self) -> f64 {
        self.0.error_rate
    }
    fn active_services(&self) -> i32 {
        saturate_i32(self.0.active_services)
    }
}

pub struct ServiceSummary(StorageServiceSummary);

#[graphql_object(context = ApiContext)]
impl ServiceSummary {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn p95_ms(&self) -> Option<f64> {
        self.0.p95_ms
    }
}

pub struct ReleaseWindow(StorageReleaseWindow);

#[graphql_object(context = ApiContext)]
impl ReleaseWindow {
    fn version(&self) -> &str {
        &self.0.version
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.0.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
}

pub struct ServiceCatalogRow(StorageServiceCatalogRow);

#[graphql_object(context = ApiContext)]
impl ServiceCatalogRow {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn service_version(&self) -> Option<&str> {
        self.0.service_version.as_deref()
    }
    fn service_namespace(&self) -> Option<&str> {
        self.0.service_namespace.as_deref()
    }
    fn deployment_environment(&self) -> Option<&str> {
        self.0.deployment_environment.as_deref()
    }
    fn telemetry_sdk_language(&self) -> Option<&str> {
        self.0.telemetry_sdk_language.as_deref()
    }
    fn telemetry_sdk_name(&self) -> Option<&str> {
        self.0.telemetry_sdk_name.as_deref()
    }
    fn telemetry_sdk_version(&self) -> Option<&str> {
        self.0.telemetry_sdk_version.as_deref()
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn instance_count(&self) -> String {
        self.0.instance_count.to_string()
    }
}

#[derive(Clone)]
pub struct ServiceNodeData {
    name: String,
    last_seen_nanos: u128,
    span_count: u64,
    error_count: u64,
    p95_ms: Option<f64>,
}

pub struct ServiceNode(ServiceNodeData);

#[graphql_object(context = ApiContext)]
impl ServiceNode {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn p95_ms(&self) -> Option<f64> {
        self.0.p95_ms
    }
}

pub struct ServiceEdge(StorageServiceEdge);

#[graphql_object(context = ApiContext)]
impl ServiceEdge {
    fn source(&self) -> &str {
        &self.0.source
    }
    fn target(&self) -> &str {
        &self.0.target
    }
    fn call_count(&self) -> String {
        self.0.call_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn p50_ms(&self) -> f64 {
        self.0.p50_ms
    }
    fn p95_ms(&self) -> f64 {
        self.0.p95_ms
    }
}

pub struct ServiceMap {
    nodes: Vec<ServiceNodeData>,
    edges: Vec<StorageServiceEdge>,
}

#[graphql_object(context = ApiContext)]
impl ServiceMap {
    fn nodes(&self) -> Vec<ServiceNode> {
        self.nodes.iter().cloned().map(ServiceNode).collect()
    }
    fn edges(&self) -> Vec<ServiceEdge> {
        self.edges.iter().cloned().map(ServiceEdge).collect()
    }
}

pub struct SpanRed(StorageSpanRed);

#[graphql_object(context = ApiContext)]
impl SpanRed {
    fn rate(&self) -> Vec<Point> {
        self.0.rate.iter().copied().map(Point).collect()
    }
    fn error_rate(&self) -> Vec<Point> {
        self.0.error_rate.iter().copied().map(Point).collect()
    }
    fn p50(&self) -> Vec<Point> {
        self.0.p50.iter().copied().map(Point).collect()
    }
    fn p95(&self) -> Vec<Point> {
        self.0.p95.iter().copied().map(Point).collect()
    }
    fn p99(&self) -> Vec<Point> {
        self.0.p99.iter().copied().map(Point).collect()
    }
}

#[derive(juniper::GraphQLEnum, Clone, Copy)]
pub enum SignalKind {
    Spans,
    Traces,
    Logs,
    Errors,
    MetricPoints,
}

impl From<SignalKind> for parallax_storage::adapter::SignalKind {
    fn from(value: SignalKind) -> Self {
        match value {
            SignalKind::Spans => Self::Spans,
            SignalKind::Traces => Self::Traces,
            SignalKind::Logs => Self::Logs,
            SignalKind::Errors => Self::Errors,
            SignalKind::MetricPoints => Self::MetricPoints,
        }
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
                    None,
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
        for name in semconv::REQUEST_DURATION_METRICS {
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

#[graphql_object(context = ApiContext)]
impl ServiceOverview {
    /// Process/system CPU, averaged per step.
    async fn cpu(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .first_nonempty_points(context, semconv::CPU_METRICS)
            .await?
            .into_iter()
            .map(Point)
            .collect())
    }

    /// Process memory, averaged per step.
    async fn memory(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .first_nonempty_points(context, semconv::MEMORY_METRICS)
            .await?
            .into_iter()
            .map(Point)
            .collect())
    }

    /// Requests per second from the request-duration histogram's sample
    /// counts.
    async fn request_rate(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        let step_secs = (self.step / 1_000_000_000).max(1) as f64;
        for name in semconv::REQUEST_DURATION_METRICS {
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

pub struct Investigation(model::Investigation);

#[graphql_object(context = ApiContext)]
impl Investigation {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    /// Opaque V1 investigation state JSON:
    /// `{version, window, pins, notes}`.
    fn state(&self) -> &str {
        &self.0.state
    }
    fn created_at_nanos(&self) -> String {
        nanos_string(self.0.created_at_nanos)
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

pub struct SavedView(model::SavedView);

#[graphql_object(context = ApiContext)]
impl SavedView {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn page(&self) -> &str {
        &self.0.page
    }
    /// URL search string captured from the page state.
    fn state(&self) -> &str {
        &self.0.state
    }
    fn created_at_nanos(&self) -> String {
        nanos_string(self.0.created_at_nanos)
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

    fn otlp_grpc_port(context: &ApiContext) -> i32 {
        i32::from(context.otlp_grpc_port)
    }

    /// Whole-system counters for an inclusive time window. Counts are strings
    /// so large telemetry volumes never saturate GraphQL Int.
    async fn overview(
        context: &ApiContext,
        from_nanos: String,
        to_nanos: String,
    ) -> FieldResult<Overview> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        Ok(Overview(
            context
                .store
                .overview_totals(from..=to)
                .await
                .map_err(field_err)?,
        ))
    }

    /// Per-signal count series for overview trend charts.
    async fn signal_count_series(
        context: &ApiContext,
        kind: SignalKind,
        service: Option<String>,
        from_nanos: String,
        to_nanos: String,
        step_seconds: Option<i32>,
    ) -> FieldResult<Vec<Point>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let series = context
            .store
            .signal_count_series(
                kind.into(),
                service.as_deref().filter(|s| !s.is_empty()),
                from..=to,
                step_nanos(step_seconds),
            )
            .await
            .map_err(field_err)?;
        Ok(series.into_iter().map(Point).collect())
    }

    /// Service summary rows for the services index.
    async fn service_list(
        context: &ApiContext,
        from_nanos: String,
        to_nanos: String,
    ) -> FieldResult<Vec<ServiceSummary>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let services = context
            .store
            .service_summaries(from..=to)
            .await
            .map_err(field_err)?;
        Ok(services.into_iter().map(ServiceSummary).collect())
    }

    /// Per-version service release windows in the selected time range.
    async fn releases(
        context: &ApiContext,
        service: String,
        from_nanos: String,
        to_nanos: String,
    ) -> FieldResult<Vec<ReleaseWindow>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let windows = context
            .store
            .release_windows(&service, from..=to)
            .await
            .map_err(field_err)?;
        Ok(windows.into_iter().map(ReleaseWindow).collect())
    }

    /// Resource-identity catalog rows for services in the selected window.
    async fn service_catalog(
        context: &ApiContext,
        from_nanos: String,
        to_nanos: String,
    ) -> FieldResult<Vec<ServiceCatalogRow>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let rows = context
            .store
            .service_catalog(from..=to)
            .await
            .map_err(field_err)?;
        Ok(rows.into_iter().map(ServiceCatalogRow).collect())
    }

    /// Trace-path service graph over a bounded set of traces in the window.
    async fn service_map(
        context: &ApiContext,
        from_nanos: String,
        to_nanos: String,
        max_traces: Option<i32>,
    ) -> FieldResult<ServiceMap> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let max_traces = clamp_limit(max_traces, 50).min(SERVICE_MAP_TRACE_CAP);
        let services = context
            .store
            .service_summaries(from..=to)
            .await
            .map_err(field_err)?;
        let edges = context
            .store
            .service_map(from..=to, max_traces)
            .await
            .map_err(field_err)?;
        let mut nodes: BTreeMap<String, ServiceNodeData> = services
            .into_iter()
            .map(|service| {
                (
                    service.name.clone(),
                    ServiceNodeData {
                        name: service.name,
                        last_seen_nanos: service.last_seen_nanos,
                        span_count: service.span_count,
                        error_count: service.error_count,
                        p95_ms: service.p95_ms,
                    },
                )
            })
            .collect();
        for edge in &edges {
            for service in [&edge.source, &edge.target] {
                nodes
                    .entry(service.clone())
                    .or_insert_with(|| ServiceNodeData {
                        name: service.clone(),
                        last_seen_nanos: 0,
                        span_count: 0,
                        error_count: 0,
                        p95_ms: None,
                    });
            }
        }
        Ok(ServiceMap {
            nodes: nodes.into_values().collect(),
            edges,
        })
    }

    /// Trace-derived RED analytics; works even when a service emits no metrics.
    async fn service_red(
        context: &ApiContext,
        service: Option<String>,
        from_nanos: String,
        to_nanos: String,
        step_seconds: Option<i32>,
    ) -> FieldResult<SpanRed> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        Ok(SpanRed(
            context
                .store
                .span_red_series(
                    service.as_deref().filter(|s| !s.is_empty()),
                    from..=to,
                    step_nanos(step_seconds),
                )
                .await
                .map_err(field_err)?,
        ))
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

    /// Parsed span events across one trace, time ascending. `namePrefix`
    /// filters by event name (for example "rpc.message" or "exception").
    async fn trace_events(
        context: &ApiContext,
        trace_id: String,
        name_prefix: Option<String>,
        limit: Option<i32>,
    ) -> FieldResult<TraceEventsOut> {
        let spans = context
            .store
            .spans_by_trace(&trace_id)
            .await
            .map_err(field_err)?;
        let name_prefix = name_prefix.as_deref().filter(|prefix| !prefix.is_empty());
        Ok(TraceEventsOut(span_events::trace_events(
            &spans,
            name_prefix,
            clamp_limit(limit, 500),
        )))
    }

    /// Summaries for traces referenced by this trace's span links.
    async fn linked_traces(
        context: &ApiContext,
        trace_id: String,
    ) -> FieldResult<Vec<TraceSummary>> {
        let spans = context
            .store
            .spans_by_trace(&trace_id)
            .await
            .map_err(field_err)?;
        let ids = linked_trace_ids(&spans, &trace_id);
        let traces = context.store.traces_by_ids(&ids).await.map_err(field_err)?;
        Ok(traces.into_iter().map(TraceSummary).collect())
    }

    /// Critical-path hops that gate one trace's latency.
    async fn trace_critical_path(
        context: &ApiContext,
        trace_id: String,
    ) -> FieldResult<CriticalPath> {
        let spans = context
            .store
            .spans_by_trace(&trace_id)
            .await
            .map_err(field_err)?;
        if spans.is_empty() {
            return Err(field_err("traceCriticalPath trace has no spans"));
        }
        Ok(CriticalPath(trace_analysis::critical_path(&spans)))
    }

    /// Structural diff between two traces' span trees.
    async fn trace_compare(
        context: &ApiContext,
        trace_id_a: String,
        trace_id_b: String,
    ) -> FieldResult<TraceDiff> {
        let spans_a = context
            .store
            .spans_by_trace(&trace_id_a)
            .await
            .map_err(field_err)?;
        if spans_a.is_empty() {
            return Err(field_err("traceCompare traceIdA has no spans"));
        }
        let spans_b = context
            .store
            .spans_by_trace(&trace_id_b)
            .await
            .map_err(field_err)?;
        if spans_b.is_empty() {
            return Err(field_err("traceCompare traceIdB has no spans"));
        }
        Ok(TraceDiff(trace_analysis::compare(&spans_a, &spans_b)))
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
        let mut trace_indexes: HashMap<String, usize> = HashMap::new();
        for span in spans {
            let trace_id = span.trace_id.clone();
            if let Some(index) = trace_indexes.get(&trace_id).copied() {
                by_trace[index].1.push(span);
            } else {
                trace_indexes.insert(trace_id.clone(), by_trace.len());
                by_trace.push((trace_id, vec![span]));
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

    /// Agent-session projection for one run when gen_ai producer spans exist.
    async fn agent_session(
        context: &ApiContext,
        run_id: String,
    ) -> FieldResult<Option<AgentSessionOut>> {
        let spans = context
            .store
            .spans_by_run(&run_id, MAX_ROWS)
            .await
            .map_err(field_err)?;
        let truncated = spans.len() == MAX_ROWS;
        Ok(agent_session::project_agent_session(&spans)
            .map(|session| AgentSessionOut { session, truncated }))
    }

    /// Deterministic story timeline for exactly one trace or run anchor.
    async fn story(
        context: &ApiContext,
        trace_id: Option<String>,
        run_id: Option<String>,
    ) -> FieldResult<Vec<StoryBeat>> {
        match (trace_id, run_id) {
            (Some(trace_id), None) => {
                let spans = context
                    .store
                    .spans_by_trace(&trace_id)
                    .await
                    .map_err(field_err)?;
                let logs = context
                    .store
                    .logs_by_trace(&trace_id)
                    .await
                    .map_err(field_err)?;
                Ok(story::project_story(&spans, &logs, &[])
                    .into_iter()
                    .map(StoryBeat)
                    .collect())
            }
            (None, Some(run_id)) => {
                let spans = context
                    .store
                    .spans_by_run(&run_id, MAX_ROWS)
                    .await
                    .map_err(field_err)?;
                let logs = context
                    .store
                    .logs_by_run(&run_id, MAX_ROWS)
                    .await
                    .map_err(field_err)?;
                Ok(story::project_story(&spans, &logs, &[])
                    .into_iter()
                    .map(StoryBeat)
                    .collect())
            }
            _ => Err(field_err(
                "story takes exactly one anchor: traceId or runId",
            )),
        }
    }

    /// Missing-evidence detector for exactly one trace or run anchor.
    async fn evidence_gaps(
        context: &ApiContext,
        trace_id: Option<String>,
        run_id: Option<String>,
    ) -> FieldResult<Vec<EvidenceGap>> {
        match (trace_id, run_id) {
            (Some(trace_id), None) => {
                let spans = context
                    .store
                    .spans_by_trace(&trace_id)
                    .await
                    .map_err(field_err)?;
                let logs = context
                    .store
                    .logs_by_trace(&trace_id)
                    .await
                    .map_err(field_err)?;
                Ok(gaps::detect_gaps(&spans, &logs)
                    .into_iter()
                    .map(EvidenceGap)
                    .collect())
            }
            (None, Some(run_id)) => {
                let spans = context
                    .store
                    .spans_by_run(&run_id, MAX_ROWS)
                    .await
                    .map_err(field_err)?;
                let logs = context
                    .store
                    .logs_by_run(&run_id, MAX_ROWS)
                    .await
                    .map_err(field_err)?;
                Ok(gaps::detect_gaps(&spans, &logs)
                    .into_iter()
                    .map(EvidenceGap)
                    .collect())
            }
            _ => Err(field_err(
                "evidenceGaps takes exactly one anchor: traceId or runId",
            )),
        }
    }

    /// Span-attribute overrepresentation in selected vs baseline windows.
    #[allow(clippy::too_many_arguments)]
    async fn attribute_compare(
        context: &ApiContext,
        selected_from_nanos: String,
        selected_to_nanos: String,
        baseline_from_nanos: String,
        baseline_to_nanos: String,
        service: Option<String>,
        error_only: Option<bool>,
        keys: Option<Vec<String>>,
        top_n: Option<i32>,
    ) -> FieldResult<Vec<AttributeCompareRow>> {
        let (selected_from, selected_to) = parse_range(&selected_from_nanos, &selected_to_nanos)?;
        let (baseline_from, baseline_to) = parse_range(&baseline_from_nanos, &baseline_to_nanos)?;
        let limit = clamp_limit(top_n, 10).min(ATTRIBUTE_COMPARE_TOP_N_CAP);
        let keys = keys.unwrap_or_default();
        Ok(context
            .store
            .attribute_compare(
                selected_from..=selected_to,
                baseline_from..=baseline_to,
                service.as_deref().filter(|service| !service.is_empty()),
                error_only.unwrap_or(false),
                &keys,
                limit,
            )
            .await
            .map_err(field_err)?
            .into_iter()
            .map(AttributeCompareRow)
            .collect())
    }

    /// Scalar span/resource attribute keys in a bounded time window.
    async fn field_keys(
        context: &ApiContext,
        from_nanos: String,
        to_nanos: String,
    ) -> FieldResult<Vec<FieldKey>> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        Ok(context
            .store
            .span_field_keys(from..=to)
            .await
            .map_err(field_err)?
            .into_iter()
            .map(FieldKey)
            .collect())
    }

    /// Bounded coverage/cardinality/top-values stats for one field key.
    async fn field_stats(
        context: &ApiContext,
        key: String,
        from_nanos: String,
        to_nanos: String,
        service: Option<String>,
    ) -> FieldResult<FieldStats> {
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let stats = context
            .store
            .span_field_stats(
                key.trim(),
                from..=to,
                service.as_deref().filter(|service| !service.is_empty()),
            )
            .await
            .map_err(field_err)?;
        Ok(FieldStats(stats))
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
        severity_max: Option<i32>,
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
                        severity_max,
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
                && severity_max.is_none_or(|max| l.severity_num <= max)
                && query
                    .as_deref()
                    .is_none_or(|needle| l.body.contains(needle))
        });
        logs.sort_by_key(|l| std::cmp::Reverse(l.ts_nanos));
        logs.truncate(limit);
        Ok(logs.into_iter().map(LogRecord).collect())
    }

    /// Logs surrounding one anchor timestamp, ascending.
    async fn logs_around(
        context: &ApiContext,
        anchor_nanos: String,
        window_seconds: Option<i32>,
        service: Option<String>,
        trace_id: Option<String>,
        limit: Option<i32>,
    ) -> FieldResult<Vec<LogRecord>> {
        let anchor: u128 = anchor_nanos
            .parse()
            .map_err(|_| field_err("invalid anchorNanos"))?;
        let window = u128::try_from(window_seconds.unwrap_or(30).clamp(1, 600)).unwrap_or(30)
            * 1_000_000_000;
        let from = anchor.saturating_sub(window);
        let to = anchor.saturating_add(window);
        let limit = clamp_limit(limit, 200);
        let mut logs =
            if let Some(trace_id) = trace_id.as_deref().filter(|trace_id| !trace_id.is_empty()) {
                context
                    .store
                    .logs_by_trace(trace_id)
                    .await
                    .map_err(field_err)?
                    .into_iter()
                    .filter(|log| {
                        log.ts_nanos >= from
                            && log.ts_nanos <= to
                            && service.as_deref().is_none_or(|svc| log.service == svc)
                    })
                    .collect::<Vec<_>>()
            } else {
                context
                    .store
                    .logs_search(service.as_deref(), from..=to, None, None, None, limit)
                    .await
                    .map_err(field_err)?
            };
        logs.sort_by_key(|log| log.ts_nanos);
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
        if lowered.starts_with("explain") && lowered.contains("analyze") {
            return Err(field_err(
                "EXPLAIN ANALYZE executes the statement and is not allowed; use EXPLAIN",
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
        Ok(cap_sql_result(result, SQL_MAX_ROWS))
    }

    /// Log counts per time bucket under the same filters as `logs` — the
    /// Discover-style histogram above the log table.
    #[allow(clippy::too_many_arguments)]
    async fn log_count_series(
        context: &ApiContext,
        from_nanos: String,
        to_nanos: String,
        service: Option<String>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
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
                severity_max,
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

    /// One saved investigation by id.
    async fn investigation(context: &ApiContext, id: String) -> FieldResult<Option<Investigation>> {
        Ok(context
            .metadata
            .investigation(&id)
            .await
            .map_err(field_err)?
            .map(Investigation))
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

    /// Run ids observed in telemetry (any tool exporting `parallax.run.id`
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
        max_duration_ms: Option<f64>,
        error_only: Option<bool>,
        query: Option<String>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: Option<TraceSort>,
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
            max_duration_ns: max_duration_ms
                .filter(|ms| *ms > 0.0)
                .map(|ms| (ms * 1e6) as u128),
            error_only: error_only.unwrap_or(false),
            name_contains: query.filter(|q| !q.trim().is_empty()),
            limit: clamp_limit(limit, 50),
            offset: offset
                .map_or(0, |value| usize::try_from(value.max(0)).unwrap_or(0))
                .min(MAX_ROWS),
            sort: sort.map(Into::into).unwrap_or_default(),
        };
        let traces = context
            .store
            .traces_search(&trace_query)
            .await
            .map_err(field_err)?;
        Ok(traces.items.into_iter().map(TraceSummary).collect())
    }

    /// Filtered, sorted, paged trace browse with total count for redesigned
    /// trace list clients.
    #[allow(clippy::too_many_arguments)]
    async fn traces_page(
        context: &ApiContext,
        service: Option<String>,
        from_nanos: Option<String>,
        to_nanos: Option<String>,
        min_duration_ms: Option<f64>,
        max_duration_ms: Option<f64>,
        error_only: Option<bool>,
        query: Option<String>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: Option<TraceSort>,
    ) -> FieldResult<TraceList> {
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
            max_duration_ns: max_duration_ms
                .filter(|ms| *ms > 0.0)
                .map(|ms| (ms * 1e6) as u128),
            error_only: error_only.unwrap_or(false),
            name_contains: query.filter(|q| !q.trim().is_empty()),
            limit: clamp_limit(limit, 50),
            offset: offset
                .map_or(0, |value| usize::try_from(value.max(0)).unwrap_or(0))
                .min(MAX_ROWS),
            sort: sort.map(Into::into).unwrap_or_default(),
        };
        let traces = context
            .store
            .traces_search(&trace_query)
            .await
            .map_err(field_err)?;
        Ok(TraceList(traces))
    }

    /// The bounded, redacted, hypothesis-ranked evidence bundle — the agent
    /// handoff artifact assembling trace + logs + metric windows together.
    /// Exactly one anchor: `fingerprint` (issue), `runId`, or `traceId`
    /// (spec §8). Null when the anchor does not exist.
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

        let mut inputs = if let Some(fingerprint) = fingerprint {
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
                metric_windows: Vec::new(),
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
            let mut seen_trace_ids = HashSet::new();
            for span in &spans {
                let trace_id = span.trace_id.clone();
                if seen_trace_ids.insert(trace_id.clone()) {
                    trace_ids.push(trace_id);
                }
            }
            let events = context
                .store
                .error_events_by_traces(&trace_ids, 50)
                .await
                .map_err(field_err)?;
            let mut fingerprints: Vec<String> = Vec::new();
            let mut seen_fingerprints = HashSet::new();
            for event in &events {
                let fingerprint = event.fingerprint.clone();
                if seen_fingerprints.insert(fingerprint.clone()) {
                    fingerprints.push(fingerprint);
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
                metric_windows: Vec::new(),
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
            let mut seen_fingerprints = HashSet::new();
            for event in &events {
                let fingerprint = event.fingerprint.clone();
                if seen_fingerprints.insert(fingerprint.clone()) {
                    fingerprints.push(fingerprint);
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
                metric_windows: Vec::new(),
            }
        };

        inputs.metric_windows = bundle_metric_windows(context, &inputs).await?;
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

    /// Groupable label/tag keys for one metric.
    async fn metric_labels(context: &ApiContext, name: String) -> FieldResult<Vec<String>> {
        validate_metric_name(&name)?;
        context.store.metric_labels(&name).await.map_err(field_err)
    }

    /// Distinct values for one metric label inside a time window.
    async fn metric_label_values(
        context: &ApiContext,
        name: String,
        label: String,
        from_nanos: String,
        to_nanos: String,
    ) -> FieldResult<Vec<String>> {
        validate_metric_name(&name)?;
        validate_metric_group_label(&label)?;
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        context
            .store
            .metric_label_values(&name, &label, from..=to)
            .await
            .map_err(field_err)
    }

    /// Distinct service names (drives the service-overview selector).
    async fn services(context: &ApiContext) -> FieldResult<Vec<String>> {
        context.store.service_names().await.map_err(field_err)
    }

    /// Runtime metric lanes, scoped to exactly one service or run.
    async fn runtime_snapshot(
        context: &ApiContext,
        service: Option<String>,
        run_id: Option<String>,
        from_nanos: String,
        to_nanos: String,
        step_seconds: i32,
    ) -> FieldResult<Vec<RuntimeMetric>> {
        match (service.as_deref(), run_id.as_deref()) {
            (Some(_), Some(_)) | (None, None) => {
                return Err(field_err("runtimeSnapshot takes exactly one scope"));
            }
            _ => {}
        }
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let rows = context
            .store
            .runtime_snapshot(
                service.as_deref(),
                run_id.as_deref(),
                from..=to,
                step_nanos(Some(step_seconds)),
            )
            .await
            .map_err(field_err)?;
        Ok(rows.into_iter().map(RuntimeMetric).collect())
    }

    /// Aggregated series for a point metric (gauge/sum); agg one of
    /// avg|min|max|sum|rate. With `groupBy` (an attribute key) one series
    /// per value; without it a single series with a null `groupValue`
    /// (spec §8 `metricSeries`). `runId` scopes to points whose resource
    /// carried `parallax.run.id` (run-anchored cross-analytics).
    #[allow(clippy::too_many_arguments)]
    async fn metric_series(
        context: &ApiContext,
        name: String,
        from_nanos: String,
        to_nanos: String,
        service: Option<String>,
        run_id: Option<String>,
        group_by: Option<String>,
        step_seconds: Option<i32>,
        agg: Option<String>,
    ) -> FieldResult<Vec<Series>> {
        validate_metric_name(&name)?;
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let agg = MetricAgg::parse(agg.as_deref().unwrap_or("avg"))
            .ok_or_else(|| field_err("agg must be avg|min|max|sum|rate"))?;
        match group_by {
            Some(group_by) => {
                validate_metric_group_label(&group_by)?;
                if run_id.is_some() {
                    return Err(field_err("runId with groupBy is not supported yet"));
                }
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
                        run_id.as_deref(),
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
        validate_metric_name(&name)?;
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

    /// Trace-linked exemplars for one metric, newest first.
    async fn metric_exemplars(
        context: &ApiContext,
        name: String,
        from_nanos: String,
        to_nanos: String,
        service: Option<String>,
        limit: Option<i32>,
    ) -> FieldResult<Vec<MetricExemplar>> {
        validate_metric_name(&name)?;
        let (from, to) = parse_range(&from_nanos, &to_nanos)?;
        let rows = context
            .store
            .metric_exemplars(&name, service.as_deref(), from..=to, clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(rows.into_iter().map(MetricExemplar).collect())
    }

    /// Saved user dashboards, most recently updated first.
    async fn dashboards(context: &ApiContext) -> FieldResult<Vec<Dashboard>> {
        let dashboards = context.metadata.dashboards().await.map_err(field_err)?;
        Ok(dashboards.into_iter().map(Dashboard).collect())
    }

    /// Saved investigations/cases, most recently updated first.
    async fn investigations(context: &ApiContext) -> FieldResult<Vec<Investigation>> {
        let investigations = context.metadata.investigations().await.map_err(field_err)?;
        Ok(investigations.into_iter().map(Investigation).collect())
    }

    /// Named saved page states, most recently updated first.
    async fn saved_views(
        context: &ApiContext,
        page: Option<String>,
    ) -> FieldResult<Vec<SavedView>> {
        let saved_views = context
            .metadata
            .saved_views(page.as_deref().filter(|page| !page.is_empty()))
            .await
            .map_err(field_err)?;
        Ok(saved_views.into_iter().map(SavedView).collect())
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

/// Fetch the anchor's metric windows: run anchors read run-scoped points
/// over the run's lifespan (5 s steps); issue/trace anchors read a
/// ±5-minute window around the anchor event (30 s steps), run-scoped when
/// the anchor's spans carry a run id, service-scoped otherwise.
async fn bundle_metric_windows(
    context: &ApiContext,
    inputs: &parallax_core::bundle::BundleInputs,
) -> FieldResult<Vec<parallax_core::bundle::MetricWindow>> {
    use parallax_core::bundle::{BundleAnchor, MetricWindow};
    const PAD_NANOS: u128 = 5 * 60 * 1_000_000_000;
    let (from, to, step_seconds, run_scope, service) = match &inputs.anchor {
        BundleAnchor::Run { run, .. } => {
            let last_activity = inputs
                .trace_logs
                .iter()
                .map(|l| l.ts_nanos)
                .chain(
                    inputs
                        .trace_spans
                        .iter()
                        .map(|s| s.ts_nanos + s.duration_ns),
                )
                .max();
            let start = run.started_at_nanos;
            let end = run
                .ended_at_nanos
                .into_iter()
                .chain(last_activity)
                .max()
                .unwrap_or(start);
            (
                start.saturating_sub(5_000_000_000),
                end + 30_000_000_000,
                5u32,
                Some(run.run_id.clone()),
                None,
            )
        }
        _ => {
            let anchor_ts = inputs
                .events
                .first()
                .map(|e| e.ts_nanos)
                .or_else(|| inputs.trace_spans.first().map(|s| s.ts_nanos));
            let Some(anchor_ts) = anchor_ts else {
                return Ok(Vec::new());
            };
            let run_id = inputs.trace_spans.iter().find_map(|s| s.run_id.clone());
            let service = inputs
                .trace_spans
                .first()
                .map(|s| s.service.clone())
                .or_else(|| inputs.events.first().map(|e| e.service.clone()));
            (
                anchor_ts.saturating_sub(PAD_NANOS),
                anchor_ts + PAD_NANOS,
                30u32,
                run_id,
                service,
            )
        }
    };
    let scope = if run_scope.is_some() {
        "run"
    } else {
        "service"
    };
    let mut windows = Vec::new();
    for metric in semconv::BUNDLE_WINDOW_METRICS {
        let points = context
            .store
            .metric_series(
                metric,
                service.as_deref(),
                run_scope.as_deref(),
                from..=to,
                u128::from(step_seconds) * 1_000_000_000,
                MetricAgg::Avg,
            )
            .await
            .map_err(field_err)?;
        if let Some(window) = MetricWindow::from_points(
            *metric,
            scope,
            from,
            to,
            step_seconds,
            points.into_iter().map(|p| (p.ts_nanos, p.value)).collect(),
        ) {
            windows.push(window);
        }
    }
    Ok(windows)
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

    /// Create or update an investigation/case state.
    async fn investigation_save(
        context: &ApiContext,
        name: String,
        state: String,
        id: Option<String>,
    ) -> FieldResult<Investigation> {
        let name = validate_investigation_name(&name)?;
        validate_investigation_state(&state)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let id = id
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| format!("case_{now:x}"));
        context
            .metadata
            .investigation_save(&id, &name, &state, now)
            .await
            .map_err(field_err)?;
        context
            .metadata
            .investigation(&id)
            .await
            .map_err(field_err)?
            .map(Investigation)
            .ok_or_else(|| field_err("investigation save did not persist"))
    }

    /// Delete an investigation/case.
    async fn investigation_delete(context: &ApiContext, id: String) -> FieldResult<bool> {
        context
            .metadata
            .investigation_delete(&id)
            .await
            .map_err(field_err)
    }

    /// Create or update a named saved page state.
    async fn saved_view_save(
        context: &ApiContext,
        name: String,
        page: String,
        state: String,
        id: Option<String>,
    ) -> FieldResult<SavedView> {
        let name = validate_saved_view_name(&name)?;
        validate_saved_view_page(&page)?;
        let existing = match id.as_deref().filter(|id| !id.is_empty()) {
            Some(id) => context.metadata.saved_view(id).await.map_err(field_err)?,
            None => None,
        };
        if existing.as_ref().is_none_or(|view| view.page != page)
            && context
                .metadata
                .saved_views(Some(&page))
                .await
                .map_err(field_err)?
                .len()
                >= SAVED_VIEWS_PER_PAGE
        {
            return Err(field_err("saved view cap reached for page"));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let id = id
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| format!("view_{now:x}"));
        context
            .metadata
            .saved_view_save(&id, &name, &page, &state, now)
            .await
            .map_err(field_err)?;
        context
            .metadata
            .saved_view(&id)
            .await
            .map_err(field_err)?
            .map(SavedView)
            .ok_or_else(|| field_err("saved view save did not persist"))
    }

    /// Delete a named saved page state.
    async fn saved_view_delete(context: &ApiContext, id: String) -> FieldResult<bool> {
        context
            .metadata
            .saved_view_delete(&id)
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

#[derive(Debug, Default)]
struct QueryShape {
    max_depth: usize,
    field_count: usize,
}

type ParsedSelection<'a> = juniper::Selection<'a, juniper::DefaultScalarValue>;

fn walk_selections<'a>(
    selections: &[ParsedSelection<'a>],
    fragments: &BTreeMap<&'a str, Vec<ParsedSelection<'a>>>,
    depth: usize,
    stats: &mut QueryShape,
    fragment_stack: &mut Vec<&'a str>,
) -> Result<(), String> {
    for selection in selections {
        match selection {
            juniper::Selection::Field(field) => {
                let field_depth = depth + 1;
                stats.max_depth = stats.max_depth.max(field_depth);
                stats.field_count += 1;
                if let Some(children) = &field.item.selection_set {
                    walk_selections(children, fragments, field_depth, stats, fragment_stack)?;
                }
            }
            juniper::Selection::InlineFragment(fragment) => {
                walk_selections(
                    &fragment.item.selection_set,
                    fragments,
                    depth,
                    stats,
                    fragment_stack,
                )?;
            }
            juniper::Selection::FragmentSpread(spread) => {
                let name = spread.item.name.item;
                if fragment_stack.contains(&name) {
                    return Err(format!("GraphQL fragment cycle includes `{name}`"));
                }
                if let Some(fragment) = fragments.get(name) {
                    fragment_stack.push(name);
                    walk_selections(fragment, fragments, depth, stats, fragment_stack)?;
                    fragment_stack.pop();
                }
            }
        }
    }
    Ok(())
}

/// Enforce coarse query-cost ceilings before Juniper execution.
///
/// Juniper 0.17 has no built-in depth/complexity middleware. Depth is selected
/// field nesting; complexity is approximated as total selected fields,
/// including fragment expansions.
pub fn check_query_limits(
    schema: &Schema,
    query: &str,
    operation_name: Option<&str>,
    max_depth: usize,
    max_complexity: usize,
) -> Result<(), String> {
    let document = juniper::parser::parse_document_source::<juniper::DefaultScalarValue>(
        query,
        &schema.schema,
    )
    .map_err(|error| format!("GraphQL query parse failed: {error}"))?;
    let fragments: BTreeMap<_, Vec<_>> = document
        .iter()
        .filter_map(|definition| match definition {
            juniper::Definition::Fragment(fragment) => {
                Some((fragment.item.name.item, fragment.item.selection_set.clone()))
            }
            _ => None,
        })
        .collect();

    let mut stats = QueryShape::default();
    let mut matched_operation = false;
    for definition in &document {
        let juniper::Definition::Operation(operation) = definition else {
            continue;
        };
        let op_name = operation.item.name.as_ref().map(|name| name.item);
        if operation_name.is_some_and(|wanted| op_name != Some(wanted)) {
            continue;
        }
        matched_operation = true;
        walk_selections(
            &operation.item.selection_set,
            &fragments,
            0,
            &mut stats,
            &mut Vec::new(),
        )?;
    }

    if !matched_operation && operation_name.is_some() {
        return Ok(());
    }
    if stats.max_depth > max_depth {
        return Err(format!(
            "GraphQL query depth {} exceeds configured maximum {}",
            stats.max_depth, max_depth
        ));
    }
    if stats.field_count > max_complexity {
        return Err(format!(
            "GraphQL query field count {} exceeds configured maximum {}",
            stats.field_count, max_complexity
        ));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_storage::adapter::{SqlResult, TelemetryStore};
    use parallax_storage::memory::MemoryStore;
    use parallax_storage::model::{
        ErrorEventRow, ErrorSource, LogRow, MetricExemplarRow, MetricPointRow, SpanRow,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DB_SEQ: AtomicU64 = AtomicU64::new(0);

    fn span(
        service: &str,
        trace_id: &str,
        span_id: &str,
        ts_nanos: u128,
        duration_ns: u128,
    ) -> SpanRow {
        SpanRow {
            ts_nanos,
            service: service.into(),
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
            name: "handler".into(),
            kind: "SPAN_KIND_SERVER".into(),
            status_code: "STATUS_CODE_UNSET".into(),
            status_message: String::new(),
            duration_ns,
            run_id: None,
            scope_name: String::new(),
            events: None,
            links: serde_json::Value::Null,
            attributes: serde_json::Value::Null,
            resource: serde_json::Value::Null,
        }
    }

    fn log_row(service: &str, trace_id: &str, ts_nanos: u128, body: &str) -> LogRow {
        LogRow {
            ts_nanos,
            event_name: String::new(),
            observed_ts_nanos: 0,
            service: service.into(),
            severity_num: 9,
            severity_text: "INFO".into(),
            body: body.into(),
            trace_id: trace_id.into(),
            span_id: format!("span-{ts_nanos}"),
            run_id: None,
            scope_name: String::new(),
            attributes: serde_json::Value::Null,
            resource: serde_json::Value::Null,
        }
    }

    fn span_with_release(
        service: &str,
        trace_id: &str,
        span_id: &str,
        ts_nanos: u128,
        version: &str,
    ) -> SpanRow {
        let mut row = span(service, trace_id, span_id, ts_nanos, 1_000);
        row.resource = serde_json::json!({ "service.version": version });
        row
    }

    async fn context_with_memory(store: Arc<MemoryStore>) -> ApiContext {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "parallax-api-test-{}-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            TEST_DB_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&path);
        let metadata = MetadataStore::open(&path).await.unwrap();
        ApiContext {
            store,
            metadata: Arc::new(metadata),
            otlp_grpc_port: 4317,
        }
    }

    fn error_messages(json: &serde_json::Value) -> Vec<String> {
        json.pointer("/errors")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
            .filter_map(|error| error.get("message").and_then(|message| message.as_str()))
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn cap_sql_result_truncates_rows_and_flags_over_cap_only() {
        let result = SqlResult {
            columns: vec!["n".into()],
            rows: vec![vec![serde_json::json!(1)], vec![serde_json::json!(2)]],
        };
        let under = cap_sql_result(result.clone(), 3);
        assert!(!under.truncated());
        assert_eq!(under.row_count(), 2);

        let at = cap_sql_result(result.clone(), 2);
        assert!(!at.truncated());
        assert_eq!(at.row_count(), 2);

        let over = cap_sql_result(result, 1);
        assert!(over.truncated());
        assert_eq!(over.row_count(), 1);
        assert_eq!(over.rows(), vec!["[1]"]);
    }

    #[tokio::test]
    async fn sql_guard_rejects_explain_analyze_but_allows_select_shape() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let analyze = juniper::http::GraphQLRequest::new(
            r#"{ sql(query: "EXPLAIN ANALYZE SELECT 1") { rowCount } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, analyze).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("EXPLAIN ANALYZE executes the statement")),
            "EXPLAIN ANALYZE rejected by GraphQL guard: {json}"
        );

        let select = juniper::http::GraphQLRequest::new(
            r#"{ sql(query: "SELECT 1") { rowCount } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, select).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("in-memory store")),
            "SELECT passes API guard and reaches memory adapter: {json}"
        );
    }

    #[tokio::test]
    async fn trace_events_filters_orders_and_reports_caps() {
        let store = Arc::new(MemoryStore::new());
        let mut root = span("checkout", "trace-a", "span-a", 1_000, 100);
        root.name = "root".into();
        root.events = Some(
            r#"[
                {"name":"exception","time_unix_nano":30,"attributes":{"message":"bad"}},
                {"name":"rpc.message.sent","timeUnixNano":"10","attributes":{"message.type":"SENT","id":7}}
            ]"#
            .into(),
        );
        let mut child = span("payments", "trace-a", "span-b", 2_000, 100);
        child.name = "client".into();
        child.events = Some(
            r#"[{"name":"rpc.message.received","time_unix_nano":20,"attributes":{"message.type":"RECEIVED"}}]"#
                .into(),
        );
        store
            .ingest_traces(vec![root, child], Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              traceEvents(traceId: "trace-a", namePrefix: "rpc.message", limit: 1) {
                truncated
                skippedSpans
                events { name spanId spanName service timeUnixNano attributes }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "traceEvents query succeeds: {json}"
        );
        assert_eq!(
            json.pointer("/data/traceEvents/truncated"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            json.pointer("/data/traceEvents/skippedSpans"),
            Some(&serde_json::json!(0))
        );
        assert_eq!(
            json.pointer("/data/traceEvents/events/0/name"),
            Some(&serde_json::json!("rpc.message.sent"))
        );
        assert_eq!(
            json.pointer("/data/traceEvents/events/0/spanId"),
            Some(&serde_json::json!("span-a"))
        );
        assert_eq!(
            json.pointer("/data/traceEvents/events/0/timeUnixNano"),
            Some(&serde_json::json!("10"))
        );
        assert_eq!(
            json.pointer("/data/traceEvents/events/0/attributes"),
            Some(&serde_json::json!(r#"{"id":"7","message.type":"SENT"}"#))
        );
    }

    #[tokio::test]
    async fn trace_events_counts_malformed_span_events() {
        let store = Arc::new(MemoryStore::new());
        let mut good = span("checkout", "trace-a", "span-a", 1_000, 100);
        good.events =
            Some(r#"[{"name":"rpc.message","time_unix_nano":10,"attributes":{}}]"#.into());
        let mut bad = span("checkout", "trace-a", "span-b", 2_000, 100);
        bad.events = Some("{not json".into());
        store
            .ingest_traces(vec![good, bad], Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              traceEvents(traceId: "trace-a") {
                truncated
                skippedSpans
                events { name spanId }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "traceEvents malformed span query succeeds: {json}"
        );
        assert_eq!(
            json.pointer("/data/traceEvents/skippedSpans"),
            Some(&serde_json::json!(1))
        );
        assert_eq!(
            json.pointer("/data/traceEvents/truncated"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            json.pointer("/data/traceEvents/events/0/name"),
            Some(&serde_json::json!("rpc.message"))
        );
    }

    #[tokio::test]
    async fn agent_session_projects_run_scoped_agent_spans() {
        let store = Arc::new(MemoryStore::new());
        let mut root = span("agent", "trace-agent", "root", 1_000, 100);
        root.name = "invoke_agent".into();
        root.run_id = Some("run-agent".into());
        root.attributes = serde_json::json!({
            "gen_ai.operation.name": "invoke_agent"
        });
        let mut tool = span("agent", "trace-agent", "tool", 1_100, 25);
        tool.name = "execute_tool".into();
        tool.parent_span_id = Some("root".into());
        tool.run_id = Some("run-agent".into());
        tool.attributes = serde_json::json!({
            "gen_ai.operation.name": "execute_tool",
            "tool.name": "inspect_repo",
            "gen_ai.usage.input_tokens": "7"
        });
        let mut shell = span("agent", "trace-agent", "shell", 1_200, 25);
        shell.name = "execute_tool".into();
        shell.parent_span_id = Some("root".into());
        shell.run_id = Some("run-agent".into());
        shell.status_code = "STATUS_CODE_ERROR".into();
        shell.attributes = serde_json::json!({
            "gen_ai.operation.name": "execute_tool",
            "tool.name": "shell_command",
            "shell.command": "false",
            "gen_ai.usage.output_tokens": 3
        });
        let mut unrelated = span("agent", "trace-other", "other", 1_050, 10);
        unrelated.name = "execute_tool".into();
        unrelated.run_id = Some("run-other".into());
        store
            .ingest_traces(vec![shell, unrelated, root, tool], Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              agentSession(runId: "run-agent") {
                rootSpanId
                truncated
                totalInputTokens
                totalOutputTokens
                errorCount
                steps {
                  kind name spanId traceId startNanos durationNs isError
                  genAiOperation inputTokens outputTokens
                }
              }
              unrelated: agentSession(runId: "run-other") { rootSpanId }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "agentSession query succeeds: {json}"
        );
        assert_eq!(
            json.pointer("/data/agentSession/rootSpanId"),
            Some(&serde_json::json!("root"))
        );
        assert_eq!(
            json.pointer("/data/agentSession/truncated"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            json.pointer("/data/agentSession/totalInputTokens"),
            Some(&serde_json::json!("7"))
        );
        assert_eq!(
            json.pointer("/data/agentSession/totalOutputTokens"),
            Some(&serde_json::json!("3"))
        );
        assert_eq!(
            json.pointer("/data/agentSession/errorCount"),
            Some(&serde_json::json!(1))
        );
        assert_eq!(
            json.pointer("/data/agentSession/steps/0/kind"),
            Some(&serde_json::json!("INVOKE_AGENT"))
        );
        assert_eq!(
            json.pointer("/data/agentSession/steps/1/name"),
            Some(&serde_json::json!("inspect_repo"))
        );
        assert_eq!(
            json.pointer("/data/agentSession/steps/2/kind"),
            Some(&serde_json::json!("SHELL"))
        );
        assert_eq!(
            json.pointer("/data/agentSession/steps/2/isError"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            json.pointer("/data/unrelated"),
            Some(&serde_json::Value::Null)
        );
    }

    #[tokio::test]
    async fn metric_name_validation_rejects_identifier_breakout() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let invalid = juniper::http::GraphQLRequest::new(
            r#"{ metricSeries(name: "evil\"name", fromNanos: "0", toNanos: "1") { groupValue } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, invalid).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("invalid metric name")),
            "invalid metric name rejected: {json}"
        );

        let valid = juniper::http::GraphQLRequest::new(
            r#"{ metricSeries(name: "http.server.request.duration", fromNanos: "0", toNanos: "1") { groupValue points { value } } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, valid).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json).is_empty(),
            "legal OTel metric name accepted: {json}"
        );
        assert!(
            json.pointer("/data/metricSeries")
                .and_then(|value| value.as_array())
                .is_some(),
            "metricSeries returns data for valid name: {json}"
        );
    }

    #[tokio::test]
    async fn metric_label_and_runtime_resolvers_query_memory_store() {
        let store = Arc::new(MemoryStore::new());
        store
            .ingest_metrics(
                {
                    let mut points = vec![
                        MetricPointRow {
                            ts_nanos: 1_000_000_000,
                            service: "checkout".into(),
                            name: "process.cpu.utilization".into(),
                            value: 0.5,
                            is_monotonic: false,
                            run_id: Some("run-a".into()),
                            attributes: serde_json::json!({
                                "runtime.name": "tokio",
                                "payment.method": "card",
                                "trace_id": "trace-a"
                            }),
                        },
                        MetricPointRow {
                            ts_nanos: 2_000_000_000,
                            service: "checkout".into(),
                            name: "jvm.memory.used".into(),
                            value: 256.0,
                            is_monotonic: false,
                            run_id: None,
                            attributes: serde_json::json!({
                                "runtime.name": "jvm"
                            }),
                        },
                    ];
                    for index in 0..110 {
                        points.push(MetricPointRow {
                            ts_nanos: 2_100_000_000 + index,
                            service: "checkout".into(),
                            name: "process.cpu.utilization".into(),
                            value: index as f64,
                            is_monotonic: false,
                            run_id: None,
                            attributes: serde_json::json!({
                                "runtime.name": format!("runtime-{index:03}")
                            }),
                        });
                    }
                    points
                },
                Vec::new(),
                Vec::new(),
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              metricLabels(name: "process.cpu.utilization")
              metricLabelValues(name: "process.cpu.utilization", label: "payment.method", fromNanos: "0", toNanos: "3000000000")
              cappedMetricLabelValues: metricLabelValues(name: "process.cpu.utilization", label: "runtime.name", fromNanos: "0", toNanos: "3000000000")
              runtimeSnapshot(service: "checkout", fromNanos: "0", toNanos: "3000000000", stepSeconds: 1) {
                family metric unit points { tsNanos value }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "metric label/runtime query: {json}"
        );
        assert_eq!(
            json.pointer("/data/metricLabels"),
            Some(&serde_json::json!(["payment.method", "runtime.name"]))
        );
        assert_eq!(
            json.pointer("/data/metricLabelValues"),
            Some(&serde_json::json!(["card"]))
        );
        assert_eq!(
            json.pointer("/data/cappedMetricLabelValues")
                .and_then(|value| value.as_array())
                .map(Vec::len),
            Some(100)
        );
        let runtime = json
            .pointer("/data/runtimeSnapshot")
            .and_then(|value| value.as_array())
            .unwrap();
        assert_eq!(runtime.len(), 2, "two runtime families returned: {json}");
        assert!(runtime.iter().any(|row| row["family"] == "process"));
        assert!(runtime.iter().any(|row| row["family"] == "jvm"));

        let denied = juniper::http::GraphQLRequest::new(
            r#"{ metricSeries(name: "process.cpu.utilization", fromNanos: "0", toNanos: "3000000000", groupBy: "trace_id") { groupValue } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, denied).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("high-cardinality identifier")),
            "denylisted groupBy rejected: {json}"
        );
    }

    #[tokio::test]
    async fn overview_service_analytics_queries_execute_against_memory_store() {
        let store = Arc::new(MemoryStore::new());
        let mut errored = span("api", "t1", "b", 1_500_000_000, 30_000_000);
        errored.status_code = "STATUS_CODE_ERROR".into();
        store
            .ingest_traces(
                vec![span("api", "t1", "a", 1_000_000_000, 10_000_000), errored],
                Default::default(),
            )
            .await
            .unwrap();
        store
            .ingest_logs(
                vec![LogRow {
                    ts_nanos: 1_250_000_000,
                    event_name: "checkout.failed".into(),
                    observed_ts_nanos: 1_300_000_000,
                    service: "api".into(),
                    severity_num: 17,
                    severity_text: "ERROR".into(),
                    body: "bad".into(),
                    trace_id: "t1".into(),
                    span_id: "b".into(),
                    run_id: None,
                    scope_name: String::new(),
                    attributes: serde_json::Value::Null,
                    resource: serde_json::Value::Null,
                }],
                Default::default(),
            )
            .await
            .unwrap();
        store
            .write_error_events(vec![ErrorEventRow {
                ts_nanos: 1_600_000_000,
                service: "api".into(),
                fingerprint: "fp".into(),
                error_type: "Error".into(),
                message: "bad".into(),
                stacktrace: None,
                source: ErrorSource::SpanStatus,
                trace_id: "t1".into(),
                span_id: "b".into(),
                attributes: serde_json::Value::Null,
            }])
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              overview(fromNanos: "0", toNanos: "2000000000") {
                spanCount traceCount logCount errorCount errorRate activeServices
              }
              signalCountSeries(kind: SPANS, service: "api", fromNanos: "0", toNanos: "2000000000", stepSeconds: 1) {
                tsNanos value
              }
              serviceList(fromNanos: "0", toNanos: "2000000000") {
                name lastSeenNanos spanCount errorCount p95Ms
              }
              serviceRed(service: "api", fromNanos: "0", toNanos: "2000000000", stepSeconds: 1) {
                rate { tsNanos value }
                errorRate { value }
                p95 { value }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json.pointer("/data/overview/spanCount"),
            Some(&serde_json::json!("2"))
        );
        assert_eq!(
            json.pointer("/data/overview/traceCount"),
            Some(&serde_json::json!("1"))
        );
        assert_eq!(
            json.pointer("/data/overview/logCount"),
            Some(&serde_json::json!("1"))
        );
        assert_eq!(
            json.pointer("/data/overview/errorCount"),
            Some(&serde_json::json!("1"))
        );
        assert_eq!(
            json.pointer("/data/signalCountSeries/0/tsNanos"),
            Some(&serde_json::json!("1000000000"))
        );
        assert_eq!(
            json.pointer("/data/signalCountSeries/0/value"),
            Some(&serde_json::json!(2.0))
        );
        assert_eq!(
            json.pointer("/data/serviceList/0/name"),
            Some(&serde_json::json!("api"))
        );
        assert_eq!(
            json.pointer("/data/serviceList/0/spanCount"),
            Some(&serde_json::json!("2"))
        );
        assert_eq!(
            json.pointer("/data/serviceRed/rate/0/value"),
            Some(&serde_json::json!(2.0))
        );
        assert_eq!(
            Overview(OverviewTotals {
                span_count: i32::MAX as u64 + 1,
                trace_count: 0,
                log_count: 0,
                metric_point_count: 0,
                error_count: 0,
                error_rate: 0.0,
                active_services: 0,
            })
            .span_count(),
            "2147483648"
        );
    }

    #[tokio::test]
    async fn logs_around_returns_windowed_ascending_rows() {
        let store = Arc::new(MemoryStore::new());
        let anchor = 100_000_000_000;
        let mut anchor_log = log_row("api", "trace-a", anchor, "anchor");
        anchor_log.event_name = "checkout.completed".into();
        anchor_log.observed_ts_nanos = anchor + 2_000_000_000;
        store
            .ingest_logs(
                vec![
                    log_row("api", "trace-a", anchor - 60_000_000_000, "too-old"),
                    log_row("api", "trace-a", anchor - 10_000_000_000, "before"),
                    anchor_log,
                    log_row("api", "trace-a", anchor + 10_000_000_000, "after"),
                    log_row("api", "trace-a", anchor + 60_000_000_000, "too-new"),
                ],
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            format!(
                r#"{{
                  logsAround(anchorNanos: "{anchor}", windowSeconds: 30, service: "api") {{
                    tsNanos body eventName observedTsNanos
                  }}
                }}"#
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "logsAround query: {json}");
        let rows = json
            .pointer("/data/logsAround")
            .and_then(|value| value.as_array())
            .unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| row["body"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["before", "anchor", "after"]
        );
        assert_eq!(
            rows[1].get("eventName"),
            Some(&serde_json::json!("checkout.completed"))
        );
        assert_eq!(
            rows[1].get("observedTsNanos"),
            Some(&serde_json::json!("102000000000"))
        );
    }

    #[tokio::test]
    async fn logs_around_can_scope_to_trace_inside_window() {
        let store = Arc::new(MemoryStore::new());
        let anchor = 100_000_000_000;
        store
            .ingest_logs(
                vec![
                    log_row("api", "trace-a", anchor - 1_000_000_000, "trace-a-before"),
                    log_row("api", "trace-b", anchor, "trace-b-anchor"),
                    log_row("api", "trace-a", anchor + 1_000_000_000, "trace-a-after"),
                ],
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            format!(
                r#"{{
                  logsAround(anchorNanos: "{anchor}", windowSeconds: 30, traceId: "trace-a") {{
                    body traceId
                  }}
                }}"#
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "logsAround trace: {json}");
        assert_eq!(
            json.pointer("/data/logsAround")
                .and_then(|value| value.as_array())
                .unwrap()
                .iter()
                .map(|row| row["body"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["trace-a-before", "trace-a-after"]
        );
    }

    #[tokio::test]
    async fn logs_around_clamps_window_and_limit() {
        let store = Arc::new(MemoryStore::new());
        let anchor = 1_000_000_000_000;
        let mut rows = (0..550)
            .map(|index| {
                log_row(
                    "api",
                    "trace-a",
                    anchor + index * 1_000_000,
                    &format!("near-{index}"),
                )
            })
            .collect::<Vec<_>>();
        rows.push(log_row(
            "api",
            "trace-a",
            anchor + 700_000_000_000,
            "beyond-clamped-window",
        ));
        store.ingest_logs(rows, Default::default()).await.unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            format!(
                r#"{{
                  logsAround(anchorNanos: "{anchor}", windowSeconds: 9999, limit: 9999) {{
                    body
                  }}
                }}"#
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "logsAround clamp: {json}");
        let rows = json
            .pointer("/data/logsAround")
            .and_then(|value| value.as_array())
            .unwrap();
        assert_eq!(rows.len(), MAX_ROWS);
        assert!(
            rows.iter()
                .all(|row| row["body"] != "beyond-clamped-window")
        );
    }

    #[tokio::test]
    async fn saved_view_resolvers_round_trip_filter_delete_and_cap() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let save = juniper::http::GraphQLRequest::new(
            r#"
            mutation {
              savedViewSave(name: "Errors", page: "/logs", state: "?sev=17&cols=trace") {
                id name page state
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, save).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "savedViewSave: {json}");
        let id = json
            .pointer("/data/savedViewSave/id")
            .and_then(|value| value.as_str())
            .unwrap()
            .to_string();
        assert_eq!(
            json.pointer("/data/savedViewSave/state"),
            Some(&serde_json::json!("?sev=17&cols=trace"))
        );

        let list = juniper::http::GraphQLRequest::new(
            r#"{ savedViews(page: "/logs") { id name page state } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, list).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "savedViews: {json}");
        assert_eq!(
            json.pointer("/data/savedViews/0/id"),
            Some(&serde_json::json!(id.as_str()))
        );

        let delete = juniper::http::GraphQLRequest::new(
            format!(r#"mutation {{ savedViewDelete(id: "{id}") }}"#),
            None,
            None,
        );
        let response = execute(&schema, &context, delete).await;
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json.pointer("/data/savedViewDelete"),
            Some(&serde_json::json!(true))
        );

        for index in 0..SAVED_VIEWS_PER_PAGE {
            context
                .metadata
                .saved_view_save(
                    &format!("view-{index}"),
                    "View",
                    "/logs",
                    "?q=x",
                    index as u128,
                )
                .await
                .unwrap();
        }
        let capped = juniper::http::GraphQLRequest::new(
            r#"mutation { savedViewSave(name: "Too many", page: "/logs", state: "?q=y") { id } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, capped).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("saved view cap")),
            "saved view cap enforced: {json}"
        );
    }

    #[tokio::test]
    async fn investigation_resolvers_round_trip_and_validate_state() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let state = r#"{"version":1,"window":{"range":"24h"},"pins":[{"kind":"trace","ref":"/traces/t1","label":"trace"}],"notes":"triage"}"#;
        let save = juniper::http::GraphQLRequest::new(
            format!(
                r#"
                mutation {{
                  investigationSave(name: "Checkout case", state: "{}") {{
                    id name state
                  }}
                }}
                "#,
                state.replace('"', "\\\"")
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, save).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json).is_empty(),
            "investigationSave: {json}"
        );
        let id = json
            .pointer("/data/investigationSave/id")
            .and_then(|value| value.as_str())
            .unwrap()
            .to_string();
        assert_eq!(
            json.pointer("/data/investigationSave/name"),
            Some(&serde_json::json!("Checkout case"))
        );

        let list = juniper::http::GraphQLRequest::new(
            r#"{ investigations { id name state updatedAtNanos } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, list).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "investigations: {json}");
        assert_eq!(
            json.pointer("/data/investigations/0/id"),
            Some(&serde_json::json!(id.as_str()))
        );

        let get = juniper::http::GraphQLRequest::new(
            format!(r#"{{ investigation(id: "{id}") {{ id name state }} }}"#),
            None,
            None,
        );
        let response = execute(&schema, &context, get).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(error_messages(&json).is_empty(), "investigation: {json}");
        assert_eq!(
            json.pointer("/data/investigation/id"),
            Some(&serde_json::json!(id.as_str()))
        );

        let delete = juniper::http::GraphQLRequest::new(
            format!(r#"mutation {{ investigationDelete(id: "{id}") }}"#),
            None,
            None,
        );
        let response = execute(&schema, &context, delete).await;
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json.pointer("/data/investigationDelete"),
            Some(&serde_json::json!(true))
        );

        let bad_json = juniper::http::GraphQLRequest::new(
            r#"mutation { investigationSave(name: "Bad", state: "{bad json") { id } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, bad_json).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("state must be valid JSON")),
            "bad JSON rejected: {json}"
        );

        let pins = (0..=INVESTIGATION_PIN_CAP)
            .map(|index| {
                serde_json::json!({
                    "kind": "trace",
                    "ref": format!("/traces/{index}"),
                    "label": format!("trace {index}")
                })
            })
            .collect::<Vec<_>>();
        let capped_state = serde_json::json!({
            "version": 1,
            "window": {"range": "24h"},
            "pins": pins,
            "notes": ""
        })
        .to_string();
        let capped = juniper::http::GraphQLRequest::new(
            format!(
                r#"mutation {{ investigationSave(name: "Too many", state: "{}") {{ id }} }}"#,
                capped_state.replace('"', "\\\"")
            ),
            None,
            None,
        );
        let response = execute(&schema, &context, capped).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("pin cap")),
            "pin cap enforced: {json}"
        );
    }

    #[tokio::test]
    async fn releases_resolver_returns_service_windows() {
        let store = Arc::new(MemoryStore::new());
        store
            .ingest_traces(
                vec![
                    span_with_release("checkout", "t1", "a", 10, "v1"),
                    span_with_release("checkout", "t2", "a", 30, "v1"),
                    span_with_release("checkout", "t3", "a", 50, "v2"),
                    span_with_release("catalog", "t4", "a", 20, "v9"),
                ],
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              releases(service: "checkout", fromNanos: "0", toNanos: "100") {
                version firstSeenNanos lastSeenNanos spanCount
              }
            }
            "#
            .into(),
            None,
            None,
        );

        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(error_messages(&json).is_empty(), "releases query: {json}");
        assert_eq!(
            json.pointer("/data/releases/0/version"),
            Some(&serde_json::json!("v1"))
        );
        assert_eq!(
            json.pointer("/data/releases/0/firstSeenNanos"),
            Some(&serde_json::json!("10"))
        );
        assert_eq!(
            json.pointer("/data/releases/0/lastSeenNanos"),
            Some(&serde_json::json!("30"))
        );
        assert_eq!(
            json.pointer("/data/releases/0/spanCount"),
            Some(&serde_json::json!("2"))
        );
        assert_eq!(
            json.pointer("/data/releases/1/version"),
            Some(&serde_json::json!("v2"))
        );
    }

    #[tokio::test]
    async fn service_catalog_resolver_returns_identity_rows() {
        let store = Arc::new(MemoryStore::new());
        let mut checkout = span("checkout", "t1", "root", 10, 1_000);
        checkout.resource = serde_json::json!({
            "service.version": "v1",
            "service.namespace": "shop",
            "deployment.environment.name": "prod",
            "telemetry.sdk.language": "rust",
            "telemetry.sdk.name": "opentelemetry",
            "telemetry.sdk.version": "0.32.1",
            "service.instance.id": "checkout-a"
        });
        store
            .ingest_traces(
                vec![checkout, span("bare", "t2", "root", 20, 1_000)],
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              serviceCatalog(fromNanos: "0", toNanos: "100") {
                name serviceVersion serviceNamespace deploymentEnvironment
                telemetrySdkLanguage telemetrySdkName telemetrySdkVersion
                lastSeenNanos instanceCount
              }
            }
            "#
            .into(),
            None,
            None,
        );

        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "serviceCatalog query: {json}"
        );
        let rows = json
            .pointer("/data/serviceCatalog")
            .unwrap()
            .as_array()
            .unwrap();
        let checkout = rows
            .iter()
            .find(|row| row.get("name") == Some(&serde_json::json!("checkout")))
            .unwrap();
        assert_eq!(
            checkout.get("serviceVersion"),
            Some(&serde_json::json!("v1"))
        );
        assert_eq!(
            checkout.get("deploymentEnvironment"),
            Some(&serde_json::json!("prod"))
        );
        assert_eq!(
            checkout.get("telemetrySdkLanguage"),
            Some(&serde_json::json!("rust"))
        );
        assert_eq!(checkout.get("instanceCount"), Some(&serde_json::json!("1")));
        let bare = rows
            .iter()
            .find(|row| row.get("name") == Some(&serde_json::json!("bare")))
            .unwrap();
        assert_eq!(bare.get("serviceVersion"), Some(&serde_json::Value::Null));
        assert_eq!(bare.get("instanceCount"), Some(&serde_json::json!("0")));
    }

    #[test]
    fn parses_typed_span_links_from_stored_json() {
        let links = serde_json::json!([
            {
                "traceId": "target-trace",
                "spanId": "target-span",
                "attributes": { "link.kind": "batch" }
            },
            {
                "trace_id": "native-target",
                "span_id": "native-span",
                "attributes": { "link.kind": "native" }
            },
            { "traceId": "", "spanId": "ignored" },
            { "spanId": "missing-trace" }
        ]);

        let parsed = span_links_from_value(&links);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].trace_id, "target-trace");
        assert_eq!(parsed[0].span_id, "target-span");
        assert_eq!(parsed[0].attributes, r#"{"link.kind":"batch"}"#);
        assert_eq!(parsed[1].trace_id, "native-target");
        assert_eq!(parsed[1].span_id, "native-span");
        assert_eq!(parsed[1].attributes, r#"{"link.kind":"native"}"#);
    }

    #[tokio::test]
    async fn linked_traces_resolves_span_link_targets() {
        let store = Arc::new(MemoryStore::new());
        let mut source = span("api", "source", "source-root", 10, 10_000_000);
        source.name = "publish".into();
        source.links = serde_json::json!([
            {
                "traceId": "target",
                "spanId": "target-root",
                "attributes": { "messaging.operation": "publish" }
            }
        ]);
        let mut target = span("worker", "target", "target-root", 20, 20_000_000);
        target.name = "consume".into();
        store
            .ingest_traces(vec![source, target], Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              trace(traceId: "source") {
                spans {
                  spanId
                  typedLinks { traceId spanId attributes }
                }
              }
              linkedTraces(traceId: "source") {
                traceId
                rootName
                service
                spanCount
                hasError
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "linkedTraces query: {json}"
        );
        assert_eq!(
            json.pointer("/data/trace/spans/0/typedLinks/0/traceId"),
            Some(&serde_json::json!("target"))
        );
        assert_eq!(
            json.pointer("/data/trace/spans/0/typedLinks/0/spanId"),
            Some(&serde_json::json!("target-root"))
        );
        assert_eq!(
            json.pointer("/data/linkedTraces/0/traceId"),
            Some(&serde_json::json!("target"))
        );
        assert_eq!(
            json.pointer("/data/linkedTraces/0/rootName"),
            Some(&serde_json::json!("consume"))
        );
        assert_eq!(
            json.pointer("/data/linkedTraces/0/service"),
            Some(&serde_json::json!("worker"))
        );
    }

    #[tokio::test]
    async fn trace_analysis_resolvers_return_path_and_diff() {
        let store = Arc::new(MemoryStore::new());
        let a_root = span("api", "a", "a-root", 0, 100);
        let mut a_db = span("db", "a", "a-db", 20, 40);
        a_db.parent_span_id = Some("a-root".into());
        let mut b_root = span("api", "b", "b-root", 0, 120);
        b_root.name = "handler".into();
        let mut b_db = span("db", "b", "b-db", 20, 60);
        b_db.parent_span_id = Some("b-root".into());
        b_db.status_code = "STATUS_CODE_ERROR".into();
        let mut b_retry = span("api", "b", "b-retry", 90, 10);
        b_retry.parent_span_id = Some("b-root".into());
        b_retry.name = "retry".into();
        store
            .ingest_traces(
                vec![a_root, a_db, b_root, b_db, b_retry],
                Default::default(),
            )
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              traceCriticalPath(traceId: "a") {
                totalGatedNs
                hops { spanId gatedByChild selfTimeNs clockSuspect }
                unattached
              }
              traceCompare(traceIdA: "a", traceIdB: "b") {
                added { name service }
                removed { name }
                changed {
                  durationDeltaNs
                  statusChanged
                  before { name statusCode }
                  after { name statusCode }
                }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "trace analysis query: {json}"
        );
        assert_eq!(
            json.pointer("/data/traceCriticalPath/totalGatedNs"),
            Some(&serde_json::json!("100"))
        );
        assert_eq!(
            json.pointer("/data/traceCriticalPath/hops/0/gatedByChild"),
            Some(&serde_json::json!("a-db"))
        );
        assert_eq!(
            json.pointer("/data/traceCompare/added/0/name"),
            Some(&serde_json::json!("retry"))
        );
        assert_eq!(
            json.pointer("/data/traceCompare/changed/0/durationDeltaNs"),
            Some(&serde_json::json!("20"))
        );
        assert_eq!(
            json.pointer("/data/traceCompare/changed/1/statusChanged"),
            Some(&serde_json::json!(true))
        );
    }

    #[tokio::test]
    async fn trace_critical_path_errors_for_empty_trace() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"{ traceCriticalPath(traceId: "missing") { totalGatedNs } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("trace has no spans")),
            "empty trace rejected: {json}"
        );
    }

    #[tokio::test]
    async fn story_resolver_returns_trace_and_run_beats() {
        let store = Arc::new(MemoryStore::new());
        let mut root = span("api", "story-trace", "root", 100, 50);
        root.run_id = Some("run-story".into());
        root.name = "checkout".into();
        root.events = Some(r#"[{"name":"exception","timeUnixNano":"120"}]"#.into());
        let mut child = span("db", "story-trace", "child", 110, 10);
        child.run_id = Some("run-story".into());
        child.parent_span_id = Some("root".into());
        child.name = "SELECT orders".into();
        child.status_code = "STATUS_CODE_ERROR".into();
        store
            .ingest_traces(vec![root, child], Default::default())
            .await
            .unwrap();
        store
            .ingest_logs(
                vec![LogRow {
                    ts_nanos: 130,
                    event_name: String::new(),
                    observed_ts_nanos: 0,
                    service: "api".into(),
                    severity_num: 17,
                    severity_text: "ERROR".into(),
                    body: "payment 123 failed".into(),
                    trace_id: "story-trace".into(),
                    span_id: "child".into(),
                    run_id: Some("run-story".into()),
                    scope_name: String::new(),
                    attributes: serde_json::Value::Null,
                    resource: serde_json::Value::Null,
                }],
                Default::default(),
            )
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              traceStory: story(traceId: "story-trace") {
                tsNanos lane kind title traceId spanId severity durationNs
              }
              runStory: story(runId: "run-story") {
                kind traceId spanId
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(error_messages(&json).is_empty(), "story query: {json}");
        assert_eq!(
            json.pointer("/data/traceStory/0/kind"),
            Some(&serde_json::json!("span.start"))
        );
        assert!(
            json.pointer("/data/traceStory")
                .and_then(|value| value.as_array())
                .is_some_and(|beats| beats.iter().any(|beat| {
                    beat["kind"] == "error" && beat["title"] == "ERROR payment <n> failed"
                })),
            "trace story has normalized error log beat: {json}"
        );
        assert!(
            json.pointer("/data/runStory")
                .and_then(|value| value.as_array())
                .is_some_and(|beats| beats
                    .iter()
                    .any(|beat| { beat["traceId"] == "story-trace" && beat["spanId"] == "child" })),
            "run story contains trace spans: {json}"
        );
    }

    #[tokio::test]
    async fn story_requires_exactly_one_anchor() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"{ story(traceId: "a", runId: "b") { kind } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("exactly one anchor")),
            "story anchor guard: {json}"
        );
    }

    #[tokio::test]
    async fn evidence_gaps_resolver_returns_trace_and_run_gaps() {
        let store = Arc::new(MemoryStore::new());
        let mut orphan = span("api", "gap-trace", "orphan", 100, 10);
        orphan.parent_span_id = Some("missing-parent".into());
        orphan.run_id = Some("gap-run".into());
        store
            .ingest_traces(vec![orphan], Default::default())
            .await
            .unwrap();
        store
            .ingest_logs(
                vec![LogRow {
                    ts_nanos: 110,
                    event_name: String::new(),
                    observed_ts_nanos: 0,
                    service: "api".into(),
                    severity_num: 9,
                    severity_text: "INFO".into(),
                    body: "uncorrelated".into(),
                    trace_id: "00000000000000000000000000000000".into(),
                    span_id: String::new(),
                    run_id: Some("gap-run".into()),
                    scope_name: String::new(),
                    attributes: serde_json::Value::Null,
                    resource: serde_json::Value::Null,
                }],
                Default::default(),
            )
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              traceGaps: evidenceGaps(traceId: "gap-trace") {
                kind subject detail
              }
              runGaps: evidenceGaps(runId: "gap-run") {
                kind subject detail
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "evidenceGaps query: {json}"
        );
        assert_eq!(
            json.pointer("/data/traceGaps/0/kind"),
            Some(&serde_json::json!("orphan_span"))
        );
        assert!(
            json.pointer("/data/traceGaps/0/detail")
                .and_then(|value| value.as_str())
                .is_some_and(|detail| detail.contains("legitimate cross-service root")),
            "orphan gap caveat: {json}"
        );
        assert!(
            json.pointer("/data/runGaps")
                .and_then(|value| value.as_array())
                .is_some_and(|gaps| gaps.iter().any(|gap| gap["kind"] == "log_without_trace")),
            "run gaps include log_without_trace: {json}"
        );
    }

    #[tokio::test]
    async fn evidence_gaps_requires_exactly_one_anchor() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"{ evidenceGaps(traceId: "a", runId: "b") { kind } }"#.into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("exactly one anchor")),
            "evidenceGaps anchor guard: {json}"
        );
    }

    #[tokio::test]
    async fn attribute_compare_resolver_returns_ranked_rows() {
        let store = Arc::new(MemoryStore::new());
        let mut spans = Vec::new();
        for index in 0..20 {
            let mut row = span("checkout", &format!("baseline-{index}"), "root", index, 10);
            row.attributes = serde_json::json!({
                "service.version": if index == 0 { "2.0.0" } else { "1.0.0" },
                "trace_id": format!("trace-baseline-{index}")
            });
            spans.push(row);
        }
        for index in 0..10 {
            let mut row = span(
                "checkout",
                &format!("selected-{index}"),
                "root",
                100 + index,
                10,
            );
            row.attributes = serde_json::json!({
                "service.version": if index < 9 { "2.0.0" } else { "1.0.0" },
                "trace_id": format!("trace-selected-{index}")
            });
            spans.push(row);
        }
        store
            .ingest_traces(spans, Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              attributeCompare(
                selectedFromNanos: "100"
                selectedToNanos: "200"
                baselineFromNanos: "0"
                baselineToNanos: "99"
                service: "checkout"
                keys: ["service.version", "trace_id"]
                topN: 5
              ) {
                key value selectedCount selectedTotal baselineCount baselineTotal score
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "attributeCompare query: {json}"
        );
        assert_eq!(
            json.pointer("/data/attributeCompare/0/key"),
            Some(&serde_json::json!("service.version"))
        );
        assert_eq!(
            json.pointer("/data/attributeCompare/0/value"),
            Some(&serde_json::json!("2.0.0"))
        );
        assert_eq!(
            json.pointer("/data/attributeCompare/0/selectedCount"),
            Some(&serde_json::json!("9"))
        );
        assert!(
            json.pointer("/data/attributeCompare")
                .and_then(|value| value.as_array())
                .is_some_and(|rows| rows.iter().all(|row| row["key"] != "trace_id")),
            "attributeCompare denies trace_id: {json}"
        );
    }

    #[tokio::test]
    async fn field_explorer_resolvers_return_keys_and_stats() {
        let store = Arc::new(MemoryStore::new());
        let mut first = span("checkout", "field-1", "root", 10, 10);
        first.attributes = serde_json::json!({
            "http.request.method": "GET",
            "request.id": "req-1"
        });
        first.resource = serde_json::json!({ "service.name": "checkout" });
        let mut second = span("checkout", "field-2", "root", 20, 10);
        second.attributes = serde_json::json!({
            "http.request.method": "GET",
            "request.id": "req-2"
        });
        second.resource = serde_json::json!({ "service.name": "checkout" });
        let mut third = span("checkout", "field-3", "root", 30, 10);
        third.attributes = serde_json::json!({
            "http.request.method": "POST",
            "request.id": "req-3"
        });
        third.resource = serde_json::json!({ "service.name": "checkout" });
        store
            .ingest_traces(vec![first, second, third], Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              fieldKeys(fromNanos: "0", toNanos: "100") {
                key namespace source nonNullCount coverage isIdentifier
              }
              fieldStats(
                key: "http.request.method"
                fromNanos: "0"
                toNanos: "100"
                service: "checkout"
              ) {
                key rowCount nonNullCount distinctCount coverage capped isIdentifier
                topValues { value count }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "field explorer query: {json}"
        );
        assert!(
            json.pointer("/data/fieldKeys")
                .and_then(|value| value.as_array())
                .is_some_and(|keys| keys.iter().any(|key| {
                    key["key"] == "resource.service.name" && key["source"] == "RESOURCE"
                })),
            "resource field exposed: {json}"
        );
        assert!(
            json.pointer("/data/fieldKeys")
                .and_then(|value| value.as_array())
                .is_some_and(|keys| keys
                    .iter()
                    .any(|key| key["key"] == "request.id" && key["isIdentifier"] == true)),
            "identifier field labeled: {json}"
        );
        assert_eq!(
            json.pointer("/data/fieldStats/topValues/0/value"),
            Some(&serde_json::json!("GET"))
        );
        assert_eq!(
            json.pointer("/data/fieldStats/topValues/0/count"),
            Some(&serde_json::json!("2"))
        );
    }

    #[tokio::test]
    async fn metric_exemplars_resolver_returns_trace_links() {
        let store = Arc::new(MemoryStore::new());
        store
            .ingest_metrics(
                Vec::new(),
                Vec::new(),
                vec![MetricExemplarRow {
                    ts_nanos: 20,
                    service: "checkout".into(),
                    name: "http.server.request.duration".into(),
                    value: 120.0,
                    trace_id: "trace-a".into(),
                    span_id: "span-a".into(),
                    run_id: Some("run-a".into()),
                    attributes: serde_json::json!({"route": "/checkout"}),
                }],
                Default::default(),
            )
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              metricExemplars(
                name: "http.server.request.duration"
                fromNanos: "0"
                toNanos: "100"
                service: "checkout"
                limit: 10
              ) {
                tsNanos service name value traceId spanId runId attributes
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "metricExemplars query: {json}"
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/traceId"),
            Some(&serde_json::json!("trace-a"))
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/spanId"),
            Some(&serde_json::json!("span-a"))
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/runId"),
            Some(&serde_json::json!("run-a"))
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/value"),
            Some(&serde_json::json!(120.0))
        );
    }

    #[tokio::test]
    async fn service_map_resolver_returns_nodes_and_edges() {
        let store = Arc::new(MemoryStore::new());
        let mut a_client = span("A", "trace-ab", "a-client", 100, 10_000_000);
        a_client.kind = "SPAN_KIND_CLIENT".into();
        let mut b_server = span("B", "trace-ab", "b-server", 101, 20_000_000);
        b_server.kind = "SPAN_KIND_SERVER".into();
        b_server.parent_span_id = Some("a-client".into());
        b_server.status_code = "STATUS_CODE_ERROR".into();
        store
            .ingest_traces(vec![a_client, b_server], Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              serviceMap(fromNanos: "0", toNanos: "200", maxTraces: 10) {
                nodes { name spanCount errorCount p95Ms }
                edges { source target callCount errorCount p50Ms p95Ms }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(error_messages(&json).is_empty(), "serviceMap query: {json}");
        assert!(
            json.pointer("/data/serviceMap/nodes")
                .and_then(|value| value.as_array())
                .is_some_and(|nodes| nodes.iter().any(|node| node["name"] == "A")
                    && nodes.iter().any(|node| node["name"] == "B")),
            "serviceMap nodes: {json}"
        );
        assert_eq!(
            json.pointer("/data/serviceMap/edges/0/source"),
            Some(&serde_json::json!("A"))
        );
        assert_eq!(
            json.pointer("/data/serviceMap/edges/0/target"),
            Some(&serde_json::json!("B"))
        );
        assert_eq!(
            json.pointer("/data/serviceMap/edges/0/errorCount"),
            Some(&serde_json::json!("1"))
        );
    }

    #[tokio::test]
    async fn traces_page_returns_total_and_span_events_json() {
        let store = Arc::new(MemoryStore::new());
        let mut mid = span("api", "mid", "b", 20, 20_000_000);
        mid.events = Some(
            r#"[{"name":"exception","timeUnixNano":"20","attributes":{"message":"bad"}}]"#
                .to_string(),
        );
        store
            .ingest_traces(
                vec![
                    span("api", "fast", "a", 10, 10_000_000),
                    mid,
                    span("api", "slow", "c", 30, 30_000_000),
                ],
                Default::default(),
            )
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              tracesPage(sort: DURATION_DESC, limit: 2, offset: 1) {
                total
                items { traceId durationNs }
              }
              trace(traceId: "mid") {
                spans { spanId events }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json.pointer("/data/tracesPage/total"),
            Some(&serde_json::json!("3"))
        );
        assert_eq!(
            json.pointer("/data/tracesPage/items/0/traceId"),
            Some(&serde_json::json!("mid"))
        );
        assert_eq!(
            json.pointer("/data/tracesPage/items/1/traceId"),
            Some(&serde_json::json!("fast"))
        );
        let events = json
            .pointer("/data/trace/spans/0/events")
            .and_then(serde_json::Value::as_str)
            .unwrap();
        assert!(events.contains("exception"));
    }
}
