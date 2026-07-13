//! GraphQL fields domain types and resolvers.

use juniper::{FieldResult, graphql_object};

use crate::{ApiContext, MAX_ROWS, clamp_limit, field_err, parse_range, retained_recent_range};

use parallax_evidence::gaps;
use parallax_storage::adapter::{
    ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow as StorageAttributeCompareRow,
    FieldKey as StorageFieldKey, FieldSource, FieldStats as StorageFieldStats,
    FieldValueCount as StorageFieldValueCount,
};

pub(crate) struct AttributeCompareRow(pub(crate) StorageAttributeCompareRow);

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

pub(crate) struct FieldKey(pub(crate) StorageFieldKey);

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

pub(crate) struct FieldValueCount(pub(crate) StorageFieldValueCount);

#[graphql_object(context = ApiContext)]
impl FieldValueCount {
    fn value(&self) -> &str {
        &self.0.value
    }
    fn count(&self) -> String {
        self.0.count.to_string()
    }
}

pub(crate) struct FieldStats(pub(crate) StorageFieldStats);

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

pub(crate) struct EvidenceGap(pub(crate) gaps::EvidenceGap);

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

pub(crate) async fn evidence_gaps(
    context: &ApiContext,
    trace_id: Option<String>,
    run_id: Option<String>,
) -> FieldResult<Vec<EvidenceGap>> {
    let trace_id = crate::validate_optional_trace_id(trace_id)?;
    match (trace_id, run_id) {
        (Some(trace_id), None) => {
            let (spans, logs) =
                tokio::try_join!(context.spans_for(&trace_id), context.logs_for(&trace_id),)?;
            Ok(gaps::detect_gaps(&spans, &logs)
                .into_iter()
                .map(EvidenceGap)
                .collect())
        }
        (None, Some(run_id)) => {
            let (spans, logs) = tokio::try_join!(
                context
                    .store
                    .spans_by_run(&run_id, MAX_ROWS, retained_recent_range()),
                context.store.logs_by_run(&run_id, MAX_ROWS),
            )
            .map_err(crate::internal_field_err)?;
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

#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn attribute_compare(
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
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(AttributeCompareRow)
        .collect())
}

pub(crate) async fn field_keys(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<FieldKey>> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    Ok(context
        .store
        .span_field_keys(from..=to)
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(FieldKey)
        .collect())
}

pub(crate) async fn field_stats(
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
        .map_err(crate::internal_field_err)?;
    Ok(FieldStats(stats))
}

#[cfg(test)]
mod tests;
