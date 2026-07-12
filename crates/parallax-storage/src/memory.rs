//! In-memory `TelemetryStore` — the fast test adapter and the engine of the
//! `--no-greptime` fallback's telemetry side (bounded).

use crate::adapter::{
    ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT, ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow,
    FIELD_KEYS_CAP, FIELD_TOP_VALUES_CAP, FieldKey, FieldSource, FieldStats, FieldValueCount,
    MAX_ROWS, OverviewTotals, ReleaseWindow, RuntimeMetricSeries, SERVICE_MAP_TRACE_CAP,
    ServiceCatalogRow, ServiceEdge, ServiceSummary, SignalKind, SpanRed, TelemetryStore,
    attribute_compare_key_allowed, attribute_compare_score, attribute_compare_value_allowed,
    field_key_identifier_like, field_key_namespace, field_value_display,
    metric_group_label_allowed, runtime_metric_family, runtime_metric_unit, span_field_key_allowed,
};
use crate::model::*;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::semconv;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::RangeInclusive;
use std::sync::Mutex;
use tokio::sync::{Mutex as AsyncMutex, oneshot};

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

fn scalar_attribute_value(attributes: &serde_json::Value, key: &str) -> Option<String> {
    let value = match attributes.get(key)? {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        _ => return None,
    };
    attribute_compare_value_allowed(&value).then_some(value)
}

fn field_scalar_value(attributes: &serde_json::Value, key: &str) -> Option<String> {
    let value = match attributes.get(key)? {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        _ => return None,
    };
    field_value_display(&value)
}

fn resource_string(resource: &serde_json::Value, key: &str) -> Option<String> {
    resource
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn span_matches_compare(
    span: &SpanRow,
    range: &RangeInclusive<u128>,
    service: Option<&str>,
    error_only: bool,
) -> bool {
    range.contains(&span.ts_nanos)
        && service.is_none_or(|svc| span.service == svc)
        && (!error_only || span.status_code == "STATUS_CODE_ERROR")
}

fn duration_quantile_ms(durations: &mut [u128], q: f64) -> f64 {
    durations.sort_unstable();
    quantile_from_sorted(durations, q) / 1_000_000.0
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

type TraceNormalizer =
    std::sync::Arc<dyn Fn(&ExportTraceServiceRequest) -> Vec<SpanRow> + Send + Sync>;
type LogNormalizer = std::sync::Arc<dyn Fn(&ExportLogsServiceRequest) -> Vec<LogRow> + Send + Sync>;

pub struct MemoryStore {
    inner: Mutex<Inner>,
    normalize_traces: Option<TraceNormalizer>,
    normalize_logs: Option<LogNormalizer>,
    traces_gate: AsyncMutex<Option<oneshot::Receiver<()>>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            normalize_traces: None,
            normalize_logs: None,
            traces_gate: AsyncMutex::new(None),
        }
    }
}

#[derive(Default)]
struct Inner {
    spans: Vec<SpanRow>,
    logs: Vec<LogRow>,
    metric_points: Vec<MetricPointRow>,
    histograms: Vec<HistogramRow>,
    metric_exemplars: Vec<MetricExemplarRow>,
    error_events: Vec<ErrorEventRow>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_normalizers(mut self, traces: TraceNormalizer, logs: LogNormalizer) -> Self {
        self.normalize_traces = Some(traces);
        self.normalize_logs = Some(logs);
        self
    }

    pub fn push_spans(&self, spans: Vec<SpanRow>) {
        self.lock().spans.extend(spans);
    }

    pub fn push_logs(&self, logs: Vec<LogRow>) {
        self.lock().logs.extend(logs);
    }

