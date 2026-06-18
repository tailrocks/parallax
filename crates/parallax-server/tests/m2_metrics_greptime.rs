//! Gated M2 metrics acceptance for the managed-engine path: downloads (once)
//! and supervises a real GreptimeDB standalone child, then round-trips SDK
//! metrics through the native per-metric tables and reads them back over
//! GraphQL. This is the metric counterpart to `m1_greptime` — it exercises the
//! native metric read stack that the memory-backed `m2_metrics_dashboards` test
//! cannot: per-metric table discovery (`information_schema` + histogram-sibling
//! collapse), `greptime_timestamp`/`greptime_value` ms→ns scaling, attribute
//! tag-column grouping, and the cumulative `_bucket` quantile math.
//!
//! Run with: `cargo test -p parallax-server --test m2_metrics_greptime -- --ignored`

use opentelemetry::metrics::MeterProvider as _;
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use std::time::Duration;

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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn managed_engine_metrics_roundtrip() {
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

    // Real SDK counter (with a grouping attribute) + histogram → native tables.
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("metric exporter");
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([opentelemetry::KeyValue::new("service.name", "m2-greptime")])
                .build(),
        )
        .build();
    let meter = meter_provider.meter("m2-greptime");
    let counter = meter.u64_counter("checkout.requests").build();
    counter.add(7, &[opentelemetry::KeyValue::new("payment.method", "card")]);
    counter.add(3, &[opentelemetry::KeyValue::new("payment.method", "wire")]);
    let histogram = meter.f64_histogram("checkout.duration").build();
    histogram.record(0.120, &[]);
    histogram.record(0.480, &[]);
    meter_provider.force_flush().expect("metric flush");

    let client = reqwest::Client::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let from = now - 3_600_000_000_000u128; // one hour back

    // Native-first: GreptimeDB's metric engine Prometheus-normalizes metric
    // names — dots become underscores and a monotonic counter gains the `_total`
    // suffix (`checkout.requests` → `checkout_requests_total`), while the
    // histogram keeps its base name with `_bucket`/`_count`/`_sum` siblings
    // (`checkout.duration` → `checkout_duration`). `metricNames` surfaces those
    // native names and reads address them as-is. Poll until they exist.
    const COUNTER: &str = "checkout_requests_total";
    const HISTOGRAM: &str = "checkout_duration";
    let mut names = serde_json::Value::Null;
    for _ in 0..100 {
        names = graphql(&client, handle.api_addr, r#"{ metricNames services }"#).await;
        let has_metric = names
            .pointer("/data/metricNames")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|n| n == COUNTER));
        if has_metric {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        names
            .pointer("/data/metricNames")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|n| n == COUNTER)),
        "counter discovered under its native (Prometheus) name: {names}"
    );
    assert!(
        names
            .pointer("/data/metricNames")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|n| n == HISTOGRAM)),
        "histogram collapses to its base name (no _bucket/_count/_sum): {names}"
    );

    // Summed counter series over the native per-metric table (ms→ns scaling).
    let series = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ metricSeries(name: "{COUNTER}", fromNanos: "{from}",
                              toNanos: "{now}", agg: "sum") {{
                   groupValue points {{ tsNanos value }} }} }}"#
        ),
    )
    .await;
    let points = series
        .pointer("/data/metricSeries/0/points")
        .and_then(|v| v.as_array())
        .expect("series array");
    assert!(!points.is_empty(), "counter series has points: {series}");
    assert!(
        points
            .iter()
            .any(|p| p["value"].as_f64().unwrap_or(0.0) >= 10.0),
        "summed counter value visible (7 + 3): {series}"
    );
    // Points must carry native ns timestamps inside the queried window.
    assert!(
        points.iter().all(|p| {
            p["tsNanos"]
                .as_str()
                .and_then(|s| s.parse::<u128>().ok())
                .is_some_and(|ts| ts >= from && ts <= now)
        }),
        "series timestamps are ns-scaled inside the window: {series}"
    );

    // groupBy splits the same metric by its native tag column.
    let grouped = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ metricSeries(name: "{COUNTER}", fromNanos: "{from}",
                              toNanos: "{now}", agg: "sum", groupBy: "payment_method") {{
                   groupValue points {{ value }} }} }}"#
        ),
    )
    .await;
    let mut group_values: Vec<&str> = grouped
        .pointer("/data/metricSeries")
        .and_then(|v| v.as_array())
        .expect("grouped series")
        .iter()
        .filter_map(|s| s["groupValue"].as_str())
        .collect();
    group_values.sort_unstable();
    assert_eq!(
        group_values,
        ["card", "wire"],
        "one series per native tag value: {grouped}"
    );

    // Histogram quantile from the native cumulative `_bucket` table.
    let quantile = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ histogramQuantile(name: "{HISTOGRAM}", fromNanos: "{from}",
                                    toNanos: "{now}", q: 0.99) {{ value }} }}"#
        ),
    )
    .await;
    let qpoints = quantile
        .pointer("/data/histogramQuantile")
        .and_then(|v| v.as_array())
        .expect("quantile array");
    assert!(
        qpoints
            .iter()
            .any(|p| p["value"].as_f64().unwrap_or(0.0) > 0.0),
        "p99 answered from native cumulative buckets: {quantile}"
    );

    handle.shutdown();
}
