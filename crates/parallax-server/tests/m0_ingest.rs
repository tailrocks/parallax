//! M0 acceptance: a real OpenTelemetry SDK (tracing-free, direct API) exports
//! traces, logs, and metrics over OTLP/gRPC into an in-process Parallax, and
//! the requests land in the spool. The OTLP/HTTP path and the health endpoint
//! are exercised with raw protobuf bytes.

use opentelemetry::KeyValue;
use opentelemetry::logs::{LogRecord as _, Logger as _, LoggerProvider as _};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::{Span as _, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use parallax_storage::spool::Signal;
use prost::Message;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config
}

#[tokio::test(flavor = "multi_thread")]
async fn real_sdk_export_lands_in_the_spool() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = parallax_server::start(&test_config(tmp.path()))
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

    // Spooled lines are valid JSON with the OTLP shape.
    let traces_file = handle.spool.dir().join("traces.ndjson");
    let first_line = std::fs::read_to_string(traces_file)
        .expect("read spool")
        .lines()
        .next()
        .expect("one line")
        .to_string();
    let value: serde_json::Value = serde_json::from_str(&first_line).expect("valid json");
    assert!(
        value.get("resourceSpans").is_some(),
        "OTLP JSON shape: {value}"
    );

    handle.shutdown();
}

#[tokio::test(flavor = "multi_thread")]
async fn otlp_http_and_health_endpoints_work() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = parallax_server::start(&test_config(tmp.path()))
        .await
        .expect("server starts");

    let health = reqwest::get(format!("http://{}/health", handle.api_addr))
        .await
        .expect("health request");
    assert_eq!(health.status(), 200);
    assert_eq!(health.text().await.expect("body"), "ok");

    // Raw protobuf OTLP/HTTP export against both the dedicated :4318-style
    // listener and the API listener's merged routes.
    let request = parallax_proto::collector_trace::ExportTraceServiceRequest::default();
    let body = request.encode_to_vec();
    let client = reqwest::Client::new();
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
