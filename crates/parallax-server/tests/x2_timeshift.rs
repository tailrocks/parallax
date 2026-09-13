#![expect(clippy::too_many_lines, reason = "measured integration scenario")]

//! X2 timeshift: `metricQuery(shiftSeconds:)` returns the previous window's
//! data over the live HTTP API. Two seeded gauge samples pin the current and
//! previous windows; a live OTLP probe in the same test proves the ingest
//! pipeline behind the server is real.

#![allow(
    clippy::expect_used,
    clippy::float_cmp,
    reason = "test fixture assertions"
)]

use opentelemetry::metrics::MeterProvider as _;
use opentelemetry_otlp::WithExportConfig;
use parallax_metadata::TursoMetadataStore;
use parallax_server::Config;
use parallax_storage::adapter::IngestStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model::MetricPointRow;
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;
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

fn point(ts_nanos: u128, value: f64) -> MetricPointRow {
    MetricPointRow {
        ts_nanos,
        service: "x2-timeshift".into(),
        name: "x2.timeshift.latency".into(),
        value,
        is_monotonic: false,
        invocation_id: None,
        attributes: serde_json::json!({}),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn shift_seconds_returns_previous_window_over_live_api() {
    let base = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let window = 300_000_000_000u128; // 300s current window
    let shift = 3_600u128; // one hour back
    let (from, to) = (base - window / 2, base + window / 2);

    // Deterministic two-window fixture: 222 in the current window, 111 exactly
    // one shift earlier. Explicit timestamps need direct seeding — OTLP stamps
    // arrivals with now.
    let store = Arc::new(MemoryStore::new().with_normalizers(
        Arc::new(parallax_ingest::normalize_traces),
        Arc::new(parallax_ingest::normalize_logs),
    ));
    store
        .ingest_metrics(
            vec![
                point(base - shift * 1_000_000_000, 111.0),
                point(base, 222.0),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Default::default(),
        )
        .await
        .expect("seed windows");

    // Same assembly as the shared harness, but with our seeded store handle.
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = test_config(tmp.path());
    let turso = Arc::new(
        TursoMetadataStore::open(config.data_dir().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let metadata: Arc<dyn MetadataStore> = turso.clone();
    let handle = parallax_server::start_with_turso(&config, store, metadata, Some(turso))
        .await
        .expect("server starts");

    // Live OTLP probe: a real SDK export must surface through the same API.
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("metric exporter");
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([opentelemetry::KeyValue::new("service.name", "x2-timeshift")])
                .build(),
        )
        .build();
    let meter = meter_provider.meter("x2-timeshift");
    let probe = meter.u64_counter("x2.timeshift.probe").build();
    probe.add(5, &[]);
    meter_provider.force_flush().expect("metric flush");

    let client = reqwest::Client::new();
    let mut names = serde_json::Value::Null;
    for _ in 0..50 {
        names = graphql(&client, handle.api_addr, r"{ metricNames }").await;
        let present = names
            .pointer("/data/metricNames")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|n| n == "x2.timeshift.probe"));
        if present {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        names
            .pointer("/data/metricNames")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|n| n == "x2.timeshift.probe")),
        "live OTLP probe ingested: {names}"
    );

    // Unshifted: current window holds the 222 sample only.
    let current = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ metricQuery(name: "x2.timeshift.latency", kind: "gauge", agg: "avg", fromNanos: "{from}", toNanos: "{to}", stepSeconds: 60) {{
                effectiveStepSeconds series {{ points {{ tsNanos value }} }} }} }}"#
        ),
    )
    .await;
    assert!(
        current.pointer("/errors").is_none(),
        "current window: {current}"
    );
    let points = current
        .pointer("/data/metricQuery/series/0/points")
        .and_then(|v| v.as_array())
        .expect("current points");
    assert_eq!(points.len(), 1, "one current bucket: {current}");
    assert_eq!(points[0].get("value").unwrap().as_f64().unwrap(), 222.0);
    let current_step = current
        .pointer("/data/metricQuery/effectiveStepSeconds")
        .expect("step")
        .clone();

    // Shifted by one window: previous window holds the 111 sample only.
    let shifted = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ metricQuery(name: "x2.timeshift.latency", kind: "gauge", agg: "avg", fromNanos: "{from}", toNanos: "{to}", stepSeconds: 60, shiftSeconds: 3600) {{
                effectiveStepSeconds series {{ points {{ tsNanos value }} }} }} }}"#
        ),
    )
    .await;
    assert!(
        shifted.pointer("/errors").is_none(),
        "shifted window: {shifted}"
    );
    assert_eq!(
        shifted
            .pointer("/data/metricQuery/effectiveStepSeconds")
            .expect("shifted step"),
        &current_step,
        "shift preserves step rounding: {shifted}"
    );
    let points = shifted
        .pointer("/data/metricQuery/series/0/points")
        .and_then(|v| v.as_array())
        .expect("shifted points");
    assert_eq!(points.len(), 1, "one previous bucket: {shifted}");
    assert_eq!(points[0].get("value").unwrap().as_f64().unwrap(), 111.0);

    handle.shutdown();
}
