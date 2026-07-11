//! GraphQL sql domain types and resolvers.

use juniper::{FieldResult, graphql_object};

use crate::{ApiContext, field_err};

use crate::SQL_MAX_ROWS;

pub struct SqlResultOut {
    result: parallax_storage::adapter::SqlResult,
    truncated: bool,
}

fn cap_sql_result(
    mut result: parallax_storage::adapter::SqlResult,
    max_rows: usize,
) -> SqlResultOut {
    let truncated = result.rows.len() > max_rows;
    if truncated {
        result.rows.truncate(max_rows);
    }
    SqlResultOut { result, truncated }
}

#[graphql_object(context = ApiContext)]
impl SqlResultOut {
    fn columns(&self) -> &[String] {
        &self.result.columns
    }
    /// Each row as a JSON array string (heterogeneous cell types).
    fn rows(&self) -> Vec<String> {
        self.result
            .rows
            .iter()
            .map(|row| serde_json::Value::Array(row.clone()).to_string())
            .collect()
    }
    fn row_count(&self) -> i32 {
        i32::try_from(self.result.rows.len()).unwrap_or(i32::MAX)
    }
    fn truncated(&self) -> bool {
        self.truncated
    }
}

pub(crate) async fn sql(context: &ApiContext, query: String) -> FieldResult<SqlResultOut> {
    let trimmed = query.trim();
    let lowered = trimmed.to_ascii_lowercase();
    let read_only = [
        "select", "with", "show", "describe", "desc", "explain", "tql",
    ]
    .iter()
    .any(|prefix| lowered.starts_with(prefix));
    if !read_only {
        return Err(field_err(
            "only read-only statements are allowed (SELECT/WITH/SHOW/DESCRIBE/EXPLAIN/TQL)",
        ));
    }
    if lowered.starts_with("explain") && lowered.contains("analyze") {
        return Err(field_err(
            "EXPLAIN ANALYZE executes the statement and is not allowed; use EXPLAIN",
        ));
    }
    if trimmed.trim_end_matches(';').contains(';') {
        return Err(field_err("multiple statements are not allowed"));
    }
    let result = context
        .store
        .raw_sql(trimmed.trim_end_matches(';'))
        .await
        .map_err(field_err)?;
    Ok(cap_sql_result(result, SQL_MAX_ROWS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};
    use parallax_storage::adapter::SqlResult;
    use parallax_storage::memory::MemoryStore;

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
                .any(|message| message.contains("in-memory store")),
            "SELECT passes API guard and reaches memory adapter: {json}"
        );
    }
}
