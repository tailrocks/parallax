//! The ingest worker: receives raw OTLP export requests from the receivers,
//! normalizes them, writes telemetry rows through the storage adapter,
//! derives error events, and upserts grouped issues in the metadata store.

use parallax_core::{derive, normalize};
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model::ErrorEventRow;
use std::sync::Arc;
use tokio::sync::mpsc;

pub enum IngestItem {
    Traces(ExportTraceServiceRequest),
    Logs(ExportLogsServiceRequest),
    Metrics(ExportMetricsServiceRequest),
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
}

impl Worker {
    pub fn new(store: Arc<dyn TelemetryStore>, metadata: Arc<MetadataStore>) -> Self {
        Self {
            store,
            metadata,
            seen_runs: std::collections::HashSet::new(),
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
            IngestItem::Traces(request) => {
                let spans = normalize::normalize_traces(&request);
                let errors = derive::derive_from_traces(&request);
                self.register_runs(
                    spans
                        .iter()
                        .filter_map(|s| s.run_id.clone().map(|run_id| (run_id, s.ts_nanos))),
                )
                .await?;
                self.store.write_spans(spans).await?;
                self.record_errors(errors).await?;
            }
            IngestItem::Logs(request) => {
                let logs = normalize::normalize_logs(&request);
                let errors = derive::derive_from_logs(&logs);
                self.register_runs(
                    logs.iter()
                        .filter_map(|l| l.run_id.clone().map(|run_id| (run_id, l.ts_nanos))),
                )
                .await?;
                self.store.write_logs(logs).await?;
                self.record_errors(errors).await?;
            }
            IngestItem::Metrics(request) => {
                let normalized = normalize::normalize_metrics(&request);
                self.store.write_metric_points(normalized.points).await?;
                self.store.write_histograms(normalized.histograms).await?;
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
