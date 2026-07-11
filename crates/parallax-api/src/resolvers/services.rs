//! GraphQL services domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    ApiContext, clamp_limit, field_err, nanos_string, parse_range, saturate_i32, step_nanos,
};

use crate::resolvers::common::Point;
use parallax_core::semconv;
use parallax_storage::adapter::{
    OverviewTotals, ReleaseWindow as StorageReleaseWindow, SERVICE_MAP_TRACE_CAP,
    ServiceCatalogRow as StorageServiceCatalogRow, ServiceEdge as StorageServiceEdge,
    ServiceSummary as StorageServiceSummary, SpanRed as StorageSpanRed,
};
use parallax_storage::model::{MetricAgg, SeriesPoint};

pub struct Overview(pub(crate) OverviewTotals);

#[graphql_object(context = ApiContext)]
impl Overview {
    fn span_count(&self) -> String {
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

pub struct ServiceSummary(pub(crate) StorageServiceSummary);

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

pub struct ReleaseWindow(pub(crate) StorageReleaseWindow);

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

pub struct ServiceCatalogRow(pub(crate) StorageServiceCatalogRow);

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
pub struct ServiceNodeData {
    name: String,
    last_seen_nanos: u128,
    span_count: u64,
    error_count: u64,
    p95_ms: Option<f64>,
}

pub struct ServiceNode(pub(crate) ServiceNodeData);

#[graphql_object(context = ApiContext)]
impl ServiceNode {
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

pub struct ServiceEdge(pub(crate) StorageServiceEdge);

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

pub struct ServiceMap {
    nodes: Vec<ServiceNodeData>,
    edges: Vec<StorageServiceEdge>,
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

pub struct SpanRed(pub(crate) StorageSpanRed);

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

#[derive(juniper::GraphQLEnum, Clone, Copy)]
pub enum SignalKind {
    Spans,
    Traces,
    Logs,
    Errors,
    MetricPoints,
}

impl From<SignalKind> for parallax_storage::adapter::SignalKind {
    fn from(value: SignalKind) -> Self {
        match value {
            SignalKind::Spans => Self::Spans,
            SignalKind::Traces => Self::Traces,
            SignalKind::Logs => Self::Logs,
            SignalKind::Errors => Self::Errors,
            SignalKind::MetricPoints => Self::MetricPoints,
        }
    }
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
pub struct ServiceOverview {
    service: String,
    from: u128,
    to: u128,
    step: u128,
    red: tokio::sync::OnceCell<Arc<RedSource>>,
    runtime: tokio::sync::OnceCell<(Vec<SeriesPoint>, Vec<SeriesPoint>)>,
}

impl ServiceOverview {
    fn new(service: String, from: u128, to: u128, step: u128) -> Self {
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
                .map_err(field_err)?;
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
                        .map_err(field_err)?;
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
                        .map_err(field_err)?;
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
            .map_err(field_err)?;
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

pub(crate) async fn overview(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Overview> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    Ok(Overview(
        context
            .store
            .overview_totals(from..=to)
            .await
            .map_err(field_err)?,
    ))
}

pub(crate) async fn signal_count_series(
    context: &ApiContext,
    kind: SignalKind,
    service: Option<String>,
    from_nanos: String,
    to_nanos: String,
    step_seconds: Option<i32>,
) -> FieldResult<Vec<Point>> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let series = context
        .store
        .signal_count_series(
            kind.into(),
            service.as_deref().filter(|s| !s.is_empty()),
            from..=to,
            step_nanos(step_seconds),
        )
        .await
        .map_err(field_err)?;
    Ok(series.into_iter().map(Point).collect())
}

pub(crate) async fn service_list(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<ServiceSummary>> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let services = context
        .store
        .service_summaries(from..=to)
        .await
        .map_err(field_err)?;
    Ok(services.into_iter().map(ServiceSummary).collect())
}

pub(crate) async fn releases(
    context: &ApiContext,
    service: String,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<ReleaseWindow>> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let windows = context
        .store
        .release_windows(&service, from..=to)
        .await
        .map_err(field_err)?;
    Ok(windows.into_iter().map(ReleaseWindow).collect())
}

pub(crate) async fn service_catalog(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<ServiceCatalogRow>> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let rows = context
        .store
        .service_catalog(from..=to)
        .await
        .map_err(field_err)?;
    Ok(rows.into_iter().map(ServiceCatalogRow).collect())
}

pub(crate) async fn service_map(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
    max_traces: Option<i32>,
) -> FieldResult<ServiceMap> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let max_traces = clamp_limit(max_traces, 50).min(SERVICE_MAP_TRACE_CAP);
    let (services, edges) = tokio::try_join!(
        context.store.service_summaries(from..=to),
        context.store.service_map(from..=to, max_traces),
    )
    .map_err(field_err)?;
    let mut nodes: BTreeMap<String, ServiceNodeData> = services
        .into_iter()
        .map(|service| {
            (
                service.name.clone(),
                ServiceNodeData {
                    name: service.name,
                    last_seen_nanos: service.last_seen_nanos,
                    span_count: service.span_count,
                    error_count: service.error_count,
                    p95_ms: service.p95_ms,
                },
            )
        })
        .collect();
    for edge in &edges {
        for service in [&edge.source, &edge.target] {
            nodes
                .entry(service.clone())
                .or_insert_with(|| ServiceNodeData {
                    name: service.clone(),
                    last_seen_nanos: 0,
                    span_count: 0,
                    error_count: 0,
                    p95_ms: None,
                });
        }
    }
    Ok(ServiceMap {
        nodes: nodes.into_values().collect(),
        edges,
    })
}

pub(crate) async fn service_red(
    context: &ApiContext,
    service: Option<String>,
    from_nanos: String,
    to_nanos: String,
    step_seconds: Option<i32>,
) -> FieldResult<SpanRed> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    Ok(SpanRed(
        context
            .store
            .span_red_series(
                service.as_deref().filter(|s| !s.is_empty()),
                from..=to,
                step_nanos(step_seconds),
            )
            .await
            .map_err(field_err)?,
    ))
}

pub(crate) async fn service_overview(
    context: &ApiContext,
    service: String,
    from_nanos: String,
    to_nanos: String,
    step_seconds: Option<i32>,
) -> FieldResult<ServiceOverview> {
    let _ = context;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    Ok(ServiceOverview::new(
        service,
        from,
        to,
        step_nanos(step_seconds),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;

    use parallax_storage::model::{ErrorEventRow, ErrorSource, LogRow};
    use std::sync::Arc;

    #[tokio::test]
    async fn overview_service_analytics_queries_execute_against_memory_store() {
        let store = Arc::new(MemoryStore::new());
        let mut errored = span("api", "t1", "b", 1_500_000_000, 30_000_000);
        errored.status_code = "STATUS_CODE_ERROR".into();
        store
            .push_spans(vec![span("api", "t1", "a", 1_000_000_000, 10_000_000), errored]
            );
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
            .write_error_events(vec![ErrorEventRow {
                ts_nanos: 1_600_000_000,
                service: "api".into(),
                fingerprint: "fp".into(),
                error_type: "Error".into(),
                message: "bad".into(),
                stacktrace: None,
                source: ErrorSource::SpanStatus,
                trace_id: "t1".into(),
                span_id: "b".into(),
                attributes: serde_json::Value::Null,
            }])
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              overview(fromNanos: "0", toNanos: "2000000000") {
                spanCount traceCount logCount errorCount errorRate activeServices
              }
              signalCountSeries(kind: SPANS, service: "api", fromNanos: "0", toNanos: "2000000000", stepSeconds: 1) {
                tsNanos value
              }
              serviceList(fromNanos: "0", toNanos: "2000000000") {
                name lastSeenNanos spanCount errorCount p95Ms
              }
              serviceRed(service: "api", fromNanos: "0", toNanos: "2000000000", stepSeconds: 1) {
                rate { tsNanos value }
                errorRate { value }
                p95 { value }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(
            json.pointer("/data/overview/spanCount"),
            Some(&serde_json::json!("2"))
        );
        assert_eq!(
            json.pointer("/data/overview/traceCount"),
            Some(&serde_json::json!("1"))
        );
        assert_eq!(
            json.pointer("/data/overview/logCount"),
            Some(&serde_json::json!("1"))
        );
        assert_eq!(
            json.pointer("/data/overview/errorCount"),
            Some(&serde_json::json!("1"))
        );
        assert_eq!(
            json.pointer("/data/signalCountSeries/0/tsNanos"),
            Some(&serde_json::json!("1000000000"))
        );
        assert_eq!(
            json.pointer("/data/signalCountSeries/0/value"),
            Some(&serde_json::json!(2.0))
        );
        assert_eq!(
            json.pointer("/data/serviceList/0/name"),
            Some(&serde_json::json!("api"))
        );
        assert_eq!(
            json.pointer("/data/serviceList/0/spanCount"),
            Some(&serde_json::json!("2"))
        );
        assert_eq!(
            json.pointer("/data/serviceRed/rate/0/value"),
            Some(&serde_json::json!(2.0))
        );
        assert_eq!(
            Overview(OverviewTotals {
                span_count: i32::MAX as u64 + 1,
                trace_count: 0,
                log_count: 0,
                metric_point_count: 0,
                error_count: 0,
                error_rate: 0.0,
                active_services: 0,
            })
            .span_count(),
            "2147483648"
        );
    }

    #[tokio::test]
    async fn releases_resolver_returns_service_windows() {
        let store = Arc::new(MemoryStore::new());
        store.push_spans(
                vec![
                    span_with_release("checkout", "t1", "a", 10, "v1"),
                    span_with_release("checkout", "t2", "a", 30, "v1"),
                    span_with_release("checkout", "t3", "a", 50, "v2"),
                    span_with_release("catalog", "t4", "a", 20, "v9"),
                ]
            );
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              releases(service: "checkout", fromNanos: "0", toNanos: "100") {
                version firstSeenNanos lastSeenNanos spanCount
              }
            }
            "#
            .into(),
            None,
            None,
        );

        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(error_messages(&json).is_empty(), "releases query: {json}");
        assert_eq!(
            json.pointer("/data/releases/0/version"),
            Some(&serde_json::json!("v1"))
        );
        assert_eq!(
            json.pointer("/data/releases/0/firstSeenNanos"),
            Some(&serde_json::json!("10"))
        );
        assert_eq!(
            json.pointer("/data/releases/0/lastSeenNanos"),
            Some(&serde_json::json!("30"))
        );
        assert_eq!(
            json.pointer("/data/releases/0/spanCount"),
            Some(&serde_json::json!("2"))
        );
        assert_eq!(
            json.pointer("/data/releases/1/version"),
            Some(&serde_json::json!("v2"))
        );
    }

    #[tokio::test]
    async fn service_catalog_resolver_returns_identity_rows() {
        let store = Arc::new(MemoryStore::new());
        let mut checkout = span("checkout", "t1", "root", 10, 1_000);
        checkout.resource = serde_json::json!({
            "service.version": "v1",
            "service.namespace": "shop",
            "deployment.environment.name": "prod",
            "telemetry.sdk.language": "rust",
            "telemetry.sdk.name": "opentelemetry",
            "telemetry.sdk.version": "0.32.1",
            "service.instance.id": "checkout-a"
        });
        store.push_spans(
                vec![checkout, span("bare", "t2", "root", 20, 1_000)]
            );
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              serviceCatalog(fromNanos: "0", toNanos: "100") {
                name serviceVersion serviceNamespace deploymentEnvironment
                telemetrySdkLanguage telemetrySdkName telemetrySdkVersion
                lastSeenNanos instanceCount
              }
            }
            "#
            .into(),
            None,
            None,
        );

        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(
            error_messages(&json).is_empty(),
            "serviceCatalog query: {json}"
        );
        let rows = json
            .pointer("/data/serviceCatalog")
            .unwrap()
            .as_array()
            .unwrap();
        let checkout = rows
            .iter()
            .find(|row| row.get("name") == Some(&serde_json::json!("checkout")))
            .unwrap();
        assert_eq!(
            checkout.get("serviceVersion"),
            Some(&serde_json::json!("v1"))
        );
        assert_eq!(
            checkout.get("deploymentEnvironment"),
            Some(&serde_json::json!("prod"))
        );
        assert_eq!(
            checkout.get("telemetrySdkLanguage"),
            Some(&serde_json::json!("rust"))
        );
        assert_eq!(checkout.get("instanceCount"), Some(&serde_json::json!("1")));
        let bare = rows
            .iter()
            .find(|row| row.get("name") == Some(&serde_json::json!("bare")))
            .unwrap();
        assert_eq!(bare.get("serviceVersion"), Some(&serde_json::Value::Null));
        assert_eq!(bare.get("instanceCount"), Some(&serde_json::json!("0")));
    }

    #[tokio::test]
    async fn service_map_resolver_returns_nodes_and_edges() {
        let store = Arc::new(MemoryStore::new());
        let mut a_client = span("A", "trace-ab", "a-client", 100, 10_000_000);
        a_client.kind = "SPAN_KIND_CLIENT".into();
        let mut b_server = span("B", "trace-ab", "b-server", 101, 20_000_000);
        b_server.kind = "SPAN_KIND_SERVER".into();
        b_server.parent_span_id = Some("a-client".into());
        b_server.status_code = "STATUS_CODE_ERROR".into();
        store.push_spans(vec![a_client, b_server]);

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              serviceMap(fromNanos: "0", toNanos: "200", maxTraces: 10) {
                nodes { name spanCount errorCount p95Ms }
                edges { source target callCount errorCount p50Ms p95Ms }
              }
            }
            "#
            .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, request).await;
        let json = serde_json::to_value(response).unwrap();

        assert!(error_messages(&json).is_empty(), "serviceMap query: {json}");
        assert!(
            json.pointer("/data/serviceMap/nodes")
                .and_then(|value| value.as_array())
                .is_some_and(|nodes| nodes.iter().any(|node| node["name"] == "A")
                    && nodes.iter().any(|node| node["name"] == "B")),
            "serviceMap nodes: {json}"
        );
        assert_eq!(
            json.pointer("/data/serviceMap/edges/0/source"),
            Some(&serde_json::json!("A"))
        );
        assert_eq!(
            json.pointer("/data/serviceMap/edges/0/target"),
            Some(&serde_json::json!("B"))
        );
        assert_eq!(
            json.pointer("/data/serviceMap/edges/0/errorCount"),
            Some(&serde_json::json!("1"))
        );
    }
}
