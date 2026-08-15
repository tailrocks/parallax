use crate::resolvers::test_support::{context_with_memory, error_messages};
use crate::{build_schema, execute};
use parallax_storage::adapter::IngestStore;
use parallax_storage::model::{ErrorEventRow, ErrorSource, IssueOccurrence};
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

async fn assert_nested_issue_reads_are_batched(page_size: usize) {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    let attributes = serde_json::json!({"test": true});
    let mut events = Vec::with_capacity(page_size * 2);
    for index in 0..page_size {
        let fingerprint = format!("issue-{index:02}");
        context
            .metadata
            .upsert_issue_occurrence(&IssueOccurrence {
                occurrence_id: fingerprint.as_str().into(),
                fingerprint: &fingerprint,
                title: format!("Issue {index}"),
                error_type: "test::Issue",
                culprit: None,
                service: "checkout",
                ts_nanos: 100 + index as u128,
                trace_id: None,
                attributes: &attributes,
            })
            .await
            .unwrap();
        for occurrence in 0..2 {
            events.push(ErrorEventRow {
                ts_nanos: 100 + occurrence,
                service: "checkout".to_string(),
                fingerprint: fingerprint.clone(),
                error_type: "test::Issue".to_string(),
                message: format!("event {occurrence}"),
                stacktrace: None,
                source: ErrorSource::LogRecord,
                trace_id: String::new(),
                span_id: String::new(),
                attributes: attributes.clone(),
            });
        }
    }
    store.write_error_events(events).await.unwrap();

    let request = juniper::http::GraphQLRequest::new(
        format!(
            r#"{{ issues(limit: {page_size}) {{ items {{ fingerprint latestEvent {{ message }} events(limit: 2, fromNanos: "0", toNanos: "1000") {{ message }} }} }} }}"#
        ),
        None,
        None,
    );
    let response = execute(&build_schema(), &context, request).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "nested issues: {json}");
    assert_eq!(
        json.pointer("/data/issues/items")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(page_size)
    );
    assert_eq!(store.error_event_read_calls(), 2);
}

#[tokio::test]
async fn nested_issue_fields_use_constant_store_calls_at_one_and_max_page() {
    assert_nested_issue_reads_are_batched(1).await;
    assert_nested_issue_reads_are_batched(50).await;
}

async fn seed_issue(
    store: &Arc<MemoryStore>,
    context: &crate::ApiContext,
    fingerprint: &str,
    service: &str,
    ts: u128,
    message: &str,
) {
    let attributes = serde_json::json!({"env": "test"});
    context
        .metadata
        .upsert_issue_occurrence(&IssueOccurrence {
            occurrence_id: fingerprint.to_string().into(),
            fingerprint,
            title: format!("Error {fingerprint}"),
            error_type: "test::Boom",
            culprit: None,
            service,
            ts_nanos: ts,
            trace_id: None,
            attributes: &attributes,
        })
        .await
        .unwrap();
    store
        .write_error_events(vec![ErrorEventRow {
            ts_nanos: ts,
            service: service.to_string(),
            fingerprint: fingerprint.to_string(),
            error_type: "test::Boom".to_string(),
            message: message.to_string(),
            stacktrace: None,
            source: ErrorSource::LogRecord,
            trace_id: String::new(),
            span_id: String::new(),
            attributes,
        }])
        .await
        .unwrap();
}

async fn gql(context: &crate::ApiContext, query: &str) -> serde_json::Value {
    let request = juniper::http::GraphQLRequest::new(query.to_string(), None, None);
    let response = execute(&build_schema(), context, request).await;
    serde_json::to_value(response).unwrap()
}

#[tokio::test]
async fn issues_filter_by_service_and_query() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "a", "checkout", 10, "alpha").await;
    seed_issue(&store, &context, "b", "billing", 20, "beta").await;
    let json = gql(
        &context,
        r#"{ issues(service: "checkout") { items { fingerprint } total } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert_eq!(
        json.pointer("/data/issues/total")
            .and_then(serde_json::Value::as_i64),
        Some(1)
    );
    assert_eq!(
        json.pointer("/data/issues/items/0/fingerprint")
            .and_then(serde_json::Value::as_str),
        Some("a")
    );
}

#[tokio::test]
async fn issues_status_must_be_open_or_resolved() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    let json = gql(&context, r#"{ issues(status: "nope") { total } }"#).await;
    assert!(
        error_messages(&json)
            .iter()
            .any(|m| m.contains("status must be open or resolved")),
        "{json}"
    );
}

#[tokio::test]
async fn issues_limit_zero_returns_no_items() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "a", "checkout", 10, "alpha").await;
    let json = gql(
        &context,
        r#"{ issues(limit: 0) { items { fingerprint } total } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert_eq!(
        json.pointer("/data/issues/items")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0)
    );
}

#[tokio::test]
async fn issues_offset_pages() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "a", "checkout", 10, "alpha").await;
    seed_issue(&store, &context, "b", "checkout", 20, "beta").await;
    let json = gql(
        &context,
        r#"{ issues(sort: LAST_SEEN, limit: 1, offset: 1) { items { fingerprint } total } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert_eq!(
        json.pointer("/data/issues/total")
            .and_then(serde_json::Value::as_i64),
        Some(2)
    );
    assert_eq!(
        json.pointer("/data/issues/items")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(1)
    );
}

