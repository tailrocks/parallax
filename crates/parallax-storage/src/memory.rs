//! In-memory `TelemetryStore` — the fast test adapter and the engine of the
//! `--no-greptime` fallback's telemetry side (bounded).

use crate::adapter::TelemetryStore;
use crate::model::*;
use std::ops::RangeInclusive;
use std::sync::Mutex;

/// Per-second rate from bucketed counter sums (monotonic resets clamp to 0).
pub(crate) fn rate_from_buckets(series: &[SeriesPoint], step_nanos: u128) -> Vec<SeriesPoint> {
    let step_secs = step_nanos as f64 / 1e9;
    series
        .windows(2)
        .map(|w| SeriesPoint {
            ts_nanos: w[1].ts_nanos,
            value: ((w[1].value - w[0].value).max(0.0)) / step_secs,
        })
        .collect()
}

/// Linear-interpolated quantile from merged explicit-bounds histograms.
pub(crate) fn quantile_from_histograms(rows: &[HistogramRow], q: f64) -> f64 {
    let Some(first) = rows.first() else {
        return 0.0;
    };
    let bounds = &first.bounds;
    let mut counts = vec![0u64; bounds.len() + 1];
    for row in rows {
        for (i, c) in row.bucket_counts.iter().enumerate() {
            if let Some(slot) = counts.get_mut(i) {
                *slot += c;
            }
        }
    }
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let target = q.clamp(0.0, 1.0) * total as f64;
    let mut cumulative = 0u64;
    for (i, count) in counts.iter().enumerate() {
        let next = cumulative + count;
        if next as f64 >= target {
            let lower = if i == 0 { 0.0 } else { bounds[i - 1] };
            let upper = bounds.get(i).copied().unwrap_or(lower);
            let within = if *count == 0 {
                0.0
            } else {
                (target - cumulative as f64) / *count as f64
            };
            return lower + (upper - lower) * within;
        }
        cumulative = next;
    }
    bounds.last().copied().unwrap_or(0.0)
}

#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    spans: Vec<SpanRow>,
    logs: Vec<LogRow>,
    metric_points: Vec<MetricPointRow>,
    histograms: Vec<HistogramRow>,
    error_events: Vec<ErrorEventRow>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // A poisoned lock only happens after a panic while holding it; the
        // data is plain rows, safe to keep serving.
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let inner = self.lock();
        (
            inner.spans.len(),
            inner.logs.len(),
            inner.metric_points.len() + inner.histograms.len(),
            inner.error_events.len(),
        )
    }
}

#[async_trait::async_trait]
impl TelemetryStore for MemoryStore {
    async fn write_spans(&self, rows: Vec<SpanRow>) -> anyhow::Result<()> {
        self.lock().spans.extend(rows);
        Ok(())
    }

    async fn write_logs(&self, rows: Vec<LogRow>) -> anyhow::Result<()> {
        self.lock().logs.extend(rows);
        Ok(())
    }

    async fn write_metric_points(&self, rows: Vec<MetricPointRow>) -> anyhow::Result<()> {
        self.lock().metric_points.extend(rows);
        Ok(())
    }

    async fn write_histograms(&self, rows: Vec<HistogramRow>) -> anyhow::Result<()> {
        self.lock().histograms.extend(rows);
        Ok(())
    }

    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> anyhow::Result<()> {
        self.lock().error_events.extend(rows);
        Ok(())
    }

    async fn spans_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<SpanRow>> {
        let mut spans: Vec<SpanRow> = self
            .lock()
            .spans
            .iter()
            .filter(|s| s.trace_id == trace_id)
            .cloned()
            .collect();
        spans.sort_by_key(|s| s.ts_nanos);
        Ok(spans)
    }

    async fn spans_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<SpanRow>> {
        let mut spans: Vec<SpanRow> = self
            .lock()
            .spans
            .iter()
            .filter(|s| s.run_id.as_deref() == Some(run_id))
            .cloned()
            .collect();
        spans.sort_by_key(|s| s.ts_nanos);
        spans.truncate(limit);
        Ok(spans)
    }

    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>> {
        let mut logs: Vec<LogRow> = self
            .lock()
            .logs
            .iter()
            .filter(|l| l.run_id.as_deref() == Some(run_id))
            .cloned()
            .collect();
        logs.sort_by_key(|l| l.ts_nanos);
        logs.truncate(limit);
        Ok(logs)
    }

    async fn logs_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<LogRow>> {
        let mut logs: Vec<LogRow> = self
            .lock()
            .logs
            .iter()
            .filter(|l| l.trace_id == trace_id)
            .cloned()
            .collect();
        logs.sort_by_key(|l| l.ts_nanos);
        Ok(logs)
    }

    async fn metric_names(&self) -> anyhow::Result<Vec<String>> {
        let inner = self.lock();
        let mut names: Vec<String> = inner
            .metric_points
            .iter()
            .map(|p| p.name.clone())
            .chain(inner.histograms.iter().map(|h| h.name.clone()))
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    async fn service_names(&self) -> anyhow::Result<Vec<String>> {
        let inner = self.lock();
        let mut names: Vec<String> = inner
            .metric_points
            .iter()
            .map(|p| p.service.clone())
            .chain(inner.spans.iter().map(|s| s.service.clone()))
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, Vec<f64>> = Default::default();
        for point in self.lock().metric_points.iter().filter(|p| {
            p.name == name
                && service.is_none_or(|svc| p.service == svc)
                && range.contains(&p.ts_nanos)
        }) {
            buckets
                .entry((point.ts_nanos / step) * step)
                .or_default()
                .push(point.value);
        }
        let mut series: Vec<SeriesPoint> = buckets
            .into_iter()
            .map(|(ts_nanos, values)| {
                let value = match agg {
                    MetricAgg::Avg => values.iter().sum::<f64>() / values.len() as f64,
                    MetricAgg::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
                    MetricAgg::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    // RATE starts from the per-bucket max of the counter.
                    MetricAgg::Sum | MetricAgg::Rate => values.iter().sum::<f64>(),
                };
                SeriesPoint { ts_nanos, value }
            })
            .collect();
        if agg == MetricAgg::Rate {
            series = rate_from_buckets(&series, step);
        }
        Ok(series)
    }

    async fn histogram_quantile(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        q: f64,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, Vec<HistogramRow>> = Default::default();
        for row in self.lock().histograms.iter().filter(|h| {
            h.name == name
                && service.is_none_or(|svc| h.service == svc)
                && range.contains(&h.ts_nanos)
        }) {
            buckets
                .entry((row.ts_nanos / step) * step)
                .or_default()
                .push(row.clone());
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, rows)| SeriesPoint {
                ts_nanos,
                value: quantile_from_histograms(&rows, q),
            })
            .collect())
    }

    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>> {
        let mut events: Vec<ErrorEventRow> = self
            .lock()
            .error_events
            .iter()
            .filter(|e| e.fingerprint == fingerprint && range.contains(&e.ts_nanos))
            .cloned()
            .collect();
        events.sort_by_key(|e| std::cmp::Reverse(e.ts_nanos));
        events.truncate(limit);
        Ok(events)
    }
}
