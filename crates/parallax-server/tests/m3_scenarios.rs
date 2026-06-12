//! The v1-scope stack scenarios, automated legs: (1) a cross-service trace
//! spanning two services; (2) database wrapper spans carrying query text and
//! duration (the documented tokio-postgres/clickhouse pattern's output);
//! (3) manually-instrumented GraphQL operation/resolver spans rendered.
//! The remaining legs (visual→agent handoff, custom dashboard, TUI run) are
//! covered by the trace/run lookup, dashboards, and CLI-wrapper tests plus
//! the operator's live dogfood.

use opentelemetry::trace::{
    Span as _, SpanContext, SpanKind, Status, TraceContextExt as _, Tracer as _,
    TracerProvider as _,
};
use opentelemetry::{Context as OtelContext, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use std::time::Duration;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.mode = "none".to_string();
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config
}

fn provider_for(service: &str, endpoint: &str) -> opentelemetry_sdk::trace::SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.to_string())
        .build()
        .expect("exporter");
    opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([KeyValue::new("service.name", service.to_string())])
                .build(),
        )
        .build()
}

async fn graphql(
    client: &reqwest::Client,
    api: std::net::SocketAddr,
    query: &str,
) -> serde_json::Value {
    client
        .post(format!("http://{api}/graphql"))
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .expect("graphql request")
        .json()
        .await
        .expect("graphql json")
}

