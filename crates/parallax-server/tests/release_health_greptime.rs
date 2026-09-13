//! R5 release health (2026-09-14): `releaseHealth` must resolve against a
//! real engine. A healthy v1 and a crashing v2 are ingested as live OTLP
//! (resource `service.version`, span `session.id`/`user.id`); the test
//! asserts per-release crash-free session/user rates and the suspect flag
//! through the exact GraphQL path the service-detail page uses.
//!
//! Run with: `cargo nextest run -p parallax-server --locked --test release_health_greptime --run-ignored all`
//! The binary is cached under target/greptime-test-bin/ across runs.

#![allow(clippy::expect_used, reason = "test fixture assertions")]

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::resource::Resource;
use parallax_server::Config;
use std::time::Duration;

const SERVICE: &str = "release-health-greptime";

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

fn emit_release(otlp_grpc_addr: &std::net::SocketAddr, version: &str, crash_first: bool) {
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{otlp_grpc_addr}"))
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(SERVICE)
                .with_attribute(KeyValue::new("service.version", version.to_string()))
                .build(),
        )
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("release-health");
    for (index, session, user) in [
        (0, format!("{version}-s1"), format!("{version}-u1")),
        (1, format!("{version}-s2"), format!("{version}-u2")),
    ] {
        let mut span = tracer.start("checkout.authorize");
        span.set_attribute(KeyValue::new("session.id", session));
        span.set_attribute(KeyValue::new("user.id", user));
        if crash_first && index == 0 {
            span.add_event(
                "exception",
                vec![
                    KeyValue::new("exception.type", "test::ReleaseHealthLive"),
                    KeyValue::new("exception.message", "v2 crashes"),
                ],
            );
            span.set_status(Status::error("boom"));
        }
        span.end();
    }
    tracer_provider.force_flush().expect("flush");
    tracer_provider.shutdown().expect("tracer shutdown");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn release_health_resolves_against_managed_engine() {
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

    emit_release(&handle.otlp_grpc_addr, "v1", false);
    emit_release(&handle.otlp_grpc_addr, "v2", true);

    let client = reqwest::Client::new();
    let query = format!(
        r#"{{ releaseHealth(service: "{SERVICE}", fromNanos: "0", toNanos: "340282366920938463463374607431768211455") {{
            version spanCount sessionCount crashedSessionCount crashFreeSessionRate
            userCount crashedUserCount crashFreeUserRate errorCount suspectRelease
        }} }}"#
    );
    let mut health = serde_json::Value::Null;
    for _ in 0..300 {
        let response = graphql(&client, &api_addr, query.clone()).await;
        assert!(
            response["errors"].is_null(),
            "releaseHealth query failed: {response}"
        );
        let rows = response["data"]["releaseHealth"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let settled = rows.len() == 2
            && rows.iter().any(|row| {
                row["version"] == "v2" && row["errorCount"] != "0" && !row["errorCount"].is_null()
            });
        if settled {
            health = serde_json::Value::Array(rows);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let rows = health.as_array().expect("two releases with v2 errors");
    assert_eq!(rows.len(), 2, "healthy v1 + crashing v2: {rows:?}");

    let v1 = rows
        .iter()
        .find(|row| row["version"] == "v1")
        .expect("v1 row");
    assert_eq!(v1["sessionCount"], "2", "{v1}");
    assert_eq!(v1["crashedSessionCount"], "0", "{v1}");
    assert_eq!(v1["crashFreeSessionRate"], 1.0, "{v1}");
    assert_eq!(v1["userCount"], "2", "{v1}");
    assert_eq!(v1["crashedUserCount"], "0", "{v1}");
    assert_eq!(v1["crashFreeUserRate"], 1.0, "{v1}");
    assert_eq!(v1["suspectRelease"], false, "{v1}");

    let v2 = rows
        .iter()
        .find(|row| row["version"] == "v2")
        .expect("v2 row");
    assert_eq!(v2["sessionCount"], "2", "{v2}");
    assert_eq!(v2["crashedSessionCount"], "1", "{v2}");
    assert_eq!(v2["crashFreeSessionRate"], 0.5, "{v2}");
    assert_eq!(v2["userCount"], "2", "{v2}");
    assert_eq!(v2["crashedUserCount"], "1", "{v2}");
    assert_eq!(v2["crashFreeUserRate"], 0.5, "{v2}");
    assert_eq!(v2["suspectRelease"], true, "v2 crash regresses vs v1: {v2}");

    handle.shutdown();
}
