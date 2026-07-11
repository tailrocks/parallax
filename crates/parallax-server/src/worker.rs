//! The ingest worker: receives raw OTLP export requests from the receivers,
//! normalizes them when needed, writes telemetry through the storage adapter,
//! derives error events, and upserts grouped issues in the metadata store.
//!
//! Three per-signal worker tasks run independently so a slow traces forward
//! does not stall logs/metrics acks (ordering across signals was never
//! guaranteed).

use parallax_core::{derive, normalize};
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model::{ErrorEventRow, ErrorSource};
use parallax_storage::spool::Signal;
use prost::Message;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};

const INGEST_RETRIES: usize = 3;
const INGEST_BACKOFF: [Duration; 3] = [
    Duration::from_millis(100),
    Duration::from_millis(500),
    Duration::from_secs(2),
];
const SEEN_RUNS_CAP: usize = 100_000;

/// One queued OTLP batch.
///
/// Memory shape per item:
/// - **Traces / logs**: decoded `Export*ServiceRequest` (needed for derive,
///   live-tail, and — for logs — identity-attribute promotion) plus raw
///   protobuf `Bytes` (zero-copy refcounted) for the native GreptimeDB OTLP
///   forward. Both are justified consumers; dropping either would re-decode
///   or re-encode on the hot path.
/// - **Metrics**: decoded request (normalize → extension tables / exemplars)
///   plus raw bytes for the native metric-engine forward.
pub enum IngestItem {
    Traces(ExportTraceServiceRequest, bytes::Bytes),
    Logs(ExportLogsServiceRequest, bytes::Bytes),
    Metrics(ExportMetricsServiceRequest, bytes::Bytes),
}

/// Per-signal senders so receivers enqueue without crossing signal FIFOs.
#[derive(Clone)]
pub struct IngestSenders {
    pub traces: mpsc::Sender<IngestItem>,
    pub logs: mpsc::Sender<IngestItem>,
    pub metrics: mpsc::Sender<IngestItem>,
}

impl IngestSenders {
    pub fn for_signal(&self, signal: Signal) -> &mpsc::Sender<IngestItem> {
        match signal {
            Signal::Traces => &self.traces,
            Signal::Logs => &self.logs,
            Signal::Metrics => &self.metrics,
        }
    }
}

pub struct IngestReceivers {
    pub traces: mpsc::Receiver<IngestItem>,
    pub logs: mpsc::Receiver<IngestItem>,
    pub metrics: mpsc::Receiver<IngestItem>,
}

/// Build three bounded channels, one per signal (`[limits] ingest_queue_batches`).
pub fn channels(buffer_per_signal: usize) -> (IngestSenders, IngestReceivers) {
    let (traces_tx, traces_rx) = mpsc::channel(buffer_per_signal);
    let (logs_tx, logs_rx) = mpsc::channel(buffer_per_signal);
    let (metrics_tx, metrics_rx) = mpsc::channel(buffer_per_signal);
    (
        IngestSenders {
            traces: traces_tx,
            logs: logs_tx,
            metrics: metrics_tx,
        },
        IngestReceivers {
            traces: traces_rx,
            logs: logs_rx,
            metrics: metrics_rx,
        },
    )
}

