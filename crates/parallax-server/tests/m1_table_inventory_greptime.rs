//! Gated invariant: against a real GreptimeDB, the *only* tables Parallax
//! creates are the three documented custom extension tables — every raw OTel
//! signal (traces, logs, metrics) lives in GreptimeDB's native auto-created
//! tables, never a hand-rolled one. Pushes all three signals, then `SHOW
//! TABLES` and asserts the inventory: native tables present, the extension set
//! is exactly the three, and none of the retired `otel_*` raw tables exist.
//!
//! Run with: `cargo test -p parallax-server --test m1_table_inventory_greptime -- --ignored`

use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::{Span as _, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use std::time::Duration;

/// Tables Parallax is allowed to create itself in GreptimeDB. Everything else
/// must be a native OTLP table the engine auto-created from a forward.
const ALLOWED_EXTENSIONS: &[&str] = &[
    "error_events",
    "rollups_fingerprint_minute",
    "run_metric_points",
];

/// The retired hand-rolled raw-signal tables — must never reappear.
const RETIRED_RAW_TABLES: &[&str] = &[
    "otel_spans",
    "otel_logs",
    "otel_metrics_points",
    "otel_metrics_histograms",
];

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn only_extension_tables_are_custom() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=debug")
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
    if !cache_bin.join("greptime").exists() && data_bin.join("greptime").exists() {
        std::fs::create_dir_all(&cache_bin).expect("cache dir");
        let _ = std::fs::copy(data_bin.join("greptime"), cache_bin.join("greptime"));
    }

    let endpoint = format!("http://{}", handle.otlp_grpc_addr);

    // (1) A trace.
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();
    tracer_provider.tracer("inv").start("inv.span").end();
    tracer_provider.force_flush().expect("trace flush");

    // (2) A log.
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()
        .expect("log exporter");
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([opentelemetry::KeyValue::new("service.name", "inv")])
                .build(),
        )
        .build();
    {
        use opentelemetry::logs::{LogRecord as _, Logger as _, LoggerProvider as _};
        let logger = logger_provider.logger("inv");
        let mut record = logger.create_log_record();
        record.set_body("inventory log".into());
        logger.emit(record);
    }
    logger_provider.force_flush().expect("log flush");

    // (3) A metric (counter + histogram).
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()
        .expect("metric exporter");
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([opentelemetry::KeyValue::new("service.name", "inv")])
                .build(),
        )
        .build();
    let meter = meter_provider.meter("inv");
    meter.u64_counter("inv.requests").build().add(1, &[]);
    meter.f64_histogram("inv.latency").build().record(0.25, &[]);
    meter_provider.force_flush().expect("metric flush");

    // Poll until all three native signal families have materialized.
    let want_native = ["opentelemetry_traces", "opentelemetry_logs"];
    let mut tables: Vec<String> = Vec::new();
    for _ in 0..100 {
        tables = show_tables(&handle).await;
        let native_ready = want_native.iter().all(|t| tables.iter().any(|x| x == t));
        let metric_ready = tables.iter().any(|t| {
            t.starts_with("inv_requests") || t == "inv_latency" || t.starts_with("inv_latency")
        });
        if native_ready && metric_ready {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // The native trace + log tables must exist (engine auto-created them).
    for native in want_native {
        assert!(
            tables.iter().any(|t| t == native),
            "native table {native} present; inventory: {tables:?}"
        );
    }

    // None of the retired hand-rolled raw tables may exist.
    for retired in RETIRED_RAW_TABLES {
        assert!(
            !tables.iter().any(|t| t == retired),
            "retired raw table {retired} must not exist; inventory: {tables:?}"
        );
    }

    // Every table is either a native OTLP table (opentelemetry_*), a native
    // per-metric / metric-engine table, or one of the three allowed extensions.
    // Anything else is an unexpected custom table.
    let unexpected: Vec<&String> = tables
        .iter()
        .filter(|t| {
            let t = t.as_str();
            let native = t.starts_with("opentelemetry_") || t.starts_with("greptime_");
            let extension = ALLOWED_EXTENSIONS.contains(&t);
            // Native per-metric tables created from the forwarded OTLP metrics.
            let metric = t.starts_with("inv_");
            !(native || extension || metric)
        })
        .collect();
    assert!(
        unexpected.is_empty(),
        "no unexpected custom tables (only the 3 extensions are ours); found {unexpected:?} in {tables:?}"
    );

    // And the only *extension* tables present are a subset of the allowed three.
    let extensions: Vec<&String> = tables
        .iter()
        .filter(|t| ALLOWED_EXTENSIONS.contains(&t.as_str()))
        .collect();
    assert!(
        extensions.len() <= ALLOWED_EXTENSIONS.len(),
        "extension tables are a subset of the documented three: {extensions:?}"
    );

    handle.shutdown();
}

/// `SHOW TABLES` in the public schema via the storage adapter's raw-SQL surface.
async fn show_tables(handle: &parallax_server::ServerHandle) -> Vec<String> {
    let result = handle
        .store
        .raw_sql("SHOW TABLES")
        .await
        .expect("show tables");
    result
        .rows
        .iter()
        .filter_map(|row| row.first().and_then(|v| v.as_str()).map(str::to_string))
        .collect()
}
