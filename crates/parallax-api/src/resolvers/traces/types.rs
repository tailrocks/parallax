//! GraphQL trace domain types and field resolvers.

use juniper::graphql_object;
use parallax_storage::model;
use std::collections::{BTreeMap, HashSet};

use crate::{ApiContext, MAX_ROWS, nanos_string, saturate_i32};

use parallax_analysis::{span_events, trace_analysis};

pub(crate) struct Span(pub(crate) model::SpanRow);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpanLink {
    pub(crate) trace_id: String,
    pub(crate) span_id: String,
    pub(crate) attributes: String,
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

pub(crate) fn span_links_from_value(links: &serde_json::Value) -> Vec<SpanLink> {
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

pub(super) fn linked_trace_ids(spans: &[model::SpanRow], anchor_trace_id: &str) -> Vec<String> {
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
    fn invocation_id(&self) -> Option<&str> {
        self.0.invocation_id.as_deref()
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
    pub(super) trace_id: String,
    pub(super) spans: Vec<model::SpanRow>,
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

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub(crate) enum TraceSort {
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

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "finite positive duration is bounded by f64-to-u128 saturation"
)]
pub(super) fn duration_ms_to_ns(ms: f64) -> Option<u128> {
    (ms.is_finite() && ms > 0.0).then_some((ms * 1e6) as u128)
}
