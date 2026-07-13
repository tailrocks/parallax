use std::sync::Arc;

use parallax_spool::{Signal, Spool};
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use tokio::task::JoinHandle;

use crate::ingest_health::{IngestHealth, QueuedItem};
use crate::worker::{self, IngestSenders, Worker};

/// Shared state handed to both OTLP transports.
#[derive(Clone, Debug)]
pub(crate) struct IngestState {
    pub spool: Arc<Spool>,
    pub health: Arc<IngestHealth>,
    senders: IngestSenders,
}

impl IngestState {
    pub(crate) async fn enqueue(
        &self,
        signal: Signal,
        item: worker::IngestItem,
        observed: bool,
    ) -> Result<(), ()> {
        let started = std::time::Instant::now();
        let Ok(permit) = self.senders.for_signal(signal).reserve().await else {
            self.health.unavailable(signal, started.elapsed());
            return Err(());
        };
        let enqueued_at = self.health.enqueued(signal, started.elapsed(), observed);
        permit.send(QueuedItem {
            item,
            enqueued_at,
            observed,
        });
        Ok(())
    }
}

pub(crate) struct IngestRuntime {
    pub state: IngestState,
    pub health: Arc<IngestHealth>,
    pub live: crate::live::LiveChannels,
    pub workers: Vec<JoinHandle<()>>,
}

pub(crate) fn assemble_ingest(
    queue_batches: usize,
    spool: Arc<Spool>,
    store: Arc<dyn TelemetryStore>,
    metadata: Arc<dyn MetadataStore>,
) -> IngestRuntime {
    let (senders, receivers) = worker::channels(queue_batches);
    let health = Arc::new(IngestHealth::new(queue_batches));
    let state = IngestState {
        spool,
        senders,
        health: health.clone(),
    };
    let live = crate::live::channels();
    let worker = Worker::new_with_health(store, metadata, live.clone(), health.clone());
    let workers = vec![
        tokio::spawn(worker.clone().run(Signal::Traces, receivers.traces)),
        tokio::spawn(worker.clone().run(Signal::Logs, receivers.logs)),
        tokio::spawn(worker.run(Signal::Metrics, receivers.metrics)),
    ];
    IngestRuntime {
        state,
        health,
        live,
        workers,
    }
}
