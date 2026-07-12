use super::*;
use crate::resolvers::test_support::*;
use crate::{build_schema, execute};
use parallax_storage::adapter::SqlResult;
use parallax_test_support::builders::MemoryStore;

use std::sync::Arc;

#[test]
fn cap_sql_result_truncates_rows_and_flags_over_cap_only() {
    let result = SqlResult {
        columns: vec!["n".into()],
        rows: vec![vec![serde_json::json!(1)], vec![serde_json::json!(2)]],
    };
    let under = cap_sql_result(result.clone(), 3);
    assert!(!under.truncated());
    assert_eq!(under.row_count(), 2);

    let at = cap_sql_result(result.clone(), 2);
    assert!(!at.truncated());
    assert_eq!(at.row_count(), 2);

    let over = cap_sql_result(result, 1);
    assert!(over.truncated());
    assert_eq!(over.row_count(), 1);
    assert_eq!(over.rows(), vec!["[1]"]);
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
            .any(|message| message.contains("test store has no SQL surface")),
        "SELECT passes API guard and reaches the test adapter: {json}"
    );
}
