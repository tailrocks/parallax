//! Trace query orchestration over storage capabilities and analysis.

use super::types::*;
use crate::{ApiContext, MAX_ROWS, clamp_limit, field_err, retained_recent_range};
use juniper::FieldResult;
use parallax_analysis::{span_events, trace_analysis};
use parallax_storage::model;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) async fn trace(context: &ApiContext, trace_id: String) -> FieldResult<Option<Trace>> {
    let trace_id = crate::validate_trace_id(trace_id)?;
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
    let trace_id = crate::validate_trace_id(trace_id)?;
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
    let trace_id = crate::validate_trace_id(trace_id)?;
    let spans = context.spans_for(&trace_id).await?;
    let ids = linked_trace_ids(&spans, &trace_id);
    let traces = context
        .store
        .traces_by_ids(&ids)
        .await
        .map_err(crate::internal_field_err)?;
    Ok(traces.into_iter().map(TraceSummary).collect())
}

pub(crate) async fn trace_critical_path(
    context: &ApiContext,
    trace_id: String,
) -> FieldResult<CriticalPath> {
    let trace_id = crate::validate_trace_id(trace_id)?;
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
    let trace_id_a = crate::validate_trace_id(trace_id_a)?;
    let trace_id_b = crate::validate_trace_id(trace_id_b)?;
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

pub(crate) async fn traces_by_invocation(
    context: &ApiContext,
    invocation_id: String,
    limit: Option<i32>,
) -> FieldResult<Vec<TraceSummary>> {
    let spans = context
        .store
        .spans_by_invocation(&invocation_id, MAX_ROWS, retained_recent_range())
        .await
        .map_err(crate::internal_field_err)?;
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
        .map_err(crate::internal_field_err)?;
    Ok(traces.into_iter().map(TraceSummary).collect())
}

#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
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
        min_duration_ns: min_duration_ms.and_then(duration_ms_to_ns),
        max_duration_ns: max_duration_ms.and_then(duration_ms_to_ns),
        error_only: error_only.unwrap_or(false),
        name_contains: query.filter(|q| !q.trim().is_empty()),
        // Plan 164: GraphQL `attributeFilters` argument lands with the editor
        // wiring; empty preserves list semantics until then.
        attribute_filters: Vec::new(),
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
        .map_err(crate::internal_field_err)?;
    Ok(traces.items.into_iter().map(TraceSummary).collect())
}

#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn traces_page(
    context: &ApiContext,
    service: Option<String>,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    min_duration_ms: Option<f64>,
    max_duration_ms: Option<f64>,
    error_only: Option<bool>,
    query: Option<String>,
    attribute_filters: Option<Vec<AttributeFilterInput>>,
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
        min_duration_ns: min_duration_ms.and_then(duration_ms_to_ns),
        max_duration_ns: max_duration_ms.and_then(duration_ms_to_ns),
        error_only: error_only.unwrap_or(false),
        name_contains: query.filter(|q| !q.trim().is_empty()),
        attribute_filters: attribute_filters
            .unwrap_or_default()
            .into_iter()
            .map(|filter| filter.into_adapter().map_err(field_err))
            .collect::<FieldResult<Vec<_>>>()?,
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
        .map_err(crate::internal_field_err)?;
    Ok(TraceList(traces))
}

/// Duration p50/p95 of the current trace filter set (plan 164 preset
/// chips). Duration bounds are deliberately not accepted: presets derive
/// from the unbounded distribution of the filtered window.
#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn trace_duration_stats(
    context: &ApiContext,
    service: Option<String>,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    error_only: Option<bool>,
    query: Option<String>,
    attribute_filters: Option<Vec<AttributeFilterInput>>,
) -> FieldResult<DurationStats> {
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
        error_only: error_only.unwrap_or(false),
        name_contains: query.filter(|q| !q.trim().is_empty()),
        attribute_filters: attribute_filters
            .unwrap_or_default()
            .into_iter()
            .map(|filter| filter.into_adapter().map_err(field_err))
            .collect::<FieldResult<Vec<_>>>()?,
        ..parallax_storage::adapter::TraceQuery::default()
    };
    let stats = context
        .store
        .trace_duration_stats(&trace_query)
        .await
        .map_err(crate::internal_field_err)?;
    Ok(DurationStats(stats))
}