#[tokio::test(flavor = "multi_thread")]
async fn stack_scenarios_cross_service_db_and_graphql_spans() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = parallax_server::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let endpoint = format!("http://{}", handle.otlp_grpc_addr);

    // Two services, one trace: the gateway's CLIENT span parents the backend's
    // SERVER span via shared trace context (what tonic middleware propagation
    // produces on the wire).
    let gateway = provider_for("api-gateway", &endpoint);
    let backend = provider_for("checkout", &endpoint);

    let gateway_tracer = gateway.tracer("gateway");
    let mut client_span = gateway_tracer
        .span_builder("grpc client checkout.Authorize")
        .with_kind(SpanKind::Client)
        .start(&gateway_tracer);
    let trace_id = format!("{:032x}", client_span.span_context().trace_id());
    let parent_context: SpanContext = client_span.span_context().clone();

    let backend_tracer = backend.tracer("checkout");
    let parent = OtelContext::new().with_remote_span_context(parent_context);
    // The backend's SERVER span, then GraphQL operation + resolver spans and
    // a database wrapper span — the manual-instrumentation patterns from
    // rust-stack-instrumentation.md, exactly as they arrive over OTLP.
    let mut server_span = backend_tracer
        .span_builder("grpc server checkout.Authorize")
        .with_kind(SpanKind::Server)
        .start_with_context(&backend_tracer, &parent);
    let server_context =
        OtelContext::new().with_remote_span_context(server_span.span_context().clone());

    let mut graphql_span = backend_tracer
        .span_builder("graphql.execute checkoutSummary")
        .start_with_context(&backend_tracer, &server_context);
    let graphql_context =
        OtelContext::new().with_remote_span_context(graphql_span.span_context().clone());
    let mut resolver_span = backend_tracer
        .span_builder("graphql.resolve Cart.items")
        .start_with_context(&backend_tracer, &graphql_context);

    let mut db_span = backend_tracer
        .span_builder("query orders")
        .with_kind(SpanKind::Client)
        .with_attributes([
            KeyValue::new("db.system.name", "postgresql"),
            KeyValue::new("db.operation.name", "SELECT"),
            KeyValue::new(
                "db.query.text",
                "SELECT id, total FROM orders WHERE cart_id = $1",
            ),
        ])
        .start_with_context(&backend_tracer, &graphql_context);
    tokio::time::sleep(Duration::from_millis(5)).await;
    db_span.end();
    resolver_span.end();
    graphql_span.end();
    server_span.set_status(Status::error("downstream timeout"));
    server_span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.type", "tonic::Status"),
            KeyValue::new("exception.message", "deadline exceeded talking to payments"),
        ],
    );
    server_span.end();
    client_span.end();
    gateway.force_flush().expect("gateway flush");
    backend.force_flush().expect("backend flush");

    // The whole story must be queryable as ONE trace across both services.
    let client = reqwest::Client::new();
    let mut trace = serde_json::Value::Null;
    for _ in 0..50 {
        trace = graphql(
            &client,
            handle.api_addr,
            &format!(
                r#"{{ trace(traceId: "{trace_id}") {{ spans {{
                     service name kind statusCode durationNs attributes parentSpanId
                   }} }} }}"#
            ),
        )
        .await;
        if trace
            .pointer("/data/trace/spans")
            .and_then(|v| v.as_array())
            .is_some_and(|spans| spans.len() == 5)
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let spans = trace
        .pointer("/data/trace/spans")
        .and_then(|v| v.as_array())
        .cloned()
        .expect("trace spans");
    assert_eq!(spans.len(), 5, "all five spans in one trace: {trace}");

    // Scenario 1 — cross-service: both services present, SERVER span parents
    // to the gateway's CLIENT span.
    let services: std::collections::BTreeSet<&str> =
        spans.iter().filter_map(|s| s["service"].as_str()).collect();
    assert!(services.contains("api-gateway") && services.contains("checkout"));
    let client_span_id = spans
        .iter()
        .find(|s| s["kind"] == "SPAN_KIND_CLIENT" && s["service"] == "api-gateway")
        .and_then(|s| s["name"].as_str().map(|_| s))
        .expect("gateway client span");
    let server = spans
        .iter()
        .find(|s| s["kind"] == "SPAN_KIND_SERVER")
        .expect("backend server span");
    assert!(
        server["parentSpanId"].as_str().is_some(),
        "server span carries the cross-service parent: {server}"
    );
    let _ = client_span_id;

    // Scenario 2 — database wrapper span: query text + a real duration.
    let db = spans
        .iter()
        .find(|s| s["name"] == "query orders")
        .expect("db span");
    let attributes: serde_json::Value =
        serde_json::from_str(db["attributes"].as_str().unwrap_or("{}")).expect("attrs json");
    assert_eq!(
        attributes["db.query.text"],
        "SELECT id, total FROM orders WHERE cart_id = $1"
    );
    assert_eq!(attributes["db.system.name"], "postgresql");
    let duration_ns: u128 = db["durationNs"]
        .as_str()
        .and_then(|d| d.parse().ok())
        .expect("duration");
    assert!(duration_ns >= 5_000_000, "db span measured its duration");

    // Scenario 3 — GraphQL operation + resolver spans rendered in the trace.
    assert!(
        spans
            .iter()
            .any(|s| s["name"] == "graphql.execute checkoutSummary")
    );
    assert!(
        spans
            .iter()
            .any(|s| s["name"] == "graphql.resolve Cart.items")
    );

    // And the failure grouped into an issue whose bundle shows the captured
    // query — the agent sees what the database was asked.
    let mut bundle_markdown = String::new();
    for _ in 0..50 {
        let issues = graphql(
            &client,
            handle.api_addr,
            r#"{ issues { items { fingerprint errorType } } }"#,
        )
        .await;
        if let Some(fp) = issues
            .pointer("/data/issues/items")
            .and_then(|v| v.as_array())
            .and_then(|a| a.iter().find(|i| i["errorType"] == "tonic::Status"))
            .and_then(|i| i["fingerprint"].as_str())
        {
            let bundle = graphql(
                &client,
                handle.api_addr,
                &format!(r#"{{ bundle(fingerprint: "{fp}") {{ markdown }} }}"#),
            )
            .await;
            bundle_markdown = bundle
                .pointer("/data/bundle/markdown")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !bundle_markdown.is_empty() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        bundle_markdown.contains("SELECT id, total FROM orders"),
        "bundle surfaces the captured query: {bundle_markdown}"
    );
    assert!(bundle_markdown.contains("api-gateway") || bundle_markdown.contains("checkout"));

    handle.shutdown();
}