#[tokio::test]
async fn issue_lookup_miss_is_null() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    let json = gql(
        &context,
        r#"{ issue(fingerprint: "missing") { fingerprint } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert!(
        json.pointer("/data/issue")
            .is_some_and(serde_json::Value::is_null)
    );
}

#[tokio::test]
async fn issue_trend_returns_points() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "a", "checkout", 10, "alpha").await;
    let json = gql(
        &context,
        r#"{ issueTrend(fingerprint: "a", hours: 1, stepSeconds: 3600) { count } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert!(
        json.pointer("/data/issueTrend")
            .and_then(serde_json::Value::as_array)
            .is_some(),
        "{json}"
    );
}

#[tokio::test]
async fn issue_set_status_persists() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "a", "checkout", 10, "alpha").await;
    let json = gql(
        &context,
        r#"mutation { issueSetStatus(fingerprint: "a", status: "resolved") { fingerprint status } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert_eq!(
        json.pointer("/data/issueSetStatus/status")
            .and_then(serde_json::Value::as_str),
        Some("resolved")
    );
}

#[tokio::test]
async fn issue_set_status_rejects_unknown() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    let json = gql(
        &context,
        r#"mutation { issueSetStatus(fingerprint: "a", status: "nope") { fingerprint } }"#,
    )
    .await;
    assert!(
        error_messages(&json)
            .iter()
            .any(|m| m.contains("status must be open or resolved")),
        "{json}"
    );
}

#[tokio::test]
async fn issues_bundle_markdown_has_stable_headers() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "a", "checkout", 10, "alpha").await;
    let json = gql(&context, r#"{ bundle(fingerprint: "a") { markdown } }"#).await;
    assert!(error_messages(&json).is_empty(), "{json}");
    let markdown = json
        .pointer("/data/bundle/markdown")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    assert!(
        markdown.contains("## "),
        "expected markdown headers: {markdown}"
    );
}

#[tokio::test]
async fn issues_bundle_json_is_byte_stable_across_repeated_reads() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "stable-fp", "checkout", 10, "alpha").await;
    let query = r#"{ bundle(fingerprint: "stable-fp", maxTokens: 4000) { json canonicalHash } }"#;
    let first = gql(&context, query).await;
    let second = gql(&context, query).await;
    assert!(error_messages(&first).is_empty(), "{first}");
    assert!(error_messages(&second).is_empty(), "{second}");
    assert_eq!(
        first.pointer("/data/bundle/json"),
        second.pointer("/data/bundle/json"),
        "wall-clock generated_at must not make sequential bundle json diverge"
    );
    assert_eq!(
        first.pointer("/data/bundle/canonicalHash"),
        second.pointer("/data/bundle/canonicalHash"),
    );
}

#[tokio::test]
async fn issues_query_text_filter() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "pay-fail", "checkout", 10, "payment").await;
    seed_issue(&store, &context, "other", "checkout", 11, "other").await;
    let json = gql(
        &context,
        r#"{ issues(query: "pay") { items { fingerprint } } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    let items = json
        .pointer("/data/issues/items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        items.iter().any(|item| item["fingerprint"] == "pay-fail"),
        "{json}"
    );
}

#[tokio::test]
async fn grouping_explanation_uses_derive_operation() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    let operation = "capsule.attach";
    let error_type = "jackin::CapsuleAttach";
    let message = "capsule attach failed for jk-alpha-demo";
    let attributes = serde_json::json!({
        "cli.command.name": operation,
        "error.type": error_type,
    });
    let fingerprint = parallax_analysis::fingerprint::fingerprint_with_operation(
        error_type,
        message,
        None,
        Some(operation),
    );
    let without_operation =
        parallax_analysis::fingerprint::fingerprint_with_operation(error_type, message, None, None);
    assert_ne!(fingerprint, without_operation);

    context
        .metadata
        .upsert_issue_occurrence(&IssueOccurrence {
            occurrence_id: fingerprint.as_str().into(),
            fingerprint: &fingerprint,
            title: format!("{error_type}: {message}"),
            error_type,
            culprit: None,
            service: "checkout",
            ts_nanos: 10,
            trace_id: None,
            attributes: &attributes,
        })
        .await
        .unwrap();
    store
        .write_error_events(vec![ErrorEventRow {
            ts_nanos: 10,
            service: "checkout".to_string(),
            fingerprint: fingerprint.clone(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            stacktrace: None,
            source: ErrorSource::LogRecord,
            trace_id: String::new(),
            span_id: String::new(),
            attributes,
        }])
        .await
        .unwrap();

    let json = gql(
        &context,
        &format!(
            r#"{{ issue(fingerprint: "{fingerprint}") {{ fingerprint groupingExplanation {{ operation inputsPresent }} }} }}"#
        ),
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
    assert_eq!(
        json.pointer("/data/issue/groupingExplanation/operation")
            .and_then(serde_json::Value::as_str),
        Some(operation)
    );
    let inputs = json
        .pointer("/data/issue/groupingExplanation/inputsPresent")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(inputs.iter().any(|value| value == "operation"), "{json}");
}

#[tokio::test]
async fn issues_time_window_filter() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue(&store, &context, "old", "checkout", 10, "old").await;
    seed_issue(&store, &context, "new", "checkout", 10_000, "new").await;
    let json = gql(
        &context,
        r#"{ issues(fromNanos: "5000", toNanos: "20000") { items { fingerprint } } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "{json}");
}
