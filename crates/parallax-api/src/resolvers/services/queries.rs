//! Service query orchestration over storage capabilities.

use super::types::*;
use crate::resolvers::common::Point;
use crate::{ApiContext, clamp_limit, parse_range, step_nanos};
use juniper::FieldResult;
use parallax_storage::adapter::SERVICE_MAP_TRACE_CAP;
use std::collections::BTreeMap;

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
            .map_err(crate::internal_field_err)?,
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
        .map_err(crate::internal_field_err)?;
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
        .map_err(crate::internal_field_err)?;
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
        .map_err(crate::internal_field_err)?;
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
        .map_err(crate::internal_field_err)?;
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
    let (services, mut edges, invocations, catalog, external) = tokio::try_join!(
        context.store.service_summaries(from..=to),
        context.store.service_map(from..=to, max_traces),
        context
            .store
            .observed_invocations(crate::MAX_ROWS, from..=to),
        context.store.service_catalog(from..=to),
        context.store.external_dependency_edges(from..=to),
    )
    .map_err(crate::internal_field_err)?;
    // Node kinds are derived from generic signals only: a service whose
    // telemetry carried `cli.invocation.id` is a CLI application; a webjs SDK
    // is a browser; everything else is a service.
    let cli_services: std::collections::BTreeSet<&str> = invocations
        .iter()
        .map(|invocation| invocation.service.as_str())
        .filter(|service| !service.is_empty())
        .collect();
    let browser_services: std::collections::BTreeSet<&str> = catalog
        .iter()
        .filter(|row| row.telemetry_sdk_language.as_deref() == Some("webjs"))
        .map(|row| row.name.as_str())
        .collect();
    let kind_for = |name: &str| {
        if cli_services.contains(name) {
            "cli".to_string()
        } else if browser_services.contains(name) {
            "browser".to_string()
        } else {
            "service".to_string()
        }
    };
    let mut nodes: BTreeMap<String, ServiceNodeData> = services
        .into_iter()
        .map(|service| {
            let kind = kind_for(&service.name);
            (
                service.name.clone(),
                ServiceNodeData {
                    name: service.name,
                    kind,
                    system: None,
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
                    kind: kind_for(service),
                    system: None,
                    last_seen_nanos: 0,
                    span_count: 0,
                    error_count: 0,
                    p95_ms: None,
                });
        }
    }
    // Plan 166: uninstrumented dependency nodes (database | queue | external)
    // derived from generic CLIENT/PRODUCER span attributes. An instrumented
    // service with the same name wins the node slot; the edge still points at
    // that name so the graph never dangles.
    for dependency in &external {
        nodes
            .entry(dependency.name.clone())
            .and_modify(|node| {
                if node.kind == dependency.kind.as_str() {
                    node.span_count += dependency.call_count;
                    node.error_count += dependency.error_count;
                }
            })
            .or_insert_with(|| ServiceNodeData {
                name: dependency.name.clone(),
                kind: dependency.kind.as_str().to_string(),
                system: dependency.system.clone(),
                last_seen_nanos: 0,
                span_count: dependency.call_count,
                error_count: dependency.error_count,
                p95_ms: Some(dependency.p95_ms),
            });
        edges.push(parallax_storage::adapter::ServiceEdge {
            source: dependency.source.clone(),
            target: dependency.name.clone(),
            call_count: dependency.call_count,
            error_count: dependency.error_count,
            p50_ms: dependency.p50_ms,
            p95_ms: dependency.p95_ms,
        });
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
            .map_err(crate::internal_field_err)?,
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
