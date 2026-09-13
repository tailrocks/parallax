//! Real-engine acceptance for ingest-converted exponential histograms: an
//! OTLP batch carrying an exp histogram (plus a gauge sibling) proves on a
//! live GreptimeDB that the stripped forward keeps the batch alive, converted
//! rows persist to `exp_histograms`, and quantile/avg/count/catalog/labels
//! serve them through the converted-exp fallback path with MemoryStore parity.
//!
//! Run with: `cargo nextest run -p parallax-server --test m12_exp_histogram_greptime --run-ignored only`

#![allow(clippy::expect_used, clippy::panic, reason = "test fixture assertions")]
#![expect(clippy::too_many_lines, reason = "one seeded end-to-end scenario")]

use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::exponential_histogram_data_point::Buckets;
use parallax_proto::metrics::{
    ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Metric, NumberDataPoint,
};
use parallax_server::Config;
use parallax_storage::model::HistogramRow;
use parallax_test_support::builders::MemoryStore;
use prost::Message;
use std::time::Duration;

const SERVICE: &str = "exp-service";
const METRIC: &str = "http.server.request.duration";
const GAUGE: &str = "process.cpu.utilization";

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

/// One batch: exp histogram (scale 0, two positive buckets) + gauge sibling.
/// The gauge proves the strip-and-forward keeps the rest of the batch alive.
fn metrics_request(base: u64) -> Vec<u8> {
    let exp_point = |offset: u64, count: u64, sum: f64| ExponentialHistogramDataPoint {
        attributes: vec![kv("route", "/pay")],
        start_time_unix_nano: base,
        time_unix_nano: base + offset,
        count,
        sum: Some(sum),
        scale: 0,
        zero_count: 0,
        positive: Some(Buckets {
            offset: 0,
            bucket_counts: vec![3, 5],
        }),
        ..Default::default()
    };
    let request = ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![kv("service.name", SERVICE)],
                ..Default::default()
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                scope: None,
                metrics: vec![
                    Metric {
                        name: METRIC.to_string(),
                        data: Some(parallax_proto::metrics::metric::Data::ExponentialHistogram(
                            ExponentialHistogram {
                                data_points: vec![
                                    exp_point(0, 8, 20.0),
                                    exp_point(61_000_000_000, 16, 44.0),
                                ],
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    },
                    Metric {
                        name: GAUGE.to_string(),
                        data: Some(parallax_proto::metrics::metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: Vec::new(),
                                start_time_unix_nano: base,
                                time_unix_nano: base,
                                value: Some(
                                    parallax_proto::metrics::number_data_point::Value::AsDouble(
                                        0.5,
                                    ),
                                ),
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    request.encode_to_vec()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn exp_histograms_convert_store_and_query_on_live_engine() {
    let _subscriber_already_installed = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=info")
        .try_init();
    let tmp = tempfile::tempdir().expect("tempdir");
    let data_bin = tmp.path().join("data/bin");
    let cache_bin = std::env::home_dir().expect("home").join(".parallax/bin");
    let seed = std::fs::read_dir(&cache_bin)
        .ok()
        .and_then(|mut entries| {
            entries.find_map(|entry| {
                let path = entry.ok()?.path();
                (path.file_name()?.to_str()? == "greptime").then_some(path)
            })
        })
        .or_else(|| {
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

    let client = reqwest::Client::new();
    let base = now_nanos();
    client
        .post(format!("http://{}/v1/metrics", handle.otlp_http_addr))
        .header("content-type", "application/x-protobuf")
        .body(metrics_request(base))
        .send()
        .await
        .expect("post metrics")
        .error_for_status()
        .expect("ingest accepted");

    let store = &handle.store;
    let from = u128::from(base);
    let to = u128::from(base) + 3_600_000_000_000;
    let step = 60_000_000_000;

    // Poll until the converted rows are queryable (async ingest worker).
    let mut ready = false;
    for _ in 0..200 {
        let catalog = store
            .metric_catalog(from..=to, None, None, 50)
            .await
            .expect("catalog");
        if catalog.iter().any(|entry| entry.name == METRIC) {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(ready, "converted exp metric appears in catalog");

    // (1) Catalog classifies the converted metric as a histogram, one count
    // per export.
    let catalog = store
        .metric_catalog(from..=to, None, None, 50)
        .await
        .expect("catalog");
    let entry = catalog
        .iter()
        .find(|entry| entry.name == METRIC)
        .expect("exp metric cataloged");
    assert_eq!(entry.kind, parallax_storage::model::MetricKind::Histogram);
    assert_eq!(entry.point_count, 2);
    assert_eq!(entry.services, vec![SERVICE.to_string()]);

    // (2) Quantiles interpolate the converted grid: bounds [2,4] counts
    // [3,5] → p50 target 4 → 2 + 2*(1/5) = 2.4, in both windows.
    let p50 = store
        .histogram_quantile(METRIC, Some(SERVICE), &[], from..=to, step, 0.5)
        .await
        .expect("quantile");
    assert_eq!(p50.len(), 2, "one point per window: {p50:?}");
    for point in &p50 {
        assert!((point.value - 2.4).abs() < 1e-9, "p50: {p50:?}");
    }

    // (3) Avg is Δsum/Δcount across the two exports: 24/8 = 3.
    let avg = store
        .histogram_avg(METRIC, Some(SERVICE), &[], from..=to, step)
        .await
        .expect("avg");
    assert_eq!(avg.len(), 1, "one delta: {avg:?}");
    assert!((avg[0].value - 3.0).abs() < 1e-9, "avg: {avg:?}");

    // (4) Count series deltas the cumulative counts: 16-8 = 8 new samples.
    let counts = store
        .histogram_count_series(METRIC, Some(SERVICE), from..=to, step)
        .await
        .expect("counts");
    assert_eq!(counts.len(), 1, "first window omitted: {counts:?}");
    assert!((counts[0].value - 8.0).abs() < 1e-9, "counts: {counts:?}");

    // (5) Labels and values come from the converted rows' attributes.
    let labels = store.metric_labels(METRIC).await.expect("labels");
    assert!(labels.contains(&"route".to_string()), "{labels:?}");
    let values = store
        .metric_label_values(METRIC, "route", from..=to)
        .await
        .expect("label values");
    assert_eq!(values, vec!["/pay".to_string()]);

    // (6) The gauge sibling survived the strip-and-forward in the same batch.
    let names = store.metric_names(from..=to).await.expect("names");
    assert!(names.iter().any(|name| name == GAUGE), "{names:?}");
    let gauge = store
        .metric_series(
            GAUGE,
            Some(SERVICE),
            None,
            &[],
            from..=to,
            step,
            parallax_storage::model::MetricAgg::Avg,
        )
        .await
        .expect("gauge series");
    assert!(!gauge.is_empty(), "sibling gauge queryable");

    // (7) MemoryStore parity: the same converted rows answer identically.
    let memory = MemoryStore::new();
    parallax_storage::adapter::IngestStore::ingest_metrics(
        &memory,
        Vec::new(),
        Vec::new(),
        vec![
            HistogramRow {
                ts_nanos: u128::from(base),
                service: SERVICE.to_string(),
                name: METRIC.to_string(),
                count: 8,
                sum: 20.0,
                bucket_counts: vec![3, 5],
                bounds: vec![2.0, 4.0],
                attributes: serde_json::json!({"route": "/pay"}),
            },
            HistogramRow {
                ts_nanos: u128::from(base) + 61_000_000_000,
                service: SERVICE.to_string(),
                name: METRIC.to_string(),
                count: 16,
                sum: 44.0,
                bucket_counts: vec![3, 5],
                bounds: vec![2.0, 4.0],
                attributes: serde_json::json!({"route": "/pay"}),
            },
        ],
        Vec::new(),
        bytes::Bytes::new(),
    )
    .await
    .expect("memory ingest");
    let memory_p50 = parallax_storage::adapter::MetricAnalyticsStore::histogram_quantile(
        &memory,
        METRIC,
        Some(SERVICE),
        &[],
        from..=to,
        step,
        0.5,
    )
    .await
    .expect("memory quantile");
    assert_eq!(memory_p50.len(), p50.len());
    for (a, b) in memory_p50.iter().zip(p50.iter()) {
        assert_eq!(a.ts_nanos, b.ts_nanos);
        assert!((a.value - b.value).abs() < 1e-9);
    }
}
