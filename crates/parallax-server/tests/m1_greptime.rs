//! Gated M1 acceptance for the managed-engine path: downloads (once) and
//! supervises a real GreptimeDB standalone child, bootstraps the DDL, and
//! round-trips telemetry through the GreptimeStore adapter.
//!
//! Run with: `cargo test -p parallax-server --test m1_greptime -- --ignored`
//! The binary is cached under target/greptime-test-bin/ across runs.

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn managed_engine_roundtrip() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=debug")
        .try_init();
    // Cache the engine binary across test runs (and reuse an existing
    // ~/.parallax/bin install when present) to avoid re-downloading 140MB.
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
    // Pre-seed the data dir's bin/ with the cached binary if we have one.
    let data_bin = tmp.path().join("bin");
    if cache_bin.join("greptime").exists() {
        std::fs::create_dir_all(&data_bin).expect("bin dir");
        std::fs::copy(cache_bin.join("greptime"), data_bin.join("greptime")).expect("seed engine");
        let _ = std::process::Command::new("chmod")
            .arg("+x")
            .arg(data_bin.join("greptime"))
            .status();
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

    // Cache the downloaded binary for the next run.
    if !cache_bin.join("greptime").exists() && data_bin.join("greptime").exists() {
        std::fs::create_dir_all(&cache_bin).expect("cache dir");
        let _ = std::fs::copy(data_bin.join("greptime"), cache_bin.join("greptime"));
    }

    // Real SDK export → worker → GreptimeDB.
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("m1-greptime");
    let mut span = tracer.start("engine.roundtrip");
    let trace_id = format!("{:032x}", span.span_context().trace_id());
    span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.type", "test::EngineRoundtrip"),
            KeyValue::new("exception.message", "stored in a real engine"),
        ],
    );
    span.set_status(Status::error("boom"));
    span.end();
    tracer_provider.force_flush().expect("flush");

    // Poll the adapter until the row is queryable in GreptimeDB.
    let mut spans = Vec::new();
    for _ in 0..100 {
        spans = handle
            .store
            .spans_by_trace(&trace_id)
            .await
            .expect("query engine");
        if !spans.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(spans.len(), 1, "span must be readable from GreptimeDB");
    assert_eq!(spans[0].name, "engine.roundtrip");
    assert_eq!(spans[0].status_code, "STATUS_CODE_ERROR");

    // The issue is derived and persisted to the metadata store on the same
    // worker pass that forwards the span, but the two stores are independent —
    // the span can become queryable in GreptimeDB a beat before the Turso
    // upsert is visible. Poll for the grouped issue rather than assuming the
    // span's visibility implies it.
    let mut issues = Vec::new();
    for _ in 0..100 {
        issues = handle.metadata.issues(10).await.expect("issues");
        if issues
            .iter()
            .any(|i| i.error_type == "test::EngineRoundtrip")
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        issues
            .iter()
            .any(|i| i.error_type == "test::EngineRoundtrip"),
        "issue grouped from engine-backed pipeline: {issues:?}"
    );

    handle.shutdown();
}