#[derive(Clone)]
pub struct Worker {
    store: Arc<dyn TelemetryStore>,
    metadata: Arc<MetadataStore>,
    seen_runs: Arc<Mutex<HashSet<String>>>,
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
            seen_runs: Arc::new(Mutex::new(HashSet::new())),
            live,
        }
    }

    pub async fn run(self, mut receiver: mpsc::Receiver<IngestItem>) {
        while let Some(item) = receiver.recv().await {
            let mut attempt = 0;
            loop {
                match self.process(&item).await {
                    Ok(()) => break,
                    Err(e) if attempt < INGEST_RETRIES => {
                        attempt += 1;
                        tracing::warn!("ingest attempt {attempt} failed, retrying: {e:#}");
                        tokio::time::sleep(INGEST_BACKOFF[attempt - 1]).await;
                    }
                    Err(e) => {
                        tracing::error!(
                            "ingest item DROPPED after {INGEST_RETRIES} retries: {e:#}"
                        );
                        break;
                    }
                }
            }
        }
    }

    async fn process(&self, item: &IngestItem) -> anyhow::Result<()> {
        match item {
            IngestItem::Traces(request, raw) => {
                let errors = derive::derive_from_traces(request);
                self.register_runs(normalize::resource_run_ids(request))
                    .await?;
                if self.live.spans.receiver_count() > 0 {
                    let spans = normalize::normalize_traces(request);
                    let _ = self.live.spans.send(crate::live::span_batch(spans));
                }
                self.store.ingest_traces(request, raw.clone()).await?;
                self.record_errors(errors).await?;
            }
            IngestItem::Logs(request, raw) => {
                let mut request = request.clone();
                let raw = if normalize::promote_log_identity_attributes(&mut request) {
                    bytes::Bytes::from(request.encode_to_vec())
                } else {
                    raw.clone()
                };
                // derive_from_logs takes normalized rows (Plan 070/026) — keep
                // normalizing on the logs path rather than refactoring derive.
                let logs = normalize::normalize_logs(&request);
                let errors = derive::derive_from_logs(&logs);
                self.register_runs(
                    logs.iter()
                        .filter_map(|l| l.run_id.clone().map(|run_id| (run_id, l.ts_nanos))),
                )
                .await?;
                if self.live.logs.receiver_count() > 0 {
                    let _ = self.live.logs.send(crate::live::log_batch(logs));
                }
                self.store.ingest_logs(&request, raw).await?;
                self.record_errors(errors).await?;
            }
            IngestItem::Metrics(request, raw) => {
                let normalized = normalize::normalize_metrics(request);
                self.store
                    .ingest_metrics(
                        normalized.points,
                        normalized.histograms,
                        normalized.exemplars,
                        raw.clone(),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    async fn register_runs(
        &self,
        run_ids: impl Iterator<Item = (String, u128)>,
    ) -> anyhow::Result<()> {
        let mut first_seen: HashMap<String, u128> = Default::default();
        {
            let seen = self.seen_runs.lock().await;
            for (run_id, ts_nanos) in run_ids {
                if run_id.is_empty() || seen.contains(&run_id) {
                    continue;
                }
                first_seen
                    .entry(run_id)
                    .and_modify(|t| *t = (*t).min(ts_nanos))
                    .or_insert(ts_nanos);
            }
        }
        for (run_id, ts_nanos) in first_seen {
            self.metadata.ensure_run(&run_id, ts_nanos).await?;
            let mut seen = self.seen_runs.lock().await;
            if seen.len() > SEEN_RUNS_CAP {
                seen.clear();
            }
            seen.insert(run_id);
        }
        Ok(())
    }

    async fn record_errors(&self, errors: Vec<ErrorEventRow>) -> anyhow::Result<()> {
        let errors = dedup_error_events(errors);
        if errors.is_empty() {
            return Ok(());
        }
        let occurrences: Vec<parallax_storage::metadata::IssueOccurrence<'_>> = errors
            .iter()
            .map(|event| parallax_storage::metadata::IssueOccurrence {
                fingerprint: &event.fingerprint,
                title: derive::issue_title(&event.error_type, &event.message),
                error_type: &event.error_type,
                culprit: derive::culprit(event.stacktrace.as_deref()),
                service: &event.service,
                ts_nanos: event.ts_nanos,
                trace_id: (!event.trace_id.is_empty() && event.trace_id.chars().any(|c| c != '0'))
                    .then_some(event.trace_id.as_str()),
                attributes: &event.attributes,
            })
            .collect();
        self.metadata.upsert_issue_occurrences(&occurrences).await?;
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
    use parallax_proto::common::any_value::Value as AnyValueEnum;
    use parallax_proto::common::{AnyValue, KeyValue};
    use parallax_proto::metrics::{
        Exemplar, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        exemplar::Value as ExemplarValue, metric::Data, number_data_point::Value as NumberValue,
    };
    use parallax_storage::memory::MemoryStore;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::oneshot;

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

    fn string_kv(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(AnyValueEnum::StringValue(value.to_string())),
            }),
            key_strindex: 0,
        }
    }

    fn metrics_request_with_exemplar() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(parallax_proto::resource::Resource {
                    attributes: vec![
                        string_kv("service.name", "checkout"),
                        string_kv("parallax.run.id", "run-a"),
                    ],
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "http.server.request.duration".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                time_unix_nano: 20,
                                value: Some(NumberValue::AsDouble(100.0)),
                                exemplars: vec![Exemplar {
                                    time_unix_nano: 21,
                                    trace_id: vec![0xab; 16],
                                    span_id: vec![0xcd; 8],
                                    value: Some(ExemplarValue::AsDouble(120.0)),
                                    filtered_attributes: vec![string_kv("route", "/checkout")],
                                }],
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
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

    #[tokio::test]
    async fn process_is_reentrant_after_failure_shape() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = Arc::new(MemoryStore::new());
        let metadata = Arc::new(
            MetadataStore::open(tmp.path().join("meta.db"))
                .await
                .expect("metadata"),
        );
        let worker = Worker::new(store.clone(), metadata, crate::live::channels());
        let item = IngestItem::Metrics(metrics_request_with_exemplar(), bytes::Bytes::new());
        worker.process(&item).await.expect("first");
        worker.process(&item).await.expect("second");
        let rows = store
            .metric_exemplars(
                "http.server.request.duration",
                Some("checkout"),
                0..=100,
                10,
            )
            .await
            .expect("metric exemplars");
        assert!(!rows.is_empty());
    }

    #[test]
    fn ingest_retry_constants_are_bounded() {
        assert_eq!(INGEST_RETRIES, 3);
        assert_eq!(INGEST_BACKOFF.len(), 3);
        assert!(INGEST_BACKOFF[0] < INGEST_BACKOFF[1]);
        assert!(INGEST_BACKOFF[1] < INGEST_BACKOFF[2]);
    }

    #[tokio::test]
    async fn metric_exemplar_round_trips_through_worker_and_store() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = Arc::new(MemoryStore::new());
        let metadata = Arc::new(
            MetadataStore::open(tmp.path().join("meta.db"))
                .await
                .expect("metadata"),
        );
        let worker = Worker::new(store.clone(), metadata, crate::live::channels());
        worker
            .process(&IngestItem::Metrics(
                metrics_request_with_exemplar(),
                bytes::Bytes::new(),
            ))
            .await
            .expect("process metrics");
        let rows = store
            .metric_exemplars(
                "http.server.request.duration",
                Some("checkout"),
                0..=100,
                10,
            )
            .await
            .expect("metric exemplars");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].value, 120.0);
        assert_eq!(rows[0].trace_id, "abababababababababababababababab");
        assert_eq!(rows[0].span_id, "cdcdcdcdcdcdcdcd");
        assert_eq!(rows[0].run_id.as_deref(), Some("run-a"));
        assert_eq!(rows[0].attributes["route"], "/checkout");
    }

    #[tokio::test]
    async fn per_signal_workers_isolate_slow_traces_from_logs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (gate_tx, gate_rx) = oneshot::channel();
        let store = Arc::new(MemoryStore::new());
        store.set_traces_gate(gate_rx).await;
        let logs_done = Arc::new(AtomicUsize::new(0));
        let metadata = Arc::new(
            MetadataStore::open(tmp.path().join("meta.db"))
                .await
                .expect("metadata"),
        );
        let live = crate::live::channels();
        let worker = Worker::new(store.clone(), metadata, live);
        let (senders, receivers) = channels(8);

        let traces_task = tokio::spawn(worker.clone().run(receivers.traces));
        let logs_done_c = logs_done.clone();
        let worker_logs = worker.clone();
        let logs_task = tokio::spawn(async move {
            let mut rx = receivers.logs;
            while let Some(item) = rx.recv().await {
                worker_logs.process(&item).await.expect("logs");
                logs_done_c.fetch_add(1, Ordering::SeqCst);
            }
        });
        let metrics_task = tokio::spawn(worker.run(receivers.metrics));

        senders
            .traces
            .send(IngestItem::Traces(
                ExportTraceServiceRequest::default(),
                bytes::Bytes::new(),
            ))
            .await
            .expect("enqueue traces");
        senders
            .logs
            .send(IngestItem::Logs(
                ExportLogsServiceRequest::default(),
                bytes::Bytes::new(),
            ))
            .await
            .expect("enqueue logs");

        // Poll until logs complete while traces remain gated (real time, bounded).
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        while logs_done.load(Ordering::SeqCst) == 0 {
            assert!(
                tokio::time::Instant::now() < deadline,
                "logs worker must not wait for a blocked traces forward"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(logs_done.load(Ordering::SeqCst), 1);

        let _ = gate_tx.send(());
        drop(senders);
        let _ = traces_task.await;
        let _ = logs_task.await;
        let _ = metrics_task.await;
    }
}
