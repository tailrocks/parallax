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
        invocation_id: None,
        session_id: None,
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
                "cli.command.name": "capsule.attach"
            }),
        ),
        log_row(
            "capsule attach failed for jk-beta-demo uid 501:20 id de4dbeef",
            json!({
                "error.type": "jackin::CapsuleAttach",
                "cli.command.name": "capsule.attach"
            }),
        ),
        log_row(
            "capsule attach failed for jk-beta-demo uid 501:20 id de4dbeef",
            json!({
                "error.type": "jackin::CapsuleAttach",
                "cli.command.name": "capsule.detach"
            }),
        ),
    ];

    let events = derive_from_logs(&rows);

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].error_type, "jackin::CapsuleAttach");
    assert_eq!(events[0].fingerprint, events[1].fingerprint);
    assert_ne!(events[0].fingerprint, events[2].fingerprint);
    assert_eq!(
        operation_from_json_attributes(&events[0].attributes),
        Some("capsule.attach")
    );
}

#[test]
fn exception_event_operation_is_stored_on_event_attributes() {
    let mut span = test_span(status::StatusCode::Ok as i32, true);
    span.events[0]
        .attributes
        .push(string_kv("cli.command.name", "capsule.attach"));
    let events = derive_from_traces(&span_request(span));
    assert_eq!(
        operation_from_json_attributes(&events[0].attributes),
        Some("capsule.attach")
    );
}

#[test]
fn structured_failure_has_one_identity_across_all_sources() {
    let mut exception_span = test_span(status::StatusCode::Ok as i32, true);
    exception_span.events[0]
        .attributes
        .push(string_kv("error.type", "test::Boom"));

    let mut status_span = test_span(status::StatusCode::Error as i32, false);
    status_span.status.as_mut().expect("status").message = "boom".to_string();
    status_span.attributes = vec![
        string_kv("error.type", "test::Boom"),
        string_kv("exception.stacktrace", "top\nbottom"),
    ];

    let trace_exception = derive_from_traces(&span_request(exception_span));
    let trace_status = derive_from_traces(&span_request(status_span));
    let logs = derive_from_logs(&[
        log_row(
            "boom",
            json!({
                "error.type": "test::Boom",
                "exception.message": "boom",
                "exception.stacktrace": "top\nbottom"
            }),
        ),
        log_row(
            "boom",
            json!({
                "error.type": "test::Boom",
                "exception.stacktrace": "top\nbottom"
            }),
        ),
    ]);

    let all = [&trace_exception[0], &trace_status[0], &logs[0], &logs[1]];
    assert!(all.iter().all(|event| event.error_type == "test::Boom"));
    assert!(
        all.iter()
            .all(|event| event.fingerprint == all[0].fingerprint)
    );
    assert_eq!(all[0].source, ErrorSource::SpanException);
    assert_eq!(all[1].source, ErrorSource::SpanStatus);
    assert_eq!(all[2].source, ErrorSource::LogException);
    assert_eq!(all[3].source, ErrorSource::LogRecord);

    let different_frame = derive_from_logs(&[log_row(
        "boom",
        json!({
            "error.type": "test::Boom",
            "exception.stacktrace": "other\nbottom"
        }),
    )]);
    assert_ne!(different_frame[0].fingerprint, all[0].fingerprint);
}

#[test]
fn issue_title_collapses_repeated_error_type_prefixes() {
    // Plan 159 finding: mark_span_error stores the reason as both the
    // error type and the status message, producing "x: x(: x)" titles.
    assert_eq!(
        issue_title("action_failure", "action_failure"),
        "action_failure"
    );
    assert_eq!(
        issue_title("action_failure", "action_failure: action_failure"),
        "action_failure"
    );
    assert_eq!(
        issue_title("redis::Timeout", "redis::Timeout: connect timed out"),
        "redis::Timeout: connect timed out"
    );
    assert_eq!(issue_title("E", "boom"), "E: boom");
}
