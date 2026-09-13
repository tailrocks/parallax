use crate::resolvers::test_support::*;
use crate::{build_schema, execute};
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

fn rum_span(
    trace: &str,
    span_id: &str,
    ts: u128,
    name: &str,
    session: Option<&str>,
    attributes: serde_json::Value,
) -> parallax_storage::model::SpanRow {
    let mut row = span("web", trace, span_id, ts, 1_000);
    row.name = name.to_string();
    row.session_id = session.map(str::to_string);
    row.attributes = attributes;
    row
}

fn seed() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    store.push_spans(vec![
        rum_span(
            "trace-a",
            "span-view-1",
            10,
            "app.screen.name",
            Some("sess-1"),
            serde_json::json!({"app.screen.name": "home", "url.path": "/"}),
        ),
        rum_span(
            "trace-a",
            "span-vital-1",
            20,
            "browser.web_vital",
            Some("sess-1"),
            serde_json::json!({"web_vital.name": "LCP", "web_vital.value": 1200.0, "web_vital.rating": "good"}),
        ),
        rum_span(
            "trace-b",
            "span-view-2",
            30,
            "app.screen.name",
            Some("sess-1"),
            serde_json::json!({"app.screen.name": "checkout", "url.path": "/checkout"}),
        ),
    ]);
    let mut error = rum_span(
        "trace-b",
        "span-error-1",
        40,
        "web.error.handled",
        Some("sess-1"),
        serde_json::json!({"error.type": "TypeError"}),
    );
    error.status_code = "STATUS_CODE_ERROR".to_string();
    error.status_message = "boom".to_string();
    store.push_spans(vec![error]);
    store.push_spans(vec![rum_span(
        "trace-c",
        "span-view-3",
        50,
        "app.screen.name",
        Some("sess-2"),
        serde_json::json!({"app.screen.name": "home", "url.path": "/"}),
    )]);
    store
}

async fn rum_query(store: Arc<MemoryStore>, query: &str) -> serde_json::Value {
    let context = context_with_memory(store).await;
    let schema = build_schema();
    let request = juniper::http::GraphQLRequest::new(query.to_string(), None, None);
    serde_json::to_value(execute(&schema, &context, request).await).unwrap()
}

#[tokio::test]
async fn rum_sessions_lists_sessions_with_counts() {
    let response = rum_query(
        seed(),
        r#"{ rumSessions(fromNanos: "0", toNanos: "99999999999999999999999") {
            sessionId service startNanos endNanos spanCount traceCount
            viewCount vitalCount errorCount hasError } }"#,
    )
    .await;
    assert!(
        response["errors"].is_null(),
        "rumSessions failed: {response}"
    );
    let sessions = response["data"]["rumSessions"]
        .as_array()
        .expect("sessions");
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0]["sessionId"], "sess-2");
    let first = &sessions[1];
    assert_eq!(first["sessionId"], "sess-1");
    assert_eq!(first["service"], "web");
    assert_eq!(first["startNanos"], "10");
    assert_eq!(first["endNanos"], "40");
    assert_eq!(first["spanCount"], 4);
    assert_eq!(first["traceCount"], 2);
    assert_eq!(first["viewCount"], 2);
    assert_eq!(first["vitalCount"], 1);
    assert_eq!(first["errorCount"], 1);
    assert_eq!(first["hasError"], true);
}

#[tokio::test]
async fn rum_sessions_honors_error_only() {
    let response = rum_query(
        seed(),
        r#"{ rumSessions(fromNanos: "0", toNanos: "99999999999999999999999", errorOnly: true) { sessionId } }"#,
    )
    .await;
    assert!(
        response["errors"].is_null(),
        "rumSessions failed: {response}"
    );
    let sessions = response["data"]["rumSessions"]
        .as_array()
        .expect("sessions");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["sessionId"], "sess-1");
}

#[tokio::test]
async fn rum_session_returns_timeline() {
    let response = rum_query(
        seed(),
        r#"{ rumSession(sessionId: "sess-1") {
            session { sessionId viewCount vitalCount errorCount }
            views { tsNanos screen path traceId spanId }
            vitals { tsNanos name value rating traceId }
            errors { tsNanos name errorType message traceId spanId } } }"#,
    )
    .await;
    assert!(
        response["errors"].is_null(),
        "rumSession failed: {response}"
    );
    let detail = &response["data"]["rumSession"];
    assert_eq!(detail["session"]["sessionId"], "sess-1");
    let views = detail["views"].as_array().expect("views");
    assert_eq!(views.len(), 2);
    assert_eq!(views[0]["screen"], "home");
    assert_eq!(views[0]["path"], "/");
    assert_eq!(views[0]["traceId"], "trace-a");
    assert_eq!(views[1]["screen"], "checkout");
    let vitals = detail["vitals"].as_array().expect("vitals");
    assert_eq!(vitals.len(), 1);
    assert_eq!(vitals[0]["name"], "LCP");
    assert_eq!(vitals[0]["value"], 1200.0);
    assert_eq!(vitals[0]["rating"], "good");
    let errors = detail["errors"].as_array().expect("errors");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["name"], "web.error.handled");
    assert_eq!(errors[0]["errorType"], "TypeError");
    assert_eq!(errors[0]["message"], "boom");
    assert_eq!(errors[0]["traceId"], "trace-b");
}

#[tokio::test]
async fn rum_session_unknown_id_is_null() {
    let response = rum_query(
        seed(),
        r#"{ rumSession(sessionId: "sess-unknown") { views { screen } } }"#,
    )
    .await;
    assert!(
        response["errors"].is_null(),
        "rumSession failed: {response}"
    );
    assert!(response["data"]["rumSession"].is_null());
}
