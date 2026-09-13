//! R2 live pipeline readout: [`parallax_api::PipelineSnapshot`] over
//! [`IngestHealth`] counters plus the declared `[sampling]` policy.

use std::sync::Arc;

use parallax_api::{IngestDrop, IngestQueue, PipelineSnapshot, SamplingPolicy};
use parallax_spool::Signal;

use crate::config::SamplingConfig;
use crate::ingest_health::{IngestHealth, drop_reason_detail, signal_name};

#[derive(Debug)]
pub(crate) struct PipelineProbe {
    pub health: Arc<IngestHealth>,
    pub sampling: SamplingConfig,
}

impl PipelineSnapshot for PipelineProbe {
    fn sampling_policies(&self) -> Vec<SamplingPolicy> {
        [
            Signal::Traces,
            Signal::Logs,
            Signal::Metrics,
            Signal::Sentry,
        ]
        .into_iter()
        .map(|signal| {
            let name = signal_name(signal);
            let rate = self.sampling.rate_for(signal);
            SamplingPolicy {
                signal: name.to_string(),
                service: None,
                rule: self.sampling.rule.clone(),
                rate,
                enforced_by: "server-ingest".to_string(),
                description: format!(
                    "Server keeps every received {name} batch ({} keep-all leg). \
                     Effective end-to-end keep rate {rate} as declared by the operator.",
                    self.sampling.rule,
                ),
            }
        })
        .collect()
    }

    fn ingest_drops(&self) -> Vec<IngestDrop> {
        self.health
            .drop_counts()
            .into_iter()
            .map(|row| IngestDrop {
                signal: row.signal.map(|signal| signal_name(signal).to_string()),
                reason: row.reason.to_string(),
                count: row.count,
                detail: drop_reason_detail(row.reason).to_string(),
            })
            .collect()
    }

    fn ingest_queues(&self) -> Vec<IngestQueue> {
        [
            Signal::Traces,
            Signal::Logs,
            Signal::Metrics,
            Signal::Sentry,
        ]
        .into_iter()
        .map(|signal| {
            let snapshot = self.health.snapshot(signal);
            IngestQueue {
                signal: signal_name(signal).to_string(),
                depth: snapshot.depth,
                capacity: snapshot.capacity,
                high_water: snapshot.high_water,
                accepted: self.health.accepted(signal),
            }
        })
        .collect()
    }
}
