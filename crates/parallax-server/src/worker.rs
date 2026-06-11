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
}

impl Worker {
    pub fn new(store: Arc<dyn TelemetryStore>, metadata: Arc<MetadataStore>) -> Self {
        Self { store, metadata }
    }

    /// Drain the channel until all senders drop.
    pub async fn run(self, mut receiver: mpsc::Receiver<IngestItem>) {
        while let Some(item) = receiver.recv().await {
            if let Err(e) = self.process(item).await {
                tracing::error!("ingest worker item failed: {e:#}");
            }
        }
    }

    async fn process(&self, item: IngestItem) -> anyhow::Result<()> {
        match item {
            IngestItem::Traces(request) => {
                let spans = normalize::normalize_traces(&request);
                let errors = derive::derive_from_traces(&request);
                self.store.write_spans(spans).await?;
                self.record_errors(errors).await?;
            }
            IngestItem::Logs(request) => {
                let logs = normalize::normalize_logs(&request);
                let errors = derive::derive_from_logs(&logs);
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
            };
            self.metadata.upsert_issue_occurrence(&occurrence).await?;
        }
        self.store.write_error_events(errors).await?;
        Ok(())
    }
}