    pub async fn set_traces_gate(&self, rx: oneshot::Receiver<()>) {
        *self.traces_gate.lock().await = Some(rx);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
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
    async fn ingest_traces(
        &self,
        request: &ExportTraceServiceRequest,
        _raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        let gate = {
            let mut g = self.traces_gate.lock().await;
            g.take()
        };
        if let Some(rx) = gate {
            let _ = rx.await;
        }
        if let Some(normalize) = &self.normalize_traces {
            self.lock().spans.extend(normalize(request));
        }
        Ok(())
    }

    async fn ingest_logs(
        &self,
        request: &ExportLogsServiceRequest,
        _raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        if let Some(normalize) = &self.normalize_logs {
            self.lock().logs.extend(normalize(request));
        }
        Ok(())
    }

    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
        _raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        let mut inner = self.lock();
        inner.metric_points.extend(points);
        inner.histograms.extend(histograms);
        inner.metric_exemplars.extend(exemplars);
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

    async fn traces_by_ids(
        &self,
        trace_ids: &[String],
    ) -> anyhow::Result<Vec<crate::adapter::TraceSummary>> {
        // O(n) dedup preserving request order (MAX_ROWS still caps fan-out).
        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        for trace_id in trace_ids.iter().filter(|trace_id| !trace_id.is_empty()) {
            if !seen.insert(trace_id.as_str()) {
                continue;
            }
            ids.push(trace_id.clone());
            if ids.len() >= MAX_ROWS {
                break;
            }
        }
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let inner = self.lock();
        let mut summaries = Vec::new();
        for trace_id in ids {
            let trace_spans: Vec<&SpanRow> = inner
                .spans
                .iter()
                .filter(|span| span.trace_id == trace_id)
                .collect();
            let Some(root) = trace_spans.iter().copied().min_by_key(|span| {
                (
                    !span.parent_span_id.as_deref().is_none_or(str::is_empty),
                    span.ts_nanos,
                )
            }) else {
                continue;
            };
            summaries.push(crate::adapter::TraceSummary {
                trace_id,
                root_name: root.name.clone(),
                service: root.service.clone(),
                start_nanos: root.ts_nanos,
                duration_ns: root.duration_ns,
                span_count: trace_spans.len() as u64,
                has_error: trace_spans
                    .iter()
                    .any(|span| span.status_code == "STATUS_CODE_ERROR"),
            });
        }
        Ok(summaries)
    }

    async fn spans_by_run(
        &self,
        run_id: &str,
        limit: usize,
        _range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<SpanRow>> {
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

    async fn spans_by_runs(
        &self,
        run_ids: &[String],
        limit_per_run: usize,
    ) -> anyhow::Result<HashMap<String, Vec<SpanRow>>> {
        let wanted: HashSet<&str> = run_ids.iter().map(String::as_str).collect();
        let mut out: HashMap<String, Vec<SpanRow>> =
            run_ids.iter().map(|id| (id.clone(), Vec::new())).collect();
        if wanted.is_empty() || limit_per_run == 0 {
            return Ok(out);
        }
        for span in self.lock().spans.iter() {
            let Some(run_id) = span.run_id.as_deref() else {
                continue;
            };
            if !wanted.contains(run_id) {
                continue;
            }
            out.entry(run_id.to_string())
                .or_default()
                .push(span.clone());
        }
        for spans in out.values_mut() {
            spans.sort_by_key(|s| std::cmp::Reverse(s.ts_nanos));
            spans.truncate(limit_per_run);
            spans.sort_by_key(|s| s.ts_nanos);
        }
        Ok(out)
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

    async fn metric_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>> {
        let inner = self.lock();
        let mut names: Vec<String> = inner
            .metric_points
            .iter()
            .filter(|p| range.contains(&p.ts_nanos))
            .map(|p| p.name.clone())
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|h| range.contains(&h.ts_nanos))
                    .map(|h| h.name.clone()),
            )
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    async fn metric_labels(&self, name: &str) -> anyhow::Result<Vec<String>> {
        let inner = self.lock();
        let mut labels = BTreeSet::new();
        for attributes in inner
            .metric_points
            .iter()
            .filter(|point| point.name == name)
            .map(|point| &point.attributes)
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|row| row.name == name)
                    .map(|row| &row.attributes),
            )
        {
            if let Some(object) = attributes.as_object() {
                for (key, value) in object {
                    if metric_group_label_allowed(key)
                        && matches!(
                            value,
                            serde_json::Value::String(_)
                                | serde_json::Value::Bool(_)
                                | serde_json::Value::Number(_)
                        )
                    {
                        labels.insert(key.clone());
                    }
                }
            }
        }
        Ok(labels.into_iter().collect())
    }

    async fn metric_label_values(
        &self,
        name: &str,
        label: &str,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<String>> {
        anyhow::ensure!(
            metric_group_label_allowed(label),
            "high-cardinality identifier - filter, don't group"
        );
        let labels = self.metric_labels(name).await?;
        anyhow::ensure!(
            labels.iter().any(|known| known == label),
            "unknown metric label"
        );
        let inner = self.lock();
        let mut values = BTreeSet::new();
        for attributes in inner
            .metric_points
            .iter()
            .filter(|point| point.name == name && range.contains(&point.ts_nanos))
            .map(|point| &point.attributes)
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|row| row.name == name && range.contains(&row.ts_nanos))
                    .map(|row| &row.attributes),
            )
        {
            if let Some(value) = scalar_attribute_value(attributes, label) {
                values.insert(value);
                if values.len() >= 100 {
                    break;
                }
            }
        }
        Ok(values.into_iter().collect())
    }

