//! Metric timestamp-precision contract (V §2b characterization, 2026-09-13).
//!
//! Parallax forwards OTLP metrics verbatim and the engine auto-creates those
//! tables with `TimestampMillisecond`; every native-metric read binds
//! millisecond bounds. Tables whose `greptime_timestamp` is nanosecond
//! precision — creatable today only by foreign writers (line protocol always
//! auto-creates ns tables regardless of its `precision` param; explicit
//! `TIMESTAMP(9)` DDL) — match no ms-bound window and read back empty (no
//! error) from catalog/labels/quantile paths.
//!
//! This test pins both halves against a real engine: the ms path resolves,
//! the ns path stays empty rather than decoding garbage. A future
//! precision-aware query path (e.g. `to_timestamp_millis()` bounds +
//! scaled decodes) must update these assertions.
//!
//! Run with: `cargo nextest run -p parallax-server --locked --test metric_precision_greptime --run-ignored all`

#![expect(clippy::too_many_lines, reason = "measured integration scenario")]
#![allow(clippy::expect_used, reason = "test fixture assertions")]

use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::metric::Data;
use parallax_proto::metrics::number_data_point::Value as NumberValue;
use parallax_proto::metrics::{Gauge, Metric, NumberDataPoint};
use parallax_server::Config;
use prost::Message;
use std::time::Duration;

const ENGINE: &str = "http://127.0.0.1:24000";

fn kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}

fn gauge_request(name: &str, time_unix_nano: u64) -> Vec<u8> {
    ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![kv("service.name", "precision-probe")],
                ..Default::default()
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                metrics: vec![Metric {
                    name: name.to_string(),
                    data: Some(Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            attributes: vec![kv("probebucket", "b0")],
                            time_unix_nano,
                            value: Some(NumberValue::AsDouble(1.0)),
                            ..Default::default()
                        }],
                    })),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

async fn graphql(client: &reqwest::Client, api_addr: &str, query: String) -> serde_json::Value {
    client
        .post(format!("http://{api_addr}/graphql"))
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .expect("graphql request")
        .json()
        .await
        .expect("graphql json")
}

async fn engine_sql(client: &reqwest::Client, sql: &str) -> serde_json::Value {
    let body = client
        .post(format!("{ENGINE}/v1/sql?db=public"))
        .form(&[("sql", sql)])
        .send()
        .await
        .expect("engine sql")
        .text()
        .await
        .expect("text");
    serde_json::from_str(&body).expect("sql json")
}

