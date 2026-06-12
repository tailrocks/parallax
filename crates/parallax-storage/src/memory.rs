//! In-memory `TelemetryStore` — the fast test adapter and the engine of the
//! `--no-greptime` fallback's telemetry side (bounded).

use crate::adapter::TelemetryStore;
use crate::model::*;
use std::ops::RangeInclusive;
use std::sync::Mutex;

/// Render one attribute value for grouping — scalars only, like the tag
/// cache; missing/nested values group under "(none)".
pub(crate) fn group_value(attributes: &serde_json::Value, key: &str) -> String {
    match attributes.get(key) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Bool(b)) => b.to_string(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        _ => "(none)".to_string(),
    }
}

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

    async fn observed_runs(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::adapter::ObservedRun>> {
        let inner = self.lock();
        let mut runs: std::collections::HashMap<String, crate::adapter::ObservedRun> =
            std::collections::HashMap::new();
        let mut absorb = |run_id: &Option<String>, ts: u128, service: &str, is_span: bool| {
            let Some(run_id) = run_id.as_deref().filter(|r| !r.is_empty()) else {
                return;
            };
            let entry =
                runs.entry(run_id.to_owned())
                    .or_insert_with(|| crate::adapter::ObservedRun {
                        run_id: run_id.to_owned(),
                        first_nanos: ts,
                        last_nanos: ts,
                        span_count: 0,
                        log_count: 0,
                        service: service.to_owned(),
                    });
            entry.first_nanos = entry.first_nanos.min(ts);
            entry.last_nanos = entry.last_nanos.max(ts);
            if is_span {
                entry.span_count += 1;
            } else {
                entry.log_count += 1;
            }
        };
        for span in &inner.spans {
            absorb(&span.run_id, span.ts_nanos, &span.service, true);
        }
        for log in &inner.logs {
            absorb(&log.run_id, log.ts_nanos, &log.service, false);
        }
        let mut runs: Vec<_> = runs.into_values().collect();
        runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
        runs.truncate(limit);
        Ok(runs)
    }

    async fn recent_traces(
        &self,
        limit: usize,
    ) -> anyhow::Result<Vec<crate::adapter::TraceSummary>> {
        let inner = self.lock();
        // Roots first (no parent); newest first.
        let mut roots: Vec<&SpanRow> = inner
            .spans
            .iter()
            .filter(|s| s.parent_span_id.as_deref().is_none_or(str::is_empty))
            .collect();
        roots.sort_by_key(|s| std::cmp::Reverse(s.ts_nanos));
        roots.truncate(limit);
        Ok(roots
            .into_iter()
            .map(|root| {
                let mut span_count = 0;
                let mut has_error = false;
                for span in &inner.spans {
                    if span.trace_id == root.trace_id {
                        span_count += 1;
                        has_error |= span.status_code == "STATUS_CODE_ERROR";
                    }
                }
                crate::adapter::TraceSummary {
                    trace_id: root.trace_id.clone(),
                    root_name: root.name.clone(),
                    service: root.service.clone(),
                    start_nanos: root.ts_nanos,
                    duration_ns: root.duration_ns,
                    span_count,
                    has_error,
                }
            })
            .collect())
    }

    async fn error_events_by_traces(
        &self,
        trace_ids: &[String],
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>> {
        if trace_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut events: Vec<ErrorEventRow> = self
            .lock()
            .error_events
            .iter()
            .filter(|e| trace_ids.contains(&e.trace_id))
            .cloned()
            .collect();
        events.sort_by_key(|e| std::cmp::Reverse(e.ts_nanos));
        events.truncate(limit);
        Ok(events)
    }

    async fn logs_search(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        body_contains: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<LogRow>> {
        let mut logs: Vec<LogRow> = self
            .lock()
            .logs
            .iter()
            .filter(|l| {
                range.contains(&l.ts_nanos)
                    && service.is_none_or(|svc| l.service == svc)
                    && severity_min.is_none_or(|min| l.severity_num >= min)
                    && body_contains.is_none_or(|needle| l.body.contains(needle))
            })
            .cloned()
            .collect();
        logs.sort_by_key(|l| std::cmp::Reverse(l.ts_nanos));
        logs.truncate(limit);
        Ok(logs)
    }

    async fn metric_series_grouped(
        &self,
        name: &str,
        service: Option<&str>,
        group_by: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<Vec<(String, Vec<SeriesPoint>)>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<(String, u128), Vec<f64>> = Default::default();
        for point in self.lock().metric_points.iter().filter(|p| {
            p.name == name
                && service.is_none_or(|svc| p.service == svc)
                && range.contains(&p.ts_nanos)
        }) {
            buckets
                .entry((
                    group_value(&point.attributes, group_by),
                    (point.ts_nanos / step) * step,
                ))
                .or_default()
                .push(point.value);
        }
        let mut groups: std::collections::BTreeMap<String, Vec<SeriesPoint>> = Default::default();
        for ((group, ts_nanos), values) in buckets {
            let value = match agg {
                MetricAgg::Avg => values.iter().sum::<f64>() / values.len() as f64,
                MetricAgg::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
                MetricAgg::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                MetricAgg::Sum | MetricAgg::Rate => values.iter().sum::<f64>(),
            };
            groups
                .entry(group)
                .or_default()
                .push(SeriesPoint { ts_nanos, value });
        }
        Ok(groups
            .into_iter()
            .map(|(group, series)| {
                let series = if agg == MetricAgg::Rate {
                    rate_from_buckets(&series, step)
                } else {
                    series
                };
                (group, series)
            })
            .collect())
    }

    async fn histogram_count_series(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, u64> = Default::default();
        for row in self.lock().histograms.iter().filter(|h| {
            h.name == name
                && service.is_none_or(|svc| h.service == svc)
                && range.contains(&h.ts_nanos)
        }) {
            *buckets.entry((row.ts_nanos / step) * step).or_default() += row.count;
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, count)| SeriesPoint {
                ts_nanos,
                value: count as f64,
            })
            .collect())
    }

    async fn error_count_series(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, u64> = Default::default();
        for event in self
            .lock()
            .error_events
            .iter()
            .filter(|e| e.service == service && range.contains(&e.ts_nanos))
        {
            *buckets.entry((event.ts_nanos / step) * step).or_default() += 1;
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, count)| SeriesPoint {
                ts_nanos,
                value: count as f64,
            })
            .collect())
    }

    async fn raw_sql(&self, _query: &str) -> anyhow::Result<crate::adapter::SqlResult> {
        anyhow::bail!(
            "raw SQL needs the GreptimeDB engine; the in-memory store \
             (storage.mode = \"none\") has no SQL surface"
        )
    }

    async fn log_count_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        body_contains: Option<&str>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, u64> = Default::default();
        for log in self.lock().logs.iter().filter(|l| {
            range.contains(&l.ts_nanos)
                && service.is_none_or(|svc| l.service == svc)
                && severity_min.is_none_or(|min| l.severity_num >= min)
                && body_contains.is_none_or(|needle| l.body.contains(needle))
        }) {
            *buckets.entry((log.ts_nanos / step) * step).or_default() += 1;
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, count)| SeriesPoint {
                ts_nanos,
                value: count as f64,
            })
            .collect())
    }
}
