//! M2 acceptance (metrics + dashboards slice): real SDK metrics become
//! queryable series, and user dashboards round-trip through the API.

use opentelemetry::metrics::MeterProvider as _;
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use std::time::Duration;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.mode = "none".to_string();
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

#[tokio::test(flavor = "multi_thread")]
async fn metrics_become_series_and_dashboards_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = parallax_server::start(&test_config(tmp.path()))
        .await
        .expect("server starts");

    // A real SDK counter + histogram.
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("metric exporter");
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([opentelemetry::KeyValue::new("service.name", "m2-metrics")])
                .build(),
        )
        .build();
    let meter = meter_provider.meter("m2-metrics");
    let counter = meter.u64_counter("checkout.requests").build();
    counter.add(7, &[]);
    let histogram = meter.f64_histogram("checkout.duration").build();
    histogram.record(0.120, &[]);
    histogram.record(0.480, &[]);
    meter_provider.force_flush().expect("metric flush");

    let client = reqwest::Client::new();

    // metricNames + a sum series with at least one point.
    let mut names = serde_json::Value::Null;
    for _ in 0..50 {
        names = graphql(&client, handle.api_addr, r#"{ metricNames services }"#).await;
        let has_metric = names
            .pointer("/data/metricNames")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|n| n == "checkout.requests"));
        if has_metric {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        names
            .pointer("/data/services")
            .and_then(|v| v.as_array())
            .is_some_and(|a| a.iter().any(|s| s == "m2-metrics")),
        "service listed: {names}"
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let from = now - 3_600_000_000_000u128; // one hour back
    let series = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ metricSeries(name: "checkout.requests", fromNanos: "{from}",
                              toNanos: "{now}", agg: "sum") {{ tsNanos value }} }}"#
        ),
    )
    .await;
    let points = series
        .pointer("/data/metricSeries")
        .and_then(|v| v.as_array())
        .expect("series array");
    assert!(!points.is_empty(), "counter series has points: {series}");
    assert!(
        points
            .iter()
            .any(|p| p["value"].as_f64().unwrap_or(0.0) >= 7.0),
        "summed counter value visible: {series}"
    );

    // Histogram quantile answers (two samples; p99 ~ upper bucket).
    let quantile = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ histogramQuantile(name: "checkout.duration", fromNanos: "{from}",
                                    toNanos: "{now}", q: 0.99) {{ value }} }}"#
        ),
    )
    .await;
    let qpoints = quantile
        .pointer("/data/histogramQuantile")
        .and_then(|v| v.as_array())
        .expect("quantile array");
    assert!(
        !qpoints.is_empty(),
        "quantile series has points: {quantile}"
    );
    assert!(
        qpoints[0]["value"].as_f64().unwrap_or(0.0) > 0.0,
        "p99 above zero: {quantile}"
    );

    // Dashboards CRUD roundtrip.
    let saved = graphql(
        &client,
        handle.api_addr,
        r#"mutation { dashboardSave(name: "ops",
             layout: "[{\"metric\":\"checkout.requests\",\"agg\":\"rate\",\"chart\":\"line\",\"title\":\"req/s\"}]") }"#,
    )
    .await;
    let id = saved
        .pointer("/data/dashboardSave")
        .and_then(|v| v.as_str())
        .expect("dashboard id")
        .to_string();

    let listed = graphql(
        &client,
        handle.api_addr,
        r#"{ dashboards { id name layout } }"#,
    )
    .await;
    assert_eq!(
        listed
            .pointer("/data/dashboards/0/name")
            .and_then(|v| v.as_str()),
        Some("ops")
    );
    assert!(
        listed
            .pointer("/data/dashboards/0/layout")
            .and_then(|v| v.as_str())
            .is_some_and(|l| l.contains("checkout.requests")),
        "layout persisted: {listed}"
    );

    let invalid = graphql(
        &client,
        handle.api_addr,
        r#"mutation { dashboardSave(name: "bad", layout: "not json") }"#,
    )
    .await;
    assert!(
        invalid.get("errors").is_some(),
        "invalid layout rejected: {invalid}"
    );

    let deleted = graphql(
        &client,
        handle.api_addr,
        &format!(r#"mutation {{ dashboardDelete(id: "{id}") }}"#),
    )
    .await;
    assert_eq!(
        deleted.pointer("/data/dashboardDelete"),
        Some(&serde_json::json!(true))
    );

    handle.shutdown();
}
