//! The ingest worker: receives raw OTLP export requests from the receivers,
//! normalizes them, writes telemetry rows through the storage adapter,
//! derives error events, and upserts grouped issues in the metadata store.

use parallax_core::{derive, normalize};
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model::{ErrorEventRow, ErrorSource};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// One queued OTLP batch: the decoded request feeds the in-process tee
/// (normalize → derive → live → run registration) while the raw OTLP bytes are
/// forwarded verbatim to GreptimeDB's native `/v1/otlp` endpoints.
pub enum IngestItem {
    Traces(ExportTraceServiceRequest, bytes::Bytes),
    Logs(ExportLogsServiceRequest, bytes::Bytes),
    Metrics(ExportMetricsServiceRequest, bytes::Bytes),
}

pub type IngestSender = mpsc::Sender<IngestItem>;

pub fn channel(buffer: usize) -> (IngestSender, mpsc::Receiver<IngestItem>) {
    mpsc::channel(buffer)
}

pub struct Worker {
    store: Arc<dyn TelemetryStore>,
    metadata: Arc<MetadataStore>,
    /// Run ids already registered this process — saves a metadata round-trip
    /// per row; `ensure_run` itself is idempotent.
    seen_runs: std::collections::HashSet<String>,
    /// Live-tail broadcasts. Publishing clones the batch only while someone
    /// is subscribed; the hot path stays clone-free otherwise.
    live: crate::live::LiveChannels,
}

impl Worker {
    pub fn new(
        store: Arc<dyn TelemetryStore>,
        metadata: Arc<MetadataStore>,
        live: crate::live::LiveChannels,
    ) -> Self {
        Self {
            store,
            metadata,
            seen_runs: std::collections::HashSet::new(),
            live,
        }
    }

    /// Drain the channel until all senders drop.
    pub async fn run(mut self, mut receiver: mpsc::Receiver<IngestItem>) {
        while let Some(item) = receiver.recv().await {
            if let Err(e) = self.process(item).await {
                tracing::error!("ingest worker item failed: {e:#}");
            }
        }
    }

    async fn process(&mut self, item: IngestItem) -> anyhow::Result<()> {
        match item {
            IngestItem::Traces(request, raw) => {
                let spans = normalize::normalize_traces(&request);
                let errors = derive::derive_from_traces(&request);
                self.register_runs(
                    spans
                        .iter()
                        .filter_map(|s| s.run_id.clone().map(|run_id| (run_id, s.ts_nanos))),
                )
                .await?;
                if self.live.spans.receiver_count() > 0 {
                    let _ = self.live.spans.send(spans.clone().into());
                }
                self.store.ingest_traces(spans, raw).await?;
                self.record_errors(errors).await?;
            }
            IngestItem::Logs(request, raw) => {
                let logs = normalize::normalize_logs(&request);
                let errors = derive::derive_from_logs(&logs);
                self.register_runs(
                    logs.iter()
                        .filter_map(|l| l.run_id.clone().map(|run_id| (run_id, l.ts_nanos))),
                )
                .await?;
                if self.live.logs.receiver_count() > 0 {
                    let _ = self.live.logs.send(logs.clone().into());
                }
                self.store.ingest_logs(logs, raw).await?;
                self.record_errors(errors).await?;
            }
            IngestItem::Metrics(request, raw) => {
                let normalized = normalize::normalize_metrics(&request);
                self.store
                    .ingest_metrics(normalized.points, normalized.histograms, raw)
                    .await?;
            }
        }
        Ok(())
    }

    /// Auto-register run ids first seen in telemetry (status `external`) so
    /// run-scoped lookups work for runs no CLI wrapper started.
    async fn register_runs(
        &mut self,
        run_ids: impl Iterator<Item = (String, u128)>,
    ) -> anyhow::Result<()> {
        let mut first_seen: std::collections::HashMap<String, u128> = Default::default();
        for (run_id, ts_nanos) in run_ids {
            if run_id.is_empty() || self.seen_runs.contains(&run_id) {
                continue;
            }
            first_seen
                .entry(run_id)
                .and_modify(|t| *t = (*t).min(ts_nanos))
                .or_insert(ts_nanos);
        }
        for (run_id, ts_nanos) in first_seen {
            self.metadata.ensure_run(&run_id, ts_nanos).await?;
            self.seen_runs.insert(run_id);
        }
        Ok(())
    }

