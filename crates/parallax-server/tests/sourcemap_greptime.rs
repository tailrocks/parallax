//! R1 live proof: minified JS stacks resolve through the artifact store
//! on the real path. Uploads a source map via the GraphQL mutation, ingests
//! OTLP spans carrying a minified `exception.stacktrace` for two releases,
//! then asserts the exact issue-detail read (`events { mappedFrames }`)
//! resolves the release with the artifact and leaves the other unresolved.
//!
//! Run with: `cargo nextest run -p parallax-server --locked --test sourcemap_greptime --run-ignored all`
//! The binary is cached under target/greptime-test-bin/ across runs.

#![allow(clippy::expect_used, reason = "test fixture assertions")]

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::resource::Resource;
use parallax_server::Config;
use std::time::Duration;

const MINIFIED_STACK: &str = "Error: boom\n    at onClick (https://cdn.example.com/app.min.js:1:25)\n    at https://cdn.example.com/app.min.js:1:10";

fn fixture_map() -> String {
    serde_json::json!({
        "version": 3,
        "file": "app.min.js",
        "sources": ["src/app.ts"],
        "names": ["onClick", "render"],
        "mappings": "AAAA,UAAIA,eAEFC"
    })
    .to_string()
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

fn emit_minified_error(otlp_grpc_addr: &std::net::SocketAddr, version: &str) {
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{otlp_grpc_addr}"))
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("sourcemap-web")
                .with_attribute(KeyValue::new("service.version", version.to_string()))
                .build(),
        )
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("sourcemap-live");
    let mut span = tracer.start("app.onClick");
    span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.type", "Error"),
            KeyValue::new("exception.message", "boom"),
            KeyValue::new("exception.stacktrace", MINIFIED_STACK),
        ],
    );
    span.set_status(Status::error("boom"));
    span.end();
    tracer_provider.force_flush().expect("flush");
    tracer_provider.shutdown().expect("tracer shutdown");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn sourcemap_resolves_live_at_issue_detail() {
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

    let client = reqwest::Client::new();

    // Upload the artifact for release 1.2.3 only; 9.9.9 stays unmapped.
    let map_json = serde_json::to_string(&fixture_map()).expect("escape map");
    let upload = graphql(
        &client,
        &api_addr,
        format!(
            r#"mutation {{ sourceMapUpload(service: "sourcemap-web", version: "1.2.3", file: "app.min.js", map: {map_json}) {{ service version file mapSha256 }} }}"#
        ),
    )
    .await;
    assert!(upload["errors"].is_null(), "upload failed: {upload}");
    assert_eq!(upload["data"]["sourceMapUpload"]["file"], "app.min.js");

    // The artifact inventory exposes metadata only — never map content.
    let listed = graphql(
        &client,
        &api_addr,
        r#"{ sourceMaps(service: "sourcemap-web", version: "1.2.3") { file mapBytes mapSha256 } }"#
            .to_string(),
    )
    .await;
    assert!(listed["errors"].is_null(), "list failed: {listed}");
    assert_eq!(
        listed["data"]["sourceMaps"].as_array().map(Vec::len),
        Some(1)
    );

    emit_minified_error(&handle.otlp_grpc_addr, "1.2.3");
    emit_minified_error(&handle.otlp_grpc_addr, "9.9.9");

    // Both releases share one fingerprint; poll until both events surface.
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

    // The exact issue-detail path: per-event frame resolution.
    let versioned = poll_versioned_events(&client, &api_addr, &fingerprint).await;
    assert_eq!(versioned.len(), 2, "both releases ingested");

    assert_versioned_frames(&versioned);

    handle.shutdown();
}

fn split_versioned(events: &[serde_json::Value]) -> Vec<(String, Vec<serde_json::Value>)> {
    events
        .iter()
        .map(|event| {
            (
                event["serviceVersion"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                event["mappedFrames"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default(),
            )
        })
        .collect()
}

async fn poll_versioned_events(
    client: &reqwest::Client,
    api_addr: &str,
    fingerprint: &str,
) -> Vec<(String, Vec<serde_json::Value>)> {
    for _ in 0..300 {
        let detail = graphql(
            client,
            api_addr,
            format!(
                r#"{{ issue(service: "sourcemap-web", fingerprint: "{fingerprint}") {{
                    events(limit: 10) {{
                        serviceVersion
                        mappedFrames {{ resolved source sourceLine sourceColumn name }}
                    }}
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
        if events.len() == 2 {
            return split_versioned(events);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Vec::new()
}

fn assert_versioned_frames(versioned: &[(String, Vec<serde_json::Value>)]) {
    for (version, frames) in versioned {
        assert_eq!(frames.len(), 2, "two V8 frames in {version}");
        if version == "1.2.3" {
            assert!(
                frames.iter().all(|frame| frame["resolved"] == true),
                "mapped release resolves: {frames:?}"
            );
            assert_eq!(frames[0]["source"], "src/app.ts");
            assert_eq!(frames[0]["sourceLine"], 3);
            assert_eq!(frames[0]["sourceColumn"], 2);
            assert_eq!(frames[0]["name"], "render");
            assert_eq!(frames[1]["name"], "onClick");
        } else {
            assert_eq!(version, "9.9.9");
            assert!(
                frames.iter().all(|frame| frame["resolved"] == false),
                "unmapped release stays unresolved: {frames:?}"
            );
        }
    }
}
