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
use parallax_storage::adapter::{IngestStore, StorageError};
use parallax_storage::metadata::{MetadataError, MetadataStore};
use parallax_storage::model::{ErrorEventRow, ErrorSource};
use prost::Message;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};

use crate::ingest_health::{IngestHealth, QueuedItem};

mod occurrence;
mod queue;
use occurrence::occurrence_id;
pub(crate) use queue::{IngestItem, IngestSenders, channels};

const INGEST_RETRIES: usize = 3;
const INGEST_BACKOFF: [Duration; 3] = [
    Duration::from_millis(100),
    Duration::from_millis(500),
    Duration::from_secs(2),
];
const SEEN_RUNS_CAP: usize = 100_000;

#[derive(Clone)]
pub(crate) struct Worker {
    store: Arc<dyn IngestStore>,
    metadata: Arc<dyn MetadataStore>,
    seen_invocations: Arc<Mutex<HashSet<String>>>,
    live: crate::live::LiveChannels,
    health: Arc<IngestHealth>,
    #[cfg(test)]
    fail_after: Arc<Mutex<Option<(FailureStage, usize)>>>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureStage {
    Registration,
    Broadcast,
    TelemetryStorage,
    IssueRecording,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum EffectStage {
    Registration,
    Broadcast,
    TelemetryStorage,
    IssueRecording,
}

#[derive(Debug, Default)]
struct EffectProgress {
    completed_through: Option<EffectStage>,
}

impl EffectProgress {
    fn completed(&self, stage: EffectStage) -> bool {
        self.completed_through.is_some_and(|done| done >= stage)
    }

    fn mark_completed(&mut self, stage: EffectStage) {
        self.completed_through = Some(stage);
    }
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Metadata(#[from] MetadataError),
    #[cfg(test)]
    #[error("injected failure after {0:?}")]
    Injected(FailureStage),
}

type WorkerResult<T> = Result<T, WorkerError>;

impl Worker {
    #[cfg(test)]
    pub(crate) fn new(
        store: Arc<dyn IngestStore>,
        metadata: Arc<dyn MetadataStore>,
        live: crate::live::LiveChannels,
    ) -> Self {
        Self::new_with_health(store, metadata, live, Arc::new(IngestHealth::new(1)))
    }

    pub(crate) fn new_with_health(
        store: Arc<dyn IngestStore>,
        metadata: Arc<dyn MetadataStore>,
        live: crate::live::LiveChannels,
        health: Arc<IngestHealth>,
    ) -> Self {
        Self {
            store,
            metadata,
            seen_invocations: Arc::new(Mutex::new(HashSet::new())),
            live,
            health,
            #[cfg(test)]
            fail_after: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(test)]
    async fn inject_failure_once_after(&self, stage: FailureStage) {
        self.inject_failures_after(stage, 1).await;
    }

    #[cfg(test)]
    async fn inject_failures_after(&self, stage: FailureStage, attempts: usize) {
        *self.fail_after.lock().await = Some((stage, attempts));
    }

    #[cfg(test)]
    async fn maybe_fail_after(&self, stage: FailureStage) -> WorkerResult<()> {
        let mut configured = self.fail_after.lock().await;
        if let Some((configured_stage, attempts)) = configured.as_mut()
            && *configured_stage == stage
        {
            *attempts -= 1;
            if *attempts == 0 {
                *configured = None;
            }
            return Err(WorkerError::Injected(stage));
        }
        Ok(())
    }

    pub(crate) async fn run(self, signal: Signal, mut receiver: mpsc::Receiver<QueuedItem>) {
        while let Some(queued) = receiver.recv().await {
            self.health
                .dequeued(signal, queued.enqueued_at.elapsed(), queued.observed);
            let item = queued.item;
            let mut attempt = 0;
            let mut progress = EffectProgress::default();
            loop {
                match self.process_with_progress(&item, &mut progress).await {
                    Ok(()) => break,
                    Err(e) if attempt < INGEST_RETRIES => {
                        attempt += 1;
                        if queued.observed {
                            self.health.retry(signal);
                        }
                        tracing::warn!("ingest attempt {attempt} failed, retrying: {e:#}");
                        tokio::time::sleep(INGEST_BACKOFF[attempt - 1]).await;
                    }
                    Err(e) => {
                        if queued.observed {
                            self.health.terminal_drop(signal);
                        }
                        tracing::error!(
                            "ingest item DROPPED after {INGEST_RETRIES} retries: {e:#}"
                        );
                        break;
                    }
                }
            }
        }
    }

    #[cfg(test)]
    async fn process(&self, item: &IngestItem) -> WorkerResult<()> {
        self.process_with_progress(item, &mut EffectProgress::default())
            .await
    }

    async fn process_with_progress(
        &self,
        item: &IngestItem,
        progress: &mut EffectProgress,
    ) -> WorkerResult<()> {
        match item {
            IngestItem::Traces(request, raw) => {
                self.process_traces(request, raw, progress).await?;
            }
            IngestItem::Logs(request, raw) => {
                self.process_logs(request, raw, progress).await?;
            }
            IngestItem::Metrics(request, raw) => {
                self.process_metrics(request, raw, progress).await?;
            }
            IngestItem::Sentry(event) => {
                self.process_sentry(event, progress).await?;
            }
        }
        Ok(())
    }

    async fn process_sentry(
        &self,
        event: &ErrorEventRow,
        progress: &mut EffectProgress,
    ) -> WorkerResult<()> {
        // No Greptime raw-signal table for Sentry (native OTLP tables only).
        // Issue membership + error_event rows are the product stores.
        if !progress.completed(EffectStage::IssueRecording) {
            self.record_errors(vec![event.clone()]).await?;
            progress.mark_completed(EffectStage::IssueRecording);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::IssueRecording).await?;
        Ok(())
    }

    async fn process_traces(
        &self,
        request: &ExportTraceServiceRequest,
        raw: &bytes::Bytes,
        progress: &mut EffectProgress,
    ) -> WorkerResult<()> {
        let errors = derive::derive_from_traces(request);
        if !progress.completed(EffectStage::Registration) {
            self.register_invocations(normalize::resource_invocation_ids(request))
                .await?;
            progress.mark_completed(EffectStage::Registration);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::Registration).await?;
        if !progress.completed(EffectStage::Broadcast) {
            if self.live.spans.receiver_count() > 0 {
                drop(
                    self.live
                        .spans
                        .send(crate::live::span_batch(normalize::normalize_traces(
                            request,
                        ))),
                );
            }
            progress.mark_completed(EffectStage::Broadcast);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::Broadcast).await?;
        if !progress.completed(EffectStage::TelemetryStorage) {
            self.store.ingest_traces(request, raw.clone()).await?;
            progress.mark_completed(EffectStage::TelemetryStorage);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::TelemetryStorage)
            .await?;
        if !progress.completed(EffectStage::IssueRecording) {
            self.record_errors(errors).await?;
            progress.mark_completed(EffectStage::IssueRecording);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::IssueRecording).await?;
        Ok(())
    }

    async fn process_logs(
        &self,
        request: &ExportLogsServiceRequest,
        raw: &bytes::Bytes,
        progress: &mut EffectProgress,
    ) -> WorkerResult<()> {
        let mut request = request.clone();
        let raw = if normalize::promote_log_identity_attributes(&mut request) {
            bytes::Bytes::from(request.encode_to_vec())
        } else {
            raw.clone()
        };
        let logs = normalize::normalize_logs(&request);
        let errors = derive::derive_from_logs(&logs);
        if !progress.completed(EffectStage::Registration) {
            self.register_invocations(
                logs.iter()
                    .filter_map(|log| log.invocation_id.clone().map(|id| (id, log.ts_nanos))),
            )
            .await?;
            progress.mark_completed(EffectStage::Registration);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::Registration).await?;
        if !progress.completed(EffectStage::Broadcast) {
            if self.live.logs.receiver_count() > 0 {
                drop(self.live.logs.send(crate::live::log_batch(logs)));
            }
            progress.mark_completed(EffectStage::Broadcast);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::Broadcast).await?;
        if !progress.completed(EffectStage::TelemetryStorage) {
            self.store.ingest_logs(&request, raw).await?;
            progress.mark_completed(EffectStage::TelemetryStorage);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::TelemetryStorage)
            .await?;
        if !progress.completed(EffectStage::IssueRecording) {
            self.record_errors(errors).await?;
            progress.mark_completed(EffectStage::IssueRecording);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::IssueRecording).await?;
        Ok(())
    }

    async fn process_metrics(
        &self,
        request: &ExportMetricsServiceRequest,
        raw: &bytes::Bytes,
        progress: &mut EffectProgress,
    ) -> WorkerResult<()> {
        if !progress.completed(EffectStage::TelemetryStorage) {
            let normalized = normalize::normalize_metrics(request);
            self.store
                .ingest_metrics(
                    normalized.points,
                    normalized.histograms,
                    normalized.exemplars,
                    raw.clone(),
                )
                .await?;
            progress.mark_completed(EffectStage::TelemetryStorage);
        }
        #[cfg(test)]
        self.maybe_fail_after(FailureStage::TelemetryStorage)
            .await?;
        Ok(())
    }

    async fn register_invocations(
        &self,
        invocation_ids: impl Iterator<Item = (String, u128)>,
    ) -> Result<(), MetadataError> {
        let mut first_seen: HashMap<String, u128> = Default::default();
        {
            let seen = self.seen_invocations.lock().await;
            for (invocation_id, ts_nanos) in invocation_ids {
                if invocation_id.is_empty() || seen.contains(&invocation_id) {
                    continue;
                }
                first_seen
                    .entry(invocation_id)
                    .and_modify(|t| *t = (*t).min(ts_nanos))
                    .or_insert(ts_nanos);
            }
        }
        for (invocation_id, ts_nanos) in first_seen {
            self.metadata
                .ensure_invocation(&invocation_id, ts_nanos)
                .await?;
            let mut seen = self.seen_invocations.lock().await;
            if seen.len() > SEEN_RUNS_CAP {
                seen.clear();
            }
            seen.insert(invocation_id);
        }
        Ok(())
    }

    async fn record_errors(&self, errors: Vec<ErrorEventRow>) -> WorkerResult<()> {
        let errors = dedup_error_events(errors);
        if errors.is_empty() {
            return Ok(());
        }
        let occurrence_ids: Vec<String> = errors.iter().map(occurrence_id).collect();
        let occurrences: Vec<parallax_storage::metadata::IssueOccurrence<'_>> = errors
            .iter()
            .zip(&occurrence_ids)
            .map(
                |(event, occurrence_id)| parallax_storage::metadata::IssueOccurrence {
                    occurrence_id: occurrence_id.as_str().into(),
                    fingerprint: &event.fingerprint,
                    title: derive::issue_title(&event.error_type, &event.message),
                    error_type: &event.error_type,
                    culprit: derive::culprit(event.stacktrace.as_deref()),
                    service: &event.service,
                    ts_nanos: event.ts_nanos,
                    trace_id: (!event.trace_id.is_empty()
                        && event.trace_id.chars().any(|c| c != '0'))
                    .then_some(event.trace_id.as_str()),
                    attributes: &event.attributes,
                },
            )
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
        // Sentry adapter rows lose to native OTLP exception evidence for the
        // same (trace, span, fingerprint) key.
        ErrorSource::SentryEnvelope => 4,
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
