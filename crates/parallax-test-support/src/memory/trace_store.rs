//! In-memory trace store capability.

use super::*;

#[async_trait::async_trait]
impl adapter::TraceStore for MemoryStore {
    async fn spans_by_trace(&self, trace_id: &str) -> StorageResult<Vec<SpanRow>> {
        let mut spans: Vec<SpanRow> = self
            .lock()
            .spans
            .iter()
            .filter(|s| s.trace_id == trace_id)
            .cloned()
            .collect();
        spans.sort_by_key(|s| s.ts_nanos);
        Ok(spans)
    }

    async fn traces_by_ids(
        &self,
        trace_ids: &[String],
    ) -> StorageResult<Vec<adapter::TraceSummary>> {
        // O(n) dedup preserving request order (MAX_ROWS still caps fan-out).
        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        for trace_id in trace_ids.iter().filter(|trace_id| !trace_id.is_empty()) {
            if !seen.insert(trace_id.as_str()) {
                continue;
            }
            ids.push(trace_id.clone());
            if ids.len() >= MAX_ROWS {
                break;
            }
        }
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let inner = self.lock();
        let mut summaries = Vec::new();
        for trace_id in ids {
            let trace_spans: Vec<&SpanRow> = inner
                .spans
                .iter()
                .filter(|span| span.trace_id == trace_id)
                .collect();
            let Some(root) = trace_spans.iter().copied().min_by_key(|span| {
                (
                    !span.parent_span_id.as_deref().is_none_or(str::is_empty),
                    span.ts_nanos,
                )
            }) else {
                continue;
            };
            summaries.push(adapter::TraceSummary {
                trace_id,
                root_name: root.name.clone(),
                service: root.service.clone(),
                start_nanos: root.ts_nanos,
                duration_ns: root.duration_ns,
                span_count: trace_spans.len() as u64,
                has_error: trace_spans
                    .iter()
                    .any(|span| span.status_code == "STATUS_CODE_ERROR"),
            });
        }
        Ok(summaries)
    }

    async fn spans_by_invocation(
        &self,
        invocation_id: &str,
        limit: usize,
        _range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<SpanRow>> {
        let mut spans: Vec<SpanRow> = self
            .lock()
            .spans
            .iter()
            .filter(|s| s.invocation_id.as_deref() == Some(invocation_id))
            .cloned()
            .collect();
        spans.sort_by_key(|s| std::cmp::Reverse(s.ts_nanos));
        spans.truncate(limit);
        spans.sort_by_key(|s| s.ts_nanos);
        Ok(spans)
    }

    async fn spans_by_invocations(
        &self,
        invocation_ids: &[String],
        limit_per_invocation: usize,
    ) -> StorageResult<HashMap<String, Vec<SpanRow>>> {
        let wanted: HashSet<&str> = invocation_ids.iter().map(String::as_str).collect();
        let mut out: HashMap<String, Vec<SpanRow>> =
            invocation_ids.iter().map(|id| (id.clone(), Vec::new())).collect();
        if wanted.is_empty() || limit_per_invocation == 0 {
            return Ok(out);
        }
        for span in self.lock().spans.iter() {
            let Some(invocation_id) = span.invocation_id.as_deref() else {
                continue;
            };
            if !wanted.contains(invocation_id) {
                continue;
            }
            out.entry(invocation_id.to_string())
                .or_default()
                .push(span.clone());
        }
        for spans in out.values_mut() {
            spans.sort_by_key(|s| std::cmp::Reverse(s.ts_nanos));
            spans.truncate(limit_per_invocation);
            spans.sort_by_key(|s| s.ts_nanos);
        }
        Ok(out)
    }
}
