use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Gauge, Histogram};
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::common::any_value::Value;
use parallax_spool::Signal;

#[derive(Debug)]
struct SignalState {
    depth: AtomicUsize,
    high_water: AtomicUsize,
    retries: AtomicU64,
    drops: AtomicU64,
    unavailable: AtomicU64,
    rejects: AtomicU64,
    spool_fails: AtomicU64,
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
            retries: AtomicU64::new(0),
            drops: AtomicU64::new(0),
            unavailable: AtomicU64::new(0),
            rejects: AtomicU64::new(0),
            spool_fails: AtomicU64::new(0),
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
    pub retries: u64,
    pub drops: u64,
}

#[derive(Debug)]
pub(crate) struct IngestHealth {
    signals: [SignalState; 4],
    enqueue_outcomes: Counter<u64>,
    enqueue_wait: Histogram<f64>,
    queue_age: Histogram<f64>,
    retries: Counter<u64>,
    drops: Counter<u64>,
    ingress_rejects: Counter<u64>,
    spool_writes: Counter<u64>,
    unsupported_metrics: Counter<u64>,
    live_tail_lags: Counter<u64>,
    drain: Histogram<f64>,
    unsupported_metric: AtomicU64,
    live_tail_lag: AtomicU64,
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
                SignalState::new("sentry", capacity),
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
            ingress_rejects: meter
                .u64_counter("parallax.ingest.loss.ingress_reject")
                .with_unit("{batch}")
                .build(),
            spool_writes: meter
                .u64_counter("parallax.ingest.loss.spool_write")
                .with_unit("{batch}")
                .build(),
            unsupported_metrics: meter
                .u64_counter("parallax.ingest.loss.unsupported_metric")
                .with_unit("{metric}")
                .build(),
            live_tail_lags: meter
                .u64_counter("parallax.ingest.loss.live_tail_lag")
                .with_unit("{batch}")
                .build(),
            drain: meter
                .f64_histogram("parallax.ingest.worker.drain")
                .with_unit("s")
                .build(),
            self_metric_batches: AtomicU64::new(0),
            unsupported_metric: AtomicU64::new(0),
            live_tail_lag: AtomicU64::new(0),
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
        state.unavailable.fetch_add(1, Ordering::Relaxed);
        self.enqueue_wait
            .record(waited.as_secs_f64(), &state.attributes);
        self.enqueue_outcomes.add(1, &state.unavailable_attributes);
    }

    pub(crate) fn ingress_reject(&self, signal: Signal) {
        let state = self.state(signal);
        state.rejects.fetch_add(1, Ordering::Relaxed);
        self.ingress_rejects.add(1, &state.attributes);
    }

    pub(crate) fn spool_failed(&self, signal: Signal) {
        let state = self.state(signal);
        state.spool_fails.fetch_add(1, Ordering::Relaxed);
        self.spool_writes.add(1, &state.attributes);
    }

    pub(crate) fn unsupported_metric(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.unsupported_metric.fetch_add(count, Ordering::Relaxed);
        self.unsupported_metrics.add(count, &[]);
    }

    pub(crate) fn live_lagged(&self, skipped: u64) {
        if skipped == 0 {
            return;
        }
        self.live_tail_lag.fetch_add(skipped, Ordering::Relaxed);
        self.live_tail_lags.add(skipped, &[]);
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
        let state = self.state(signal);
        state.retries.fetch_add(1, Ordering::Relaxed);
        self.retries.add(1, &state.attributes);
    }

    pub(crate) fn terminal_drop(&self, signal: Signal) {
        let state = self.state(signal);
        state.drops.fetch_add(1, Ordering::Relaxed);
        self.drops.add(1, &state.attributes);
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
        for signal in [
            Signal::Traces,
            Signal::Logs,
            Signal::Metrics,
            Signal::Sentry,
        ] {
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
            retries: state.retries.load(Ordering::Relaxed),
            drops: state.drops.load(Ordering::Relaxed),
        }
    }

    pub(crate) fn degradation(&self) -> Option<String> {
        let mut reasons = Vec::new();
        let full = [
            ("traces", self.snapshot(Signal::Traces)),
            ("logs", self.snapshot(Signal::Logs)),
            ("metrics", self.snapshot(Signal::Metrics)),
        ]
        .into_iter()
        .filter(|(_, snapshot)| snapshot.depth >= snapshot.capacity)
        .map(|(signal, snapshot)| format!("{signal}={}/{}", snapshot.depth, snapshot.capacity))
        .collect::<Vec<_>>();
        if !full.is_empty() {
            reasons.push(format!("ingest queue full ({})", full.join(", ")));
        }
        let drops = self.sum_signal(|state| state.drops.load(Ordering::Relaxed));
        if drops > 0 {
            reasons.push(format!("ingest terminal drop ({drops})"));
        }
        let spool = self.sum_signal(|state| state.spool_fails.load(Ordering::Relaxed));
        if spool > 0 {
            reasons.push(format!("spool write failed ({spool})"));
        }
        (!reasons.is_empty()).then(|| reasons.join("; "))
    }

    pub(crate) fn loss_json(&self) -> String {
        format!(
            "{{\"queue_unavailable\":{},\"terminal_drop\":{},\"ingress_reject\":{},\"spool_write\":{},\"unsupported_metric\":{},\"live_tail_lag\":{}}}",
            self.sum_signal(|state| state.unavailable.load(Ordering::Relaxed)),
            self.sum_signal(|state| state.drops.load(Ordering::Relaxed)),
            self.sum_signal(|state| state.rejects.load(Ordering::Relaxed)),
            self.sum_signal(|state| state.spool_fails.load(Ordering::Relaxed)),
            self.unsupported_metric.load(Ordering::Relaxed),
            self.live_tail_lag.load(Ordering::Relaxed),
        )
    }

    fn sum_signal(&self, read: impl Fn(&SignalState) -> u64) -> u64 {
        self.signals.iter().map(read).sum()
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
            Signal::Sentry => 3,
        }]
    }
}

pub(crate) async fn health_handler(
    State(health): State<std::sync::Arc<IngestHealth>>,
) -> impl IntoResponse {
    if let Some(reason) = health.degradation() {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("degraded: {reason}"),
        )
    } else {
        (StatusCode::OK, "ok".to_string())
    }
}

pub(crate) async fn loss_handler(
    State(health): State<std::sync::Arc<IngestHealth>>,
) -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        health.loss_json(),
    )
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
