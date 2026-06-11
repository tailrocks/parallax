//! M2 acceptance (API slice): the GraphQL surface answers over real ingested
//! telemetry — issues with nested events, trace, correlated logs, and the
//! issue-status mutation.

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
async fn graphql_surface_answers_over_ingested_telemetry() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = parallax_server::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let grpc_endpoint = format!("http://{}", handle.otlp_grpc_addr);

    // Emit a failing span and a correlated log in its context.
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint.clone())
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("m2-test");
    let mut span = tracer.start("api.surface");
    let trace_id = format!("{:032x}", span.span_context().trace_id());
    let span_context: SpanContext = span.span_context().clone();
    span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.type", "test::ApiSurface"),
            KeyValue::new("exception.message", "api surface check"),
        ],
    );
    span.set_status(Status::error("boom"));
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
    let logger = logger_provider.logger("m2-test");
    let mut record = logger.create_log_record();
    record.set_severity_number(Severity::Info);
    record.set_body(AnyValue::from("inside the failing request"));
    record.set_trace_context(span_context.trace_id(), span_context.span_id(), None);
    logger.emit(record);
    logger_provider.force_flush().expect("log flush");

    // Poll GraphQL until the pipeline lands the issue.
    let client = reqwest::Client::new();
    let mut fingerprint = String::new();
    for _ in 0..50 {
        let response = graphql(
            &client,
            handle.api_addr,
            r#"{ issues { fingerprint errorType eventCount status } }"#,
        )
        .await;
        if let Some(issue) = response
            .pointer("/data/issues")
            .and_then(|v| v.as_array())
            .and_then(|a| a.iter().find(|i| i["errorType"] == "test::ApiSurface"))
        {
            fingerprint = issue["fingerprint"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(!fingerprint.is_empty(), "issue visible through GraphQL");

    // Issue with nested events.
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ issue(fingerprint: "{fingerprint}") {{
                 title status events {{ message traceId source }}
               }} }}"#
        ),
    )
    .await;
    let events = response
        .pointer("/data/issue/events")
        .and_then(|v| v.as_array())
        .expect("events array");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["traceId"], trace_id.as_str());
    assert_eq!(events[0]["source"], "span_exception");

    // Trend rollup reaches the sparkline query.
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(r#"{{ issueTrend(fingerprint: "{fingerprint}") {{ tsNanos count }} }}"#),
    )
    .await;
    let trend = response
        .pointer("/data/issueTrend")
        .and_then(|v| v.as_array())
        .expect("trend array");
    assert_eq!(
        trend
            .iter()
            .map(|p| p["count"].as_i64().unwrap_or(0))
            .sum::<i64>(),
        1,
        "one occurrence counted in the trend: {trend:?}"
    );

    // Trace and correlated logs.
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ trace(traceId: "{trace_id}") {{ spans {{ name statusCode }} }}
                 logsByTrace(traceId: "{trace_id}") {{ body severityText }} }}"#
        ),
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/trace/spans/0/name")
            .and_then(|v| v.as_str()),
        Some("api.surface")
    );
    assert_eq!(
        response
            .pointer("/data/logsByTrace/0/body")
            .and_then(|v| v.as_str()),
        Some("inside the failing request"),
        "log correlated to the trace through the SDK context: {response}"
    );

    // Mutation: resolve the issue.
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"mutation {{ issueSetStatus(fingerprint: "{fingerprint}", status: "resolved") }}"#
        ),
    )
    .await;
    assert_eq!(
        response.pointer("/data/issueSetStatus"),
        Some(&serde_json::json!(true))
    );
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(r#"{{ issue(fingerprint: "{fingerprint}") {{ status }} }}"#),
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/issue/status")
            .and_then(|v| v.as_str()),
        Some("resolved")
    );

    // Run-scoped reads: a span emitted under a parallax.run_id resource
    // attribute is reachable through tracesByRun.
    let run_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("run span exporter");
    let run_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(run_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "m2-run-service"),
                    KeyValue::new("parallax.run_id", "run_m2test"),
                ])
                .build(),
        )
        .build();
    let run_tracer = run_provider.tracer("m2-run");
    let mut run_span = run_tracer.start("inside.the.run");
    run_span.end();
    run_provider.force_flush().expect("run flush");

    let mut run_traces = serde_json::Value::Null;
    for _ in 0..50 {
        let response = graphql(
            &client,
            handle.api_addr,
            r#"{ tracesByRun(runId: "run_m2test") { traceId spans { name runId } } }"#,
        )
        .await;
        if response
            .pointer("/data/tracesByRun/0/spans/0/name")
            .and_then(|v| v.as_str())
            == Some("inside.the.run")
        {
            run_traces = response;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        run_traces
            .pointer("/data/tracesByRun/0/spans/0/runId")
            .and_then(|v| v.as_str()),
        Some("run_m2test"),
        "run-tagged span reachable through tracesByRun: {run_traces}"
    );

    handle.shutdown();
}
