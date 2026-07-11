//! GraphQL metrics domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;

use crate::{retained_recent_range, 
    ApiContext, clamp_limit, field_err, nanos_string, parse_range, step_nanos,
    validate_metric_group_label, validate_metric_name,
};

use crate::resolvers::common::Point;
use parallax_storage::adapter::RuntimeMetricSeries as StorageRuntimeMetricSeries;
use parallax_storage::model::{MetricAgg, SeriesPoint};

pub struct Series {
    group_value: Option<String>,
    points: Vec<SeriesPoint>,
}

#[graphql_object(context = ApiContext)]
impl Series {
    fn group_value(&self) -> Option<&str> {
        self.group_value.as_deref()
    }
    fn points(&self) -> Vec<Point> {
        self.points.iter().copied().map(Point).collect()
    }
}

pub struct RuntimeMetric(pub(crate) StorageRuntimeMetricSeries);

#[graphql_object(context = ApiContext)]
impl RuntimeMetric {
    fn family(&self) -> &str {
        &self.0.family
    }
    fn metric(&self) -> &str {
        &self.0.metric
    }
    fn unit(&self) -> Option<&str> {
        self.0.unit.as_deref()
    }
    fn points(&self) -> Vec<Point> {
        self.0.points.iter().copied().map(Point).collect()
    }
}

pub struct MetricExemplar(pub(crate) model::MetricExemplarRow);

#[graphql_object(context = ApiContext)]
impl MetricExemplar {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn value(&self) -> f64 {
        self.0.value
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn run_id(&self) -> Option<&str> {
        self.0.run_id.as_deref()
    }
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
    }
}

pub(crate) async fn metric_names(
    context: &ApiContext,
    prefix: Option<String>,
) -> FieldResult<Vec<String>> {
    let mut names = context.store.metric_names(retained_recent_range()).await.map_err(field_err)?;
    if let Some(prefix) = prefix {
        names.retain(|n| n.starts_with(&prefix));
    }
    Ok(names)
}

pub(crate) async fn metric_labels(context: &ApiContext, name: String) -> FieldResult<Vec<String>> {
    validate_metric_name(&name)?;
    context.store.metric_labels(&name).await.map_err(field_err)
}

