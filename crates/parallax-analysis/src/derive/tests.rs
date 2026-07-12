use super::*;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::{AnyValue, KeyValue, any_value};
use parallax_proto::resource::Resource;
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span, Status, span, status};
use serde_json::json;

fn string_kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn span_request(span: Span) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![string_kv("service.name", "checkout")],
                ..Default::default()
            }),
            scope_spans: vec![ScopeSpans {
                spans: vec![span],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn test_span(status_code: i32, exception: bool) -> Span {
    Span {
        trace_id: vec![1; 16],
        span_id: vec![2; 8],
        name: "checkout.authorize".to_string(),
        end_time_unix_nano: 99,
        status: Some(Status {
            code: status_code,
            message: "status failed".to_string(),
        }),
        events: exception
            .then(|| span::Event {
                time_unix_nano: 42,
                name: "exception".to_string(),
                attributes: vec![
                    string_kv("exception.type", "test::Boom"),
                    string_kv("exception.message", "boom"),
                    string_kv("exception.stacktrace", "top\nbottom"),
                ],
                ..Default::default()
            })
            .into_iter()
            .collect(),
        ..Default::default()
    }
}

fn log_row(body: &str, attributes: serde_json::Value) -> LogRow {
    LogRow {
        ts_nanos: 1,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: "checkout".to_string(),
        severity_num: SEVERITY_ERROR,
        severity_text: "ERROR".to_string(),
        body: body.to_string(),
        trace_id: "trace".to_string(),
        span_id: "span".to_string(),
        run_id: None,
        scope_name: "test".to_string(),
        attributes,
        resource: json!({}),
    }
}

#[test]
fn ok_span_exception_produces_span_exception_error() {
    let request = span_request(test_span(status::StatusCode::Ok as i32, true));

    let events = derive_from_traces(&request);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, ErrorSource::SpanException);
    assert_eq!(events[0].error_type, "test::Boom");
    assert_eq!(events[0].message, "boom");
    assert_eq!(events[0].stacktrace.as_deref(), Some("top\nbottom"));
}

#[test]
fn error_span_without_exception_still_produces_span_status_error() {
    let request = span_request(test_span(status::StatusCode::Error as i32, false));

    let events = derive_from_traces(&request);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, ErrorSource::SpanStatus);
    assert_eq!(events[0].error_type, "span_error");
    assert_eq!(events[0].message, "status failed");
}

#[test]
fn logs_prefer_structured_error_type_and_operation_for_fingerprint() {
    let rows = vec![
        log_row(
            "capsule attach failed for jk-alpha-demo uid 501:0 id a1b2c3d4",
            json!({
                "error.type": "jackin::CapsuleAttach",
                "jackin.operation": "capsule.attach"
            }),
        ),
        log_row(
            "capsule attach failed for jk-beta-demo uid 501:20 id de4dbeef",
            json!({
                "error.type": "jackin::CapsuleAttach",
                "jackin.operation": "capsule.attach"
            }),
        ),
        log_row(
            "capsule attach failed for jk-beta-demo uid 501:20 id de4dbeef",
            json!({
                "error.type": "jackin::CapsuleAttach",
                "jackin.operation": "capsule.detach"
            }),
        ),
    ];

    let events = derive_from_logs(&rows);

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].error_type, "jackin::CapsuleAttach");
    assert_eq!(events[0].fingerprint, events[1].fingerprint);
    assert_ne!(events[0].fingerprint, events[2].fingerprint);
}
