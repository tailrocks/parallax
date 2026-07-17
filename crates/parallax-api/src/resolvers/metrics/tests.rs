#![expect(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "exact telemetry fixture scenario"
)]

use crate::resolvers::test_support::*;
use crate::{build_schema, execute};
use parallax_storage::adapter::IngestStore;
use parallax_test_support::builders::MemoryStore;

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
                        invocation_id: Some("run-a".into()),
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
                        invocation_id: None,
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
                        invocation_id: None,
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
                invocation_id: Some("run-a".into()),
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
            tsNanos service name value traceId spanId invocationId attributes
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
        json.pointer("/data/metricExemplars/0/invocationId"),
        Some(&serde_json::json!("run-a"))
    );
    assert_eq!(
        json.pointer("/data/metricExemplars/0/value"),
        Some(&serde_json::json!(120.0))
    );
}

#[tokio::test]
async fn metric_catalog_classifies_kinds_and_counts_finite_window_samples() {
    use parallax_storage::model::HistogramRow;
    let store = Arc::new(MemoryStore::new());
    store
        .ingest_metrics(
            vec![
                MetricPointRow {
                    ts_nanos: 1_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.load".into(),
                    value: 6.0,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
                MetricPointRow {
                    ts_nanos: 2_000_000_000,
                    service: "billing".into(),
                    name: "shapes.region.load".into(),
                    value: 3.0,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "us"}),
                },
                // NaN sample must not count (metric-summary contract).
                MetricPointRow {
                    ts_nanos: 2_500_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.load".into(),
                    value: f64::NAN,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({}),
                },
                MetricPointRow {
                    ts_nanos: 3_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.requests_total".into(),
                    value: 42.0,
                    is_monotonic: true,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
                // Outside the queried window: must be absent from counts.
                MetricPointRow {
                    ts_nanos: 9_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.load".into(),
                    value: 1.0,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({}),
                },
            ],
            vec![HistogramRow {
                ts_nanos: 1_500_000_000,
                service: "checkout".into(),
                name: "http.server.request.duration".into(),
                count: 7,
                sum: 3.5,
                bucket_counts: vec![3, 3, 1],
                bounds: vec![0.1, 1.0],
                attributes: serde_json::json!({}),
            }],
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
          metricCatalog(fromNanos: "0", toNanos: "4000000000") {
            name kind unit services lastDatapointNanos pointCount
          }
          gauges: metricCatalog(fromNanos: "0", toNanos: "4000000000", kind: "gauge") { name }
          searched: metricCatalog(fromNanos: "0", toNanos: "4000000000", q: "REQUESTS") { name kind }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "metricCatalog: {json}");

    let rows = json
        .pointer("/data/metricCatalog")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(rows.len(), 3, "three catalog families: {json}");
    let by_name = |name: &str| {
        rows.iter()
            .find(|row| row["name"] == name)
            .unwrap_or_else(|| panic!("{name} in catalog: {json}"))
    };
    let gauge = by_name("shapes.region.load");
    assert_eq!(gauge["kind"], "gauge");
    assert_eq!(gauge["pointCount"], "2", "NaN and out-of-window excluded");
    assert_eq!(gauge["lastDatapointNanos"], "2000000000");
    assert_eq!(
        gauge["services"],
        serde_json::json!(["billing", "checkout"])
    );
    let sum = by_name("shapes.region.requests_total");
    assert_eq!(sum["kind"], "sum");
    assert_eq!(sum["pointCount"], "1");
    let histogram = by_name("http.server.request.duration");
    assert_eq!(histogram["kind"], "histogram");
    assert_eq!(histogram["pointCount"], "1", "one count per export");

    let gauges = json
        .pointer("/data/gauges")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(gauges.len(), 1, "kind filter: {json}");
    let searched = json
        .pointer("/data/searched")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(searched.len(), 1, "case-insensitive q filter: {json}");
    assert_eq!(searched[0]["name"], "shapes.region.requests_total");
}

#[tokio::test]
async fn metric_catalog_rejects_unknown_kind_and_reversed_range() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let bad_kind = juniper::http::GraphQLRequest::new(
        r#"{ metricCatalog(fromNanos: "0", toNanos: "1", kind: "counter") { name } }"#.into(),
        None,
        None,
    );
    let response = execute(&schema, &context, bad_kind).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("gauge|sum|histogram")),
        "unknown kind rejected: {json}"
    );

    let reversed = juniper::http::GraphQLRequest::new(
        r#"{ metricCatalog(fromNanos: "5", toNanos: "1") { name } }"#.into(),
        None,
        None,
    );
    let response = execute(&schema, &context, reversed).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        !error_messages(&json).is_empty(),
        "reversed range rejected: {json}"
    );
}

#[test]
fn effective_step_rounds_up_to_contract_bucket_cap() {
    use super::effective_step_seconds;
    // 1h window, no step: default ceil(3600/60) = 60s.
    assert_eq!(effective_step_seconds(0, 3_600_000_000_000, None), 60);
    // Requested 1s over 1h = 3600 buckets: rounded up to 30s (120 buckets).
    assert_eq!(effective_step_seconds(0, 3_600_000_000_000, Some(1)), 30);
    // Requested step already coarse enough is kept.
    assert_eq!(effective_step_seconds(0, 3_600_000_000_000, Some(120)), 120);
    // Zero/negative requested falls back to the default.
    assert_eq!(effective_step_seconds(0, 3_600_000_000_000, Some(0)), 60);
    // Tiny window: minimum one second.
    assert_eq!(effective_step_seconds(0, 1, None), 1);
}

