//! The storage adapter boundary. Everything engine-specific lives behind
//! `TelemetryStore`; product code never sees an engine.

use crate::model::*;
use std::ops::RangeInclusive;

#[async_trait::async_trait]
pub trait TelemetryStore: Send + Sync {
    async fn write_spans(&self, rows: Vec<SpanRow>) -> anyhow::Result<()>;
    async fn write_logs(&self, rows: Vec<LogRow>) -> anyhow::Result<()>;
    async fn write_metric_points(&self, rows: Vec<MetricPointRow>) -> anyhow::Result<()>;
    async fn write_histograms(&self, rows: Vec<HistogramRow>) -> anyhow::Result<()>;
    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> anyhow::Result<()>;

    /// Anchored read: every span of one trace, start-time ascending.
    async fn spans_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<SpanRow>>;
    /// Anchored read: every log of one trace, time ascending.
    async fn logs_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<LogRow>>;
    /// Error events for a fingerprint within a time range, newest first.
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>>;
}
