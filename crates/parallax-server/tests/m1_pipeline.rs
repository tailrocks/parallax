//! M1 acceptance: a real SDK export carrying an exception span and an ERROR
//! log flows receiver → worker → storage adapter + metadata, producing a
//! grouped issue and anchored-readable telemetry.

use opentelemetry::KeyValue;
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity};
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use parallax_server::Config;
use std::time::Duration;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config
}

#[tokio::test(flavor = "multi_thread")]
async fn error_telemetry_becomes_a_grouped_issue() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = parallax_server::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let grpc_endpoint = format!("http://{}", handle.otlp_grpc_addr);

    // A span that fails with an exception event — twice, with volatile
    // message differences, so grouping is exercised.
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint.clone())
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("m1-test");
    let mut captured_trace_id = String::new();
    for attempt in [2, 4] {
        let mut span = tracer.start("payment.authorize");
        captured_trace_id = format!("{:032x}", span.span_context().trace_id());
        span.add_event(
            "exception",
            vec![
                KeyValue::new("exception.type", "redis::ConnectionTimeout"),
                KeyValue::new(
                    "exception.message",
                    format!("timed out connecting to redis://cache-7:6379 (attempt {attempt})"),
                ),
                KeyValue::new(
                    "exception.stacktrace",
                    "checkout::payment::authorize at src/payment.rs:184",
                ),
            ],
        );
        span.set_status(Status::error("connection timed out"));
        span.end();
    }
    tracer_provider.force_flush().expect("trace flush");

    // An ERROR log on the same service: a second fingerprint.
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint)
        .build()
        .expect("log exporter");
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .build();
    let logger = logger_provider.logger("m1-test");
    let mut record = logger.create_log_record();
    record.set_severity_number(Severity::Error);
    record.set_severity_text("ERROR");
    record.set_body(AnyValue::from(
        "payment authorization failed: retries exhausted",
    ));
    logger.emit(record);
    logger_provider.force_flush().expect("log flush");

    // The worker is async; poll the metadata store until grouping lands.
    let mut issues = Vec::new();
    for _ in 0..50 {
        issues = handle.metadata.issues(10).await.expect("issues query");
        let exception_grouped = issues
            .iter()
            .any(|i| i.error_type == "redis::ConnectionTimeout" && i.event_count == 2);
        if exception_grouped && issues.len() >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let exception_issue = issues
        .iter()
        .find(|i| i.error_type == "redis::ConnectionTimeout")
        .expect("exception issue grouped");
    assert_eq!(
        exception_issue.event_count, 2,
        "two volatile-message occurrences must group into one issue"
    );
    assert_eq!(
        exception_issue.culprit.as_deref(),
        Some("checkout::payment::authorize at src/payment.rs:184")
    );
    assert_eq!(exception_issue.status, "open");
    assert!(exception_issue.last_trace_id.is_some());
    assert!(
        issues.iter().any(|i| i.error_type == "log_error"),
        "the ERROR log must form its own issue: {issues:?}"
    );

    // Anchored reads work through the adapter.
    let spans = handle
        .store
        .spans_by_trace(&captured_trace_id)
        .await
        .expect("spans read");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "payment.authorize");
    assert_eq!(spans[0].status_code, "STATUS_CODE_ERROR");
    let events = handle
        .store
        .error_events_by_fingerprint(&exception_issue.fingerprint, 0..=u128::MAX, 10)
        .await
        .expect("error events read");
    assert_eq!(events.len(), 2);

    handle.shutdown();
}