#[tokio::test]
async fn metric_query_enforces_typed_aggregation_legality() {
    let store = Arc::new(MemoryStore::new());
    store
        .ingest_metrics(
            vec![
                MetricPointRow {
                    ts_nanos: 1_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.requests_total".into(),
                    value: 10.0,
                    is_monotonic: true,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
                MetricPointRow {
                    ts_nanos: 61_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.requests_total".into(),
                    value: 40.0,
                    is_monotonic: true,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
            ],
            Vec::new(),
            Vec::new(),
            Default::default(),
        )
        .await
        .unwrap();
    let schema = build_schema();
    let context = context_with_memory(store).await;

    let illegal = juniper::http::GraphQLRequest::new(
        r#"{ metricQuery(name: "shapes.region.requests_total", kind: "sum", agg: "avg", fromNanos: "0", toNanos: "120000000000") { kind } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, illegal).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("illegal for kind 'sum'")
                && message.contains("sum|rate")),
        "illegal agg names the legal set: {json}"
    );

    let legal = juniper::http::GraphQLRequest::new(
        r#"{ metricQuery(name: "shapes.region.requests_total", kind: "sum", agg: "rate", fromNanos: "0", toNanos: "120000000000", stepSeconds: 60) {
            kind effectiveStepSeconds series { groupValue points { tsNanos value } }
        } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, legal).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "legal rate query: {json}");
    assert_eq!(json.pointer("/data/metricQuery/kind").unwrap(), "sum");
    assert_eq!(
        json.pointer("/data/metricQuery/effectiveStepSeconds")
            .unwrap(),
        60
    );
    let points = json
        .pointer("/data/metricQuery/series/0/points")
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(!points.is_empty(), "rate series has points: {json}");

    let histogram_group_by = juniper::http::GraphQLRequest::new(
        r#"{ metricQuery(name: "http.server.request.duration", kind: "histogram", agg: "p95", groupBy: "region", fromNanos: "0", toNanos: "120000000000") { kind } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, histogram_group_by).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("groupBy is not supported for histogram")),
        "histogram groupBy rejected: {json}"
    );
}

#[tokio::test]
async fn metric_query_supports_last_and_increase_aggregations() {
    let store = Arc::new(MemoryStore::new());
    store
        .ingest_metrics(
            vec![
                MetricPointRow {
                    ts_nanos: 1_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.requests_total".into(),
                    value: 10.0,
                    is_monotonic: true,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
                MetricPointRow {
                    ts_nanos: 61_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.requests_total".into(),
                    value: 40.0,
                    is_monotonic: true,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
                MetricPointRow {
                    ts_nanos: 2_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.load".into(),
                    value: 3.0,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
                MetricPointRow {
                    ts_nanos: 30_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.load".into(),
                    value: 7.0,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
            ],
            Vec::new(),
            Vec::new(),
            Default::default(),
        )
        .await
        .unwrap();
    let schema = build_schema();
    let context = context_with_memory(store).await;

    // increase: reset-clamped counter growth per bucket, not divided by step.
    let increase = juniper::http::GraphQLRequest::new(
        r#"{ metricQuery(name: "shapes.region.requests_total", kind: "sum", agg: "increase", fromNanos: "0", toNanos: "120000000000", stepSeconds: 60) {
            series { points { tsNanos value } }
        } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, increase).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "increase legal: {json}");
    let points = json
        .pointer("/data/metricQuery/series/0/points")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(points.len(), 1, "one delta bucket: {json}");
    assert_eq!(points[0].get("value").unwrap().as_f64().unwrap(), 30.0);

    // last: latest gauge sample inside the single bucket.
    let last = juniper::http::GraphQLRequest::new(
        r#"{ metricQuery(name: "shapes.region.load", kind: "gauge", agg: "last", fromNanos: "0", toNanos: "60000000000", stepSeconds: 60) {
            series { points { tsNanos value } }
        } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, last).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "last legal: {json}");
    let points = json
        .pointer("/data/metricQuery/series/0/points")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(points.len(), 1, "single bucket: {json}");
    assert_eq!(points[0].get("value").unwrap().as_f64().unwrap(), 7.0);
}

#[tokio::test]
async fn metric_query_applies_attribute_where_filters() {
    let store = Arc::new(MemoryStore::new());
    store
        .ingest_metrics(
            vec![
                MetricPointRow {
                    ts_nanos: 1_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.load".into(),
                    value: 6.0,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "eu"}),
                },
                MetricPointRow {
                    ts_nanos: 2_000_000_000,
                    service: "checkout".into(),
                    name: "shapes.region.load".into(),
                    value: 1.0,
                    is_monotonic: false,
                    invocation_id: None,
                    attributes: serde_json::json!({"region": "ap"}),
                },
            ],
            Vec::new(),
            Vec::new(),
            Default::default(),
        )
        .await
        .unwrap();
    let schema = build_schema();
    let context = context_with_memory(store).await;

    let filtered = juniper::http::GraphQLRequest::new(
        r#"{ metricQuery(name: "shapes.region.load", kind: "gauge", agg: "max", fromNanos: "0", toNanos: "60000000000", stepSeconds: 60, attributeFilters: [{key: "region", op: "=", value: "ap"}]) {
            series { points { value } }
        } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, filtered).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "filtered legal: {json}");
    let points = json
        .pointer("/data/metricQuery/series/0/points")
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(points.len(), 1, "one bucket: {json}");
    assert_eq!(points[0].get("value").unwrap().as_f64().unwrap(), 1.0);

    let rejected = juniper::http::GraphQLRequest::new(
        r#"{ metricQuery(name: "shapes.region.load", kind: "gauge", agg: "max", fromNanos: "0", toNanos: "60000000000", attributeFilters: [{key: "region", op: "~", value: "ap"}]) { kind } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, rejected).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("invalid attribute filter operator")),
        "bad operator rejected: {json}"
    );
}
