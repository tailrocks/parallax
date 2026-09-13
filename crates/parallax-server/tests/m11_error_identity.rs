#![expect(clippy::too_many_lines, reason = "measured integration scenario")]

//! Error identity (correlated investigation): OTLP resource identity
//! (`cli.invocation.id`, `session.id`, `service.version`, environment) must
//! reach persisted error rows and the GraphQL contracts, and an occurrence
//! must link to its registered run.

#![allow(clippy::expect_used, reason = "test fixture assertions")]

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::resource::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use parallax_server::Config;
use std::time::Duration;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config
}

async fn graphql(client: &reqwest::Client, api_addr: &str, query: String) -> serde_json::Value {
    client
        .post(format!("http://{api_addr}/graphql"))
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .expect("graphql request")
        .json()
        .await
        .expect("graphql json")
}

#[path = "support/harness.rs"]
mod support;

#[tokio::test(flavor = "multi_thread")]
async fn error_identity_flows_from_ingestion_to_api() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let api_addr = handle.api_addr.to_string();
    let grpc_endpoint = format!("http://{}", handle.otlp_grpc_addr);
    let client = reqwest::Client::new();

    // The run must be registered before navigation, mirroring the CLI wrapper.
    let start = graphql(
        &client,
        &api_addr,
        r#"mutation { invocationStart(invocationId: "run-m11", command: "m11 check", startedAtNanos: "1") }"#
            .to_string(),
    )
    .await;
    assert_eq!(
        start["data"]["invocationStart"],
        serde_json::Value::Bool(true),
        "run registration: {start}"
    );

    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint)
        .build()
        .expect("span exporter");
    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("m11-svc")
                .with_attribute(KeyValue::new("service.version", "2.0.0"))
                .with_attribute(KeyValue::new("deployment.environment.name", "prod"))
                .build(),
        )
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("m11-identity");
    let mut span = tracer.start("checkout.authorize");
    span.set_attribute(KeyValue::new("cli.invocation.id", "run-m11"));
    span.set_attribute(KeyValue::new("session.id", "sess-m11"));
    span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.type", "redis::ConnectionTimeout"),
            KeyValue::new(
                "exception.message",
                "timed out connecting to redis://cache-7:6379 after 2000ms",
            ),
        ],
    );
    span.set_status(Status::error("timeout"));
    span.end();
    tracer_provider.force_flush().expect("trace flush");
    tracer_provider.shutdown().expect("tracer shutdown");

    // Ingestion is asynchronous: poll the canonical API until the issue
    // surfaces, then assert the full identity contract in one query.
    let query = r#"{ issues { items { service fingerprint events {
        service invocationId sessionId serviceVersion environment
        invocation { invocationId command }
    } } } }"#;
    let mut events: Vec<serde_json::Value> = Vec::new();
    for _ in 0..100 {
        let response = graphql(&client, &api_addr, query.to_string()).await;
        assert!(
            response["errors"].is_null(),
            "issues query failed: {response}"
        );
        events = response["data"]["issues"]["items"]
            .as_array()
            .expect("items")
            .iter()
            .flat_map(|issue| issue["events"].as_array().cloned().unwrap_or_default())
            .collect();
        if !events.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(events.len(), 1, "one derived error expected: {events:?}");
    let event = &events[0];
    assert_eq!(event["service"], "m11-svc");
    assert_eq!(event["invocationId"], "run-m11");
    assert_eq!(event["sessionId"], "sess-m11");
    assert_eq!(event["serviceVersion"], "2.0.0");
    assert_eq!(event["environment"], "prod");
    assert_eq!(event["invocation"]["invocationId"], "run-m11");
    assert_eq!(event["invocation"]["command"], "m11 check");

    // The service-scoped issue query exposes the same identity.
    let fingerprint = graphql(
        &client,
        &api_addr,
        r#"{ issues { items { service fingerprint } } }"#.to_string(),
    )
    .await;
    let item = &fingerprint["data"]["issues"]["items"][0];
    assert_eq!(item["service"], "m11-svc");
    let service = item["service"].as_str().expect("service");
    let fingerprint = item["fingerprint"].as_str().expect("fingerprint");
    let single = graphql(
        &client,
        &api_addr,
        format!(
            r#"{{ issue(service: "{service}", fingerprint: "{fingerprint}") {{
                service fingerprint
                latestEvent {{ invocationId serviceVersion }}
                trend {{ count }}
            }} }}"#
        ),
    )
    .await;
    assert_eq!(single["data"]["issue"]["service"], "m11-svc");
    assert_eq!(
        single["data"]["issue"]["latestEvent"]["invocationId"],
        "run-m11"
    );
    assert_eq!(
        single["data"]["issue"]["latestEvent"]["serviceVersion"],
        "2.0.0"
    );
}
