//! P0 regression (2026-09-13): `issue.events` must resolve against a real
//! engine. The ranked multi-key SQL broke live (`No field named ts`) while
//! mocked/memory-backed contract tests stayed green, so this test drives the
//! exact GraphQL path the issue-detail page uses through managed GreptimeDB.
//!
//! Run with: `cargo nextest run -p parallax-server --locked --test issue_events_greptime --run-ignored all`
//! The binary is cached under target/greptime-test-bin/ across runs.

#![expect(clippy::too_many_lines, reason = "measured integration scenario")]
#![allow(clippy::expect_used, reason = "test fixture assertions")]

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::resource::Resource;
use parallax_server::Config;
use std::time::Duration;

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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn issue_events_resolve_against_managed_engine() {
    let cache_bin = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("greptime-bin");
    let home_bin = std::env::home_dir()
        .map(|h| h.join(".parallax/bin/greptime"))
        .filter(|p| p.exists());
    if let Some(existing) = home_bin
        && !cache_bin.join("greptime").exists()
    {
        std::fs::create_dir_all(&cache_bin).expect("cache dir");
        std::fs::copy(existing, cache_bin.join("greptime")).expect("copy cached engine");
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    let data_bin = tmp.path().join("bin");
    if cache_bin.join("greptime").exists() {
        std::fs::create_dir_all(&data_bin).expect("bin dir");
        std::fs::copy(cache_bin.join("greptime"), data_bin.join("greptime")).expect("seed engine");
    }

    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.mode = "managed".to_string();
    config.storage.data_dir = tmp.path().to_string_lossy().into_owned();

    let handle = parallax_server::start(&config)
        .await
        .expect("managed server starts");
    let api_addr = handle.api_addr.to_string();

    if !cache_bin.join("greptime").exists() && data_bin.join("greptime").exists() {
        std::fs::create_dir_all(&cache_bin).expect("cache dir");
        std::fs::copy(data_bin.join("greptime"), cache_bin.join("greptime"))
            .expect("cache downloaded engine");
    }

    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("issue-events-greptime")
                .build(),
        )
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("issue-events");
    let mut span = tracer.start("checkout.authorize");
    span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.type", "test::IssueEventsLive"),
            KeyValue::new("exception.message", "ranked read must resolve"),
        ],
    );
    span.set_status(Status::error("boom"));
    span.end();
    tracer_provider.force_flush().expect("flush");
    tracer_provider.shutdown().expect("tracer shutdown");

    let client = reqwest::Client::new();
    let mut fingerprint = String::new();
    for _ in 0..300 {
        let response = graphql(
            &client,
            &api_addr,
            r#"{ issues { items { service fingerprint } } }"#.to_string(),
        )
        .await;
        assert!(
            response["errors"].is_null(),
            "issues query failed: {response}"
        );
        let found = response["data"]["issues"]["items"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item["fingerprint"].as_str())
            .filter(|found| !found.is_empty());
        if let Some(found) = found {
            fingerprint = found.to_string();
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(!fingerprint.is_empty(), "issue grouped from live pipeline");

    // The exact IssueDetail path: multi-key ranked read over GreptimeDB.
    let detail = graphql(
        &client,
        &api_addr,
        format!(
            r#"{{ issue(service: "issue-events-greptime", fingerprint: "{fingerprint}") {{
                service fingerprint
                events(limit: 5) {{ tsNanos service fingerprint errorType message }}
            }} }}"#
        ),
    )
    .await;
    assert!(
        detail["errors"].is_null(),
        "issue.events must resolve live: {detail}"
    );
    let events = detail["data"]["issue"]["events"]
        .as_array()
        .expect("events array");
    assert_eq!(events.len(), 1, "one occurrence expected: {detail}");
    assert_eq!(events[0]["service"], "issue-events-greptime");
    assert_eq!(events[0]["errorType"], "test::IssueEventsLive");

    handle.shutdown();
}
