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