pub(crate) async fn metric_label_values(
    context: &ApiContext,
    name: String,
    label: String,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<String>> {
    validate_metric_name(&name)?;
    validate_metric_group_label(&label)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    context
        .store
        .metric_label_values(&name, &label, from..=to)
        .await
        .map_err(field_err)
}

pub(crate) async fn services(context: &ApiContext) -> FieldResult<Vec<String>> {
    context.store.service_names(retained_recent_range()).await.map_err(field_err)
}

pub(crate) async fn runtime_snapshot(
    context: &ApiContext,
    service: Option<String>,
    run_id: Option<String>,
    from_nanos: String,
    to_nanos: String,
    step_seconds: i32,
) -> FieldResult<Vec<RuntimeMetric>> {
    match (service.as_deref(), run_id.as_deref()) {
        (Some(_), Some(_)) | (None, None) => {
            return Err(field_err("runtimeSnapshot takes exactly one scope"));
        }
        _ => {}
    }
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let rows = context
        .store
        .runtime_snapshot(
            service.as_deref(),
            run_id.as_deref(),
            from..=to,
            step_nanos(Some(step_seconds)),
        )
        .await
        .map_err(field_err)?;
    Ok(rows.into_iter().map(RuntimeMetric).collect())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn metric_series(
    context: &ApiContext,
    name: String,
    from_nanos: String,
    to_nanos: String,
    service: Option<String>,
    run_id: Option<String>,
    group_by: Option<String>,
    step_seconds: Option<i32>,
    agg: Option<String>,
) -> FieldResult<Vec<Series>> {
    validate_metric_name(&name)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let agg = MetricAgg::parse(agg.as_deref().unwrap_or("avg"))
        .ok_or_else(|| field_err("agg must be avg|min|max|sum|rate"))?;
    match group_by {
        Some(group_by) => {
            validate_metric_group_label(&group_by)?;
            if run_id.is_some() {
                return Err(field_err("runId with groupBy is not supported yet"));
            }
            let groups = context
                .store
                .metric_series_grouped(
                    &name,
                    service.as_deref(),
                    &group_by,
                    from..=to,
                    step_nanos(step_seconds),
                    agg,
                )
                .await
                .map_err(field_err)?;
            Ok(groups
                .into_iter()
                .map(|(group_value, points)| Series {
                    group_value: Some(group_value),
                    points,
                })
                .collect())
        }
        None => {
            let points = context
                .store
                .metric_series(
                    &name,
                    service.as_deref(),
                    run_id.as_deref(),
                    from..=to,
                    step_nanos(step_seconds),
                    agg,
                )
                .await
                .map_err(field_err)?;
            Ok(vec![Series {
                group_value: None,
                points,
            }])
        }
    }
}

pub(crate) async fn histogram_quantile(
    context: &ApiContext,
    name: String,
    from_nanos: String,
    to_nanos: String,
    q: f64,
    service: Option<String>,
    step_seconds: Option<i32>,
) -> FieldResult<Vec<Point>> {
    validate_metric_name(&name)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let series = context
        .store
        .histogram_quantile(
            &name,
            service.as_deref(),
            from..=to,
            step_nanos(step_seconds),
            q,
        )
        .await
        .map_err(field_err)?;
    Ok(series.into_iter().map(Point).collect())
}

pub(crate) async fn metric_exemplars(
    context: &ApiContext,
    name: String,
    from_nanos: String,
    to_nanos: String,
    service: Option<String>,
    limit: Option<i32>,
) -> FieldResult<Vec<MetricExemplar>> {
    validate_metric_name(&name)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let rows = context
        .store
        .metric_exemplars(&name, service.as_deref(), from..=to, clamp_limit(limit, 50))
        .await
        .map_err(field_err)?;
    Ok(rows.into_iter().map(MetricExemplar).collect())
}

#[cfg(test)]
mod tests {

    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;

    use parallax_storage::model::{MetricExemplarRow, MetricPointRow};
    use std::sync::Arc;

    #[tokio::test]
    async fn metric_name_validation_rejects_identifier_breakout() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let invalid = juniper::http::GraphQLRequest::new(
            r#"{ metricSeries(name: "evil\"name", fromNanos: "0", toNanos: "1") { groupValue } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, invalid).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("invalid metric name")),
            "invalid metric name rejected: {json}"
        );

        let valid = juniper::http::GraphQLRequest::new(
            r#"{ metricSeries(name: "http.server.request.duration", fromNanos: "0", toNanos: "1") { groupValue points { value } } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, valid).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json).is_empty(),
            "legal OTel metric name accepted: {json}"
        );
        assert!(
            json.pointer("/data/metricSeries")
                .and_then(|value| value.as_array())
                .is_some(),
            "metricSeries returns data for valid name: {json}"
        );
    }

    #[tokio::test]
    async fn metric_label_and_runtime_resolvers_query_memory_store() {
        let store = Arc::new(MemoryStore::new());
        store
            .ingest_metrics(
                {
                    let mut points = vec![
                        MetricPointRow {
                            ts_nanos: 1_000_000_000,
                            service: "checkout".into(),
                            name: "process.cpu.utilization".into(),
                            value: 0.5,
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
                            name: "jvm.memory.used".into(),
                            value: 256.0,
                            is_monotonic: false,
                            run_id: None,
                            attributes: serde_json::json!({
                                "runtime.name": "jvm"
                            }),
                        },
                    ];
                    for index in 0..110 {
                        points.push(MetricPointRow {
                            ts_nanos: 2_100_000_000 + index,
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
                    points
                },
                Vec::new(),
                Vec::new(),
                Default::default(),
            )
            .await
            .unwrap();
        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              metricLabels(name: "process.cpu.utilization")
              metricLabelValues(name: "process.cpu.utilization", label: "payment.method", fromNanos: "0", toNanos: "3000000000")
              cappedMetricLabelValues: metricLabelValues(name: "process.cpu.utilization", label: "runtime.name", fromNanos: "0", toNanos: "3000000000")
              runtimeSnapshot(service: "checkout", fromNanos: "0", toNanos: "3000000000", stepSeconds: 1) {
                family metric unit points { tsNanos value }
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
            "metric label/runtime query: {json}"
        );
        assert_eq!(
            json.pointer("/data/metricLabels"),
            Some(&serde_json::json!(["payment.method", "runtime.name"]))
        );
        assert_eq!(
            json.pointer("/data/metricLabelValues"),
            Some(&serde_json::json!(["card"]))
        );
        assert_eq!(
            json.pointer("/data/cappedMetricLabelValues")
                .and_then(|value| value.as_array())
                .map(Vec::len),
            Some(100)
        );
        let runtime = json
            .pointer("/data/runtimeSnapshot")
            .and_then(|value| value.as_array())
            .unwrap();
        assert_eq!(runtime.len(), 2, "two runtime families returned: {json}");
        assert!(runtime.iter().any(|row| row["family"] == "process"));
        assert!(runtime.iter().any(|row| row["family"] == "jvm"));

        let denied = juniper::http::GraphQLRequest::new(
            r#"{ metricSeries(name: "process.cpu.utilization", fromNanos: "0", toNanos: "3000000000", groupBy: "trace_id") { groupValue } }"#
                .into(),
            None,
            None,
        );
        let response = execute(&schema, &context, denied).await;
        let json = serde_json::to_value(response).unwrap();
        assert!(
            error_messages(&json)
                .iter()
                .any(|message| message.contains("high-cardinality identifier")),
            "denylisted groupBy rejected: {json}"
        );
    }

    #[tokio::test]
    async fn metric_exemplars_resolver_returns_trace_links() {
        let store = Arc::new(MemoryStore::new());
        store
            .ingest_metrics(
                Vec::new(),
                Vec::new(),
                vec![MetricExemplarRow {
                    ts_nanos: 20,
                    service: "checkout".into(),
                    name: "http.server.request.duration".into(),
                    value: 120.0,
                    trace_id: "trace-a".into(),
                    span_id: "span-a".into(),
                    run_id: Some("run-a".into()),
                    attributes: serde_json::json!({"route": "/checkout"}),
                }],
                Default::default(),
            )
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              metricExemplars(
                name: "http.server.request.duration"
                fromNanos: "0"
                toNanos: "100"
                service: "checkout"
                limit: 10
              ) {
                tsNanos service name value traceId spanId runId attributes
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
            "metricExemplars query: {json}"
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/traceId"),
            Some(&serde_json::json!("trace-a"))
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/spanId"),
            Some(&serde_json::json!("span-a"))
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/runId"),
            Some(&serde_json::json!("run-a"))
        );
        assert_eq!(
            json.pointer("/data/metricExemplars/0/value"),
            Some(&serde_json::json!(120.0))
        );
    }
}
