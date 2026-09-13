use super::*;

#[async_trait::async_trait]
impl adapter::RumSessionStore for MemoryStore {
    async fn rum_sessions(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        error_only: bool,
        limit: usize,
    ) -> StorageResult<Vec<adapter::RumSession>> {
        let spans: Vec<SpanRow> = self
            .lock()
            .spans
            .iter()
            .filter(|span| range.contains(&span.ts_nanos))
            .cloned()
            .collect();
        Ok(parallax_storage::projections::summarize_rum_sessions(
            &spans, service, error_only, limit,
        ))
    }

    async fn rum_session_detail(
        &self,
        session_id: &str,
        limit: usize,
    ) -> StorageResult<Option<adapter::RumSessionDetail>> {
        let spans: Vec<SpanRow> = self.lock().spans.clone();
        Ok(parallax_storage::projections::project_rum_session_detail(
            &spans, session_id, limit,
        ))
    }
}
