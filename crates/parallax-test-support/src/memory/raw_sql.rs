//! In-memory raw sql capability.

use super::*;

#[async_trait::async_trait]
impl adapter::RawSqlStore for MemoryStore {
    async fn raw_sql(&self, query: &str) -> StorageResult<adapter::SqlResult> {
        let lowered = query.trim().to_ascii_lowercase();
        if lowered.contains("information_schema.columns") {
            return Ok(adapter::SqlResult {
                columns: vec![
                    "table_name".into(),
                    "column_name".into(),
                    "data_type".into(),
                ],
                rows: vec![
                    vec![
                        serde_json::json!("opentelemetry_logs"),
                        serde_json::json!("body"),
                        serde_json::json!("STRING"),
                    ],
                    vec![
                        serde_json::json!("opentelemetry_traces"),
                        serde_json::json!("trace_id"),
                        serde_json::json!("STRING"),
                    ],
                ],
            });
        }
        if lowered.starts_with("select count(*)") {
            let (_, logs, _, _) = self.counts();
            return Ok(adapter::SqlResult {
                columns: vec!["count(*)".into()],
                rows: vec![vec![serde_json::json!(logs)]],
            });
        }
        Err(adapter::StorageError::query(anyhow::anyhow!(
            "memory store cannot execute {query}"
        )))
    }
}