fn rows_of(response: &serde_json::Value) -> Vec<serde_json::Value> {
    response["output"][0]["records"]["rows"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

async fn timestamp_type(client: &reqwest::Client, table: &str) -> Option<String> {
    let desc = engine_sql(client, &format!("DESCRIBE TABLE {table}")).await;
    rows_of(&desc).iter().find_map(|row| {
        (row[0] == "greptime_timestamp").then(|| row[1].as_str().unwrap_or_default().to_string())
    })
}

async fn poll_timestamp_type(client: &reqwest::Client, table: &str) -> Option<String> {
    for _ in 0..100 {
        if let Some(found) = timestamp_type(client, table).await {
            return Some(found);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    None
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn metric_timestamp_precision_contract() {
    let cache_bin = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("greptime-bin");
    let tmp = tempfile::tempdir().expect("tempdir");
    let data_bin = tmp.path().join("bin");
    if cache_bin.join("greptime").exists() {
        std::fs::create_dir_all(&data_bin).expect("bin dir");
        std::fs::copy(cache_bin.join("greptime"), data_bin.join("greptime")).expect("seed engine");
    }
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.mode = "managed".to_string();
    config.storage.data_dir = tmp.path().to_string_lossy().into_owned();
    let handle = parallax_server::start(&config).await.expect("start");
    let api_addr = handle.api_addr.to_string();
    let otlp_http = handle.otlp_http_addr.to_string();
    let client = reqwest::Client::new();

    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let now_nanos_u64 = u64::try_from(now_nanos).expect("nanos fit u64");
    let from_nanos = now_nanos.saturating_sub(15 * 60 * 1_000_000_000);
    let to_nanos = now_nanos + 60 * 1_000_000_000;

    // Half 1 (in contract): OTLP-forwarded metrics land in ms tables and read.
    let ms_name = "p0prec.ms.gauge";
    let resp = client
        .post(format!("http://{otlp_http}/v1/metrics"))
        .header("content-type", "application/x-protobuf")
        .body(gauge_request(ms_name, now_nanos_u64))
        .send()
        .await
        .expect("post metrics");
    assert!(resp.status().is_success(), "otlp ingest: {resp:?}");
    let ms_table = "p0prec_ms_gauge";
    let ms_type = poll_timestamp_type(&client, ms_table).await;
    assert_eq!(ms_type.as_deref(), Some("TimestampMillisecond"));
    let mut ms_values = Vec::new();
    for _ in 0..100 {
        let response = graphql(
            &client,
            &api_addr,
            format!(
                r#"{{ metricLabelValues(name: "{ms_name}", label: "probebucket", fromNanos: "{from_nanos}", toNanos: "{to_nanos}") }}"#
            ),
        )
        .await;
        assert!(response["errors"].is_null(), "labels query: {response}");
        ms_values = response["data"]["metricLabelValues"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if !ms_values.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(ms_values, vec![serde_json::json!("b0")]);

    // Half 2 (out of contract): ns writers stay silently empty, never garbage.
    // Trigger A: line protocol with precision=ns auto-creates ns tables.
    let lp_table = "p0prec_lp_ns";
    let lp_resp = client
        .post(format!("{ENGINE}/v1/influxdb/write?db=public&precision=ns"))
        .body(format!(
            "{lp_table},service_name=precision-probe,probebucket=b0 greptime_value=1 {now_nanos_u64}"
        ))
        .send()
        .await
        .expect("line protocol write");
    assert!(lp_resp.status().is_success(), "lp write: {lp_resp:?}");
    // Trigger B: explicit ns DDL + insert.
    let ddl_table = "p0prec_ddlns";
    engine_sql(
        &client,
        &format!(
            "CREATE TABLE {ddl_table} (greptime_timestamp TIMESTAMP(9) TIME INDEX, \
             greptime_value DOUBLE, service_name STRING, probebucket STRING, \
             PRIMARY KEY(service_name, probebucket))"
        ),
    )
    .await;
    engine_sql(
        &client,
        &format!("INSERT INTO {ddl_table} VALUES ({now_nanos}, 1.0, 'precision-probe', 'b0')"),
    )
    .await;
    for table in [lp_table, ddl_table] {
        let precision = poll_timestamp_type(&client, table).await;
        assert_eq!(precision.as_deref(), Some("TimestampNanosecond"), "{table}");
        let response = graphql(
            &client,
            &api_addr,
            format!(
                r#"{{ metricLabelValues(name: "{table}", label: "probebucket", fromNanos: "{from_nanos}", toNanos: "{to_nanos}") }}"#
            ),
        )
        .await;
        assert!(
            response["errors"].is_null(),
            "ns reads stay empty, never error: {response}"
        );
        assert_eq!(
            response["data"]["metricLabelValues"],
            serde_json::json!([]),
            "{table}: ns rows match no ms window"
        );
        let catalog = graphql(
            &client,
            &api_addr,
            format!(
                r#"{{ metricCatalog(fromNanos: "{from_nanos}", toNanos: "{to_nanos}", q: "{table}") {{ name }} }}"#
            ),
        )
        .await;
        assert!(catalog["errors"].is_null(), "catalog query: {catalog}");
        assert_eq!(
            catalog["data"]["metricCatalog"],
            serde_json::json!([]),
            "{table}: zero-point families stay out of the catalog"
        );
    }

    handle.shutdown();
}
