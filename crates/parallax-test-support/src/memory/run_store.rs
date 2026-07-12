use super::*;

#[async_trait::async_trait]
impl adapter::RunStore for MemoryStore {
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>> {
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
    ) -> anyhow::Result<HashMap<String, Vec<ErrorEventRow>>> {
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

    async fn observed_runs(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<adapter::ObservedRun>> {
        let inner = self.lock();
        let mut runs: HashMap<String, adapter::ObservedRun> = HashMap::new();
        let mut absorb = |run_id: &Option<String>, ts: u128, service: &str, is_span: bool| {
            if !range.contains(&ts) {
                return;
            }
            let Some(run_id) = run_id.as_deref().filter(|r| !r.is_empty()) else {
                return;
            };
            let entry = runs
                .entry(run_id.to_owned())
                .or_insert_with(|| adapter::ObservedRun {
                    run_id: run_id.to_owned(),
                    first_nanos: ts,
                    last_nanos: ts,
                    span_count: 0,
                    log_count: 0,
                    service: service.to_owned(),
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
            absorb(&span.run_id, span.ts_nanos, &span.service, true);
        }
        for log in &inner.logs {
            absorb(&log.run_id, log.ts_nanos, &log.service, false);
        }
        let mut runs: Vec<_> = runs.into_values().collect();
        runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
        runs.truncate(limit);
        Ok(runs)
    }
}
