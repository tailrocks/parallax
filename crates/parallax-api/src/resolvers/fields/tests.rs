use crate::resolvers::test_support::*;
use crate::{build_schema, execute};

use parallax_storage::memory::MemoryStore;

use parallax_storage::model::LogRow;
use std::sync::Arc;

#[tokio::test]
async fn evidence_gaps_resolver_returns_trace_and_run_gaps() {
    let store = Arc::new(MemoryStore::new());
    let mut orphan = span("api", "gap-trace", "orphan", 100, 10);
    orphan.parent_span_id = Some("missing-parent".into());
    orphan.run_id = Some("gap-run".into());
    store.push_spans(vec![orphan]);
    store.push_logs(vec![LogRow {
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
    }]);

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
    store.push_spans(spans);

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
    store.push_spans(vec![first, second, third]);

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
