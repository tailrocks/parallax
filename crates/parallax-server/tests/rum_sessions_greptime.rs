//! R3 live proof: browser RUM payloads ingested through public OTLP must
//! resolve as a first-class session — `rumSessions` returns the session
//! entity with linked counts, `rumSession` returns the timeline of page
//! views, vitals, and errors. Drives the exact GraphQL path the /rum
//! sessions surface uses through managed GreptimeDB (no memory doubles on
//! this path).
//!
//! Run with: `cargo nextest run -p parallax-server --locked --test rum_sessions_greptime --run-ignored all`
//! The binary is cached under target/greptime-test-bin/ across runs.

#![expect(clippy::too_many_lines, reason = "measured integration scenario")]
#![allow(clippy::expect_used, reason = "test fixture assertions")]

use opentelemetry::KeyValue;
use opentelemetry::trace::{Span as _, Status, Tracer as _, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::resource::Resource;
use parallax_server::Config;
use std::time::Duration;

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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn rum_session_resolves_against_managed_engine() {
    let cache_bin = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("greptime-bin");
    let home_bin = std::env::home_dir()
        .map(|h| h.join(".parallax/bin/greptime"))
        .filter(|p| p.exists());
    if let Some(existing) = home_bin
        && !cache_bin.join("greptime").exists()
    {
        std::fs::create_dir_all(&cache_bin).expect("cache dir");
        std::fs::copy(existing, cache_bin.join("greptime")).expect("copy cached engine");
    }

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

    let handle = parallax_server::start(&config)
        .await
        .expect("managed server starts");
    let api_addr = handle.api_addr.to_string();

    if !cache_bin.join("greptime").exists() && data_bin.join("greptime").exists() {
        std::fs::create_dir_all(&cache_bin).expect("cache dir");
        std::fs::copy(data_bin.join("greptime"), cache_bin.join("greptime"))
            .expect("cache downloaded engine");
    }

    // One browser session: resource carries `session.id` (the web shape —
    // no cli.invocation.id, no session.start/end events).
    let session_id = format!("sess-rum-live-{}", std::process::id());
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(format!("http://{}", handle.otlp_grpc_addr))
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name("rum-live-web")
                .with_attribute(KeyValue::new("session.id", session_id.clone()))
                .build(),
        )
        .with_batch_exporter(span_exporter)
        .build();
    let tracer = tracer_provider.tracer("rum-live");

    let mut view = tracer.start("app.screen.name");
    view.set_attribute(KeyValue::new("app.screen.name", "home"));
    view.set_attribute(KeyValue::new("url.path", "/"));
    view.end();

    let mut vital = tracer.start("browser.web_vital");
    vital.set_attribute(KeyValue::new("web_vital.name", "LCP"));
    vital.set_attribute(KeyValue::new("web_vital.value", 1200.0));
    vital.set_attribute(KeyValue::new("web_vital.rating", "good"));
    vital.end();

    let mut checkout = tracer.start("app.screen.name");
    checkout.set_attribute(KeyValue::new("app.screen.name", "checkout"));
    checkout.set_attribute(KeyValue::new("url.path", "/checkout"));
    checkout.end();

    let mut error = tracer.start("web.error.handled");
    error.set_attribute(KeyValue::new("error.type", "TypeError"));
    error.set_status(Status::error("boom"));
    error.end();

    tracer_provider.force_flush().expect("flush");
    tracer_provider.shutdown().expect("tracer shutdown");

    // Ingestion is asynchronous: poll the sessions inbox until the live
    // session surfaces, then assert the entity contract in one query.
    let client = reqwest::Client::new();
    let mut session = serde_json::Value::Null;
    for _ in 0..300 {
        let response = graphql(
            &client,
            &api_addr,
            r#"{ rumSessions(fromNanos: "0", toNanos: "99999999999999999999999") {
                sessionId service spanCount traceCount viewCount vitalCount errorCount hasError
            } }"#
                .to_string(),
        )
        .await;
        assert!(
            response["errors"].is_null(),
            "rumSessions query failed: {response}"
        );
        let found = response["data"]["rumSessions"]
            .as_array()
            .and_then(|sessions| {
                sessions
                    .iter()
                    .find(|row| row["sessionId"] == session_id)
                    .cloned()
            });
        if let Some(found) = found {
            session = found;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(session["sessionId"], session_id, "live session grouped");
    assert_eq!(session["service"], "rum-live-web");
    assert_eq!(session["spanCount"], 4);
    assert_eq!(session["traceCount"], 4);
    assert_eq!(session["viewCount"], 2);
    assert_eq!(session["vitalCount"], 1);
    assert_eq!(session["errorCount"], 1);
    assert_eq!(session["hasError"], true);

    // The session detail timeline: views, vitals, and errors linked to traces.
    let detail = graphql(
        &client,
        &api_addr,
        format!(
            r#"{{ rumSession(sessionId: "{session_id}") {{
                session {{ sessionId startNanos endNanos }}
                views {{ screen path traceId spanId }}
                vitals {{ name value rating traceId }}
                errors {{ name errorType message traceId spanId }} }} }}"#
        ),
    )
    .await;
    assert!(
        detail["errors"].is_null(),
        "rumSession must resolve live: {detail}"
    );
    let body = &detail["data"]["rumSession"];
    assert_eq!(body["session"]["sessionId"], session_id);
    let views = body["views"].as_array().expect("views array");
    assert_eq!(views.len(), 2, "two page views expected: {detail}");
    assert_eq!(views[0]["screen"], "home");
    assert_eq!(views[0]["path"], "/");
    assert!(!views[0]["traceId"].as_str().unwrap_or_default().is_empty());
    assert_eq!(views[1]["screen"], "checkout");
    assert_eq!(views[1]["path"], "/checkout");
    let vitals = body["vitals"].as_array().expect("vitals array");
    assert_eq!(vitals.len(), 1, "one vital expected: {detail}");
    assert_eq!(vitals[0]["name"], "LCP");
    assert_eq!(vitals[0]["value"], 1200.0);
    assert_eq!(vitals[0]["rating"], "good");
    let errors = body["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1, "one error expected: {detail}");
    assert_eq!(errors[0]["name"], "web.error.handled");
    assert_eq!(errors[0]["errorType"], "TypeError");
    assert_eq!(errors[0]["message"], "boom");
    assert!(!errors[0]["traceId"].as_str().unwrap_or_default().is_empty());

    // errorOnly narrows the inbox to the errored session.
    let errored = graphql(
        &client,
        &api_addr,
        r#"{ rumSessions(fromNanos: "0", toNanos: "99999999999999999999999", errorOnly: true) { sessionId } }"#
            .to_string(),
    )
    .await;
    assert!(
        errored["errors"].is_null(),
        "errorOnly query failed: {errored}"
    );
    let rows = errored["data"]["rumSessions"].as_array().expect("rows");
    assert!(
        rows.iter().any(|row| row["sessionId"] == session_id),
        "live session in errorOnly inbox: {errored}"
    );

    handle.shutdown();
}
