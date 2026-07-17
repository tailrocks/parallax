//! GraphQL service domain types and field resolvers.

use juniper::{FieldResult, graphql_object};
use std::sync::Arc;

use crate::{ApiContext, nanos_string, saturate_i32};

use crate::resolvers::common::Point;
use parallax_analysis::semconv;
use parallax_storage::adapter::{
    OverviewTotals, ReleaseWindow as StorageReleaseWindow,
    ServiceCatalogRow as StorageServiceCatalogRow, ServiceEdge as StorageServiceEdge,
    ServiceSummary as StorageServiceSummary, SpanRed as StorageSpanRed,
};
use parallax_storage::model::{MetricAgg, SeriesPoint};

pub(crate) struct Overview(pub(crate) OverviewTotals);

#[graphql_object(context = ApiContext)]
impl Overview {
    pub(super) fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
    fn trace_count(&self) -> String {
        self.0.trace_count.to_string()
    }
    fn log_count(&self) -> String {
        self.0.log_count.to_string()
    }
    fn metric_point_count(&self) -> String {
        self.0.metric_point_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn error_rate(&self) -> f64 {
        self.0.error_rate
    }
    fn active_services(&self) -> i32 {
        saturate_i32(self.0.active_services)
    }
}

pub(crate) struct ServiceSummary(pub(crate) StorageServiceSummary);

#[graphql_object(context = ApiContext)]
impl ServiceSummary {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn p95_ms(&self) -> Option<f64> {
        self.0.p95_ms
    }
}

pub(crate) struct ReleaseWindow(pub(crate) StorageReleaseWindow);

#[graphql_object(context = ApiContext)]
impl ReleaseWindow {
    fn version(&self) -> &str {
        &self.0.version
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.0.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
}

pub(crate) struct ServiceCatalogRow(pub(crate) StorageServiceCatalogRow);

#[graphql_object(context = ApiContext)]
impl ServiceCatalogRow {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn service_version(&self) -> Option<&str> {
        self.0.service_version.as_deref()
    }
    fn service_namespace(&self) -> Option<&str> {
        self.0.service_namespace.as_deref()
    }
    fn deployment_environment(&self) -> Option<&str> {
        self.0.deployment_environment.as_deref()
    }
    fn telemetry_sdk_language(&self) -> Option<&str> {
        self.0.telemetry_sdk_language.as_deref()
    }
    fn telemetry_sdk_name(&self) -> Option<&str> {
        self.0.telemetry_sdk_name.as_deref()
    }
    fn telemetry_sdk_version(&self) -> Option<&str> {
        self.0.telemetry_sdk_version.as_deref()
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn instance_count(&self) -> String {
        self.0.instance_count.to_string()
    }
}

#[derive(Clone)]
pub(crate) struct ServiceNodeData {
    pub(super) name: String,
    pub(super) kind: String,
    /// Generic system label for derived external nodes (`postgresql`,
    /// `kafka`, a host, …); `None` for instrumented services (plan 166).
    pub(super) system: Option<String>,
    pub(super) last_seen_nanos: u128,
    pub(super) span_count: u64,
    pub(super) error_count: u64,
    pub(super) p95_ms: Option<f64>,
}

pub(crate) struct ServiceNode(pub(crate) ServiceNodeData);

#[graphql_object(context = ApiContext)]
impl ServiceNode {
    fn name(&self) -> &str {
        &self.0.name
    }
    /// cli | browser | service | database | queue | external — derived from
    /// generic signals only.
    fn kind(&self) -> &str {
        &self.0.kind
    }
    /// System label for external dependency nodes (postgresql, kafka, host).
    fn system(&self) -> Option<&str> {
        self.0.system.as_deref()
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn span_count(&self) -> String {
        self.0.span_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn p95_ms(&self) -> Option<f64> {
        self.0.p95_ms
    }
}

pub(crate) struct ServiceEdge(pub(crate) StorageServiceEdge);

#[graphql_object(context = ApiContext)]
impl ServiceEdge {
    fn source(&self) -> &str {
        &self.0.source
    }
    fn target(&self) -> &str {
        &self.0.target
    }
    fn call_count(&self) -> String {
        self.0.call_count.to_string()
    }
    fn error_count(&self) -> String {
        self.0.error_count.to_string()
    }
    fn p50_ms(&self) -> f64 {
        self.0.p50_ms
    }
    fn p95_ms(&self) -> f64 {
        self.0.p95_ms
    }
}

pub(crate) struct ServiceMap {
    pub(super) nodes: Vec<ServiceNodeData>,
    pub(super) edges: Vec<StorageServiceEdge>,
}

#[graphql_object(context = ApiContext)]
impl ServiceMap {
    fn nodes(&self) -> Vec<ServiceNode> {
        self.nodes.iter().cloned().map(ServiceNode).collect()
    }
    fn edges(&self) -> Vec<ServiceEdge> {
        self.edges.iter().cloned().map(ServiceEdge).collect()
    }
}

pub(crate) struct SpanRed(pub(crate) StorageSpanRed);

#[graphql_object(context = ApiContext)]
impl SpanRed {
    fn rate(&self) -> Vec<Point> {
        self.0.rate.iter().copied().map(Point).collect()
    }
    fn error_rate(&self) -> Vec<Point> {
        self.0.error_rate.iter().copied().map(Point).collect()
    }
    fn p50(&self) -> Vec<Point> {
        self.0.p50.iter().copied().map(Point).collect()
    }
    fn p95(&self) -> Vec<Point> {
        self.0.p95.iter().copied().map(Point).collect()
    }
    fn p99(&self) -> Vec<Point> {
        self.0.p99.iter().copied().map(Point).collect()
    }
}

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub(crate) enum SignalKind {
    Spans,
    Traces,
    Logs,
    Errors,
    MetricPoints,
}

/// Shared RED series for one service overview window.
struct RedSource {
    latency_p50: Vec<SeriesPoint>,
    latency_p95: Vec<SeriesPoint>,
    latency_p99: Vec<SeriesPoint>,
    request_rate: Vec<SeriesPoint>,
}

/// The predefined per-service overview (spec §8): well-known metric names,
/// graceful absence — a missing instrument yields an empty series.
pub(crate) struct ServiceOverview {
    service: String,
    from: u128,
    to: u128,
    step: u128,
    red: tokio::sync::OnceCell<Arc<RedSource>>,
    runtime: tokio::sync::OnceCell<(Vec<SeriesPoint>, Vec<SeriesPoint>)>,
}

impl ServiceOverview {
    pub(super) fn new(service: String, from: u128, to: u128, step: u128) -> Self {
        Self {
            service,
            from,
            to,
            step,
            red: tokio::sync::OnceCell::new(),
            runtime: tokio::sync::OnceCell::new(),
        }
    }

    async fn first_nonempty_points(
        &self,
        context: &ApiContext,
        candidates: &[&str],
    ) -> FieldResult<Vec<SeriesPoint>> {
        for name in candidates {
            let series = context
                .store
                .metric_series(
                    name,
                    Some(&self.service),
                    None,
                    self.from..=self.to,
                    self.step,
                    MetricAgg::Avg,
                )
                .await
                .map_err(crate::internal_field_err)?;
            if !series.is_empty() {
                return Ok(series);
            }
        }
        Ok(Vec::new())
    }

    async fn red_source(&self, context: &ApiContext) -> FieldResult<Arc<RedSource>> {
        self.red
            .get_or_try_init(|| async {
                let step_secs = (self.step / 1_000_000_000).max(1) as f64;
                let quantiles = [0.50_f64, 0.95, 0.99];
                let mut latency_p50 = Vec::new();
                let mut latency_p95 = Vec::new();
                let mut latency_p99 = Vec::new();
                let mut request_rate = Vec::new();
                for name in semconv::REQUEST_DURATION_METRICS {
                    let series = context
                        .store
                        .histogram_quantiles(
                            name,
                            Some(&self.service),
                            self.from..=self.to,
                            self.step,
                            &quantiles,
                        )
                        .await
                        .map_err(crate::internal_field_err)?;
                    if !series.iter().any(|points| !points.is_empty()) {
                        continue;
                    }
                    latency_p50 = series.first().cloned().unwrap_or_default();
                    latency_p95 = series.get(1).cloned().unwrap_or_default();
                    latency_p99 = series.get(2).cloned().unwrap_or_default();
                    let counts = context
                        .store
                        .histogram_count_series(
                            name,
                            Some(&self.service),
                            self.from..=self.to,
                            self.step,
                        )
                        .await
                        .map_err(crate::internal_field_err)?;
                    request_rate = counts
                        .into_iter()
                        .map(|p| SeriesPoint {
                            ts_nanos: p.ts_nanos,
                            value: p.value / step_secs,
                        })
                        .collect();
                    break;
                }
                Ok(Arc::new(RedSource {
                    latency_p50,
                    latency_p95,
                    latency_p99,
                    request_rate,
                }))
            })
            .await
            .cloned()
    }

    async fn runtime_series(
        &self,
        context: &ApiContext,
    ) -> FieldResult<&(Vec<SeriesPoint>, Vec<SeriesPoint>)> {
        self.runtime
            .get_or_try_init(|| async {
                let (cpu, memory) = tokio::try_join!(
                    self.first_nonempty_points(context, semconv::CPU_METRICS),
                    self.first_nonempty_points(context, semconv::MEMORY_METRICS),
                )?;
                Ok((cpu, memory))
            })
            .await
    }
}

#[graphql_object(context = ApiContext)]
impl ServiceOverview {
    async fn cpu(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .runtime_series(context)
            .await?
            .0
            .iter()
            .copied()
            .map(Point)
            .collect())
    }
    async fn memory(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .runtime_series(context)
            .await?
            .1
            .iter()
            .copied()
            .map(Point)
            .collect())
    }
    async fn request_rate(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .red_source(context)
            .await?
            .request_rate
            .iter()
            .copied()
            .map(Point)
            .collect())
    }
    async fn latency_p50(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .red_source(context)
            .await?
            .latency_p50
            .iter()
            .copied()
            .map(Point)
            .collect())
    }
    async fn latency_p95(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .red_source(context)
            .await?
            .latency_p95
            .iter()
            .copied()
            .map(Point)
            .collect())
    }
    async fn latency_p99(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        Ok(self
            .red_source(context)
            .await?
            .latency_p99
            .iter()
            .copied()
            .map(Point)
            .collect())
    }
    async fn error_rate(&self, context: &ApiContext) -> FieldResult<Vec<Point>> {
        let step_secs = (self.step / 1_000_000_000).max(1) as f64;
        let counts = context
            .store
            .error_count_series(&self.service, self.from..=self.to, self.step)
            .await
            .map_err(crate::internal_field_err)?;
        Ok(counts
            .into_iter()
            .map(|p| {
                Point(SeriesPoint {
                    ts_nanos: p.ts_nanos,
                    value: p.value / step_secs,
                })
            })
            .collect())
    }
}
