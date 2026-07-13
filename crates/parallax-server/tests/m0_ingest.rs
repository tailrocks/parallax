//! M0 acceptance: a real OpenTelemetry SDK (tracing-free, direct API) exports
//! traces, logs, and metrics over OTLP/gRPC into an in-process Parallax, and
//! the requests land in the spool. The OTLP/HTTP path and the health endpoint
//! are exercised with raw protobuf bytes.

use opentelemetry::KeyValue;
use opentelemetry::logs::{LogRecord as _, Logger as _, LoggerProvider as _};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::{Span as _, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use parallax_metadata::TursoMetadataStore;
use parallax_server::Config;
use parallax_spool::Signal;
use parallax_test_support::builders::MemoryStore;
use prost::Message;
use std::sync::Arc;
use tokio::sync::oneshot;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config
}

async fn wait_for_healthy(client: &reqwest::Client, url: &str) -> Result<String, String> {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if response.status().is_success() {
                return response.text().await.map_err(|error| error.to_string());
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tokio::test(flavor = "multi_thread")]
async fn real_sdk_export_lands_in_the_spool() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let grpc_endpoint = format!("http://{}", handle.otlp_grpc_addr);

    // Traces.
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint.clone())
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("m0-test");
    let mut span = tracer.start("m0 smoke span");
    span.set_attribute(KeyValue::new("test.case", "m0"));
    span.end();
    tracer_provider.force_flush().expect("trace flush");

    // Logs.
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint.clone())
        .build()
        .expect("log exporter");
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .build();
    let logger = logger_provider.logger("m0-test");
    let mut record = logger.create_log_record();
    record.set_severity_number(opentelemetry::logs::Severity::Error);
    record.set_body("m0 smoke log".into());
    logger.emit(record);
    logger_provider.force_flush().expect("log flush");

    // Metrics.
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint)
        .build()
        .expect("metric exporter");
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .build();
    let meter = meter_provider.meter("m0-test");
    meter.u64_counter("m0.smoke").build().add(1, &[]);
    meter_provider.force_flush().expect("metric flush");

    assert!(
        handle.spool.line_count(Signal::Traces).expect("count") >= 1,
        "trace spooled"
    );
    assert!(
        handle.spool.line_count(Signal::Logs).expect("count") >= 1,
        "log spooled"
    );
    assert!(
        handle.spool.line_count(Signal::Metrics).expect("count") >= 1,
        "metric spooled"
    );

    // Spooled frames are raw protobuf under the PSPL1 magic.
    let traces_file = handle.spool.dir().join("traces.pspl");
    let bytes = std::fs::read(&traces_file).expect("read spool");
    assert!(
        bytes.starts_with(b"PSPL1"),
        "spool magic missing: {} bytes",
        bytes.len()
    );
    assert!(bytes.len() > 5 + 4, "expected at least one frame");

    handle.shutdown();
}

#[tokio::test(flavor = "multi_thread")]
async fn otlp_http_and_health_endpoints_work() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");

    let health = reqwest::get(format!("http://{}/health", handle.api_addr))
        .await
        .expect("health request");
    assert_eq!(health.status(), 200);
    assert_eq!(health.text().await.expect("body"), "ok");

    let client = reqwest::Client::new();
    let graphql: serde_json::Value = client
        .post(format!("http://{}/graphql", handle.api_addr))
        .json(&serde_json::json!({ "query": "{ otlpGrpcPort }" }))
        .send()
        .await
        .expect("graphql request")
        .json()
        .await
        .expect("graphql json");
    assert_eq!(
        graphql["data"]["otlpGrpcPort"].as_u64(),
        Some(u64::from(handle.otlp_grpc_addr.port()))
    );

    // Raw protobuf OTLP/HTTP export against both the dedicated :4318-style
    // listener and the API listener's merged routes.
    let request = parallax_proto::collector_trace::ExportTraceServiceRequest::default();
    let body = request.encode_to_vec();
    for addr in [handle.otlp_http_addr, handle.api_addr] {
        let response = client
            .post(format!("http://{addr}/v1/traces"))
            .header("content-type", "application/x-protobuf")
            .body(body.clone())
            .send()
            .await
            .expect("otlp/http post");
        assert_eq!(response.status(), 200, "addr {addr}");
    }
    assert!(handle.spool.line_count(Signal::Traces).expect("count") >= 2);

    let bad = client
        .post(format!("http://{}/v1/traces", handle.otlp_http_addr))
        .header("content-type", "application/x-protobuf")
        .body(vec![0xffu8, 0x01, 0x02])
        .send()
        .await
        .expect("bad post");
    assert_eq!(bad.status(), 400, "garbage protobuf must be rejected");

    handle.shutdown();
}

#[tokio::test(flavor = "multi_thread")]
async fn health_reports_real_queue_overload_and_recovery() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let mut config = test_config(tmp.path());
    config.limits.ingest_queue_batches = 1;
    let (release_tx, release_rx) = oneshot::channel();
    let store = Arc::new(MemoryStore::new());
    store.set_traces_gate(release_rx).await;
    let metadata = Arc::new(
        TursoMetadataStore::open(tmp.path().join("meta.db"))
            .await
            .map_err(|error| error.to_string())?,
    );
    let handle = parallax_server::start_with_capabilities(&config, store, metadata)
        .await
        .map_err(|error| error.to_string())?;
    let client = reqwest::Client::new();
    let ingest_url = format!("http://{}/v1/traces", handle.otlp_http_addr);
    let health_url = format!("http://{}/health", handle.api_addr);
    let body =
        parallax_proto::collector_trace::ExportTraceServiceRequest::default().encode_to_vec();

    for _ in 0..2 {
        let response = client
            .post(&ingest_url)
            .header("content-type", "application/x-protobuf")
            .body(body.clone())
            .send()
            .await
            .map_err(|error| error.to_string())?;
        if !response.status().is_success() {
            return Err(format!("initial ingest failed: {}", response.status()));
        }
    }

    let overloaded = client
        .get(&health_url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let overloaded_status = overloaded.status();
    let overloaded_body = overloaded.text().await.map_err(|error| error.to_string())?;
    if overloaded_status != reqwest::StatusCode::SERVICE_UNAVAILABLE
        || overloaded_body != "degraded: ingest queue full (traces=1/1)"
    {
        return Err(format!(
            "overload health mismatch: {overloaded_status} {overloaded_body:?}"
        ));
    }

    release_tx
        .send(())
        .map_err(|()| "traces gate closed before release".to_string())?;
    let recovered = wait_for_healthy(&client, &health_url).await?;
    if recovered != "ok" {
        return Err(format!("recovery health mismatch: {recovered:?}"));
    }

    handle.shutdown_graceful().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn gzip_compressed_otlp_http_is_accepted() {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");

    let request = parallax_proto::collector_trace::ExportTraceServiceRequest::default();
    let plain = request.encode_to_vec();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&plain).expect("gzip");
    let gz = encoder.finish().expect("finish gzip");

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}/v1/traces", handle.otlp_http_addr))
        .header("content-type", "application/x-protobuf")
        .header("content-encoding", "gzip")
        .body(gz)
        .send()
        .await
        .expect("gzip post");
    assert_eq!(response.status(), 200, "gzip OTLP/HTTP must be accepted");
    assert!(
        handle.spool.line_count(Signal::Traces).expect("count") >= 1,
        "gzip body must land in the spool"
    );
    handle.shutdown();
}
#[path = "support/harness.rs"]
mod support;
