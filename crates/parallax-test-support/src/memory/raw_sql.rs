//! In-memory raw sql capability.

use super::*;

#[async_trait::async_trait]
impl adapter::RawSqlStore for MemoryStore {
    async fn raw_sql(&self, _query: &str) -> anyhow::Result<adapter::SqlResult> {
        anyhow::bail!("raw SQL needs the GreptimeDB engine; the test store has no SQL surface")
    }
}
