//! X1 acceptance: multi-environment error telemetry groups into one issue
//! while the environment filter and per-env rollup stay exact end to end —
//! metadata predicate, store predicate, and the GraphQL surface.

#![allow(clippy::expect_used, reason = "test fixture assertions")]
#![expect(clippy::too_many_lines, reason = "measured integration scenario")]

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::resource::Resource;
use parallax_server::Config;
use parallax_storage::model::{IssueQuery, IssueSortKey};
use std::time::Duration;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config
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

/// Emit `count` identical exception spans from one deployment environment
/// (`None` = no environment on the resource).
fn emit_exceptions(grpc_endpoint: &str, environment: Option<&str>, count: usize) {
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint.to_string())
        .build()
        .expect("span exporter");
    let mut resource = Resource::builder().with_service_name("checkout-x1");
    if let Some(environment) = environment {
        resource = resource.with_attributes([KeyValue::new(
            "deployment.environment.name",
            environment.to_string(),
        )]);
    }
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource.build())
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("x1-test");
    for _ in 0..count {
        let mut span = tracer.start("payment.authorize");
        span.add_event(
            "exception",
            vec![
                KeyValue::new("exception.type", "redis::ConnectionTimeout"),
                KeyValue::new(
                    "exception.message",
                    "timed out connecting to redis://cache-7:6379",
                ),
                KeyValue::new(
                    "exception.stacktrace",
                    "checkout::payment::authorize at src/payment.rs:184",
                ),
            ],
        );
        span.set_status(Status::error("connection timed out"));
        span.end();
    }
    tracer_provider.force_flush().expect("trace flush");
    tracer_provider.shutdown().expect("tracer shutdown");
}

#[tokio::test(flavor = "multi_thread")]
async fn environment_filter_and_rollup_cover_multi_env_issue() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let grpc_endpoint = format!("http://{}", handle.otlp_grpc_addr);

    // Same service + same exception shape in two environments: one issue,
    // three occurrences. A fourth occurrence carries no environment.
    emit_exceptions(&grpc_endpoint, Some("production"), 2);
    emit_exceptions(&grpc_endpoint, Some("staging"), 1);
    emit_exceptions(&grpc_endpoint, None, 1);

    let mut issues = Vec::new();
    for _ in 0..100 {
        issues = handle.metadata.issues(10).await.expect("issues query");
        if issues
            .iter()
            .any(|i| i.service == "checkout-x1" && i.event_count == 4)
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let issue = issues
        .iter()
        .find(|i| i.service == "checkout-x1")
        .expect("multi-env issue grouped");
    assert_eq!(issue.event_count, 4, "all occurrences grouped: {issue:?}");

    // Store-side error events land after the metadata upsert; poll until
    // the full occurrence set is readable.
    for _ in 0..100 {
        let events = handle
            .store
            .error_events_by_fingerprint(
                &issue.service,
                &issue.fingerprint,
                0..=u128::MAX,
                10,
                None,
            )
            .await
            .expect("error events read");
        if events.len() == 4 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Rollup: only the three environment-carrying occurrences count.
    let rollup: std::collections::BTreeMap<String, u64> =
        serde_json::from_str(&issue.environments).expect("environments json");
    assert_eq!(
        rollup,
        std::collections::BTreeMap::from([
            ("production".to_string(), 2),
            ("staging".to_string(), 1),
        ]),
        "per-env rollup: {rollup:?}"
    );

    // Metadata predicate.
    for (environment, expected) in [("production", 1), ("staging", 1), ("qa", 0)] {
        let (_, total) = handle
            .metadata
            .issues_filtered(
                &IssueQuery {
                    environment: Some(environment.to_string()),
                    ..Default::default()
                },
                IssueSortKey::LastSeen,
                10,
                0,
            )
            .await
            .expect("filtered issues");
        assert_eq!(total, expected, "issues_filtered({environment})");
    }

    // Store predicate.
    for (environment, expected) in [
        (None, 4),
        (Some("production"), 2),
        (Some("staging"), 1),
        (Some("qa"), 0),
    ] {
        let events = handle
            .store
            .error_events_by_fingerprint(
                &issue.service,
                &issue.fingerprint,
                0..=u128::MAX,
                10,
                environment,
            )
            .await
            .expect("filtered error events");
        assert_eq!(events.len(), expected, "store filter({environment:?})");
        if environment.is_some() {
            assert!(
                events
                    .iter()
                    .all(|event| event.environment.as_deref() == environment),
                "store filter({environment:?}): {events:?}"
            );
        }
    }

    // GraphQL surface end to end.
    let client = reqwest::Client::new();
    let api = handle.api_addr;
    let filtered = graphql(
        &client,
        api,
        r#"{ issues(environment: "production") { total } }"#,
    )
    .await;
    assert_eq!(
        filtered["data"]["issues"]["total"], 1,
        "graphql issues(environment): {filtered}"
    );
    let absent = graphql(&client, api, r#"{ issues(environment: "qa") { total } }"#).await;
    assert_eq!(
        absent["data"]["issues"]["total"], 0,
        "graphql issues(absent env): {absent}"
    );
    let detail = graphql(
        &client,
        api,
        &format!(
            r#"{{ issue(service: "checkout-x1", fingerprint: "{}") {{ environmentCounts {{ environment count }} events(environment: "staging") {{ environment }} }} }}"#,
            issue.fingerprint,
        ),
    )
    .await;
    assert_eq!(
        detail["data"]["issue"]["environmentCounts"],
        serde_json::json!([
            { "environment": "production", "count": 2 },
            { "environment": "staging", "count": 1 },
        ]),
        "graphql rollup: {detail}"
    );
    assert_eq!(
        detail["data"]["issue"]["events"],
        serde_json::json!([{ "environment": "staging" }]),
        "graphql events(environment): {detail}"
    );

    handle.shutdown();
}

#[path = "support/harness.rs"]
mod support;
