//! Real-engine acceptance for plan-164 structured attribute filters: an
//! OTLP-seeded corpus with a known `http.request.method` split (7 GET /
//! 2 POST / 1 DELETE) proves on a live GreptimeDB that `attributeFilters`
//! narrows `traces_search` and `logs_search`, that an injection-shaped
//! value returns zero rows instead of escaping its literal, that
//! `trace_facets`/`log_facets` count the exact split, and that
//! `trace_duration_stats` answers over the filtered set.
//!
//! Run with: `cargo nextest run -p parallax-server --test m9_attribute_filters_greptime --run-ignored only`

#![allow(clippy::expect_used, clippy::panic, reason = "test fixture assertions")]
#![expect(clippy::too_many_lines, reason = "one seeded end-to-end scenario")]

use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_server::Config;
use parallax_storage::adapter::{AttributeFilter, AttributeFilterOp, TraceQuery};
use prost::Message;
use std::time::Duration;

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

fn filter(key: &str, op: AttributeFilterOp, value: &str) -> AttributeFilter {
    AttributeFilter {
        key: key.to_string(),
        op,
        value: value.to_string(),
    }
}

fn spans_request(base: u64, methods: &[&str]) -> Vec<u8> {
    let spans = methods
        .iter()
        .enumerate()
        .map(|(index, method)| {
            let id = u8::try_from(index + 1).expect("small corpus");
            parallax_proto::trace::Span {
                trace_id: vec![id; 16],
                span_id: vec![id; 8],
                parent_span_id: Vec::new(),
                name: format!("handler-{method}"),
                kind: 2,
                start_time_unix_nano: base + (index as u64) * 1_000_000,
                end_time_unix_nano: base + (index as u64) * 1_000_000 + 5_000_000,
                attributes: vec![
                    kv("http.request.method", method),
                    kv("http.route", "/api/items"),
                ],
                ..Default::default()
            }
        })
        .collect();
    ExportTraceServiceRequest {
        resource_spans: vec![parallax_proto::trace::ResourceSpans {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![kv("service.name", "filter-api")],
                ..Default::default()
            }),
            scope_spans: vec![parallax_proto::trace::ScopeSpans {
                spans,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

fn logs_request(base: u64, methods: &[&str]) -> Vec<u8> {
    ExportLogsServiceRequest {
        resource_logs: vec![parallax_proto::logs::ResourceLogs {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![kv("service.name", "filter-api")],
                ..Default::default()
            }),
            scope_logs: vec![parallax_proto::logs::ScopeLogs {
                log_records: methods
                    .iter()
                    .enumerate()
                    .map(|(index, method)| parallax_proto::logs::LogRecord {
                        time_unix_nano: base + (index as u64) * 1_000_000,
                        observed_time_unix_nano: base + (index as u64) * 1_000_000,
                        severity_number: 9,
                        severity_text: "INFO".to_string(),
                        body: Some(AnyValue {
                            value: Some(AnyValueEnum::StringValue(format!("handled {method}"))),
                        }),
                        attributes: vec![kv("http.request.method", method)],
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

async fn post_otlp(client: &reqwest::Client, url: &str, body: Vec<u8>) {
    let response = client
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(body)
        .send()
        .await
        .unwrap_or_else(|error| panic!("POST {url}: {error}"));
    assert!(
        response.status().is_success(),
        "POST {url}: {}",
        response.status()
    );
}

/// 7 GET / 2 POST / 1 DELETE — the same split discipline as the playground
/// `f-attrs` scenario, exactly assertable.
const METHODS: [&str; 10] = [
    "GET", "GET", "GET", "GET", "GET", "GET", "GET", "POST", "POST", "DELETE",
];

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn attribute_filters_narrow_and_count_on_live_engine() {
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
        &format!("http://{}/v1/traces", handle.otlp_http_addr),
        spans_request(base, &METHODS),
    )
    .await;
    post_otlp(
        &client,
        &format!("http://{}/v1/logs", handle.otlp_http_addr),
        logs_request(base, &METHODS),
    )
    .await;

    let store = &handle.store;
    let from = u128::from(base) - 3_600_000_000_000;
    let to = u128::from(base) + 3_600_000_000_000;
    let base_query = TraceQuery {
        from_nanos: Some(from),
        to_nanos: Some(to),
        limit: 50,
        ..TraceQuery::default()
    };

    // Poll until the whole corpus is queryable (async ingest worker).
    let mut total = 0;
    for _ in 0..200 {
        total = store
            .traces_search(&base_query)
            .await
            .expect("unfiltered traces_search")
            .total;
        if total >= METHODS.len() as u64 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(total, METHODS.len() as u64, "corpus fully ingested");

    // (1) Structured filter narrows to the exact split.
    let post_query = TraceQuery {
        attribute_filters: vec![filter("http.request.method", AttributeFilterOp::Eq, "POST")],
        ..base_query.clone()
    };
    let post = store.traces_search(&post_query).await.expect("POST filter");
    assert_eq!(post.total, 2, "POST narrows to 2 traces");

    // (2) Injection-shaped value stays one literal: zero rows, no escape.
    let injection = store
        .traces_search(&TraceQuery {
            attribute_filters: vec![filter(
                "http.request.method",
                AttributeFilterOp::Eq,
                "x' OR 1=1--",
            )],
            ..base_query.clone()
        })
        .await
        .expect("injection-shaped filter still plans");
    assert_eq!(injection.total, 0, "injection value matches nothing");

    // (3) Facets count the exact distribution, DISTINCT traces per value.
    let facets = store.trace_facets(&base_query).await.expect("trace facets");
    let method_facet = facets
        .iter()
        .find(|facet| facet.dimension == "http.request.method")
        .expect("method facet present");
    let counts: Vec<(&str, u64)> = method_facet
        .values
        .iter()
        .map(|value| (value.value.as_str(), value.count))
        .collect();
    assert_eq!(
        counts,
        vec![("GET", 7), ("POST", 2), ("DELETE", 1)],
        "facet counts match the seeded 70/20/10 split"
    );

    // (4) Duration stats answer over the filtered set.
    let stats = store
        .trace_duration_stats(&post_query)
        .await
        .expect("duration stats");
    assert!(stats.p50_ns.is_some(), "p50 present: {stats:?}");
    assert!(stats.p95_ns.is_some(), "p95 present: {stats:?}");

    // (5) Logs: filter narrows rows and log facets count the split.
    let post_logs = store
        .logs_search(
            None,
            from..=to,
            None,
            None,
            None,
            &[filter("http.request.method", AttributeFilterOp::Eq, "POST")],
            50,
        )
        .await
        .expect("filtered logs_search");
    assert_eq!(post_logs.len(), 2, "POST narrows to 2 log rows");
    let log_facets = store
        .log_facets(None, from..=to, None, None, None, &[])
        .await
        .expect("log facets");
    let log_method_facet = log_facets
        .iter()
        .find(|facet| facet.dimension == "http.request.method")
        .expect("log method facet present");
    let log_counts: Vec<(&str, u64)> = log_method_facet
        .values
        .iter()
        .map(|value| (value.value.as_str(), value.count))
        .collect();
    assert_eq!(
        log_counts,
        vec![("GET", 7), ("POST", 2), ("DELETE", 1)],
        "log facet counts match the seeded split"
    );

    handle.shutdown_graceful().await.expect("server shutdown");
}
