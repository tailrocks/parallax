//! In-memory `TelemetryStore` for tests and explicit test-support builds only.

mod math;

use self::math::{
    duration_quantile_ms, field_scalar_value, group_value, quantile_from_histograms,
    quantile_from_sorted, resource_string, scalar_attribute_value, span_matches_compare,
};
use crate::normalizers::{LogNormalizer, TraceNormalizer};
use parallax_model::*;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::semconv;
use parallax_storage::adapter::{
    self, ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT, ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow,
    FIELD_KEYS_CAP, FIELD_TOP_VALUES_CAP, FieldKey, FieldSource, FieldStats, FieldValueCount,
    MAX_ROWS, MetricStore, OverviewTotals, ReleaseWindow, RuntimeMetricSeries,
    SERVICE_MAP_TRACE_CAP, ServiceCatalogRow, ServiceEdge, ServiceSummary, SignalKind, SpanRed,
    TelemetryStore, attribute_compare_key_allowed, attribute_compare_score,
    field_key_identifier_like, field_key_namespace, field_value_display,
    metric_group_label_allowed, runtime_metric_family, runtime_metric_unit, span_field_key_allowed,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::RangeInclusive;
use std::sync::Mutex;
use tokio::sync::{Mutex as AsyncMutex, oneshot};

#[expect(missing_debug_implementations, reason = "opaque normalizers")]
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
impl adapter::IngestStore for MemoryStore {
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
            crate::warn_error(rx.await, "memory adapter test gate");
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
}

#[async_trait::async_trait]
impl adapter::TraceStore for MemoryStore {
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
    ) -> anyhow::Result<Vec<adapter::TraceSummary>> {
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
            summaries.push(adapter::TraceSummary {
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
}

#[async_trait::async_trait]
impl adapter::LogStore for MemoryStore {
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
}

#[async_trait::async_trait]
impl MetricStore for MemoryStore {
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
}

#[async_trait::async_trait]
impl TelemetryStore for MemoryStore {
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
            .collect::<BTreeSet<_>>()
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
            .collect::<BTreeSet<_>>()
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
        let mut buckets: BTreeMap<u128, u64> = Default::default();
        match kind {
            SignalKind::Spans => {
                for span in inner.spans.iter().filter(|s| {
                    range.contains(&s.ts_nanos) && service.is_none_or(|svc| s.service == svc)
                }) {
                    *buckets.entry((span.ts_nanos / step) * step).or_default() += 1;
                }
            }
            SignalKind::Traces => {
                let mut traces: BTreeMap<u128, BTreeSet<&str>> = Default::default();
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
        let mut by_service: BTreeMap<&str, Vec<&SpanRow>> = Default::default();
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
        let mut buckets: BTreeMap<u128, Vec<&SpanRow>> = Default::default();
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
        let mut buckets: BTreeMap<u128, Vec<f64>> = Default::default();
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
            series = adapter::rate_from_buckets(&series, step);
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
        let mut latest: BTreeMap<u128, HistogramRow> = Default::default();
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
    ) -> anyhow::Result<Vec<adapter::ObservedRun>> {
        let inner = self.lock();
        let mut runs: HashMap<String, adapter::ObservedRun> = HashMap::new();
        let mut absorb = |run_id: &Option<String>, ts: u128, service: &str, is_span: bool| {
            if !range.contains(&ts) {
                return;
            }
            let Some(run_id) = run_id.as_deref().filter(|r| !r.is_empty()) else {
                return;
            };
            let entry = runs
                .entry(run_id.to_owned())
                .or_insert_with(|| adapter::ObservedRun {
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
        query: &adapter::TraceQuery,
    ) -> anyhow::Result<adapter::TraceList> {
        let inner = self.lock();
        // `service` matches any trace the service participates in (a span of
        // that service anywhere), not only the root span.
        // Windowed participation + aggregates (plan 075; aligned both adapters).
        let in_window = |ts: u128| {
            query.from_nanos.is_none_or(|from| ts >= from)
                && query.to_nanos.is_none_or(|to| ts <= to)
        };
        let participating: Option<HashSet<&str>> = query.service.as_deref().map(|svc| {
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
        let mut rep: HashMap<&str, &SpanRow> = HashMap::new();
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
                adapter::TraceSummary {
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
            adapter::TraceSort::StartDesc => {
                traces.sort_by_key(|t| std::cmp::Reverse(t.start_nanos));
            }
            adapter::TraceSort::DurationDesc => {
                traces.sort_by_key(|t| std::cmp::Reverse(t.duration_ns));
            }
            adapter::TraceSort::DurationAsc => traces.sort_by_key(|t| t.duration_ns),
            adapter::TraceSort::SpanCountDesc => {
                traces.sort_by_key(|t| std::cmp::Reverse(t.span_count));
            }
        }
        let total = traces.len() as u64;
        let items = traces
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();
        Ok(adapter::TraceList { items, total })
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
        let mut buckets: BTreeMap<(String, u128), Vec<f64>> = Default::default();
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
        let mut groups: BTreeMap<String, Vec<SeriesPoint>> = Default::default();
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
                    adapter::rate_from_buckets(&series, step)
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
        let mut buckets: BTreeMap<u128, u64> = Default::default();
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
        let mut buckets: BTreeMap<u128, u64> = Default::default();
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

    async fn raw_sql(&self, _query: &str) -> anyhow::Result<adapter::SqlResult> {
        anyhow::bail!("raw SQL needs the GreptimeDB engine; the test store has no SQL surface")
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
        let mut buckets: BTreeMap<u128, u64> = Default::default();
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
mod tests;
