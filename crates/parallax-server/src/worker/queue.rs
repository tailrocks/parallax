use parallax_storage::model::ErrorEventRow;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_spool::Signal;
use tokio::sync::mpsc;

use crate::ingest_health::QueuedItem;

/// One queued ingest batch.
///
/// Traces/logs/metrics move their decoded request plus refcounted raw protobuf
/// bytes forward. Sentry carries a single normalized error row (plan 118);
/// the durable spool holds that row's JSON, not the raw envelope.
#[derive(Debug)]
pub(crate) enum IngestItem {
    Traces(ExportTraceServiceRequest, bytes::Bytes),
    Logs(ExportLogsServiceRequest, bytes::Bytes),
    Metrics(ExportMetricsServiceRequest, bytes::Bytes),
    // Boxed: one small variant beside three large decoded-request variants
    // would otherwise inflate every queued item.
    Sentry(Box<ErrorEventRow>),
}

#[derive(Clone, Debug)]
pub(crate) struct IngestSenders {
    pub(crate) traces: mpsc::Sender<QueuedItem>,
    pub(crate) logs: mpsc::Sender<QueuedItem>,
    pub(crate) metrics: mpsc::Sender<QueuedItem>,
    pub(crate) sentry: mpsc::Sender<QueuedItem>,
}

impl IngestSenders {
    pub(crate) fn for_signal(&self, signal: Signal) -> &mpsc::Sender<QueuedItem> {
        match signal {
            Signal::Traces => &self.traces,
            Signal::Logs => &self.logs,
            Signal::Metrics => &self.metrics,
            Signal::Sentry => &self.sentry,
        }
    }
}

#[derive(Debug)]
pub(crate) struct IngestReceivers {
    pub traces: mpsc::Receiver<QueuedItem>,
    pub logs: mpsc::Receiver<QueuedItem>,
    pub metrics: mpsc::Receiver<QueuedItem>,
    pub sentry: mpsc::Receiver<QueuedItem>,
}

/// Build four bounded channels, one per signal (`ingest_queue_batches`).
pub(crate) fn channels(buffer_per_signal: usize) -> (IngestSenders, IngestReceivers) {
    let (traces_tx, traces_rx) = mpsc::channel(buffer_per_signal);
    let (logs_tx, logs_rx) = mpsc::channel(buffer_per_signal);
    let (metrics_tx, metrics_rx) = mpsc::channel(buffer_per_signal);
    let (sentry_tx, sentry_rx) = mpsc::channel(buffer_per_signal);
    (
        IngestSenders {
            traces: traces_tx,
            logs: logs_tx,
            metrics: metrics_tx,
            sentry: sentry_tx,
        },
        IngestReceivers {
            traces: traces_rx,
            logs: logs_rx,
            metrics: metrics_rx,
            sentry: sentry_rx,
        },
    )
}
