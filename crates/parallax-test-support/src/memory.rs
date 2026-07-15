//! In-memory `TelemetryStore` for tests and explicit test-support builds only.

mod error_analytics;
mod log_analytics;
mod log_count;
mod log_store;
mod math;
mod metric_analytics;
mod metric_store;
mod raw_sql;
mod run_store;
mod runtime_metrics;
mod seed;
mod service_analytics;
mod trace_analytics;
mod trace_search;
mod trace_store;

use self::math::{
    duration_quantile_ms, field_scalar_value, group_value, quantile_from_histograms,
    quantile_from_sorted, resource_string, scalar_attribute_value, span_matches_compare,
};
use crate::normalizers::{LogNormalizer, TraceNormalizer};
use parallax_model::*;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_semconv as semconv;
use parallax_storage::adapter::{
    self, ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT, ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow,
    FIELD_KEYS_CAP, FIELD_TOP_VALUES_CAP, FieldKey, FieldSource, FieldStats, FieldValueCount,
    MAX_ROWS, MetricAnalyticsStore, MetricStore, OverviewTotals, ReleaseWindow,
    RuntimeMetricSeries, SERVICE_MAP_TRACE_CAP, ServiceCatalogRow, ServiceEdge, ServiceSummary,
    SignalKind, SpanRed, StorageResult, attribute_compare_key_allowed, attribute_compare_score,
    field_key_identifier_like, field_key_namespace, field_value_display,
    metric_group_label_allowed, runtime_metric_family, runtime_metric_unit, span_field_key_allowed,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::RangeInclusive;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Mutex as AsyncMutex, oneshot};

#[expect(missing_debug_implementations, reason = "opaque normalizers")]
pub struct MemoryStore {
    inner: Mutex<Inner>,
    normalize_traces: Option<TraceNormalizer>,
    normalize_logs: Option<LogNormalizer>,
    traces_gate: AsyncMutex<Option<oneshot::Receiver<()>>>,
    error_event_read_calls: AtomicUsize,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            normalize_traces: None,
            normalize_logs: None,
            traces_gate: AsyncMutex::new(None),
            error_event_read_calls: AtomicUsize::new(0),
        }
    }
}

#[derive(Default)]
pub(super) struct Inner {
    pub(super) spans: Vec<SpanRow>,
    pub(super) logs: Vec<LogRow>,
    pub(super) metric_points: Vec<MetricPointRow>,
    pub(super) histograms: Vec<HistogramRow>,
    pub(super) metric_exemplars: Vec<MetricExemplarRow>,
    pub(super) error_events: Vec<ErrorEventRow>,
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

    pub fn error_event_read_calls(&self) -> usize {
        self.error_event_read_calls.load(Ordering::Relaxed)
    }

    pub(super) fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[async_trait::async_trait]
impl adapter::IngestStore for MemoryStore {
    async fn ingest_traces(
        &self,
        request: &ExportTraceServiceRequest,
        _raw: bytes::Bytes,
    ) -> StorageResult<()> {
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
    ) -> StorageResult<()> {
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
    ) -> StorageResult<()> {
        let mut inner = self.lock();
        inner.metric_points.extend(points);
        inner.histograms.extend(histograms);
        inner.metric_exemplars.extend(exemplars);
        Ok(())
    }

    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> StorageResult<()> {
        self.lock().error_events.extend(rows);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
