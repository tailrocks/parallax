//! GraphQL fields domain types and resolvers.

use juniper::{FieldResult, graphql_object};

use crate::{ApiContext, MAX_ROWS, clamp_limit, field_err, parse_range};

use parallax_core::gaps;
use parallax_storage::adapter::{
    ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow as StorageAttributeCompareRow,
    FieldKey as StorageFieldKey, FieldSource, FieldStats as StorageFieldStats,
    FieldValueCount as StorageFieldValueCount,
};

pub struct AttributeCompareRow(pub(crate) StorageAttributeCompareRow);

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

pub struct FieldKey(pub(crate) StorageFieldKey);

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

pub struct FieldValueCount(pub(crate) StorageFieldValueCount);

#[graphql_object(context = ApiContext)]
impl FieldValueCount {
    fn value(&self) -> &str {
        &self.0.value
    }
    fn count(&self) -> String {
        self.0.count.to_string()
    }
}

pub struct FieldStats(pub(crate) StorageFieldStats);

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

pub struct EvidenceGap(pub(crate) gaps::EvidenceGap);

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
                context.store.spans_by_run(&run_id, MAX_ROWS),
                context.store.logs_by_run(&run_id, MAX_ROWS),
            )
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

#[allow(clippy::too_many_arguments)]
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
        .map_err(field_err)?
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
        .map_err(field_err)?
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
        .map_err(field_err)?;
    Ok(FieldStats(stats))
}

#[cfg(test)]
mod tests {

    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;

    use parallax_storage::model::LogRow;
    use std::sync::Arc;

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
}
