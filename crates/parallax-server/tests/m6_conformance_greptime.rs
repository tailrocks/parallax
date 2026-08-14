//! Gated adapter conformance against a real `GreptimeDB` (plan 074).
//!
//! Run with: `cargo nextest run -p parallax-server m6_conformance --run-ignored only`

#![allow(clippy::expect_used, reason = "test fixture assertions")]

use parallax_greptime::GreptimeStore;
use parallax_server::Config;
use parallax_storage::adapter::TelemetryStore;
use parallax_test_support::{builders, conformance};
use prost::Message;
use std::ops::RangeInclusive;

fn make_executable(path: &std::path::Path) -> anyhow::Result<()> {
    let status = std::process::Command::new("chmod")
        .arg("+x")
        .arg(path)
        .status()?;
    anyhow::ensure!(status.success(), "chmod cached engine exited with {status}");
    Ok(())
}

fn seed_cached_engine(tmp: &std::path::Path) {
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

    let data_bin = tmp.join("bin");
    if cache_bin.join("greptime").exists() {
        std::fs::create_dir_all(&data_bin).expect("bin dir");
        std::fs::copy(cache_bin.join("greptime"), data_bin.join("greptime")).expect("seed engine");
        make_executable(&data_bin.join("greptime")).expect("mark cached engine executable");
    }
}

async fn post<M: Message>(addr: std::net::SocketAddr, path: &str, request: &M) {
    let response = reqwest::Client::new()
        .post(format!("http://{addr}/v1/{path}"))
        .header("content-type", "application/x-protobuf")
        .body(request.encode_to_vec())
        .send()
        .await
        .expect("OTLP conformance request");
    assert_eq!(response.status(), 200, "{path}: {response:?}");
}

async fn wait_until_seeded(store: &dyn TelemetryStore, window: RangeInclusive<u128>) {
    let mut seeded = false;
    let mut last_error = None;
    for _ in 0..100 {
        match conformance::assert_seeded(store, "conformance_duration", window.clone()).await {
            Ok(()) => {
                seeded = true;
                break;
            }
            Err(error) => last_error = Some(error),
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(seeded, "seed never became queryable: {last_error:?}");
}

async fn run_seeded_scenarios(store: &dyn TelemetryStore, window: RangeInclusive<u128>) {
    let exp = conformance::SeededExpectations {
        metric_name: "conformance_duration",
        window,
    };
    conformance::trace_search_scenario(store, &exp)
        .await
        .expect("engine trace_search");
    conformance::log_count_series_scenario(store, &exp)
        .await
        .expect("engine log_count_series");
    conformance::overview_totals_scenario(store, &exp)
        .await
        .expect("engine overview_totals");
    conformance::attribute_compare_scenario(store, &exp)
        .await
        .expect("engine attribute_compare");
    conformance::service_map_scenario(store, &exp)
        .await
        .expect("engine service_map");
}

/// Same SQL through JSON and Arrow HTTP transports must decode to equal rows.
async fn assert_arrow_json_transport_parity() {
    let store = GreptimeStore::connect("http://127.0.0.1:24000", "7d", "7d", "7d")
        .await
        .expect("connect managed engine HTTP");
    let sql = r#"SELECT "trace_id" FROM opentelemetry_traces LIMIT 8"#;
    let json = store.sql(sql).await.expect("json transport");
    let arrow = store.sql_arrow(sql).await.expect("arrow transport");
    assert_eq!(json, arrow, "arrow vs json decoded rows");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn greptime_conformance_scenarios() {
    let _subscriber_already_installed = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=info")
        .try_init();

    let tmp = tempfile::tempdir().expect("tempdir");
    seed_cached_engine(tmp.path());

    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.mode = "managed".to_string();
    config.storage.data_dir = tmp.path().to_string_lossy().into_owned();

    let handle = parallax_server::start(&config)
        .await
        .expect("managed server starts");
    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let window = start..=(start + 10_000_000_000);
    conformance::assert_empty(handle.store.as_ref(), window.clone())
        .await
        .expect("fresh engine empty-window conformance");

    let start = u64::try_from(start).expect("fixture timestamp");
    post(
        handle.otlp_http_addr,
        "traces",
        &builders::conformance_traces(conformance::SERVICE, start),
    )
    .await;
    post(
        handle.otlp_http_addr,
        "logs",
        &builders::conformance_logs(conformance::SERVICE, start),
    )
    .await;
    post(
        handle.otlp_http_addr,
        "metrics",
        &builders::conformance_metrics(conformance::SERVICE, start),
    )
    .await;

    wait_until_seeded(handle.store.as_ref(), window.clone()).await;
    run_seeded_scenarios(handle.store.as_ref(), window.clone()).await;
    assert_arrow_json_transport_parity().await;

    handle
        .shutdown_graceful()
        .await
        .expect("managed server stops before restart");
    let restarted = parallax_server::start(&config)
        .await
        .expect("managed server restarts");
    conformance::assert_seeded(restarted.store.as_ref(), "conformance_duration", window)
        .await
        .expect("restarted engine retains conformance seed");
    restarted
        .shutdown_graceful()
        .await
        .expect("restarted managed server stops");
}
