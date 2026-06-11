//! Error-event derivation from normalized rows — graduated from
//! `poc/evidence-loop/src/derive.rs` with identical rules: span `exception`
//! events, span ERROR status, ERROR/FATAL logs, and exception-attribute logs
//! (the post-Span-Events encoding). No fourth signal.

use crate::fingerprint::fingerprint;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::KeyValue;
use parallax_storage::model::{ErrorEventRow, ErrorSource, LogRow};

use crate::normalize::{attr_str, attributes_to_json, hex};

pub const SEVERITY_ERROR: i32 = 17;

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
        let service = attr_str(resource_attrs, "service.name")
            .unwrap_or("unknown")
            .to_string();
        for ss in &rs.scope_spans {
            for span in &ss.spans {
                let is_error = span.status.as_ref().map(|s| s.code == 2).unwrap_or(false);
                if !is_error {
                    continue;
                }
                let exception = span.events.iter().find(|e| e.name == "exception");
                let (source, error_type, message, stacktrace, ts) = match exception {
                    Some(event) => (
                        ErrorSource::SpanException,
                        attr_str(&event.attributes, "exception.type")
                            .unwrap_or("unknown")
                            .to_string(),
                        attr_str(&event.attributes, "exception.message")
                            .unwrap_or("")
                            .to_string(),
                        attr_str(&event.attributes, "exception.stacktrace").map(str::to_string),
                        u128::from(event.time_unix_nano),
                    ),
                    None => (
                        ErrorSource::SpanStatus,
                        "span_error".to_string(),
                        span.status
                            .as_ref()
                            .map(|s| s.message.clone())
                            .filter(|m| !m.is_empty())
                            .unwrap_or_else(|| span.name.clone()),
                        None,
                        u128::from(span.end_time_unix_nano),
                    ),
                };
                let fp = fingerprint(&error_type, &message, stacktrace.as_deref());
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
            .get("exception.type")
            .and_then(|v| v.as_str());
        let exception_message = row
            .attributes
            .get("exception.message")
            .and_then(|v| v.as_str());
        let has_exception_attrs = exception_type.is_some() || exception_message.is_some();
        let is_error_severity = row.severity_num >= SEVERITY_ERROR
            || matches!(row.severity_text.as_str(), "ERROR" | "FATAL");
        if !is_error_severity && !has_exception_attrs {
            continue;
        }
        let (source, error_type, message, stacktrace) = if has_exception_attrs {
            (
                ErrorSource::LogException,
                exception_type.unwrap_or("unknown").to_string(),
                exception_message.unwrap_or(&row.body).to_string(),
                row.attributes
                    .get("exception.stacktrace")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            )
        } else {
            (
                ErrorSource::LogRecord,
                "log_error".to_string(),
                row.body.clone(),
                None,
            )
        };
        let fp = fingerprint(&error_type, &message, stacktrace.as_deref());
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
