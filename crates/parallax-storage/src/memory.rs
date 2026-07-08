//! In-memory `TelemetryStore` — the fast test adapter and the engine of the
//! `--no-greptime` fallback's telemetry side (bounded).

use crate::adapter::{OverviewTotals, ServiceSummary, SignalKind, SpanRed, TelemetryStore};
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

fn quantile_from_sorted(values: &[u128], q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    if values.len() == 1 {
        return values[0] as f64;
    }
    let pos = q.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return values[lo] as f64;
    }
    let weight = pos - lo as f64;
    values[lo] as f64 + (values[hi] as f64 - values[lo] as f64) * weight
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
    // The in-memory adapter has no proto dependency, so it stores the decoded
    // tee rows and ignores the raw OTLP bytes the native forward would use.
    async fn ingest_traces(&self, spans: Vec<SpanRow>, _raw: bytes::Bytes) -> anyhow::Result<()> {
        self.lock().spans.extend(spans);
        Ok(())
    }

    async fn ingest_logs(&self, logs: Vec<LogRow>, _raw: bytes::Bytes) -> anyhow::Result<()> {
        self.lock().logs.extend(logs);
        Ok(())
    }

    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        histograms: Vec<HistogramRow>,
        _raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        let mut inner = self.lock();
        inner.metric_points.extend(points);
        inner.histograms.extend(histograms);
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
        spans.sort_by_key(|s| std::cmp::Reverse(s.ts_nanos));
        spans.truncate(limit);
        spans.sort_by_key(|s| s.ts_nanos);
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
        logs.sort_by_key(|l| std::cmp::Reverse(l.ts_nanos));
        logs.truncate(limit);
        logs.sort_by_key(|l| l.ts_nanos);
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
            .chain(inner.logs.iter().map(|l| l.service.clone()))
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    async fn overview_totals(&self, range: RangeInclusive<u128>) -> anyhow::Result<OverviewTotals> {
        let inner = self.lock();
        let spans: Vec<&SpanRow> = inner
            .spans
            .iter()
            .filter(|s| range.contains(&s.ts_nanos))
            .collect();
        let logs = inner
            .logs
            .iter()
            .filter(|l| range.contains(&l.ts_nanos))
            .count() as u64;
        let metric_points = inner
            .metric_points
            .iter()
            .filter(|p| range.contains(&p.ts_nanos))
            .count() as u64
            + inner
                .histograms
                .iter()
                .filter(|h| range.contains(&h.ts_nanos))
                .count() as u64;
        let errors = spans
            .iter()
            .filter(|s| s.status_code == "STATUS_CODE_ERROR")
            .count() as u64;
        let trace_count = spans
            .iter()
            .map(|s| s.trace_id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len() as u64;
        let active_services = spans
            .iter()
            .map(|s| s.service.as_str())
            .chain(
                inner
                    .logs
                    .iter()
                    .filter(|l| range.contains(&l.ts_nanos))
                    .map(|l| l.service.as_str()),
            )
            .chain(
                inner
                    .metric_points
                    .iter()
                    .filter(|p| range.contains(&p.ts_nanos))
                    .map(|p| p.service.as_str()),
            )
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|h| range.contains(&h.ts_nanos))
                    .map(|h| h.service.as_str()),
            )
            .collect::<std::collections::BTreeSet<_>>()
            .len() as u64;
        let span_count = spans.len() as u64;
        Ok(OverviewTotals {
            span_count,
            trace_count,
            log_count: logs,
            metric_point_count: metric_points,
            error_count: errors,
            error_rate: if span_count == 0 {
                0.0
            } else {
                errors as f64 / span_count as f64
            },
            active_services,
        })
    }

    async fn signal_count_series(
        &self,
        kind: SignalKind,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let inner = self.lock();
        let mut buckets: std::collections::BTreeMap<u128, u64> = Default::default();
        match kind {
            SignalKind::Spans => {
                for span in inner.spans.iter().filter(|s| {
                    range.contains(&s.ts_nanos) && service.is_none_or(|svc| s.service == svc)
                }) {
                    *buckets.entry((span.ts_nanos / step) * step).or_default() += 1;
                }
            }
            SignalKind::Traces => {
                let mut traces: std::collections::BTreeMap<u128, std::collections::BTreeSet<&str>> =
                    Default::default();
                for span in inner.spans.iter().filter(|s| {
                    range.contains(&s.ts_nanos) && service.is_none_or(|svc| s.service == svc)
                }) {
                    traces
                        .entry((span.ts_nanos / step) * step)
                        .or_default()
                        .insert(span.trace_id.as_str());
                }
                return Ok(traces
                    .into_iter()
                    .map(|(ts_nanos, trace_ids)| SeriesPoint {
                        ts_nanos,
                        value: trace_ids.len() as f64,
                    })
                    .collect());
            }
            SignalKind::Logs => {
                for log in inner.logs.iter().filter(|l| {
                    range.contains(&l.ts_nanos) && service.is_none_or(|svc| l.service == svc)
                }) {
                    *buckets.entry((log.ts_nanos / step) * step).or_default() += 1;
                }
            }
            SignalKind::Errors => {
                for event in inner.error_events.iter().filter(|e| {
                    range.contains(&e.ts_nanos) && service.is_none_or(|svc| e.service == svc)
                }) {
                    *buckets.entry((event.ts_nanos / step) * step).or_default() += 1;
                }
            }
            SignalKind::MetricPoints => {
                for point in inner.metric_points.iter().filter(|p| {
                    range.contains(&p.ts_nanos) && service.is_none_or(|svc| p.service == svc)
                }) {
                    *buckets.entry((point.ts_nanos / step) * step).or_default() += 1;
                }
                for row in inner.histograms.iter().filter(|h| {
                    range.contains(&h.ts_nanos) && service.is_none_or(|svc| h.service == svc)
                }) {
                    *buckets.entry((row.ts_nanos / step) * step).or_default() += 1;
                }
            }
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, count)| SeriesPoint {
                ts_nanos,
                value: count as f64,
            })
            .collect())
    }

    async fn service_summaries(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceSummary>> {
        let inner = self.lock();
        let mut by_service: std::collections::BTreeMap<&str, Vec<&SpanRow>> = Default::default();
        for span in inner.spans.iter().filter(|s| range.contains(&s.ts_nanos)) {
            by_service.entry(&span.service).or_default().push(span);
        }
        let mut summaries: Vec<_> = by_service
            .into_iter()
            .map(|(name, spans)| {
                let mut durations: Vec<u128> = spans.iter().map(|s| s.duration_ns).collect();
                durations.sort_unstable();
                ServiceSummary {
                    name: name.to_owned(),
                    last_seen_nanos: spans.iter().map(|s| s.ts_nanos).max().unwrap_or(0),
                    span_count: spans.len() as u64,
                    error_count: spans
                        .iter()
                        .filter(|s| s.status_code == "STATUS_CODE_ERROR")
                        .count() as u64,
                    p95_ms: Some(quantile_from_sorted(&durations, 0.95) / 1_000_000.0),
                }
            })
            .collect();
        summaries.sort_by_key(|s| std::cmp::Reverse(s.last_seen_nanos));
        Ok(summaries)
    }

    async fn span_red_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<SpanRed> {
        let step = step_nanos.max(1);
        let step_secs = step as f64 / 1_000_000_000.0;
        let inner = self.lock();
        let mut buckets: std::collections::BTreeMap<u128, Vec<&SpanRow>> = Default::default();
        for span in inner
            .spans
            .iter()
            .filter(|s| range.contains(&s.ts_nanos) && service.is_none_or(|svc| s.service == svc))
        {
            buckets
                .entry((span.ts_nanos / step) * step)
                .or_default()
                .push(span);
        }
        let mut red = SpanRed::default();
        for (ts_nanos, spans) in buckets {
            let count = spans.len() as f64;
            let errors = spans
                .iter()
                .filter(|s| s.status_code == "STATUS_CODE_ERROR")
                .count() as f64;
            let mut durations: Vec<u128> = spans.iter().map(|s| s.duration_ns).collect();
            durations.sort_unstable();
            red.rate.push(SeriesPoint {
                ts_nanos,
                value: count / step_secs,
            });
            red.error_rate.push(SeriesPoint {
                ts_nanos,
                value: if count == 0.0 { 0.0 } else { errors / count },
            });
            red.p50.push(SeriesPoint {
                ts_nanos,
                value: quantile_from_sorted(&durations, 0.50) / 1_000_000.0,
            });
            red.p95.push(SeriesPoint {
                ts_nanos,
                value: quantile_from_sorted(&durations, 0.95) / 1_000_000.0,
            });
            red.p99.push(SeriesPoint {
                ts_nanos,
                value: quantile_from_sorted(&durations, 0.99) / 1_000_000.0,
            });
        }
        Ok(red)
    }

    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
        run_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, Vec<f64>> = Default::default();
        for point in self.lock().metric_points.iter().filter(|p| {
            p.name == name
                && service.is_none_or(|svc| p.service == svc)
                && run_id.is_none_or(|id| p.run_id.as_deref() == Some(id))
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

    async fn traces_search(
        &self,
        query: &crate::adapter::TraceQuery,
    ) -> anyhow::Result<crate::adapter::TraceList> {
        let inner = self.lock();
        // `service` matches any trace the service participates in (a span of
        // that service anywhere), not only the root span.
        let participating: Option<std::collections::HashSet<&str>> =
            query.service.as_deref().map(|svc| {
                inner
                    .spans
                    .iter()
                    .filter(|s| s.service == svc)
                    .map(|s| s.trace_id.as_str())
                    .collect()
            });
        // Representative span per trace: the root (no parent), else — when no
        // root was stored — the earliest span, so all-INTERNAL traces still
        // list instead of vanishing.
        let mut rep: std::collections::HashMap<&str, &SpanRow> = std::collections::HashMap::new();
        for span in &inner.spans {
            let is_root = span.parent_span_id.as_deref().is_none_or(str::is_empty);
            match rep.get(span.trace_id.as_str()) {
                None => {
                    rep.insert(&span.trace_id, span);
                }
                Some(cur) => {
                    let cur_root = cur.parent_span_id.as_deref().is_none_or(str::is_empty);
                    // Prefer a root; among the same class prefer the earliest.
                    let replace = match (cur_root, is_root) {
                        (false, true) => true,
                        (true, false) => false,
                        _ => span.ts_nanos < cur.ts_nanos,
                    };
                    if replace {
                        rep.insert(&span.trace_id, span);
                    }
                }
            }
        }
        // Representative-span filters; newest first.
        let roots: Vec<&SpanRow> = rep
            .into_values()
            .filter(|s| {
                participating
                    .as_ref()
                    .is_none_or(|set| set.contains(s.trace_id.as_str()))
            })
            .filter(|s| query.from_nanos.is_none_or(|from| s.ts_nanos >= from))
            .filter(|s| query.to_nanos.is_none_or(|to| s.ts_nanos <= to))
            .filter(|s| query.min_duration_ns.is_none_or(|min| s.duration_ns >= min))
            .filter(|s| query.max_duration_ns.is_none_or(|max| s.duration_ns <= max))
            .filter(|s| {
                query
                    .name_contains
                    .as_deref()
                    .is_none_or(|needle| s.name.contains(needle))
            })
            .collect();
        let mut traces: Vec<_> = roots
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
            .collect();
        if query.error_only {
            traces.retain(|t| t.has_error);
        }
        match query.sort {
            crate::adapter::TraceSort::StartDesc => {
                traces.sort_by_key(|t| std::cmp::Reverse(t.start_nanos));
            }
            crate::adapter::TraceSort::DurationDesc => {
                traces.sort_by_key(|t| std::cmp::Reverse(t.duration_ns));
            }
            crate::adapter::TraceSort::DurationAsc => traces.sort_by_key(|t| t.duration_ns),
            crate::adapter::TraceSort::SpanCountDesc => {
                traces.sort_by_key(|t| std::cmp::Reverse(t.span_count));
            }
        }
        let total = traces.len() as u64;
        let items = traces
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();
        Ok(crate::adapter::TraceList { items, total })
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
        severity_max: Option<i32>,
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
                    && severity_max.is_none_or(|max| l.severity_num <= max)
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
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, u64> = Default::default();
        for log in self.lock().logs.iter().filter(|l| {
            range.contains(&l.ts_nanos)
                && service.is_none_or(|svc| l.service == svc)
                && severity_min.is_none_or(|min| l.severity_num >= min)
                && severity_max.is_none_or(|max| l.severity_num <= max)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::{TelemetryStore, TraceQuery, TraceSort};

    fn span(trace: &str, span_id: &str, parent: Option<&str>, service: &str, ts: u128) -> SpanRow {
        SpanRow {
            ts_nanos: ts,
            service: service.into(),
            trace_id: trace.into(),
            span_id: span_id.into(),
            parent_span_id: parent.map(Into::into),
            name: format!("{service}-{span_id}"),
            kind: "SPAN_KIND_INTERNAL".into(),
            status_code: "STATUS_CODE_UNSET".into(),
            status_message: String::new(),
            duration_ns: 1_000,
            run_id: None,
            scope_name: String::new(),
            events: None,
            links: serde_json::Value::Null,
            attributes: serde_json::Value::Null,
            resource: serde_json::Value::Null,
        }
    }

    fn span_with_duration(
        trace: &str,
        span_id: &str,
        parent: Option<&str>,
        service: &str,
        ts: u128,
        duration_ns: u128,
    ) -> SpanRow {
        let mut row = span(trace, span_id, parent, service, ts);
        row.duration_ns = duration_ns;
        row
    }

    fn error_event(service: &str, ts: u128) -> ErrorEventRow {
        ErrorEventRow {
            ts_nanos: ts,
            service: service.into(),
            fingerprint: format!("{service}-fp"),
            error_type: "Error".into(),
            message: "boom".into(),
            stacktrace: None,
            source: ErrorSource::SpanStatus,
            trace_id: format!("{service}-trace"),
            span_id: format!("{service}-span"),
            attributes: serde_json::Value::Null,
        }
    }

    fn log(run_id: Option<&str>, ts: u128, severity_num: i32) -> LogRow {
        LogRow {
            ts_nanos: ts,
            service: "api".into(),
            severity_num,
            severity_text: format!("S{severity_num}"),
            body: format!("log-{ts}"),
            trace_id: format!("trace-{ts}"),
            span_id: format!("span-{ts}"),
            run_id: run_id.map(Into::into),
            scope_name: String::new(),
            attributes: serde_json::Value::Null,
            resource: serde_json::Value::Null,
        }
    }

    fn query(service: Option<&str>) -> TraceQuery {
        TraceQuery {
            service: service.map(Into::into),
            limit: 50,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn run_anchored_reads_keep_newest_limit_in_ascending_order() {
        let store = MemoryStore::new();
        let mut spans = Vec::new();
        let mut logs = Vec::new();
        for index in 0..250u128 {
            let mut span = span(
                &format!("trace-{index}"),
                &format!("span-{index}"),
                None,
                "api",
                index,
            );
            span.run_id = Some("run-1".into());
            spans.push(span);
            logs.push(log(Some("run-1"), index, 9));
        }
        store
            .ingest_traces(spans, bytes::Bytes::new())
            .await
            .unwrap();
        store.ingest_logs(logs, bytes::Bytes::new()).await.unwrap();

        let spans = store.spans_by_run("run-1", 200).await.unwrap();
        let logs = store.logs_by_run("run-1", 200).await.unwrap();

        assert_eq!(spans.len(), 200);
        assert_eq!(logs.len(), 200);
        assert_eq!(spans.first().map(|span| span.ts_nanos), Some(50));
        assert_eq!(logs.first().map(|log| log.ts_nanos), Some(50));
        assert_eq!(spans.last().map(|span| span.ts_nanos), Some(249));
        assert_eq!(logs.last().map(|log| log.ts_nanos), Some(249));
    }

    #[tokio::test]
    async fn log_severity_max_bounds_search_and_count_series() {
        let store = MemoryStore::new();
        store
            .ingest_logs(
                vec![
                    log(None, 5, 5),
                    log(None, 9, 9),
                    log(None, 13, 13),
                    log(None, 17, 17),
                ],
                bytes::Bytes::new(),
            )
            .await
            .unwrap();

        let logs = store
            .logs_search(None, 0..=100, Some(5), Some(8), None, 10)
            .await
            .unwrap();
        let series = store
            .log_count_series(None, 0..=100, Some(5), Some(8), None, 1)
            .await
            .unwrap();

        assert_eq!(
            logs.iter().map(|log| log.severity_num).collect::<Vec<_>>(),
            vec![5]
        );
        assert_eq!(series.iter().map(|point| point.value).sum::<f64>(), 1.0);
    }

    // A non-root span of a participating service surfaces the whole trace,
    // represented by its real root (the cross-service `--service catalog` bug).
    #[tokio::test]
    async fn service_filter_matches_participation_not_just_root() {
        let store = MemoryStore::new();
        store
            .ingest_traces(
                vec![
                    span("t1", "a", None, "checkout", 10),
                    span("t1", "b", Some("a"), "catalog", 20),
                ],
                bytes::Bytes::new(),
            )
            .await
            .unwrap();

        let by_catalog = store.traces_search(&query(Some("catalog"))).await.unwrap();
        let by_catalog = by_catalog.items;
        assert_eq!(by_catalog.len(), 1, "catalog participates in t1");
        assert_eq!(by_catalog[0].trace_id, "t1");
        assert_eq!(
            by_catalog[0].service, "checkout",
            "summary uses the trace root, not the filtered service"
        );
        assert_eq!(by_catalog[0].span_count, 2);

        let absent = store.traces_search(&query(Some("payment"))).await.unwrap();
        assert!(absent.items.is_empty(), "payment is in no trace");
    }

    // A trace with no stored root (all spans parented elsewhere) still lists,
    // represented by its earliest span.
    #[tokio::test]
    async fn rootless_trace_lists_via_earliest_span() {
        let store = MemoryStore::new();
        store
            .ingest_traces(
                vec![
                    span("t2", "y", Some("missing-parent"), "catalog", 30),
                    span("t2", "x", Some("missing-parent"), "catalog", 15),
                ],
                bytes::Bytes::new(),
            )
            .await
            .unwrap();

        let traces = store.traces_search(&query(None)).await.unwrap();
        let traces = traces.items;
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].trace_id, "t2");
        assert_eq!(
            traces[0].start_nanos, 15,
            "earliest span represents a rootless trace"
        );
        assert_eq!(traces[0].span_count, 2);
    }

    #[tokio::test]
    async fn trace_search_sorts_offsets_and_filters_duration_band() {
        let store = MemoryStore::new();
        store
            .ingest_traces(
                vec![
                    span_with_duration("fast", "a", None, "api", 10, 10),
                    span_with_duration("mid", "b", None, "api", 20, 20),
                    span_with_duration("slow", "c", None, "api", 30, 30),
                    span_with_duration("wide", "d", None, "api", 40, 25),
                    span_with_duration("wide", "e", Some("d"), "api", 45, 5),
                ],
                bytes::Bytes::new(),
            )
            .await
            .unwrap();

        let result = store
            .traces_search(&TraceQuery {
                min_duration_ns: Some(15),
                max_duration_ns: Some(30),
                sort: TraceSort::DurationDesc,
                limit: 2,
                offset: 1,
                ..TraceQuery::default()
            })
            .await
            .unwrap();
        assert_eq!(result.total, 3);
        assert_eq!(
            result
                .items
                .iter()
                .map(|t| t.trace_id.as_str())
                .collect::<Vec<_>>(),
            vec!["wide", "mid"]
        );

        let by_span_count = store
            .traces_search(&TraceQuery {
                sort: TraceSort::SpanCountDesc,
                limit: 1,
                ..TraceQuery::default()
            })
            .await
            .unwrap();
        assert_eq!(by_span_count.items[0].trace_id, "wide");
        assert_eq!(by_span_count.items[0].span_count, 2);
    }

    #[tokio::test]
    async fn overview_totals_and_signal_series_cover_seeded_window() {
        let store = MemoryStore::new();
        let mut ok = span("t1", "a", None, "api", 1_000_000_000);
        ok.duration_ns = 1_000_000;
        let mut err = span("t1", "b", Some("a"), "api", 1_500_000_000);
        err.status_code = "STATUS_CODE_ERROR".into();
        err.duration_ns = 9_000_000;
        store
            .ingest_traces(vec![ok, err], bytes::Bytes::new())
            .await
            .unwrap();
        store
            .ingest_logs(
                vec![LogRow {
                    ts_nanos: 1_250_000_000,
                    service: "api".into(),
                    severity_num: 17,
                    severity_text: "ERROR".into(),
                    body: "bad".into(),
                    trace_id: "t1".into(),
                    span_id: "b".into(),
                    run_id: None,
                    scope_name: String::new(),
                    attributes: serde_json::Value::Null,
                    resource: serde_json::Value::Null,
                }],
                bytes::Bytes::new(),
            )
            .await
            .unwrap();
        store
            .write_error_events(vec![error_event("api", 1_600_000_000)])
            .await
            .unwrap();

        let totals = store.overview_totals(0..=2_000_000_000).await.unwrap();
        assert_eq!(totals.span_count, 2);
        assert_eq!(totals.trace_count, 1);
        assert_eq!(totals.log_count, 1);
        assert_eq!(totals.error_count, 1);
        assert_eq!(totals.active_services, 1);
        assert_eq!(totals.error_rate, 0.5);

        let spans = store
            .signal_count_series(
                SignalKind::Spans,
                Some("api"),
                0..=2_000_000_000,
                1_000_000_000,
            )
            .await
            .unwrap();
        assert_eq!(spans[0].value, 2.0);
        let errors = store
            .signal_count_series(
                SignalKind::Errors,
                Some("api"),
                0..=2_000_000_000,
                1_000_000_000,
            )
            .await
            .unwrap();
        assert_eq!(errors[0].value, 1.0);
    }

    #[tokio::test]
    async fn service_summaries_and_red_use_trace_durations() {
        let store = MemoryStore::new();
        let mut fast = span("t1", "a", None, "api", 1_000_000_000);
        fast.duration_ns = 10_000_000;
        let mut slow = span("t2", "b", None, "api", 1_500_000_000);
        slow.duration_ns = 30_000_000;
        slow.status_code = "STATUS_CODE_ERROR".into();
        let mut other = span("t3", "c", None, "worker", 1_800_000_000);
        other.duration_ns = 50_000_000;
        store
            .ingest_traces(vec![fast, slow, other], bytes::Bytes::new())
            .await
            .unwrap();

        let summaries = store.service_summaries(0..=2_000_000_000).await.unwrap();
        assert_eq!(summaries[0].name, "worker");
        let api = summaries.iter().find(|s| s.name == "api").unwrap();
        assert_eq!(api.span_count, 2);
        assert_eq!(api.error_count, 1);
        assert_eq!(api.p95_ms, Some(29.0));

        let red = store
            .span_red_series(Some("api"), 0..=2_000_000_000, 1_000_000_000)
            .await
            .unwrap();
        assert_eq!(red.rate[0].value, 2.0);
        assert_eq!(red.error_rate[0].value, 0.5);
        assert_eq!(red.p50[0].value, 20.0);
        assert_eq!(red.p95[0].value, 29.0);
        assert_eq!(red.p99[0].value, 29.8);
    }
}
