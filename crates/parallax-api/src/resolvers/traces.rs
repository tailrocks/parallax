//! GraphQL traces domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::{
    ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string, retained_recent_range, saturate_i32,
};

use parallax_core::{span_events, trace_analysis};

pub(crate) struct Span(pub(crate) model::SpanRow);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpanLink {
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
    /// `OTel` span links as JSON — spans in other traces this span causally
    /// references (batch/async sub-operations).
    fn links(&self) -> String {
        self.0.links.to_string()
    }
    /// Typed span-link targets, beside the raw JSON string for compatibility.
    fn typed_links(&self) -> Vec<SpanLink> {
        span_links_from_value(&self.0.links)
    }
    /// `OTel` span events as JSON: `[{name, timeUnixNano, attributes}]`.
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

pub(crate) struct Trace {
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

pub(crate) struct TraceEvent(pub(crate) span_events::TraceEvent);

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

pub(crate) struct TraceEventsOut(pub(crate) span_events::TraceEvents);

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

pub(crate) struct TraceSummary(pub(crate) parallax_storage::adapter::TraceSummary);

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

pub(crate) struct TraceList(pub(crate) parallax_storage::adapter::TraceList);

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

pub(crate) struct CriticalHop(pub(crate) trace_analysis::CriticalHop);

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

pub(crate) struct CriticalPath(pub(crate) trace_analysis::CriticalPath);

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

pub(crate) struct DiffSpan(pub(crate) trace_analysis::DiffSpan);

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

pub(crate) struct ChangedSpan(pub(crate) trace_analysis::ChangedSpan);

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

pub(crate) struct TraceDiff(pub(crate) trace_analysis::TraceDiff);

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

pub(crate) async fn trace(context: &ApiContext, trace_id: String) -> FieldResult<Option<Trace>> {
    let spans = context.spans_for(&trace_id).await?;
    if spans.is_empty() {
        return Ok(None);
    }
    Ok(Some(Trace {
        trace_id,
        spans: Arc::unwrap_or_clone(spans),
    }))
}

pub(crate) async fn trace_events(
    context: &ApiContext,
    trace_id: String,
    name_prefix: Option<String>,
    limit: Option<i32>,
) -> FieldResult<TraceEventsOut> {
    let spans = context.spans_for(&trace_id).await?;
    let name_prefix = name_prefix.as_deref().filter(|prefix| !prefix.is_empty());
    Ok(TraceEventsOut(span_events::trace_events(
        &spans,
        name_prefix,
        clamp_limit(limit, 500),
    )))
}

pub(crate) async fn linked_traces(
    context: &ApiContext,
    trace_id: String,
) -> FieldResult<Vec<TraceSummary>> {
    let spans = context.spans_for(&trace_id).await?;
    let ids = linked_trace_ids(&spans, &trace_id);
    let traces = context.store.traces_by_ids(&ids).await.map_err(field_err)?;
    Ok(traces.into_iter().map(TraceSummary).collect())
}

pub(crate) async fn trace_critical_path(
    context: &ApiContext,
    trace_id: String,
) -> FieldResult<CriticalPath> {
    let spans = context.spans_for(&trace_id).await?;
    if spans.is_empty() {
        return Err(field_err("traceCriticalPath trace has no spans"));
    }
    Ok(CriticalPath(trace_analysis::critical_path(&spans)))
}

pub(crate) async fn trace_compare(
    context: &ApiContext,
    trace_id_a: String,
    trace_id_b: String,
) -> FieldResult<TraceDiff> {
    let (spans_a, spans_b) = tokio::try_join!(
        context.spans_for(&trace_id_a),
        context.spans_for(&trace_id_b),
    )?;
    if spans_a.is_empty() {
        return Err(field_err("traceCompare traceIdA has no spans"));
    }
    if spans_b.is_empty() {
        return Err(field_err("traceCompare traceIdB has no spans"));
    }
    Ok(TraceDiff(trace_analysis::compare(&spans_a, &spans_b)))
}

pub(crate) async fn traces_by_run(
    context: &ApiContext,
    run_id: String,
    limit: Option<i32>,
) -> FieldResult<Vec<TraceSummary>> {
    let spans = context
        .store
        .spans_by_run(&run_id, MAX_ROWS, retained_recent_range())
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

pub(crate) async fn recent_traces(
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn traces(
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn traces_page(
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

#[cfg(test)]
mod tests;
