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
