//! Parallax GraphQL API — the V1 surface from the implementation spec §8,
//! served by **Juniper** (operator instruction, 2026-06-12: the library he
//! uses in his own services). Every client (CLI, UI, agents) goes through
//! this schema; none touch storage directly.
//!
//! Juniper notes (per the spec's dependency table): GraphQL `Int` is i32 —
//! counts saturate; nanosecond timestamps cross as strings; field names are
//! auto-camelCased; cost limits are resolver-level caps in V1.

use juniper::{EmptySubscription, FieldError, FieldResult, RootNode, graphql_object};
use parallax_core::{gaps, story, trace_analysis};
use parallax_storage::adapter::{
    ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow as StorageAttributeCompareRow, OverviewTotals,
    SERVICE_MAP_TRACE_CAP, ServiceEdge as StorageServiceEdge,
    ServiceSummary as StorageServiceSummary, SpanRed as StorageSpanRed, TelemetryStore,
};
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model;
use parallax_storage::model::{MetricAgg, SeriesPoint};
use std::collections::BTreeMap;
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

fn clamp_limit(limit: Option<i32>, default: usize) -> usize {
    limit
        .map_or(default, |l| usize::try_from(l.max(0)).unwrap_or(default))
        .min(MAX_ROWS)
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
            let trace_id = link.get("traceId")?.as_str()?.to_string();
            if trace_id.is_empty() {
                return None;
            }
            let span_id = link
                .get("spanId")
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
    for span in spans {
        for link in span_links_from_value(&span.links) {
            if link.trace_id == anchor_trace_id || ids.contains(&link.trace_id) {
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
    /// OTel span links as JSON: `[{traceId, spanId, attributes}]` — spans in
    /// other traces this span causally references (batch/async sub-operations).
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
        Ok(SqlResultOut(result))
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

    /// Distinct service names (drives the service-overview selector).
    async fn services(context: &ApiContext) -> FieldResult<Vec<String>> {
        context.store.service_names().await.map_err(field_err)
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

/// Well-known process metrics correlated into every bundle (spec §8
/// correlation sections).
const BUNDLE_WINDOW_METRICS: &[&str] = &[
    "process.cpu.utilization",
    "process.memory.usage",
    "tokio.runtime.alive_tasks",
];

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
    for metric in BUNDLE_WINDOW_METRICS {
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
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;
    use parallax_storage::model::{ErrorEventRow, ErrorSource, LogRow, SpanRow};

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

    async fn context_with_memory(store: Arc<MemoryStore>) -> ApiContext {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "parallax-api-test-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let metadata = MetadataStore::open(&path).await.unwrap();
        let _ = std::fs::remove_file(path);
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

    #[test]
    fn parses_typed_span_links_from_stored_json() {
        let links = serde_json::json!([
            {
                "traceId": "target-trace",
                "spanId": "target-span",
                "attributes": { "link.kind": "batch" }
            },
            { "traceId": "", "spanId": "ignored" },
            { "spanId": "missing-trace" }
        ]);

        let parsed = span_links_from_value(&links);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].trace_id, "target-trace");
        assert_eq!(parsed[0].span_id, "target-span");
        assert_eq!(parsed[0].attributes, r#"{"link.kind":"batch"}"#);
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
