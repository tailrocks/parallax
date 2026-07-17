#![expect(clippy::too_many_lines, reason = "measured integration scenario")]

use super::*;
use crate::resolvers::test_support::*;
use crate::{build_schema, execute};
use parallax_storage::adapter::{IngestStore, OverviewTotals};
use parallax_test_support::builders::MemoryStore;

use parallax_storage::model::{ErrorEventRow, ErrorSource, LogRow};
use std::sync::Arc;

#[tokio::test]
async fn overview_service_analytics_queries_execute_against_memory_store() {
    let store = Arc::new(MemoryStore::new());
    let mut errored = span("api", "t1", "b", 1_500_000_000, 30_000_000);
    errored.status_code = "STATUS_CODE_ERROR".into();
    store.push_spans(vec![
        span("api", "t1", "a", 1_000_000_000, 10_000_000),
        errored,
    ]);
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
        invocation_id: None,
        session_id: None,
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
    store.push_spans(vec![
        span_with_release("checkout", "t1", "a", 10, "v1"),
        span_with_release("checkout", "t2", "a", 30, "v1"),
        span_with_release("checkout", "t3", "a", 50, "v2"),
        span_with_release("catalog", "t4", "a", 20, "v9"),
    ]);
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
    store.push_spans(vec![checkout, span("bare", "t2", "root", 20, 1_000)]);
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

#[tokio::test]
async fn service_map_derives_external_dependency_nodes_from_generic_attributes() {
    let store = Arc::new(MemoryStore::new());

    // Database dependency: CLIENT span with db.* attributes, no child.
    let mut db_client = span("checkout", "trace-db", "db-client", 100, 5_000_000);
    db_client.kind = "SPAN_KIND_CLIENT".into();
    db_client.attributes = serde_json::json!({
        "db.system.name": "postgresql",
        "db.namespace": "orders",
        "server.address": "pg.internal"
    });

    // Queue dependency: PRODUCER span with messaging.* attributes, no child.
    let mut producer = span("fulfillment", "trace-q", "producer", 110, 7_000_000);
    producer.kind = "SPAN_KIND_PRODUCER".into();
    producer.attributes = serde_json::json!({
        "messaging.system": "kafka",
        "messaging.destination.name": "shipments"
    });

    // External HTTP dependency: CLIENT span with only server.address.
    let mut http_client = span("checkout", "trace-http", "http-client", 120, 9_000_000);
    http_client.kind = "SPAN_KIND_CLIENT".into();
    http_client.status_code = "STATUS_CODE_ERROR".into();
    http_client.attributes = serde_json::json!({ "server.address": "api.stripe.test" });

    // Instrumented pair (negative): CLIENT span whose SERVER child lives in
    // another instrumented service must NOT create an external node.
    let mut internal_client = span("checkout", "trace-int", "int-client", 130, 4_000_000);
    internal_client.kind = "SPAN_KIND_CLIENT".into();
    internal_client.attributes = serde_json::json!({ "server.address": "pricing.internal" });
    let mut internal_server = span("pricing", "trace-int", "int-server", 131, 3_000_000);
    internal_server.kind = "SPAN_KIND_SERVER".into();
    internal_server.parent_span_id = Some("int-client".into());

    store.push_spans(vec![
        db_client,
        producer,
        http_client,
        internal_client,
        internal_server,
    ]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          serviceMap(fromNanos: "0", toNanos: "200", maxTraces: 10) {
            nodes { name kind system spanCount errorCount }
            edges { source target callCount errorCount }
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

    let nodes = json
        .pointer("/data/serviceMap/nodes")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let node = |name: &str| {
        nodes
            .iter()
            .find(|node| node["name"] == name)
            .cloned()
            .unwrap_or_else(|| panic!("node {name} missing: {json}"))
    };

    let database = node("orders");
    assert_eq!(database["kind"], "database", "database node: {json}");
    assert_eq!(database["system"], "postgresql", "database system: {json}");

    let queue = node("shipments");
    assert_eq!(queue["kind"], "queue", "queue node: {json}");
    assert_eq!(queue["system"], "kafka", "queue system: {json}");

    let external = node("api.stripe.test");
    assert_eq!(external["kind"], "external", "external node: {json}");
    assert_eq!(external["errorCount"], "1", "external errors: {json}");

    // Instrumented pair stays an internal service edge — no external node.
    assert!(
        !nodes.iter().any(|node| node["name"] == "pricing.internal"),
        "instrumented pair must not derive an external node: {json}"
    );

    let edges = json
        .pointer("/data/serviceMap/edges")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    assert!(
        edges
            .iter()
            .any(|edge| edge["source"] == "checkout" && edge["target"] == "orders"),
        "checkout → orders edge: {json}"
    );
    assert!(
        edges
            .iter()
            .any(|edge| edge["source"] == "fulfillment" && edge["target"] == "shipments"),
        "fulfillment → shipments edge: {json}"
    );
    assert!(
        edges
            .iter()
            .any(|edge| edge["source"] == "checkout" && edge["target"] == "pricing"),
        "instrumented checkout → pricing edge: {json}"
    );
}
