use super::*;

#[test]
fn log_event_serializes_typed_log_identity() {
    let value = log_event(&LogRow {
        ts_nanos: 1_000_000_000,
        event_name: "checkout.completed".to_string(),
        observed_ts_nanos: 3_000_000_000,
        service: "checkout".to_string(),
        severity_num: 9,
        severity_text: "INFO".to_string(),
        body: "checkout.completed".to_string(),
        trace_id: "trace-a".to_string(),
        span_id: "span-a".to_string(),
        invocation_id: Some("run-a".to_string()),
        session_id: None,
        scope_name: "seed".to_string(),
        attributes: serde_json::json!({"event.name": "checkout.completed"}),
        resource: serde_json::json!({"service.name": "checkout"}),
    });

    assert_eq!(value["eventName"], "checkout.completed");
    assert_eq!(value["observedTsNanos"], "3000000000");
    assert_eq!(value["tsNanos"], "1000000000");
}