    async fn record_errors(&self, errors: Vec<ErrorEventRow>) -> anyhow::Result<()> {
        let errors = dedup_error_events(errors);
        if errors.is_empty() {
            return Ok(());
        }
        for event in &errors {
            let occurrence = parallax_storage::metadata::IssueOccurrence {
                fingerprint: &event.fingerprint,
                title: derive::issue_title(&event.error_type, &event.message),
                error_type: &event.error_type,
                culprit: derive::culprit(event.stacktrace.as_deref()),
                service: &event.service,
                ts_nanos: event.ts_nanos,
                trace_id: (!event.trace_id.is_empty() && event.trace_id.chars().any(|c| c != '0'))
                    .then_some(event.trace_id.as_str()),
                attributes: &event.attributes,
            };
            self.metadata.upsert_issue_occurrence(&occurrence).await?;
        }
        self.store.write_error_events(errors).await?;
        Ok(())
    }
}

fn source_precedence(source: ErrorSource) -> u8 {
    match source {
        ErrorSource::SpanException => 0,
        ErrorSource::LogException => 1,
        ErrorSource::LogRecord => 2,
        ErrorSource::SpanStatus => 3,
    }
}

fn dedup_error_events(errors: Vec<ErrorEventRow>) -> Vec<ErrorEventRow> {
    let mut seen: HashMap<(String, String, String), usize> = HashMap::new();
    let mut deduped: Vec<ErrorEventRow> = Vec::with_capacity(errors.len());
    for event in errors {
        if event.trace_id.is_empty() || event.span_id.is_empty() {
            deduped.push(event);
            continue;
        }
        let key = (
            event.trace_id.clone(),
            event.span_id.clone(),
            event.fingerprint.clone(),
        );
        if let Some(&index) = seen.get(&key) {
            if source_precedence(event.source) < source_precedence(deduped[index].source) {
                deduped[index] = event;
            }
        } else {
            seen.insert(key, deduped.len());
            deduped.push(event);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;
    use serde_json::json;

    fn error_event(source: ErrorSource, span_id: &str, fingerprint: &str) -> ErrorEventRow {
        ErrorEventRow {
            ts_nanos: 1,
            service: "checkout".to_string(),
            fingerprint: fingerprint.to_string(),
            error_type: "test::Boom".to_string(),
            message: "boom".to_string(),
            stacktrace: Some("top\nbottom".to_string()),
            source,
            trace_id: "trace".to_string(),
            span_id: span_id.to_string(),
            attributes: json!({}),
        }
    }

    #[test]
    fn dedup_prefers_span_exception_for_same_failure() {
        let events = dedup_error_events(vec![
            error_event(ErrorSource::LogException, "span-a", "fp"),
            error_event(ErrorSource::SpanException, "span-a", "fp"),
        ]);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, ErrorSource::SpanException);
    }

    #[test]
    fn dedup_preserves_distinct_span_failures() {
        let events = dedup_error_events(vec![
            error_event(ErrorSource::SpanException, "span-a", "fp"),
            error_event(ErrorSource::SpanException, "span-b", "fp"),
        ]);

        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn record_errors_counts_one_occurrence_after_dedup() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = Arc::new(MemoryStore::new());
        let metadata = Arc::new(
            MetadataStore::open(tmp.path().join("meta.db"))
                .await
                .expect("metadata"),
        );
        let worker = Worker::new(store.clone(), metadata.clone(), crate::live::channels());

        worker
            .record_errors(vec![
                error_event(ErrorSource::LogException, "span-a", "fp"),
                error_event(ErrorSource::SpanException, "span-a", "fp"),
            ])
            .await
            .expect("record errors");

        let issues = metadata.issues(10).await.expect("issues");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].event_count, 1);
        let events = store
            .error_events_by_fingerprint("fp", 0..=u128::MAX, 10)
            .await
            .expect("error events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, ErrorSource::SpanException);
    }
}
