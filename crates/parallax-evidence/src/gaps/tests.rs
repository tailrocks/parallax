use super::*;
use serde_json::json;

fn span(id: &str, parent: Option<&str>, kind: &str) -> SpanRow {
    SpanRow {
        ts_nanos: 100,
        service: "api".to_string(),
        trace_id: "trace-a".to_string(),
        span_id: id.to_string(),
        parent_span_id: parent.map(str::to_string),
        name: id.to_string(),
        kind: kind.to_string(),
        status_code: "STATUS_CODE_UNSET".to_string(),
        status_message: String::new(),
        duration_ns: 10,
        invocation_id: None,
        session_id: None,
        scope_name: String::new(),
        events: None,
        links: Value::Null,
        attributes: json!({}),
        resource: json!({}),
    }
}

fn log(trace_id: &str) -> LogRow {
    LogRow {
        ts_nanos: 100,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: "api".to_string(),
        severity_num: 9,
        severity_text: "INFO".to_string(),
        body: "body".to_string(),
        trace_id: trace_id.to_string(),
        span_id: String::new(),
        invocation_id: None,
        session_id: None,
        scope_name: String::new(),
        attributes: json!({}),
        resource: json!({}),
    }
}

#[test]
fn detects_orphan_span_with_caveat() {
    let gaps = detect_gaps(&[span("child", Some("missing"), "SPAN_KIND_SERVER")], &[]);

    assert_eq!(gaps[0].kind, "orphan_span");
    assert!(gaps[0].detail.contains("legitimate cross-service root"));
}

#[test]
fn detects_log_without_trace() {
    let gaps = detect_gaps(&[], &[log("00000000000000000000000000000000")]);

    assert_eq!(gaps[0].kind, "log_without_trace");
}

#[test]
fn detects_producer_without_consumer_in_set_only() {
    let mut producer = span("producer", None, "SPAN_KIND_PRODUCER");
    producer.links = json!([{ "traceId": "trace-a", "spanId": "missing-consumer" }]);

    let gaps = detect_gaps(&[producer], &[]);

    assert_eq!(gaps[0].kind, "producer_without_consumer");
}

#[test]
fn detects_browser_without_backend() {
    let mut browser = span("browser", None, "SPAN_KIND_CLIENT");
    browser.resource = json!({ "telemetry.sdk.language": "webjs" });

    let gaps = detect_gaps(&[browser], &[]);

    assert_eq!(gaps[0].kind, "browser_without_backend");
}

#[test]
fn clean_trace_has_no_gaps_and_output_is_deterministic() {
    let mut client = span("client", None, "SPAN_KIND_CLIENT");
    client.attributes = json!({ "http.route": "/checkout" });
    let server = span("server", Some("client"), "SPAN_KIND_SERVER");
    let mut producer = span("producer", None, "SPAN_KIND_PRODUCER");
    producer.links = json!([{ "traceId": "trace-a", "spanId": "consumer" }]);
    let consumer = span("consumer", None, "SPAN_KIND_CONSUMER");
    let spans = vec![client, server, producer, consumer];
    let logs = vec![log("trace-a")];

    assert!(detect_gaps(&spans, &logs).is_empty());
    assert_eq!(detect_gaps(&spans, &logs), detect_gaps(&spans, &logs));
}
