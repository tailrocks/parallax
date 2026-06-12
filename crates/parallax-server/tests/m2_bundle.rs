//! M2 acceptance (bundle slice): the bundle query returns bounded, redacted,
//! hypothesis-ranked evidence — and seeded secrets never reach it.

use opentelemetry::KeyValue;
use opentelemetry::logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity};
use opentelemetry::trace::{Span as _, SpanContext, Status, Tracer as _, TracerProvider as _};
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

#[tokio::test(flavor = "multi_thread")]
async fn bundle_is_bounded_redacted_and_hypothesis_ranked() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = parallax_server::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let grpc_endpoint = format!("http://{}", handle.otlp_grpc_addr);

    // A timeout-shaped failure with a seeded canary secret in the log.
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint.clone())
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("m2-bundle");
    let mut span = tracer.start("payment.authorize");
    let span_context: SpanContext = span.span_context().clone();
    span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.type", "redis::ConnectionTimeout"),
            KeyValue::new(
                "exception.message",
                "timed out connecting to redis://cache-7:6379 after 2000ms",
            ),
            KeyValue::new(
                "exception.stacktrace",
                "checkout::payment::authorize at src/payment.rs:184",
            ),
        ],
    );
    span.set_status(Status::error("timeout"));
    span.end();
    tracer_provider.force_flush().expect("trace flush");

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint)
        .build()
        .expect("log exporter");
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .build();
    let logger = logger_provider.logger("m2-bundle");
    let mut record = logger.create_log_record();
    record.set_severity_number(Severity::Info);
    record.set_body(AnyValue::from(
        "retrying with auth=Bearer abc123secrettoken key=AKIAIOSFODNN7EXAMPLE",
    ));
    record.set_trace_context(span_context.trace_id(), span_context.span_id(), None);
    logger.emit(record);
    logger_provider.force_flush().expect("log flush");

    // A process gauge in the same window (same default-resource service) —
    // the bundle must correlate it as a metric window.
    use opentelemetry::metrics::MeterProvider as _;
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("metric exporter");
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .build();
    let gauge = meter_provider
        .meter("m2-bundle")
        .f64_gauge("process.cpu.utilization")
        .build();
    gauge.record(0.37, &[]);
    meter_provider.force_flush().expect("metric flush");

    // Find the issue, then fetch its bundle.
    let client = reqwest::Client::new();
    let mut fingerprint = String::new();
    for _ in 0..50 {
        let response: serde_json::Value = client
            .post(format!("http://{}/graphql", handle.api_addr))
            .json(&serde_json::json!({"query": "{ issues { items { fingerprint errorType } } }"}))
            .send()
            .await
            .expect("issues request")
            .json()
            .await
            .expect("issues json");
        if let Some(issue) = response
            .pointer("/data/issues/items")
            .and_then(|v| v.as_array())
            .and_then(|a| {
                a.iter()
                    .find(|i| i["errorType"] == "redis::ConnectionTimeout")
            })
        {
            fingerprint = issue["fingerprint"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(!fingerprint.is_empty());

    let response: serde_json::Value = client
        .post(format!("http://{}/graphql", handle.api_addr))
        .json(&serde_json::json!({"query": format!(
            r#"{{ bundle(fingerprint: "{fingerprint}") {{ json markdown canonicalHash }} }}"#
        )}))
        .send()
        .await
        .expect("bundle request")
        .json()
        .await
        .expect("bundle json");

    let json = response
        .pointer("/data/bundle/json")
        .and_then(|v| v.as_str())
        .expect("bundle json field");
    let markdown = response
        .pointer("/data/bundle/markdown")
        .and_then(|v| v.as_str())
        .expect("bundle markdown field");
    let hash = response
        .pointer("/data/bundle/canonicalHash")
        .and_then(|v| v.as_str())
        .expect("bundle hash");

    // Redaction: canaries never reach either projection.
    for projection in [json, markdown] {
        assert!(
            !projection.contains("abc123secrettoken"),
            "bearer canary leaked"
        );
        assert!(
            !projection.contains("AKIAIOSFODNN7EXAMPLE"),
            "aws canary leaked"
        );
    }
    assert!(
        json.contains("REDACTED"),
        "redaction report visible in bundle"
    );

    // Hypotheses: the timeout shape is recognized; markdown carries the
    // sections the agent reads.
    assert!(
        json.contains("dependency_failure"),
        "timeout hypothesis ranked: {json}"
    );
    assert!(markdown.contains("## Trace"), "trace section present");
    assert!(
        markdown.contains("## Correlated logs"),
        "logs section present"
    );
    assert!(
        markdown.contains("## Hypotheses"),
        "hypotheses section present"
    );
    assert!(hash.starts_with("sha256:"));

    // Correlation: the same-window process gauge appears as a metric window
    // (poll — the metric batch may land just after the issue).
    let mut correlated = String::new();
    for _ in 0..50 {
        let response: serde_json::Value = client
            .post(format!("http://{}/graphql", handle.api_addr))
            .json(&serde_json::json!({"query": format!(
                r#"{{ bundle(fingerprint: "{fingerprint}") {{ json markdown }} }}"#
            )}))
            .send()
            .await
            .expect("bundle request")
            .json()
            .await
            .expect("bundle json");
        let json = response
            .pointer("/data/bundle/json")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if json.contains("process.cpu.utilization") {
            correlated = response
                .pointer("/data/bundle/markdown")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let parsed: serde_json::Value = serde_json::from_str(json).expect("bundle json parses");
            let window = parsed
                .pointer("/metric_windows/0")
                .expect("metric window present");
            assert_eq!(window["metric"], "process.cpu.utilization");
            assert_eq!(window["scope"], "service");
            assert!(
                window["stats"]["last"].as_f64().unwrap_or(0.0) > 0.3,
                "gauge value visible in stats: {window}"
            );
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        correlated.contains("## Metric windows"),
        "markdown carries the metric-window section: {correlated}"
    );

    // Bounding: a tiny budget still yields a valid bundle that fits.
    let response: serde_json::Value = client
        .post(format!("http://{}/graphql", handle.api_addr))
        .json(&serde_json::json!({"query": format!(
            r#"{{ bundle(fingerprint: "{fingerprint}", maxTokens: 500) {{ json }} }}"#
        )}))
        .send()
        .await
        .expect("bounded request")
        .json()
        .await
        .expect("bounded json");
    let bounded_json = response
        .pointer("/data/bundle/json")
        .and_then(|v| v.as_str())
        .expect("bounded bundle");
    let parsed: serde_json::Value = serde_json::from_str(bounded_json).expect("valid json");
    let estimated = parsed
        .pointer("/bounded/estimated_tokens")
        .and_then(|v| v.as_u64())
        .expect("estimate present");
    assert!(
        estimated <= 700,
        "bounded bundle near budget, got {estimated}"
    );

    handle.shutdown();
}
