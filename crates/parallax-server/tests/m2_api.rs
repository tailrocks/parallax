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

    // Poll GraphQL until the pipeline lands the issue (spec §8: issues
    // returns IssueList { items, total }).
    let client = reqwest::Client::new();
    let mut fingerprint = String::new();
    for _ in 0..50 {
        let response = graphql(
            &client,
            handle.api_addr,
            r#"{ issues { total items { fingerprint errorType eventCount status } } }"#,
        )
        .await;
        if let Some(issue) = response
            .pointer("/data/issues/items")
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

    // Filtered listing: the issue's service matches, a wrong service does
    // not, and the search query matches the error type.
    let response = graphql(
        &client,
        handle.api_addr,
        r#"{ issues(query: "ApiSurface", sort: EVENTS) { total items { fingerprint } }
             none: issues(service: "no-such-service") { total } }"#,
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/issues/items/0/fingerprint")
            .and_then(|v| v.as_str()),
        Some(fingerprint.as_str()),
        "query filter finds the issue: {response}"
    );
    assert_eq!(
        response.pointer("/data/none/total"),
        Some(&serde_json::json!(0))
    );

    // Issue with nested events, tags cache, and latestEvent.
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"{{ issue(fingerprint: "{fingerprint}") {{
                 title status tags trend {{ count }}
                 latestEvent {{ traceId }}
                 events {{ message traceId source }}
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
    assert_eq!(
        response
            .pointer("/data/issue/latestEvent/traceId")
            .and_then(|v| v.as_str()),
        Some(trace_id.as_str())
    );
    let tags: serde_json::Value = serde_json::from_str(
        response
            .pointer("/data/issue/tags")
            .and_then(|v| v.as_str())
            .expect("tags JSON string"),
    )
    .expect("tags parse");
    assert!(tags.is_object(), "tags cache is a JSON object: {tags}");
    let trend_total: i64 = response
        .pointer("/data/issue/trend")
        .and_then(|v| v.as_array())
        .expect("embedded trend")
        .iter()
        .map(|p| p["count"].as_i64().unwrap_or(0))
        .sum();
    assert_eq!(trend_total, 1, "embedded per-issue trend counts the event");

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

    // Mutation: resolve the issue — returns the updated Issue (spec §8).
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(
            r#"mutation {{ issueSetStatus(fingerprint: "{fingerprint}", status: "resolved") {{ fingerprint status }} }}"#
        ),
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/issueSetStatus/status")
            .and_then(|v| v.as_str()),
        Some("resolved"),
        "mutation returns the updated issue: {response}"
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

    // Unified logs browse (spec §8 `logs`): the correlated log is reachable
    // by service-less severity/query filters, newest first.
    let response = graphql(
        &client,
        handle.api_addr,
        r#"{ logs(query: "failing request") { body severityText }
             nothing: logs(query: "no-such-needle") { body } }"#,
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/logs/0/body")
            .and_then(|v| v.as_str()),
        Some("inside the failing request"),
        "unified logs finds by body substring: {response}"
    );
    assert_eq!(
        response
            .pointer("/data/nothing")
            .and_then(|v| v.as_array())
            .map(Vec::len),
        Some(0)
    );

    // serviceOverview answers with graceful absence (no well-known metrics
    // were sent): empty series, not an error.
    let response = graphql(
        &client,
        handle.api_addr,
        r#"{ serviceOverview(service: "m2-run-service",
                             fromNanos: "0", toNanos: "9223372036854775807") {
               cpu { value } requestRate { value } errorRate { value } } }"#,
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/serviceOverview/cpu")
            .and_then(|v| v.as_array())
            .map(Vec::len),
        Some(0),
        "absent instruments yield empty series: {response}"
    );

    // Run-scoped reads: a span emitted under a parallax.run.id resource
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
                    KeyValue::new("parallax.run.id", "run_m2test"),
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
            r#"{ tracesByRun(runId: "run_m2test") {
                   traceId rootName service spanCount hasError } }"#,
        )
        .await;
        if response
            .pointer("/data/tracesByRun/0/rootName")
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
            .pointer("/data/tracesByRun/0/service")
            .and_then(|v| v.as_str()),
        Some("m2-run-service"),
        "run-tagged trace summarized through tracesByRun: {run_traces}"
    );
    let run_trace_id = run_traces
        .pointer("/data/tracesByRun/0/traceId")
        .and_then(|v| v.as_str())
        .expect("trace id in summary")
        .to_string();

    // The summary's trace opens with full spans carrying the run id.
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(r#"{{ trace(traceId: "{run_trace_id}") {{ spans {{ name runId }} }} }}"#),
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/trace/spans/0/runId")
            .and_then(|v| v.as_str()),
        Some("run_m2test")
    );

    // The worker auto-registered the externally-seen run id (no CLI
    // runStart): run(runId) answers with status `external` and counts.
    let response = graphql(
        &client,
        handle.api_addr,
        r#"{ run(runId: "run_m2test") {
               runId status errorCount traceCount issues { fingerprint } } }"#,
    )
    .await;
    assert_eq!(
        response
            .pointer("/data/run/status")
            .and_then(|v| v.as_str()),
        Some("external"),
        "externally-seen run auto-registered: {response}"
    );
    assert_eq!(
        response.pointer("/data/run/traceCount"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        response.pointer("/data/run/errorCount"),
        Some(&serde_json::json!(0))
    );

    // Standard-alias run correlation (spec §7): an emitter using only the
    // OTel `session.id` convention — no parallax.run.id — resolves to the
    // same run model (run-id-standardization.md).
    let session_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("session span exporter");
    let session_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(session_exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", "m2-session-service"),
                    KeyValue::new("session.id", "sess_0042"),
                ])
                .build(),
        )
        .build();
    let session_tracer = session_provider.tracer("m2-session");
    let mut session_span = session_tracer.start("inside.the.session");
    session_span.end();
    session_provider.force_flush().expect("session flush");

    let mut session_run = serde_json::Value::Null;
    for _ in 0..50 {
        let response = graphql(
            &client,
            handle.api_addr,
            r#"{ run(runId: "sess_0042") { runId status traceCount } }"#,
        )
        .await;
        if response
            .pointer("/data/run/traceCount")
            .and_then(|v| v.as_i64())
            == Some(1)
        {
            session_run = response;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        session_run
            .pointer("/data/run/status")
            .and_then(|v| v.as_str()),
        Some("external"),
        "session.id alias resolves to an auto-registered run: {session_run}"
    );

    // Run-anchored bundle (spec §8 bundle(runId:)) renders without errors.
    let response = graphql(
        &client,
        handle.api_addr,
        r#"{ bundle(runId: "run_m2test") { markdown canonicalHash } }"#,
    )
    .await;
    let markdown = response
        .pointer("/data/bundle/markdown")
        .and_then(|v| v.as_str())
        .expect("run bundle markdown");
    assert!(
        markdown.contains("run_m2test"),
        "run bundle names the run: {markdown}"
    );
    assert!(
        markdown.contains("No grouped issues"),
        "clean run says so: {markdown}"
    );

    // Trace-anchored bundle resolves from the same data.
    let response = graphql(
        &client,
        handle.api_addr,
        &format!(r#"{{ bundle(traceId: "{trace_id}") {{ markdown }} }}"#),
    )
    .await;
    let markdown = response
        .pointer("/data/bundle/markdown")
        .and_then(|v| v.as_str())
        .expect("trace bundle markdown");
    assert!(
        markdown.contains("api surface check"),
        "trace bundle carries the primary issue's event: {markdown}"
    );

    // Exactly-one-anchor rule.
    let response = graphql(
        &client,
        handle.api_addr,
        r#"{ bundle(fingerprint: "a", runId: "b") { markdown } }"#,
    )
    .await;
    assert!(
        response.pointer("/errors/0").is_some(),
        "two anchors rejected: {response}"
    );

    handle.shutdown();
}
