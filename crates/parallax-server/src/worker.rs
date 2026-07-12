#![expect(
    clippy::excessive_nesting,
    reason = "measured staged ingest transaction"
)]

//! The ingest worker: receives raw OTLP export requests from the receivers,
//! normalizes them when needed, writes telemetry through the storage adapter,
//! derives error events, and upserts grouped issues in the metadata store.
//!
//! Three per-signal worker tasks run independently so a slow traces forward
//! does not stall logs/metrics acks (ordering across signals was never
//! guaranteed).

use parallax_analysis::derive;
use parallax_ingest as normalize;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_spool::Signal;
use parallax_storage::adapter::IngestStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model::{ErrorEventRow, ErrorSource};
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
#[derive(Debug)]
pub(crate) enum IngestItem {
    Traces(ExportTraceServiceRequest, bytes::Bytes),
    Logs(ExportLogsServiceRequest, bytes::Bytes),
    Metrics(ExportMetricsServiceRequest, bytes::Bytes),
}

/// Per-signal senders so receivers enqueue without crossing signal FIFOs.
#[derive(Clone, Debug)]
pub(crate) struct IngestSenders {
    pub traces: mpsc::Sender<IngestItem>,
    pub logs: mpsc::Sender<IngestItem>,
    pub metrics: mpsc::Sender<IngestItem>,
}

impl IngestSenders {
    pub(crate) fn for_signal(&self, signal: Signal) -> &mpsc::Sender<IngestItem> {
        match signal {
            Signal::Traces => &self.traces,
            Signal::Logs => &self.logs,
            Signal::Metrics => &self.metrics,
        }
    }
}

#[derive(Debug)]
pub(crate) struct IngestReceivers {
    pub traces: mpsc::Receiver<IngestItem>,
    pub logs: mpsc::Receiver<IngestItem>,
    pub metrics: mpsc::Receiver<IngestItem>,
}

/// Build three bounded channels, one per signal (`[limits] ingest_queue_batches`).
pub(crate) fn channels(buffer_per_signal: usize) -> (IngestSenders, IngestReceivers) {
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
pub(crate) struct Worker {
    store: Arc<dyn IngestStore>,
    metadata: Arc<dyn MetadataStore>,
    seen_runs: Arc<Mutex<HashSet<String>>>,
    live: crate::live::LiveChannels,
    #[cfg(test)]
    fail_once_after: Arc<Mutex<Option<FailureStage>>>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureStage {
    Registration,
    Broadcast,
    TelemetryStorage,
    IssueRecording,
}

impl Worker {
    pub(crate) fn new(
        store: Arc<dyn IngestStore>,
        metadata: Arc<dyn MetadataStore>,
        live: crate::live::LiveChannels,
    ) -> Self {
        Self {
            store,
            metadata,
            seen_runs: Arc::new(Mutex::new(HashSet::new())),
            live,
            #[cfg(test)]
            fail_once_after: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(test)]
    async fn inject_failure_once_after(&self, stage: FailureStage) {
        *self.fail_once_after.lock().await = Some(stage);
    }

    #[cfg(test)]
    async fn maybe_fail_after(&self, stage: FailureStage) -> anyhow::Result<()> {
        let mut configured = self.fail_once_after.lock().await;
        if configured.as_ref() == Some(&stage) {
            *configured = None;
            anyhow::bail!("injected failure after {stage:?}");
        }
        Ok(())
    }

    pub(crate) async fn run(self, mut receiver: mpsc::Receiver<IngestItem>) {
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
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::Registration).await?;
                if self.live.spans.receiver_count() > 0 {
                    let spans = normalize::normalize_traces(request);
                    drop(self.live.spans.send(crate::live::span_batch(spans)));
                }
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::Broadcast).await?;
                self.store.ingest_traces(request, raw.clone()).await?;
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::TelemetryStorage)
                    .await?;
                self.record_errors(errors).await?;
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::IssueRecording).await?;
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
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::Registration).await?;
                if self.live.logs.receiver_count() > 0 {
                    drop(self.live.logs.send(crate::live::log_batch(logs)));
                }
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::Broadcast).await?;
                self.store.ingest_logs(&request, raw).await?;
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::TelemetryStorage)
                    .await?;
                self.record_errors(errors).await?;
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::IssueRecording).await?;
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
                #[cfg(test)]
                self.maybe_fail_after(FailureStage::TelemetryStorage)
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
mod tests;
