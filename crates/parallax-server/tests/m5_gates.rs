//! Gated M5 gate measurements on the real managed engine: ingest-to-queryable
//! p95 and warm bundle-assembly latency, printed for the gates report
//! (docs/research/architecture/v1-gates-report.md).
//!
//! Run with: `cargo test -p parallax-server --test m5_gates -- --ignored --nocapture`

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use std::time::{Duration, Instant};

fn percentile(sorted_millis: &[u128], p: f64) -> u128 {
    let rank = ((sorted_millis.len() as f64 - 1.0) * p).round() as usize;
    sorted_millis[rank.min(sorted_millis.len() - 1)]
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "runs a real GreptimeDB; run with --ignored --nocapture"]
async fn measure_m5_gates() {
    // Engine binary cache handling identical to m1_greptime.
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

    let warm_start = Instant::now();
    let handle = parallax_server::start(&config)
        .await
        .expect("server starts");
    let warm_start_millis = warm_start.elapsed().as_millis();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("exporter");
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([KeyValue::new("service.name", "gate-bench")])
                .build(),
        )
        .build();
    let tracer = provider.tracer("m5");
    let client = reqwest::Client::new();

    // Gate: ingest-to-queryable ≤ 5s p95. Twenty error spans, each timed
    // from "the app hands the batch to the exporter" (flush) to "the trace
    // answers over GraphQL".
    let mut latencies_millis = Vec::new();
    for i in 0..20 {
        let mut span = tracer.start(format!("gate.iteration.{i}"));
        let trace_id = format!("{:032x}", span.span_context().trace_id());
        span.add_event(
            "exception",
            vec![
                KeyValue::new("exception.type", "gate::Timeout"),
                KeyValue::new(
                    "exception.message",
                    format!("simulated timeout after {}ms", 100 + i),
                ),
                KeyValue::new(
                    "exception.stacktrace",
                    "gate::bench::run at src/bench.rs:42",
                ),
            ],
        );
        span.set_status(Status::error("gate"));
        span.end();
        let t0 = Instant::now();
        provider.force_flush().expect("flush");
        loop {
            let response: serde_json::Value = client
                .post(format!("http://{}/graphql", handle.api_addr))
                .json(&serde_json::json!({"query": format!(
                    r#"{{ trace(traceId: "{trace_id}") {{ spans {{ name }} }} }}"#
                )}))
                .send()
                .await
                .expect("trace request")
                .json()
                .await
                .expect("trace json");
            if response
                .pointer("/data/trace/spans")
                .is_some_and(|s| !s.is_null())
            {
                break;
            }
            assert!(
                t0.elapsed() < Duration::from_secs(30),
                "iteration {i} not queryable within 30s"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        latencies_millis.push(t0.elapsed().as_millis());
    }
    latencies_millis.sort_unstable();

    // Gate: a real panic becomes a grouped issue in ~5 s. The documented
    // capture path (capture/rust.md): a panic hook emits an OTLP log record
    // carrying exception.* attributes; derivation groups it. catch_unwind
    // keeps the harness alive — the panic itself is real.
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("log exporter");
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([opentelemetry::KeyValue::new("service.name", "gate-bench")])
                .build(),
        )
        .build();
    {
        use opentelemetry::logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _};
        let logger = logger_provider.logger("panic-hook");
        let panic_info = std::panic::catch_unwind(|| {
            panic!("checkout total overflowed at row 4242");
        })
        .expect_err("the panic is real");
        let message = panic_info
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| panic_info.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic".to_string());
        let mut record = logger.create_log_record();
        record.set_severity_number(opentelemetry::logs::Severity::Fatal);
        record.set_severity_text("FATAL");
        record.set_body(AnyValue::from(message.clone()));
        record.add_attribute("exception.type", "panic");
        record.add_attribute("exception.message", message);
        record.add_attribute(
            "exception.stacktrace",
            "gate::bench::checkout at src/bench.rs:99",
        );
        logger.emit(record);
    }
    let panic_t0 = Instant::now();
    logger_provider.force_flush().expect("log flush");
    let panic_visible_millis = loop {
        let response: serde_json::Value = client
            .post(format!("http://{}/graphql", handle.api_addr))
            .json(&serde_json::json!({"query": "{ issues { errorType } }"}))
            .send()
            .await
            .expect("issues request")
            .json()
            .await
            .expect("issues json");
        if response
            .pointer("/data/issues")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|i| i["errorType"] == "panic"))
        {
            break panic_t0.elapsed().as_millis();
        }
        assert!(
            panic_t0.elapsed() < Duration::from_secs(30),
            "panic issue not grouped within 30s"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    };

    // Gate: bundle assembly ≤ 300ms warm. First call warms; then ten timed.
    let issues: serde_json::Value = client
        .post(format!("http://{}/graphql", handle.api_addr))
        .json(&serde_json::json!({"query": "{ issues { fingerprint errorType } }"}))
        .send()
        .await
        .expect("issues request")
        .json()
        .await
        .expect("issues json");
    let fingerprint = issues
        .pointer("/data/issues")
        .and_then(|v| v.as_array())
        .and_then(|a| a.iter().find(|i| i["errorType"] == "gate::Timeout"))
        .and_then(|i| i["fingerprint"].as_str())
        .expect("gate issue grouped")
        .to_string();
    let bundle_query = serde_json::json!({"query": format!(
        r#"{{ bundle(fingerprint: "{fingerprint}") {{ canonicalHash }} }}"#
    )});
    let mut bundle_millis = Vec::new();
    for i in 0..11 {
        let t0 = Instant::now();
        let response: serde_json::Value = client
            .post(format!("http://{}/graphql", handle.api_addr))
            .json(&bundle_query)
            .send()
            .await
            .expect("bundle request")
            .json()
            .await
            .expect("bundle json");
        assert!(
            response.pointer("/data/bundle/canonicalHash").is_some(),
            "bundle answers: {response}"
        );
        if i > 0 {
            bundle_millis.push(t0.elapsed().as_millis());
        }
    }
    bundle_millis.sort_unstable();

    println!("== M5 gate measurements (managed GreptimeDB, this machine) ==");
    println!("warm server start (engine cached): {warm_start_millis} ms");
    println!(
        "ingest-to-queryable over {} runs: p50 {} ms, p95 {} ms, max {} ms (gate: p95 <= 5000)",
        latencies_millis.len(),
        percentile(&latencies_millis, 0.50),
        percentile(&latencies_millis, 0.95),
        latencies_millis.last().expect("non-empty"),
    );
    println!("real panic -> grouped issue visible: {panic_visible_millis} ms (gate: ~5000)");
    println!(
        "warm bundle assembly over {} runs: p50 {} ms, p95 {} ms, max {} ms (gate: <= 300 warm)",
        bundle_millis.len(),
        percentile(&bundle_millis, 0.50),
        percentile(&bundle_millis, 0.95),
        bundle_millis.last().expect("non-empty"),
    );

    let p95 = percentile(&latencies_millis, 0.95);
    assert!(p95 <= 5_000, "ingest-to-queryable p95 {p95} ms over gate");
    let bundle_p95 = percentile(&bundle_millis, 0.95);
    assert!(
        bundle_p95 <= 300,
        "bundle warm p95 {bundle_p95} ms over gate"
    );

    handle.shutdown();
}
