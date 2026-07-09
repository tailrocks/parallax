//! Error-event derivation from normalized rows — graduated from
//! `poc/evidence-loop/src/derive.rs` with identical rules: span `exception`
//! events, span ERROR status, ERROR/FATAL logs, and exception-attribute logs
//! (the post-Span-Events encoding). Producer-stated `error.type` and
//! `jackin.operation` attributes refine grouping when present.

use crate::fingerprint::fingerprint_with_operation;
use crate::semconv;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::KeyValue;
use parallax_storage::model::{ErrorEventRow, ErrorSource, LogRow};

use crate::normalize::{attr_str, attributes_to_json, hex};

pub const SEVERITY_ERROR: i32 = 17;

fn json_attr_str<'a>(attributes: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    attributes
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

/// Derive error events from a trace export request (span exceptions + span
/// ERROR statuses). Works on the raw request so exception span *events* are
/// visible (they are not part of `SpanRow`).
pub fn derive_from_traces(request: &ExportTraceServiceRequest) -> Vec<ErrorEventRow> {
    let mut events = Vec::new();
    for rs in &request.resource_spans {
        let resource_attrs: &[KeyValue] = rs
            .resource
            .as_ref()
            .map(|r| r.attributes.as_slice())
            .unwrap_or(&[]);
        let service = attr_str(resource_attrs, semconv::SERVICE_NAME)
            .unwrap_or("unknown")
            .to_string();
        for ss in &rs.scope_spans {
            for span in &ss.spans {
                let is_error = span.status.as_ref().map(|s| s.code == 2).unwrap_or(false);
                let exception = span
                    .events
                    .iter()
                    .find(|e| e.name == semconv::EXCEPTION_EVENT_NAME);
                let Some((source, error_type, message, stacktrace, ts, operation)) = exception
                    .map_or_else(
                        || {
                            is_error.then(|| {
                                (
                                    ErrorSource::SpanStatus,
                                    attr_str(&span.attributes, semconv::ERROR_TYPE)
                                        .unwrap_or("span_error")
                                        .to_string(),
                                    span.status
                                        .as_ref()
                                        .map(|s| s.message.clone())
                                        .filter(|m| !m.is_empty())
                                        .unwrap_or_else(|| span.name.clone()),
                                    None,
                                    u128::from(span.end_time_unix_nano),
                                    attr_str(&span.attributes, semconv::JACKIN_OPERATION)
                                        .map(str::to_string),
                                )
                            })
                        },
                        |event| {
                            let fallback_type =
                                attr_str(&event.attributes, semconv::EXCEPTION_TYPE)
                                    .unwrap_or("unknown");
                            Some((
                                ErrorSource::SpanException,
                                attr_str(&event.attributes, semconv::ERROR_TYPE)
                                    .or_else(|| attr_str(&span.attributes, semconv::ERROR_TYPE))
                                    .unwrap_or(fallback_type)
                                    .to_string(),
                                attr_str(&event.attributes, semconv::EXCEPTION_MESSAGE)
                                    .unwrap_or("")
                                    .to_string(),
                                attr_str(&event.attributes, semconv::EXCEPTION_STACKTRACE)
                                    .map(str::to_string),
                                u128::from(event.time_unix_nano),
                                attr_str(&event.attributes, semconv::JACKIN_OPERATION)
                                    .or_else(|| {
                                        attr_str(&span.attributes, semconv::JACKIN_OPERATION)
                                    })
                                    .map(str::to_string),
                            ))
                        },
                    )
                else {
                    continue;
                };
                let fp = fingerprint_with_operation(
                    &error_type,
                    &message,
                    stacktrace.as_deref(),
                    operation.as_deref(),
                );
                events.push(ErrorEventRow {
                    ts_nanos: ts,
                    service: service.clone(),
                    fingerprint: fp,
                    error_type,
                    message,
                    stacktrace,
                    source,
                    trace_id: hex(&span.trace_id),
                    span_id: hex(&span.span_id),
                    attributes: attributes_to_json(&span.attributes),
                });
            }
        }
    }
    events
}

/// Derive error events from normalized log rows (ERROR/FATAL severity and the
/// exception-as-log encoding).
pub fn derive_from_logs(rows: &[LogRow]) -> Vec<ErrorEventRow> {
    let mut events = Vec::new();
    for row in rows {
        let exception_type = row
            .attributes
            .get(semconv::EXCEPTION_TYPE)
            .and_then(|v| v.as_str());
        let exception_message = row
            .attributes
            .get(semconv::EXCEPTION_MESSAGE)
            .and_then(|v| v.as_str());
        let has_exception_attrs = exception_type.is_some() || exception_message.is_some();
        let error_severity = row.severity_num >= SEVERITY_ERROR
            || matches!(row.severity_text.as_str(), "ERROR" | "FATAL");
        if !error_severity && !has_exception_attrs {
            continue;
        }
        let structured_error_type = json_attr_str(&row.attributes, semconv::ERROR_TYPE);
        let operation = json_attr_str(&row.attributes, semconv::JACKIN_OPERATION);
        let (source, error_type, message, stacktrace) = if has_exception_attrs {
            (
                ErrorSource::LogException,
                structured_error_type
                    .unwrap_or_else(|| exception_type.unwrap_or("unknown"))
                    .to_string(),
                exception_message.unwrap_or(&row.body).to_string(),
                row.attributes
                    .get(semconv::EXCEPTION_STACKTRACE)
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            )
        } else {
            (
                ErrorSource::LogRecord,
                structured_error_type.unwrap_or("log_error").to_string(),
                row.body.clone(),
                None,
            )
        };
        let fp =
            fingerprint_with_operation(&error_type, &message, stacktrace.as_deref(), operation);
        events.push(ErrorEventRow {
            ts_nanos: row.ts_nanos,
            service: row.service.clone(),
            fingerprint: fp,
            error_type,
            message,
            stacktrace,
            source,
            trace_id: row.trace_id.clone(),
            span_id: row.span_id.clone(),
            attributes: row.attributes.clone(),
        });
    }
    events
}

/// Issue title: `error_type: first line of the normalized-ish message`.
pub fn issue_title(error_type: &str, message: &str) -> String {
    let head = message.lines().next().unwrap_or("").trim();
    if head.is_empty() {
        error_type.to_string()
    } else {
        format!("{error_type}: {head}")
    }
}

/// Culprit: the top stack frame, when a stacktrace exists.
pub fn culprit(stacktrace: Option<&str>) -> Option<String> {
    stacktrace
        .and_then(|s| s.lines().next())
        .map(|l| l.trim().to_string())
}

#[cfg(test)]
mod tests {
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
}
