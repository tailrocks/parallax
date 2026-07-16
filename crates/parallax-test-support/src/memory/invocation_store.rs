use super::*;

#[async_trait::async_trait]
impl adapter::InvocationStore for MemoryStore {
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<ErrorEventRow>> {
        self.error_event_read_calls.fetch_add(1, Ordering::Relaxed);
        let mut events: Vec<ErrorEventRow> = self
            .lock()
            .error_events
            .iter()
            .filter(|e| e.fingerprint == fingerprint && range.contains(&e.ts_nanos))
            .cloned()
            .collect();
        events.sort_by_key(|e| std::cmp::Reverse(e.ts_nanos));
        events.truncate(limit);
        Ok(events)
    }

    async fn error_events_by_fingerprints(
        &self,
        fingerprints: &[String],
        range: RangeInclusive<u128>,
        limit_per_fingerprint: usize,
    ) -> StorageResult<HashMap<String, Vec<ErrorEventRow>>> {
        self.error_event_read_calls.fetch_add(1, Ordering::Relaxed);
        let wanted: HashSet<_> = fingerprints.iter().map(String::as_str).collect();
        let mut events: HashMap<String, Vec<ErrorEventRow>> = fingerprints
            .iter()
            .map(|fingerprint| (fingerprint.clone(), Vec::new()))
            .collect();
        for event in &self.lock().error_events {
            if wanted.contains(event.fingerprint.as_str()) && range.contains(&event.ts_nanos) {
                events
                    .entry(event.fingerprint.clone())
                    .or_default()
                    .push(event.clone());
            }
        }
        for rows in events.values_mut() {
            rows.sort_by_key(|event| std::cmp::Reverse(event.ts_nanos));
            rows.truncate(limit_per_fingerprint);
        }
        Ok(events)
    }

    async fn observed_invocations(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<adapter::ObservedInvocation>> {
        let inner = self.lock();
        let mut runs: HashMap<String, adapter::ObservedInvocation> = HashMap::new();
        let mut absorb =
            |invocation_id: &Option<String>, ts: u128, service: &str, is_span: bool| {
                if !range.contains(&ts) {
                    return;
                }
                let Some(invocation_id) = invocation_id.as_deref().filter(|r| !r.is_empty()) else {
                    return;
                };
                let entry = runs.entry(invocation_id.to_owned()).or_insert_with(|| {
                    adapter::ObservedInvocation {
                        invocation_id: invocation_id.to_owned(),
                        first_nanos: ts,
                        last_nanos: ts,
                        span_count: 0,
                        log_count: 0,
                        service: service.to_owned(),
                        last_command: None,
                        app_mode: None,
                    }
                });
                entry.first_nanos = entry.first_nanos.min(ts);
                entry.last_nanos = entry.last_nanos.max(ts);
                if is_span {
                    entry.span_count += 1;
                } else {
                    entry.log_count += 1;
                }
            };
        for span in &inner.spans {
            absorb(&span.invocation_id, span.ts_nanos, &span.service, true);
        }
        for log in &inner.logs {
            absorb(&log.invocation_id, log.ts_nanos, &log.service, false);
        }
        for span in &inner.spans {
            let Some(invocation_id) = span.invocation_id.as_deref() else {
                continue;
            };
            let Some(entry) = runs.get_mut(invocation_id) else {
                continue;
            };
            if entry.last_command.is_none() {
                entry.last_command = span
                    .attributes
                    .get(parallax_semconv::CLI_COMMAND_NAME)
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
            }
            if entry.app_mode.is_none() {
                entry.app_mode = span
                    .attributes
                    .get(parallax_semconv::APP_MODE)
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
            }
        }
        let mut runs: Vec<_> = runs.into_values().collect();
        runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
        runs.truncate(limit);
        Ok(runs)
    }

    async fn sessions_by_invocation(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<adapter::InvocationSession>> {
        let rows: Vec<LogRow> = self
            .lock()
            .logs
            .iter()
            .filter(|log| {
                log.invocation_id.as_deref() == Some(invocation_id) && range.contains(&log.ts_nanos)
            })
            .cloned()
            .collect();
        Ok(parallax_storage::projections::pair_sessions(&rows, limit))
    }

    async fn screen_visits(
        &self,
        invocation_id: Option<&str>,
        session_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<adapter::ScreenVisit>> {
        let rows: Vec<LogRow> = self
            .lock()
            .logs
            .iter()
            .filter(|log| {
                range.contains(&log.ts_nanos)
                    && invocation_id.is_none_or(|invocation_id| {
                        log.invocation_id.as_deref() == Some(invocation_id)
                    })
            })
            .cloned()
            .collect();
        Ok(parallax_storage::projections::pair_screen_visits(
            &rows, session_id, limit,
        ))
    }

    async fn ui_actions(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<adapter::UiAction>> {
        let spans = self.invocation_spans(Some(invocation_id), &range);
        Ok(parallax_storage::projections::project_ui_actions(
            &spans, limit,
        ))
    }

    async fn background_cycles(
        &self,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<adapter::BackgroundCycleSummary>> {
        let spans = self.invocation_spans(invocation_id, &range);
        Ok(parallax_storage::projections::summarize_background_cycles(
            &spans, limit,
        ))
    }

    async fn jobs(
        &self,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<adapter::JobSummary>> {
        let spans = self.invocation_spans(invocation_id, &range);
        Ok(parallax_storage::projections::summarize_jobs(&spans, limit))
    }

    async fn conversations(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<adapter::ConversationSummary>> {
        let spans = self.invocation_spans(Some(invocation_id), &range);
        Ok(parallax_storage::projections::summarize_conversations(
            &spans, limit,
        ))
    }
}

impl MemoryStore {
    /// Bounded span window for the invocation projections, matching whole
    /// traces whose resolved invocation id equals the scope (the GreptimeDB
    /// adapter matches trace-wide too).
    fn invocation_spans(
        &self,
        invocation_id: Option<&str>,
        range: &RangeInclusive<u128>,
    ) -> Vec<SpanRow> {
        let inner = self.lock();
        let trace_ids: HashSet<&str> = inner
            .spans
            .iter()
            .filter(|span| invocation_id.is_none_or(|id| span.invocation_id.as_deref() == Some(id)))
            .map(|span| span.trace_id.as_str())
            .collect();
        inner
            .spans
            .iter()
            .filter(|span| {
                range.contains(&span.ts_nanos) && trace_ids.contains(span.trace_id.as_str())
            })
            .cloned()
            .collect()
    }
}
