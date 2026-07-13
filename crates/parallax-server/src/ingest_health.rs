use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Gauge, Histogram};
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::common::any_value::Value;
use parallax_spool::Signal;

#[derive(Debug)]
struct SignalState {
    depth: AtomicUsize,
    high_water: AtomicUsize,
    capacity: usize,
    attributes: [KeyValue; 1],
    accepted_attributes: [KeyValue; 2],
    unavailable_attributes: [KeyValue; 2],
    queue_times: Mutex<VecDeque<Instant>>,
}

impl SignalState {
    fn new(signal: &'static str, capacity: usize) -> Self {
        Self {
            depth: AtomicUsize::new(0),
            high_water: AtomicUsize::new(0),
            capacity,
            attributes: [KeyValue::new("signal", signal)],
            accepted_attributes: [
                KeyValue::new("signal", signal),
                KeyValue::new("outcome", "accepted"),
            ],
            unavailable_attributes: [
                KeyValue::new("signal", signal),
                KeyValue::new("outcome", "unavailable"),
            ],
            queue_times: Mutex::new(VecDeque::with_capacity(capacity)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct QueueSnapshot {
    pub depth: usize,
    pub capacity: usize,
    pub high_water: usize,
}

#[derive(Debug)]
pub(crate) struct IngestHealth {
    signals: [SignalState; 3],
    enqueue_outcomes: Counter<u64>,
    enqueue_wait: Histogram<f64>,
    queue_age: Histogram<f64>,
    retries: Counter<u64>,
    drops: Counter<u64>,
    drain: Histogram<f64>,
    self_metric_batches: AtomicU64,
    depth_gauge: Gauge<u64>,
    capacity_gauge: Gauge<u64>,
    high_water_gauge: Gauge<u64>,
    oldest_age_gauge: Gauge<f64>,
    spool_bytes_gauge: Gauge<u64>,
    spool_oldest_age_gauge: Gauge<f64>,
    spool_reclaimed: Counter<u64>,
}

impl IngestHealth {
    pub(crate) fn new(capacity: usize) -> Self {
        let meter = opentelemetry::global::meter_provider().meter("parallax.ingest");
        Self {
            signals: [
                SignalState::new("traces", capacity),
                SignalState::new("logs", capacity),
                SignalState::new("metrics", capacity),
            ],
            enqueue_outcomes: meter
                .u64_counter("parallax.ingest.enqueue.outcomes")
                .with_unit("{batch}")
                .build(),
            enqueue_wait: meter
                .f64_histogram("parallax.ingest.enqueue.wait")
                .with_unit("s")
                .build(),
            queue_age: meter
                .f64_histogram("parallax.ingest.queue.age")
                .with_unit("s")
                .build(),
            retries: meter
                .u64_counter("parallax.ingest.worker.retries")
                .with_unit("{retry}")
                .build(),
            drops: meter
                .u64_counter("parallax.ingest.worker.drops")
                .with_unit("{batch}")
                .build(),
            drain: meter
                .f64_histogram("parallax.ingest.worker.drain")
                .with_unit("s")
                .build(),
            self_metric_batches: AtomicU64::new(0),
            depth_gauge: meter
                .u64_gauge("parallax.ingest.queue.depth")
                .with_unit("{batch}")
                .build(),
            capacity_gauge: meter
                .u64_gauge("parallax.ingest.queue.capacity")
                .with_unit("{batch}")
                .build(),
            high_water_gauge: meter
                .u64_gauge("parallax.ingest.queue.high_water")
                .with_unit("{batch}")
                .build(),
            oldest_age_gauge: meter
                .f64_gauge("parallax.ingest.queue.oldest_age")
                .with_unit("s")
                .build(),
            spool_bytes_gauge: meter
                .u64_gauge("parallax.ingest.spool.bytes")
                .with_unit("By")
                .build(),
            spool_oldest_age_gauge: meter
                .f64_gauge("parallax.ingest.spool.oldest_age")
                .with_unit("s")
                .build(),
            spool_reclaimed: meter
                .u64_counter("parallax.ingest.spool.reclaimed")
                .with_unit("By")
                .build(),
        }
    }

    pub(crate) fn enqueued(&self, signal: Signal, waited: Duration, observed: bool) -> Instant {
        let enqueued_at = Instant::now();
        if !observed {
            self.self_metric_batches.fetch_add(1, Ordering::Relaxed);
            return enqueued_at;
        }
        let state = self.state(signal);
        let depth = state.depth.fetch_add(1, Ordering::AcqRel) + 1;
        state
            .queue_times
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push_back(enqueued_at);
        state.high_water.fetch_max(depth, Ordering::Relaxed);
        self.enqueue_wait
            .record(waited.as_secs_f64(), &state.attributes);
        self.enqueue_outcomes.add(1, &state.accepted_attributes);
        self.record_gauges(state);
        enqueued_at
    }

    pub(crate) fn unavailable(&self, signal: Signal, waited: Duration) {
        let state = self.state(signal);
        self.enqueue_wait
            .record(waited.as_secs_f64(), &state.attributes);
        self.enqueue_outcomes.add(1, &state.unavailable_attributes);
    }

    pub(crate) fn dequeued(&self, signal: Signal, age: Duration, observed: bool) {
        if !observed {
            return;
        }
        let state = self.state(signal);
        state.depth.fetch_sub(1, Ordering::AcqRel);
        state
            .queue_times
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front();
        self.queue_age.record(age.as_secs_f64(), &state.attributes);
        self.record_gauges(state);
    }

    pub(crate) fn retry(&self, signal: Signal) {
        self.retries.add(1, &self.state(signal).attributes);
    }

    pub(crate) fn terminal_drop(&self, signal: Signal) {
        self.drops.add(1, &self.state(signal).attributes);
    }

    pub(crate) fn drained(&self, elapsed: Duration, completed: bool) {
        self.drain.record(
            elapsed.as_secs_f64(),
            &[KeyValue::new(
                "outcome",
                if completed { "completed" } else { "timeout" },
            )],
        );
    }

    pub(crate) fn observe_spool(&self, spool: &parallax_spool::Spool) -> std::io::Result<()> {
        for signal in [Signal::Traces, Signal::Logs, Signal::Metrics] {
            let state = self.state(signal);
            let health = spool.health(signal, std::time::SystemTime::now())?;
            self.spool_bytes_gauge
                .record(health.bytes, &state.attributes);
            self.spool_oldest_age_gauge
                .record(health.oldest_age.as_secs_f64(), &state.attributes);
        }
        Ok(())
    }

    pub(crate) fn spool_reclaimed(&self, bytes: u64) {
        self.spool_reclaimed.add(bytes, &[]);
    }

    pub(crate) fn snapshot(&self, signal: Signal) -> QueueSnapshot {
        let state = self.state(signal);
        QueueSnapshot {
            depth: state.depth.load(Ordering::Acquire),
            capacity: state.capacity,
            high_water: state.high_water.load(Ordering::Relaxed),
        }
    }

    fn record_gauges(&self, state: &SignalState) {
        self.depth_gauge.record(
            u64::try_from(state.depth.load(Ordering::Acquire)).unwrap_or(u64::MAX),
            &state.attributes,
        );
        self.capacity_gauge.record(
            u64::try_from(state.capacity).unwrap_or(u64::MAX),
            &state.attributes,
        );
        self.high_water_gauge.record(
            u64::try_from(state.high_water.load(Ordering::Relaxed)).unwrap_or(u64::MAX),
            &state.attributes,
        );
        let oldest_age = state
            .queue_times
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .front()
            .map_or(Duration::ZERO, Instant::elapsed)
            .as_secs_f64();
        self.oldest_age_gauge.record(oldest_age, &state.attributes);
    }

    fn state(&self, signal: Signal) -> &SignalState {
        &self.signals[match signal {
            Signal::Traces => 0,
            Signal::Logs => 1,
            Signal::Metrics => 2,
        }]
    }
}

#[derive(Debug)]
pub(crate) struct QueuedItem {
    pub item: crate::worker::IngestItem,
    pub enqueued_at: Instant,
    pub observed: bool,
}

impl QueuedItem {
    #[cfg(test)]
    pub(crate) fn fixture(item: crate::worker::IngestItem) -> Self {
        Self {
            item,
            enqueued_at: Instant::now(),
            observed: true,
        }
    }
}

pub(crate) fn is_self_metrics(request: &ExportMetricsServiceRequest) -> bool {
    request.resource_metrics.iter().any(|metrics| {
        metrics.resource.as_ref().is_some_and(|resource| {
            resource.attributes.iter().any(|attribute| {
                attribute.key == "service.name"
                    && matches!(
                        attribute.value.as_ref().and_then(|value| value.value.as_ref()),
                        Some(Value::StringValue(service)) if service == "parallax"
                    )
            })
        })
    })
}

#[cfg(test)]
mod tests;
