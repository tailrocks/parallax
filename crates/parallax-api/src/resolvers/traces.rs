//! GraphQL traces domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::{retained_recent_range, ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string, saturate_i32};

use parallax_core::{span_events, trace_analysis};

pub struct Span(pub(crate) model::SpanRow);

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

pub struct TraceEvent(pub(crate) span_events::TraceEvent);

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

pub struct TraceEventsOut(pub(crate) span_events::TraceEvents);

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

pub struct TraceSummary(pub(crate) parallax_storage::adapter::TraceSummary);

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

pub struct TraceList(pub(crate) parallax_storage::adapter::TraceList);

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

pub struct CriticalHop(pub(crate) trace_analysis::CriticalHop);

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

pub struct CriticalPath(pub(crate) trace_analysis::CriticalPath);

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

pub struct DiffSpan(pub(crate) trace_analysis::DiffSpan);

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

pub struct ChangedSpan(pub(crate) trace_analysis::ChangedSpan);

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

pub struct TraceDiff(pub(crate) trace_analysis::TraceDiff);

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
mod tests {
    use super::*;
    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};
    
    use parallax_storage::memory::MemoryStore;

    use parallax_storage::model::SpanRow;
    use std::sync::Arc;

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
            .push_spans(vec![root, child]);

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
            .push_spans(vec![good, bad]);

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
            .push_spans(vec![source, target]);

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
        store.push_spans(
                vec![a_root, a_db, b_root, b_db, b_retry]
            );

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
    async fn traces_page_returns_total_and_span_events_json() {
        let store = Arc::new(MemoryStore::new());
        let mut mid = span("api", "mid", "b", 20, 20_000_000);
        mid.events = Some(
            r#"[{"name":"exception","timeUnixNano":"20","attributes":{"message":"bad"}}]"#
                .to_string(),
        );
        store.push_spans(
                vec![
                    span("api", "fast", "a", 10, 10_000_000),
                    mid,
                    span("api", "slow", "c", 30, 30_000_000),
                ]
            );

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

    #[test]
    fn compare_two_500_span_traces_timing() {
        let make = |trace_id: &str| -> Vec<SpanRow> {
            (0..500)
                .map(|i| {
                    let mut row = span(
                        "svc",
                        trace_id,
                        &format!("{trace_id}-{i}"),
                        1_000_000_000 + i as u128 * 1_000,
                        5_000,
                    );
                    if i > 0 {
                        row.parent_span_id = Some(format!("{trace_id}-{}", i / 2));
                    }
                    row.name = format!("op.{}", i % 40);
                    row
                })
                .collect()
        };
        let start = std::time::Instant::now();
        let _ = parallax_core::trace_analysis::compare(&make("a"), &make("b"));
        let elapsed = start.elapsed();
        eprintln!(
            "trace_analysis::compare on 2x500 spans: {:.3} ms",
            elapsed.as_secs_f64() * 1000.0
        );
        assert!(elapsed.as_millis() < 50, "compare slow: {elapsed:?}");
    }
}
