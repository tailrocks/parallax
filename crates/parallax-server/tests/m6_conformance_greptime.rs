//! Gated adapter conformance against a real `GreptimeDB` (plan 074).
//!
//! Run with: `cargo nextest run -p parallax-server m6_conformance --run-ignored only`

#![allow(clippy::expect_used, reason = "test fixture assertions")]

use parallax_server::Config;
use parallax_test_support::{builders, conformance};
use prost::Message;

fn make_executable(path: &std::path::Path) -> anyhow::Result<()> {
    let status = std::process::Command::new("chmod")
        .arg("+x")
        .arg(path)
        .status()?;
    anyhow::ensure!(status.success(), "chmod cached engine exited with {status}");
    Ok(())
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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn greptime_conformance_scenarios() {
    let _subscriber_already_installed = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=info")
        .try_init();

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
        make_executable(&data_bin.join("greptime")).expect("mark cached engine executable");
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

    let mut seeded = false;
    let mut last_error = None;
    for _ in 0..100 {
        match conformance::assert_seeded(
            handle.store.as_ref(),
            "conformance_duration",
            window.clone(),
        )
        .await
        {
            Ok(()) => {
                seeded = true;
                break;
            }
            Err(error) => last_error = Some(error),
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(seeded, "seed never became queryable: {last_error:?}");

    handle.shutdown_graceful().await;
    let restarted = parallax_server::start(&config)
        .await
        .expect("managed server restarts");
    conformance::assert_seeded(restarted.store.as_ref(), "conformance_duration", window)
        .await
        .expect("restarted engine retains conformance seed");
    restarted.shutdown_graceful().await;
}
