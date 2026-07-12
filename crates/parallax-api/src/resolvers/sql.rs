//! GraphQL sql domain types and resolvers.

use juniper::{FieldResult, graphql_object};

use crate::{ApiContext, field_err};

use crate::SQL_MAX_ROWS;

pub(crate) struct SqlResultOut {
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
mod tests;
