//! Real-engine acceptance for plan-105 metric summaries: an OTLP-seeded
//! gauge corpus (dotted name, one NaN sample, one invocation-tagged subset)
//! proves on a live GreptimeDB that overview totals count windowed finite
//! samples, the MetricPoints signal trend buckets the same counts, service
//! discovery includes the metric-only service, and
//! `invocation_metric_summaries` returns canonical native-family names with
//! MemoryStore parity.
//!
//! Run with: `cargo nextest run -p parallax-server --test m10_metric_summaries_greptime --run-ignored only`

#![allow(clippy::expect_used, clippy::panic, reason = "test fixture assertions")]
#![expect(clippy::too_many_lines, reason = "one seeded end-to-end scenario")]

use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_server::Config;
use parallax_storage::adapter::SignalKind;
use parallax_storage::model::MetricPointRow;
use parallax_test_support::builders::MemoryStore;
use prost::Message;
use std::time::Duration;

const INVOCATION: &str = "11111111-2222-3333-4444-555555555555";
const SERVICE: &str = "metrics-only-service";
const METRIC: &str = "app.render.time";

fn kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn now_nanos() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
    )
    .expect("fits u64")
}

/// Three finite samples plus one NaN, all invocation-tagged, one gauge.
fn metrics_request(base: u64) -> Vec<u8> {
    let point = |offset: u64, value: f64| parallax_proto::metrics::NumberDataPoint {
        attributes: Vec::new(),
        start_time_unix_nano: base,
        time_unix_nano: base + offset,
        exemplars: Vec::new(),
        flags: 0,
        value: Some(parallax_proto::metrics::number_data_point::Value::AsDouble(
            value,
        )),
    };
    let request = ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![
                    kv("service.name", SERVICE),
                    kv("cli.invocation.id", INVOCATION),
                ],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                scope: None,
                metrics: vec![parallax_proto::metrics::Metric {
                    name: METRIC.to_string(),
                    description: String::new(),
                    unit: "ms".to_string(),
                    metadata: Vec::new(),
                    data: Some(parallax_proto::metrics::metric::Data::Gauge(
                        parallax_proto::metrics::Gauge {
                            data_points: vec![
                                point(1_000_000_000, 5.0),
                                point(2_000_000_000, 7.0),
                                point(3_000_000_000, 9.0),
                                point(4_000_000_000, f64::NAN),
                            ],
                        },
                    )),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };
    request.encode_to_vec()
}

async fn post_otlp(client: &reqwest::Client, url: &str, body: Vec<u8>) {
    let response = client
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(body)
        .send()
        .await
        .expect("otlp post");
    assert!(
        response.status().is_success(),
        "otlp accepted: {:?}",
        response.status()
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn metric_summaries_conform_on_live_engine() {
    let _subscriber_already_installed = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=info")
        .try_init();
    let cache_bin = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("greptime-bin");
    let home_bin = std::env::home_dir()
        .map(|home| home.join(".parallax/bin/greptime"))
        .filter(|path| path.exists());
    let tmp = tempfile::tempdir().expect("tempdir");
    let data_bin = tmp.path().join("bin");
    let seed = home_bin.or_else(|| {
        let cached = cache_bin.join("greptime");
        cached.exists().then_some(cached)
    });
    if let Some(existing) = seed {
        std::fs::create_dir_all(&data_bin).expect("bin dir");
        std::fs::copy(&existing, data_bin.join("greptime")).expect("seed engine");
        let status = std::process::Command::new("chmod")
            .arg("+x")
            .arg(data_bin.join("greptime"))
            .status()
            .expect("chmod");
        assert!(status.success());
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
        std::fs::copy(data_bin.join("greptime"), cache_bin.join("greptime"))
            .expect("cache downloaded engine");
    }

    let client = reqwest::Client::new();
    let base = now_nanos();
    post_otlp(
        &client,
        &format!("http://{}/v1/metrics", handle.otlp_http_addr),
        metrics_request(base),
    )
    .await;

    let store = &handle.store;
    let from = u128::from(base);
    let to = u128::from(base) + 3_600_000_000_000;

    // Poll until the finite samples are queryable (async ingest worker).
    let mut totals = None;
    for _ in 0..200 {
        let overview = store
            .overview_totals(from..=to)
            .await
            .expect("overview totals");
        if overview.metric_point_count >= 3 {
            totals = Some(overview);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let totals = totals.expect("finite metric samples become visible");

    // (1) Windowed finite counting: NaN sample never counts.
    assert_eq!(totals.metric_point_count, 3, "3 finite samples");

    // (2) Metric-only service discovery: overview and service_names see it.
    assert!(totals.active_services >= 1, "metric-only service is active");
    let names = store.service_names(from..=to).await.expect("service names");
    assert!(
        names.iter().any(|name| name == SERVICE),
        "metric-only service discovered: {names:?}"
    );

    // (3) Signal trend buckets sum to the same finite count.
    let trend = store
        .signal_count_series(SignalKind::MetricPoints, None, from..=to, 60_000_000_000)
        .await
        .expect("metric trend");
    let trend_total: f64 = trend.iter().map(|point| point.value).sum();
    assert_eq!(trend_total as u64, 3, "trend buckets: {trend:?}");

    // (4) Invocation summaries: canonical name, finite-only, last value.
    // The extension write can trail native visibility; poll like the corpus.
    let mut summaries = Vec::new();
    for _ in 0..200 {
        summaries = store
            .invocation_metric_summaries(INVOCATION, from..=to, 50)
            .await
            .expect("invocation summaries");
        if !summaries.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(summaries.len(), 1, "one family: {summaries:?}");
    assert_eq!(summaries[0].name, "app_render_time", "canonical identity");
    assert_eq!(summaries[0].point_count, 3, "finite points only");
    assert_eq!(summaries[0].last_value, 9.0, "latest finite sample");

    // (5) Unknown invocation is empty, never an error.
    let unknown = store
        .invocation_metric_summaries("not-an-invocation", from..=to, 50)
        .await
        .expect("unknown invocation");
    assert!(unknown.is_empty());

    // (6) MemoryStore parity over identical seeds.
    let memory = MemoryStore::new();
    let rows: Vec<MetricPointRow> = [(1u64, 5.0), (2, 7.0), (3, 9.0)]
        .iter()
        .map(|(offset, value)| MetricPointRow {
            ts_nanos: u128::from(base + offset * 1_000_000_000),
            service: SERVICE.to_string(),
            name: METRIC.to_string(),
            value: *value,
            is_monotonic: false,
            invocation_id: Some(INVOCATION.to_string()),
            attributes: serde_json::json!({}),
        })
        .chain(std::iter::once(MetricPointRow {
            ts_nanos: u128::from(base + 4_000_000_000),
            service: SERVICE.to_string(),
            name: METRIC.to_string(),
            value: f64::NAN,
            is_monotonic: false,
            invocation_id: Some(INVOCATION.to_string()),
            attributes: serde_json::json!({}),
        }))
        .collect();
    parallax_storage::adapter::IngestStore::ingest_metrics(
        &memory,
        rows,
        Vec::new(),
        Vec::new(),
        Default::default(),
    )
    .await
    .expect("memory ingest");
    let memory_summaries =
        parallax_storage::adapter::MetricAnalyticsStore::invocation_metric_summaries(
            &memory,
            INVOCATION,
            from..=to,
            50,
        )
        .await
        .expect("memory summaries");
    assert_eq!(memory_summaries, summaries, "adapter parity");

    handle.shutdown();
}