    async fn service_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>> {
        let inner = self.lock();
        let mut names: Vec<String> = inner
            .metric_points
            .iter()
            .filter(|p| range.contains(&p.ts_nanos))
            .map(|p| p.service.clone())
            .chain(
                inner
                    .spans
                    .iter()
                    .filter(|s| range.contains(&s.ts_nanos))
                    .map(|s| s.service.clone()),
            )
            .chain(
                inner
                    .logs
                    .iter()
                    .filter(|l| range.contains(&l.ts_nanos))
                    .map(|l| l.service.clone()),
            )
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

    async fn release_windows(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ReleaseWindow>> {
        let inner = self.lock();
        let mut by_version: BTreeMap<String, ReleaseWindow> = BTreeMap::new();
        for span in inner
            .spans
            .iter()
            .filter(|s| s.service == service && range.contains(&s.ts_nanos))
        {
            let Some(version) = span
                .resource
                .get(semconv::SERVICE_VERSION)
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let window = by_version
                .entry(version.to_string())
                .or_insert_with(|| ReleaseWindow {
                    version: version.to_string(),
                    first_seen_nanos: span.ts_nanos,
                    last_seen_nanos: span.ts_nanos,
                    span_count: 0,
                });
            window.first_seen_nanos = window.first_seen_nanos.min(span.ts_nanos);
            window.last_seen_nanos = window.last_seen_nanos.max(span.ts_nanos);
            window.span_count += 1;
        }
        let mut windows: Vec<_> = by_version.into_values().collect();
        windows.sort_by_key(|window| (window.first_seen_nanos, window.version.clone()));
        Ok(windows)
    }

    async fn service_catalog(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceCatalogRow>> {
        #[derive(Default)]
        struct CatalogAgg {
            latest: Option<SpanRow>,
            instances: BTreeSet<String>,
        }

        let inner = self.lock();
        let mut by_service: BTreeMap<String, CatalogAgg> = BTreeMap::new();
        for span in inner.spans.iter().filter(|s| range.contains(&s.ts_nanos)) {
            let entry = by_service.entry(span.service.clone()).or_default();
            if entry
                .latest
                .as_ref()
                .is_none_or(|latest| span.ts_nanos >= latest.ts_nanos)
            {
                entry.latest = Some(span.clone());
            }
            if let Some(instance) = resource_string(&span.resource, "service.instance.id") {
                entry.instances.insert(instance);
            }
        }

        let mut rows = Vec::new();
        for (name, agg) in by_service {
            let Some(latest) = agg.latest else { continue };
            rows.push(ServiceCatalogRow {
                name,
                service_version: resource_string(&latest.resource, semconv::SERVICE_VERSION),
                service_namespace: resource_string(&latest.resource, semconv::SERVICE_NAMESPACE),
                deployment_environment: resource_string(
                    &latest.resource,
                    semconv::DEPLOYMENT_ENVIRONMENT_NAME,
                )
                .or_else(|| resource_string(&latest.resource, semconv::DEPLOYMENT_ENVIRONMENT)),
                telemetry_sdk_language: resource_string(&latest.resource, "telemetry.sdk.language"),
                telemetry_sdk_name: resource_string(&latest.resource, "telemetry.sdk.name"),
                telemetry_sdk_version: resource_string(&latest.resource, "telemetry.sdk.version"),
                last_seen_nanos: latest.ts_nanos,
                instance_count: agg.instances.len() as u64,
            });
        }
        rows.sort_by_key(|row| row.name.clone());
        rows.truncate(MAX_ROWS);
        Ok(rows)
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
        // Latest sample per window (plan 085) — align with greptime MAX merge.
        let step = step_nanos.max(1);
        let mut latest: std::collections::BTreeMap<u128, HistogramRow> = Default::default();
        for row in self.lock().histograms.iter().filter(|h| {
            h.name == name
                && service.is_none_or(|svc| h.service == svc)
                && range.contains(&h.ts_nanos)
        }) {
            let window = (row.ts_nanos / step) * step;
            match latest.get(&window) {
                Some(cur) if cur.ts_nanos >= row.ts_nanos => {}
                _ => {
                    latest.insert(window, row.clone());
                }
            }
        }
        Ok(latest
            .into_iter()
            .map(|(ts_nanos, row)| SeriesPoint {
                ts_nanos,
                value: quantile_from_histograms(&[row], q),
            })
            .collect())
    }

    async fn metric_exemplars(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<MetricExemplarRow>> {
        let mut rows: Vec<MetricExemplarRow> = self
            .lock()
            .metric_exemplars
            .iter()
            .filter(|row| {
                row.name == name
                    && service.is_none_or(|svc| row.service == svc)
                    && range.contains(&row.ts_nanos)
            })
            .cloned()
            .collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.ts_nanos));
        rows.truncate(limit.min(MAX_ROWS));
        Ok(rows)
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
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<crate::adapter::ObservedRun>> {
        let inner = self.lock();
        let mut runs: std::collections::HashMap<String, crate::adapter::ObservedRun> =
            std::collections::HashMap::new();
        let mut absorb = |run_id: &Option<String>, ts: u128, service: &str, is_span: bool| {
            if !range.contains(&ts) {
                return;
            }
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
        // Windowed participation + aggregates (plan 075; aligned both adapters).
        let in_window = |ts: u128| {
            query.from_nanos.is_none_or(|from| ts >= from)
                && query.to_nanos.is_none_or(|to| ts <= to)
        };
        let participating: Option<std::collections::HashSet<&str>> =
            query.service.as_deref().map(|svc| {
                inner
                    .spans
                    .iter()
                    .filter(|s| s.service == svc && in_window(s.ts_nanos))
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
                    if span.trace_id == root.trace_id && in_window(span.ts_nanos) {
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

    async fn attribute_compare(
        &self,
        selected: RangeInclusive<u128>,
        baseline: RangeInclusive<u128>,
        service: Option<&str>,
        error_only: bool,
        keys: &[String],
        top_n: usize,
    ) -> anyhow::Result<Vec<AttributeCompareRow>> {
        let limit = top_n.min(ATTRIBUTE_COMPARE_TOP_N_CAP);
        if limit == 0 {
            return Ok(Vec::new());
        }

        let spans = self.lock().spans.clone();
        let candidate_keys: Vec<String> = if keys.is_empty() {
            let mut discovered = BTreeSet::new();
            for span in spans.iter().filter(|span| {
                (span_matches_compare(span, &selected, service, error_only)
                    || span_matches_compare(span, &baseline, service, error_only))
                    && span.attributes.is_object()
            }) {
                if let Some(attributes) = span.attributes.as_object() {
                    for key in attributes.keys() {
                        if attribute_compare_key_allowed(key)
                            && scalar_attribute_value(&span.attributes, key).is_some()
                        {
                            discovered.insert(key.clone());
                        }
                    }
                }
            }
            discovered
                .into_iter()
                .take(ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT)
                .collect()
        } else {
            keys.iter()
                .map(|key| key.trim())
                .filter(|key| attribute_compare_key_allowed(key))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .take(ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT)
                .map(str::to_string)
                .collect()
        };

        let mut rows = Vec::new();
        for key in candidate_keys {
            let mut selected_counts: BTreeMap<String, u64> = BTreeMap::new();
            let mut baseline_counts: BTreeMap<String, u64> = BTreeMap::new();
            let mut selected_total = 0;
            let mut baseline_total = 0;

            for span in &spans {
                if span_matches_compare(span, &selected, service, error_only)
                    && let Some(value) = scalar_attribute_value(&span.attributes, &key)
                {
                    selected_total += 1;
                    *selected_counts.entry(value).or_default() += 1;
                }
                if span_matches_compare(span, &baseline, service, error_only)
                    && let Some(value) = scalar_attribute_value(&span.attributes, &key)
                {
                    baseline_total += 1;
                    *baseline_counts.entry(value).or_default() += 1;
                }
            }

            for (value, selected_count) in selected_counts {
                let baseline_count = baseline_counts.get(&value).copied().unwrap_or(0);
                let score = attribute_compare_score(
                    selected_count,
                    selected_total,
                    baseline_count,
                    baseline_total,
                );
                if score > 0.0 {
                    rows.push(AttributeCompareRow {
                        key: key.clone(),
                        value,
                        selected_count,
                        selected_total,
                        baseline_count,
                        baseline_total,
                        score,
                    });
                }
            }
        }

        rows.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| b.selected_count.cmp(&a.selected_count))
                .then_with(|| a.key.cmp(&b.key))
                .then_with(|| a.value.cmp(&b.value))
        });
        rows.truncate(limit);
        Ok(rows)
    }

    async fn span_field_keys(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<FieldKey>> {
        let spans = self.lock().spans.clone();
        let window: Vec<SpanRow> = spans
            .into_iter()
            .filter(|span| range.contains(&span.ts_nanos))
            .collect();
        let row_count = window.len() as u64;
        let mut counts: BTreeMap<String, (FieldSource, u64)> = BTreeMap::new();

        for span in &window {
            if !span.service.trim().is_empty() {
                counts
                    .entry(format!("resource.{}", semconv::SERVICE_NAME))
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert((FieldSource::Resource, 1));
            }
            if let Some(attributes) = span.attributes.as_object() {
                for key in attributes.keys() {
                    if !span_field_key_allowed(key) {
                        continue;
                    }
                    if field_scalar_value(&span.attributes, key).is_some() {
                        counts
                            .entry(key.clone())
                            .and_modify(|(_, count)| *count += 1)
                            .or_insert((FieldSource::Span, 1));
                    }
                }
            }
            if let Some(resource) = span.resource.as_object() {
                for key in resource.keys() {
                    if key == semconv::SERVICE_NAME {
                        continue;
                    }
                    let exposed = format!("resource.{key}");
                    if !span_field_key_allowed(&exposed) {
                        continue;
                    }
                    if field_scalar_value(&span.resource, key).is_some() {
                        counts
                            .entry(exposed)
                            .and_modify(|(_, count)| *count += 1)
                            .or_insert((FieldSource::Resource, 1));
                    }
                }
            }
        }

        Ok(counts
            .into_iter()
            .take(FIELD_KEYS_CAP)
            .map(|(key, (source, non_null_count))| FieldKey {
                namespace: field_key_namespace(&key),
                coverage: if row_count == 0 {
                    0.0
                } else {
                    non_null_count as f64 / row_count as f64
                },
                is_identifier: field_key_identifier_like(&key),
                key,
                source,
                row_count,
                non_null_count,
            })
            .collect())
    }

    async fn span_field_stats(
        &self,
        key: &str,
        range: RangeInclusive<u128>,
        service: Option<&str>,
    ) -> anyhow::Result<FieldStats> {
        anyhow::ensure!(span_field_key_allowed(key), "invalid field key");
        let discovered = self.span_field_keys(range.clone()).await?;
        let Some(discovered_key) = discovered.iter().find(|field| field.key == key) else {
            anyhow::bail!("unknown span field key");
        };
        let (source, raw_key) = match key.strip_prefix("resource.") {
            Some(raw) => (FieldSource::Resource, raw),
            None => (FieldSource::Span, key),
        };

        let spans = self.lock().spans.clone();
        let window: Vec<SpanRow> = spans
            .into_iter()
            .filter(|span| {
                range.contains(&span.ts_nanos) && service.is_none_or(|svc| span.service == svc)
            })
            .collect();
        let row_count = window.len() as u64;
        let mut values = Vec::new();
        for span in &window {
            let value = match source {
                FieldSource::Span => field_scalar_value(&span.attributes, raw_key),
                FieldSource::Resource if raw_key == semconv::SERVICE_NAME => {
                    field_value_display(&span.service)
                }
                FieldSource::Resource => field_scalar_value(&span.resource, raw_key),
            };
            if let Some(value) = value {
                values.push(value);
            }
        }

        let non_null_count = values.len() as u64;
        let capped = values.len() > MAX_ROWS;
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        let mut distinct = BTreeSet::new();
        for value in values.into_iter().take(MAX_ROWS) {
            distinct.insert(value.clone());
            *counts.entry(value).or_default() += 1;
        }

        let mut top_values: Vec<FieldValueCount> = counts
            .into_iter()
            .map(|(value, count)| FieldValueCount { value, count })
            .collect();
        top_values.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
        top_values.truncate(FIELD_TOP_VALUES_CAP);
        let sample_count = non_null_count.min(MAX_ROWS as u64);
        let is_identifier = discovered_key.is_identifier
            || (sample_count >= 20 && distinct.len() as u64 >= sample_count.saturating_sub(1));

        Ok(FieldStats {
            key: key.to_string(),
            namespace: field_key_namespace(key),
            source,
            row_count,
            non_null_count,
            distinct_count: distinct.len() as u64,
            coverage: if row_count == 0 {
                0.0
            } else {
                non_null_count as f64 / row_count as f64
            },
            capped,
            is_identifier,
            top_values,
        })
    }

    async fn service_map(
        &self,
        range: RangeInclusive<u128>,
        max_traces: usize,
    ) -> anyhow::Result<Vec<ServiceEdge>> {
        let trace_limit = max_traces.min(SERVICE_MAP_TRACE_CAP);
        if trace_limit == 0 {
            return Ok(Vec::new());
        }

        let spans = self.lock().spans.clone();
        let mut trace_last_seen: BTreeMap<String, u128> = BTreeMap::new();
        for span in spans.iter().filter(|span| range.contains(&span.ts_nanos)) {
            trace_last_seen
                .entry(span.trace_id.clone())
                .and_modify(|last| *last = (*last).max(span.ts_nanos))
                .or_insert(span.ts_nanos);
        }
        let mut traces: Vec<_> = trace_last_seen.into_iter().collect();
        traces.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let trace_ids: BTreeSet<String> = traces
            .into_iter()
            .take(trace_limit)
            .map(|(trace_id, _)| trace_id)
            .collect();

        let by_trace_span: HashMap<(String, String), &SpanRow> = spans
            .iter()
            .filter(|span| trace_ids.contains(&span.trace_id))
            .map(|span| ((span.trace_id.clone(), span.span_id.clone()), span))
            .collect();
        let mut grouped: BTreeMap<(String, String), (u64, u64, Vec<u128>)> = BTreeMap::new();
        for span in spans.iter().filter(|span| {
            trace_ids.contains(&span.trace_id)
                && range.contains(&span.ts_nanos)
                && span.kind == "SPAN_KIND_SERVER"
        }) {
            let Some(parent_id) = span.parent_span_id.as_deref().filter(|id| !id.is_empty()) else {
                continue;
            };
            let Some(parent) = by_trace_span.get(&(span.trace_id.clone(), parent_id.to_string()))
            else {
                continue;
            };
            if parent.service == span.service {
                continue;
            }
            let entry = grouped
                .entry((parent.service.clone(), span.service.clone()))
                .or_default();
            entry.0 += 1;
            if span.status_code == "STATUS_CODE_ERROR" {
                entry.1 += 1;
            }
            entry.2.push(span.duration_ns);
        }

        Ok(grouped
            .into_iter()
            .map(
                |((source, target), (call_count, error_count, mut durations))| {
                    let p50_ms = duration_quantile_ms(&mut durations, 0.5);
                    let p95_ms = duration_quantile_ms(&mut durations, 0.95);
                    ServiceEdge {
                        source,
                        target,
                        call_count,
                        error_count,
                        p50_ms,
                        p95_ms,
                    }
                },
            )
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
        anyhow::ensure!(
            metric_group_label_allowed(group_by),
            "high-cardinality identifier - filter, don't group"
        );
        let labels = self.metric_labels(name).await?;
        anyhow::ensure!(
            labels.iter().any(|label| label == group_by),
            "unknown metric label"
        );
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

    async fn runtime_snapshot(
        &self,
        service: Option<&str>,
        run_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<RuntimeMetricSeries>> {
        let mut rows = Vec::new();
        for metric in self.metric_names(range.clone()).await? {
            let Some(family) = runtime_metric_family(&metric) else {
                continue;
            };
            let points = self
                .metric_series(
                    &metric,
                    service,
                    run_id,
                    range.clone(),
                    step_nanos,
                    MetricAgg::Avg,
                )
                .await?;
            if points.is_empty() {
                continue;
            }
            rows.push(RuntimeMetricSeries {
                family: family.to_string(),
                metric: metric.clone(),
                unit: runtime_metric_unit(&metric),
                points,
            });
        }
        rows.sort_by(|a, b| a.family.cmp(&b.family).then(a.metric.cmp(&b.metric)));
        Ok(rows)
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
            event_name: String::new(),
            observed_ts_nanos: 0,
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

    fn span_with_attrs(trace: &str, span_id: &str, ts: u128, attrs: serde_json::Value) -> SpanRow {
        let mut row = span(trace, span_id, None, "checkout", ts);
        row.attributes = attrs;
        row
    }

    fn span_with_release(trace: &str, span_id: &str, ts: u128, version: &str) -> SpanRow {
        let mut row = span(trace, span_id, None, "checkout", ts);
        row.resource = serde_json::json!({ "service.version": version });
        row
    }

    fn span_with_resource(
        trace: &str,
        span_id: &str,
        service: &str,
        ts: u128,
        resource: serde_json::Value,
    ) -> SpanRow {
        let mut row = span(trace, span_id, None, service, ts);
        row.resource = resource;
        row
    }

    #[tokio::test]
    async fn span_field_keys_and_stats_cover_span_and_resource_attrs() {
        let store = MemoryStore::new();
        let mut s1 = span_with_attrs(
            "trace-1",
            "root",
            10,
            serde_json::json!({
                "http.request.method": "GET",
                "request.id": "req-1"
            }),
        );
        s1.resource = serde_json::json!({ "service.name": "checkout" });
        let mut s2 = span_with_attrs(
            "trace-2",
            "root",
            20,
            serde_json::json!({
                "http.request.method": "GET",
                "request.id": "req-2"
            }),
        );
        s2.resource = serde_json::json!({ "service.name": "checkout" });
        let mut s3 = span_with_attrs(
            "trace-3",
            "root",
            30,
            serde_json::json!({
                "http.request.method": "POST",
                "request.id": "req-3"
            }),
        );
        s3.resource = serde_json::json!({ "service.name": "checkout" });
        let mut s4 = span("trace-4", "root", None, "checkout", 40);
        s4.resource = serde_json::json!({ "service.name": "checkout" });
        store.push_spans(vec![s1, s2, s3, s4]);

        let keys = store.span_field_keys(0..=100).await.unwrap();
        let method_key = keys
            .iter()
            .find(|key| key.key == "http.request.method")
            .unwrap();
        assert_eq!(method_key.namespace, "http");
        assert_eq!(method_key.non_null_count, 3);
        assert!((method_key.coverage - 0.75).abs() < f64::EPSILON);
        assert!(
            keys.iter().any(
                |key| key.key == "resource.service.name" && key.source == FieldSource::Resource
            )
        );
        assert!(
            keys.iter()
                .any(|key| key.key == "request.id" && key.is_identifier)
        );

        let stats = store
            .span_field_stats("http.request.method", 0..=100, Some("checkout"))
            .await
            .unwrap();
        assert_eq!(stats.row_count, 4);
        assert_eq!(stats.non_null_count, 3);
        assert_eq!(stats.distinct_count, 2);
        assert_eq!(stats.top_values[0].value, "GET");
        assert_eq!(stats.top_values[0].count, 2);
    }

    #[tokio::test]
    async fn span_field_stats_rejects_disallowed_keys() {
        let store = MemoryStore::new();
        store.push_spans(vec![span_with_attrs(
            "trace-1",
            "root",
            10,
            serde_json::json!({ "authorization": "secret" }),
        )]);

        assert!(!span_field_key_allowed("authorization"));
        let err = store
            .span_field_stats("authorization", 0..=100, None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("invalid field key"));
    }

    #[tokio::test]
    async fn service_catalog_returns_identity_and_nulls() {
        let store = MemoryStore::new();
        store.push_spans(vec![
            span_with_resource(
                "checkout-old",
                "root",
                "checkout",
                10,
                serde_json::json!({
                    "service.version": "v1",
                    "service.namespace": "shop",
                    "deployment.environment.name": "staging",
                    "telemetry.sdk.language": "rust",
                    "telemetry.sdk.name": "opentelemetry",
                    "telemetry.sdk.version": "0.31.0",
                    "service.instance.id": "checkout-a"
                }),
            ),
            span_with_resource(
                "checkout-new",
                "root",
                "checkout",
                20,
                serde_json::json!({
                    "service.version": "v2",
                    "service.namespace": "shop",
                    "deployment.environment.name": "prod",
                    "telemetry.sdk.language": "rust",
                    "telemetry.sdk.name": "opentelemetry",
                    "telemetry.sdk.version": "0.32.1",
                    "service.instance.id": "checkout-b"
                }),
            ),
            span("bare", "root", None, "bare", 30),
        ]);

        let rows = store.service_catalog(0..=100).await.unwrap();

        let bare = rows.iter().find(|row| row.name == "bare").unwrap();
        assert_eq!(bare.service_version, None);
        assert_eq!(bare.telemetry_sdk_language, None);
        assert_eq!(bare.instance_count, 0);
        let checkout = rows.iter().find(|row| row.name == "checkout").unwrap();
        assert_eq!(checkout.service_version.as_deref(), Some("v2"));
        assert_eq!(checkout.service_namespace.as_deref(), Some("shop"));
        assert_eq!(checkout.deployment_environment.as_deref(), Some("prod"));
        assert_eq!(checkout.telemetry_sdk_language.as_deref(), Some("rust"));
        assert_eq!(
            checkout.telemetry_sdk_name.as_deref(),
            Some("opentelemetry")
        );
        assert_eq!(checkout.telemetry_sdk_version.as_deref(), Some("0.32.1"));
        assert_eq!(checkout.last_seen_nanos, 20);
        assert_eq!(checkout.instance_count, 2);
    }

    #[tokio::test]
    async fn release_windows_group_versions_by_service_and_range() {
        let store = MemoryStore::new();
        store.push_spans(vec![
            span_with_release("t1", "a", 10, "v1"),
            span_with_release("t2", "a", 20, "v1"),
            span_with_release("t3", "a", 40, "v2"),
            span_with_release("t4", "a", 60, "v2"),
            span("other", "a", None, "catalog", 30),
            span_with_release("too-late", "a", 90, "v3"),
        ]);

        let windows = store.release_windows("checkout", 0..=80).await.unwrap();

        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].version, "v1");
        assert_eq!(windows[0].first_seen_nanos, 10);
        assert_eq!(windows[0].last_seen_nanos, 20);
        assert_eq!(windows[0].span_count, 2);
        assert_eq!(windows[1].version, "v2");
        assert_eq!(windows[1].first_seen_nanos, 40);
        assert_eq!(windows[1].last_seen_nanos, 60);
        assert_eq!(windows[1].span_count, 2);
    }

    #[tokio::test]
    async fn attribute_compare_ranks_overrepresented_value_first() {
        let store = MemoryStore::new();
        let mut spans = Vec::new();
        for index in 0..20 {
            let version = if index == 0 { "2.0.0" } else { "1.0.0" };
            spans.push(span_with_attrs(
                &format!("baseline-{index}"),
                "root",
                index,
                serde_json::json!({
                    "service.version": version,
                    "http.route": "/checkout"
                }),
            ));
        }
        for index in 0..10 {
            let version = if index < 9 { "2.0.0" } else { "1.0.0" };
            spans.push(span_with_attrs(
                &format!("selected-{index}"),
                "root",
                100 + index,
                serde_json::json!({
                    "service.version": version,
                    "http.route": "/checkout"
                }),
            ));
        }
        store.push_spans(spans);

        let rows = store
            .attribute_compare(100..=200, 0..=99, Some("checkout"), false, &[], 10)
            .await
            .unwrap();

        let first = rows.first().expect("overrepresented value");
        assert_eq!(first.key, "service.version");
        assert_eq!(first.value, "2.0.0");
        assert_eq!(first.selected_count, 9);
        assert_eq!(first.selected_total, 10);
        assert_eq!(first.baseline_count, 1);
        assert_eq!(first.baseline_total, 20);
        assert!(first.score > 0.8, "{first:?}");
    }

    #[tokio::test]
    async fn metric_exemplars_filters_by_metric_service_range_and_limit() {
        let store = MemoryStore::new();
        store
            .ingest_metrics(
                Vec::new(),
                Vec::new(),
                vec![
                    MetricExemplarRow {
                        ts_nanos: 20,
                        service: "checkout".into(),
                        name: "http.server.request.duration".into(),
                        value: 120.0,
                        trace_id: "trace-a".into(),
                        span_id: "span-a".into(),
                        run_id: Some("run-a".into()),
                        attributes: serde_json::json!({"route": "/checkout"}),
                    },
                    MetricExemplarRow {
                        ts_nanos: 10,
                        service: "checkout".into(),
                        name: "http.server.request.duration".into(),
                        value: 90.0,
                        trace_id: "trace-b".into(),
                        span_id: "span-b".into(),
                        run_id: None,
                        attributes: serde_json::Value::Null,
                    },
                    MetricExemplarRow {
                        ts_nanos: 30,
                        service: "catalog".into(),
                        name: "http.server.request.duration".into(),
                        value: 80.0,
                        trace_id: "trace-c".into(),
                        span_id: "span-c".into(),
                        run_id: None,
                        attributes: serde_json::Value::Null,
                    },
                ],
                bytes::Bytes::new(),
            )
            .await
            .unwrap();

        let rows = store
            .metric_exemplars("http.server.request.duration", Some("checkout"), 0..=25, 1)
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].trace_id, "trace-a");
        assert_eq!(rows[0].run_id.as_deref(), Some("run-a"));
        assert_eq!(rows[0].attributes["route"], "/checkout");
    }

    #[tokio::test]
    async fn metric_labels_values_and_runtime_snapshot_derive_from_points() {
        let store = MemoryStore::new();
        store
            .ingest_metrics(
                vec![
                    MetricPointRow {
                        ts_nanos: 1_000_000_000,
                        service: "checkout".into(),
                        name: "process.cpu.utilization".into(),
                        value: 0.42,
                        is_monotonic: false,
                        run_id: Some("run-a".into()),
                        attributes: serde_json::json!({
                            "runtime.name": "tokio",
                            "payment.method": "card",
                            "trace_id": "trace-a"
                        }),
                    },
                    MetricPointRow {
                        ts_nanos: 2_000_000_000,
                        service: "checkout".into(),
                        name: "jvm.gc.time".into(),
                        value: 12.0,
                        is_monotonic: false,
                        run_id: None,
                        attributes: serde_json::json!({
                            "payment.method": "wire"
                        }),
                    },
                ],
                Vec::new(),
                Vec::new(),
                bytes::Bytes::new(),
            )
            .await
            .unwrap();

        let labels = store
            .metric_labels("process.cpu.utilization")
            .await
            .unwrap();
        assert!(labels.contains(&"runtime.name".to_string()));
        assert!(labels.contains(&"payment.method".to_string()));
        assert!(!labels.contains(&"trace_id".to_string()));

        let values = store
            .metric_label_values(
                "process.cpu.utilization",
                "payment.method",
                0..=3_000_000_000,
            )
            .await
            .unwrap();
        assert_eq!(values, vec!["card".to_string()]);

        let mut capped_points = Vec::new();
        for index in 0..110 {
            capped_points.push(MetricPointRow {
                ts_nanos: 4_000_000_000 + index,
                service: "checkout".into(),
                name: "process.cpu.utilization".into(),
                value: index as f64,
                is_monotonic: false,
                run_id: None,
                attributes: serde_json::json!({
                    "runtime.name": format!("runtime-{index:03}")
                }),
            });
        }
        store
            .ingest_metrics(capped_points, Vec::new(), Vec::new(), bytes::Bytes::new())
            .await
            .unwrap();

        let capped = store
            .metric_label_values("process.cpu.utilization", "runtime.name", 0..=5_000_000_000)
            .await
            .unwrap();
        assert_eq!(capped.len(), 100);

        let runtime = store
            .runtime_snapshot(Some("checkout"), None, 0..=3_000_000_000, 1_000_000_000)
            .await
            .unwrap();
        assert_eq!(runtime.len(), 2);
        assert!(runtime.iter().any(|row| row.family == "process"));
        assert!(runtime.iter().any(|row| row.family == "jvm"));

        let run_runtime = store
            .runtime_snapshot(None, Some("run-a"), 0..=3_000_000_000, 1_000_000_000)
            .await
            .unwrap();
        assert_eq!(run_runtime.len(), 1);
        assert_eq!(run_runtime[0].metric, "process.cpu.utilization");

        let denied = store
            .metric_series_grouped(
                "process.cpu.utilization",
                Some("checkout"),
                "trace_id",
                0..=3_000_000_000,
                1_000_000_000,
                MetricAgg::Avg,
            )
            .await
            .unwrap_err()
            .to_string();
        assert!(denied.contains("high-cardinality identifier"));
    }

    #[tokio::test]
    async fn attribute_compare_denies_identifier_keys() {
        let store = MemoryStore::new();
        store.push_spans(vec![
            span_with_attrs(
                "baseline",
                "root",
                1,
                serde_json::json!({
                    "service.version": "1.0.0",
                    "trace_id": "trace-baseline",
                    "run_id": "run-baseline",
                    "session.id": "session-baseline",
                    "user.id": "user-baseline"
                }),
            ),
            span_with_attrs(
                "selected",
                "root",
                100,
                serde_json::json!({
                    "service.version": "2.0.0",
                    "trace_id": "trace-selected",
                    "run_id": "run-selected",
                    "session.id": "session-selected",
                    "user.id": "user-selected"
                }),
            ),
        ]);
        let keys = vec![
            "trace_id".to_string(),
            "run_id".to_string(),
            "session.id".to_string(),
            "user.id".to_string(),
            "service.version".to_string(),
        ];

        let rows = store
            .attribute_compare(100..=200, 0..=99, None, false, &keys, 10)
            .await
            .unwrap();

        assert!(rows.iter().all(|row| {
            !matches!(
                row.key.as_str(),
                "trace_id" | "run_id" | "session.id" | "user.id"
            )
        }));
        assert!(rows.iter().any(|row| row.key == "service.version"));
    }

    #[tokio::test]
    async fn attribute_compare_is_deterministic() {
        let store = MemoryStore::new();
        store.push_spans(vec![
            span_with_attrs(
                "baseline-a",
                "root",
                1,
                serde_json::json!({"service.version": "1.0.0"}),
            ),
            span_with_attrs(
                "selected-a",
                "root",
                100,
                serde_json::json!({"service.version": "2.0.0"}),
            ),
        ]);

        let first = store
            .attribute_compare(100..=200, 0..=99, None, false, &[], 10)
            .await
            .unwrap();
        let second = store
            .attribute_compare(100..=200, 0..=99, None, false, &[], 10)
            .await
            .unwrap();

        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn service_map_derives_trace_path_edges() {
        let store = MemoryStore::new();
        let mut a_client = span("trace-ab", "a-client", None, "A", 100);
        a_client.kind = "SPAN_KIND_CLIENT".into();
        let mut b_server = span("trace-ab", "b-server", Some("a-client"), "B", 101);
        b_server.kind = "SPAN_KIND_SERVER".into();
        b_server.status_code = "STATUS_CODE_ERROR".into();
        b_server.duration_ns = 20_000_000;
        let mut b_client = span("trace-bc", "b-client", None, "B", 110);
        b_client.kind = "SPAN_KIND_CLIENT".into();
        let mut c_server = span("trace-bc", "c-server", Some("b-client"), "C", 111);
        c_server.kind = "SPAN_KIND_SERVER".into();
        c_server.duration_ns = 30_000_000;
        let mut outside_client = span("trace-out", "a-client", None, "A", 1_000);
        outside_client.kind = "SPAN_KIND_CLIENT".into();
        let mut outside_server = span("trace-out", "d-server", Some("a-client"), "D", 1_001);
        outside_server.kind = "SPAN_KIND_SERVER".into();
        store.push_spans(vec![
            a_client,
            b_server,
            b_client,
            c_server,
            outside_client,
            outside_server,
        ]);

        let edges = store.service_map(0..=200, 100).await.unwrap();

        let edge_ab = edges
            .iter()
            .find(|edge| edge.source == "A" && edge.target == "B")
            .expect("A -> B edge");
        assert_eq!(edge_ab.call_count, 1);
        assert_eq!(edge_ab.error_count, 1);
        assert_eq!(edge_ab.p50_ms, 20.0);
        assert!(
            edges
                .iter()
                .any(|edge| edge.source == "B" && edge.target == "C")
        );
        assert!(!edges.iter().any(|edge| edge.target == "D"));
    }

    #[tokio::test]
    async fn service_map_is_deterministic_and_trace_bounded() {
        let store = MemoryStore::new();
        let mut a_client = span("trace-ab", "a-client", None, "A", 100);
        a_client.kind = "SPAN_KIND_CLIENT".into();
        let mut b_server = span("trace-ab", "b-server", Some("a-client"), "B", 101);
        b_server.kind = "SPAN_KIND_SERVER".into();
        let mut b_client = span("trace-bc", "b-client", None, "B", 110);
        b_client.kind = "SPAN_KIND_CLIENT".into();
        let mut c_server = span("trace-bc", "c-server", Some("b-client"), "C", 111);
        c_server.kind = "SPAN_KIND_SERVER".into();
        store.push_spans(vec![a_client, b_server, b_client, c_server]);

        let first = store.service_map(0..=200, 100).await.unwrap();
        let second = store.service_map(0..=200, 100).await.unwrap();
        let bounded = store.service_map(0..=200, 1).await.unwrap();

        assert_eq!(first, second);
        assert_eq!(bounded.len(), 1);
        assert_eq!(bounded[0].source, "B");
        assert_eq!(bounded[0].target, "C");
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
        store.push_spans(spans);
        store.push_logs(logs);

        let spans = store
            .spans_by_run("run-1", 200, 0..=u128::MAX)
            .await
            .unwrap();
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
        store.push_logs(vec![
            log(None, 5, 5),
            log(None, 9, 9),
            log(None, 13, 13),
            log(None, 17, 17),
        ]);

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
        store.push_spans(vec![
            span("t1", "a", None, "checkout", 10),
            span("t1", "b", Some("a"), "catalog", 20),
        ]);

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
        store.push_spans(vec![
            span("t2", "y", Some("missing-parent"), "catalog", 30),
            span("t2", "x", Some("missing-parent"), "catalog", 15),
        ]);

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
    async fn traces_by_ids_preserves_requested_order_and_summarizes_targets() {
        let store = MemoryStore::new();
        let mut target_b = span("target-b", "root-b", None, "worker", 20);
        target_b.name = "consume-b".into();
        target_b.status_code = "STATUS_CODE_ERROR".into();
        let mut target_a = span("target-a", "root-a", None, "api", 10);
        target_a.name = "consume-a".into();
        store.push_spans(vec![
            target_a,
            span("target-a", "child-a", Some("root-a"), "api", 12),
            target_b,
        ]);

        let summaries = store
            .traces_by_ids(&[
                "target-b".to_string(),
                "missing".to_string(),
                "target-a".to_string(),
                "target-b".to_string(),
            ])
            .await
            .unwrap();

        assert_eq!(
            summaries
                .iter()
                .map(|summary| summary.trace_id.as_str())
                .collect::<Vec<_>>(),
            vec!["target-b", "target-a"]
        );
        assert_eq!(summaries[0].service, "worker");
        assert_eq!(summaries[0].root_name, "consume-b");
        assert!(summaries[0].has_error);
        assert_eq!(summaries[1].span_count, 2);
    }

    #[tokio::test]
    async fn trace_search_sorts_offsets_and_filters_duration_band() {
        let store = MemoryStore::new();
        store.push_spans(vec![
            span_with_duration("fast", "a", None, "api", 10, 10),
            span_with_duration("mid", "b", None, "api", 20, 20),
            span_with_duration("slow", "c", None, "api", 30, 30),
            span_with_duration("wide", "d", None, "api", 40, 25),
            span_with_duration("wide", "e", Some("d"), "api", 45, 5),
        ]);

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
        store.push_spans(vec![ok, err]);
        store.push_logs(vec![LogRow {
            ts_nanos: 1_250_000_000,
            event_name: "checkout.failed".into(),
            observed_ts_nanos: 1_300_000_000,
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
        }]);
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

        let logs = store.logs_by_trace("t1").await.unwrap();
        assert_eq!(logs[0].event_name, "checkout.failed");
        assert_eq!(logs[0].observed_ts_nanos, 1_300_000_000);

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
        store.push_spans(vec![fast, slow, other]);

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

    #[tokio::test]
    async fn conformance_scenarios_pass_on_memory() {
        let store = MemoryStore::new();
        crate::conformance::trace_search_scenario(&store)
            .await
            .expect("trace_search");
        crate::conformance::log_count_series_scenario(&store)
            .await
            .expect("log_count_series");
        crate::conformance::overview_totals_scenario(&store)
            .await
            .expect("overview_totals");
        crate::conformance::attribute_compare_scenario(&store)
            .await
            .expect("attribute_compare");
        crate::conformance::service_map_scenario(&store)
            .await
            .expect("service_map");
    }
}
