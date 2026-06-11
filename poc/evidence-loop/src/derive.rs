//! Error-event derivation from OTLP data.
//!
//! Parallax does not invent a fourth signal: error events are derived from
//! (a) spans with ERROR status carrying `exception` span events,
//! (b) spans with ERROR status alone,
//! (c) ERROR/FATAL log records,
//! (d) log records carrying `exception.*` attributes — the encoding OTel moves
//!     exceptions toward after the 2026-03 Span Events deprecation.
//! Both exception encodings (span event and log record) must converge to the
//! same fingerprint; the test suite asserts this.

use crate::fingerprint::fingerprint;
use crate::otlp::{attr, LogsData, TraceData};
use serde::Serialize;

pub const SEVERITY_ERROR: i32 = 17; // OTLP SeverityNumber: ERROR

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ErrorSource {
    SpanException,
    SpanStatus,
    LogRecord,
    LogException,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorEvent {
    pub source: ErrorSource,
    pub error_type: String,
    pub message: String,
    pub stacktrace: Option<String>,
    pub trace_id: String,
    pub span_id: String,
    pub time_unix_nano: String,
    pub service_name: String,
    pub fingerprint: String,
}

pub fn derive_from_trace(trace: &TraceData) -> Vec<ErrorEvent> {
    let mut events = Vec::new();
    for rs in &trace.resource_spans {
        let service = rs
            .resource
            .as_ref()
            .and_then(|r| attr(&r.attributes, "service.name"))
            .unwrap_or("unknown")
            .to_string();
        for ss in &rs.scope_spans {
            for span in &ss.spans {
                let is_error = span
                    .status
                    .as_ref()
                    .map(|s| s.code == "STATUS_CODE_ERROR")
                    .unwrap_or(false);
                if !is_error {
                    continue;
                }
                let exception = span.events.iter().find(|e| e.name == "exception");
                let (source, error_type, message, stacktrace, time) = match exception {
                    Some(ev) => (
                        ErrorSource::SpanException,
                        attr(&ev.attributes, "exception.type").unwrap_or("unknown").to_string(),
                        attr(&ev.attributes, "exception.message").unwrap_or("").to_string(),
                        attr(&ev.attributes, "exception.stacktrace").map(str::to_string),
                        ev.time_unix_nano.clone(),
                    ),
                    None => (
                        ErrorSource::SpanStatus,
                        "span_error".to_string(),
                        span.status
                            .as_ref()
                            .and_then(|s| s.message.clone())
                            .unwrap_or_else(|| span.name.clone()),
                        None,
                        span.end_time_unix_nano.clone(),
                    ),
                };
                let fp = fingerprint(&error_type, &message, stacktrace.as_deref());
                events.push(ErrorEvent {
                    source,
                    error_type,
                    message,
                    stacktrace,
                    trace_id: span.trace_id.clone(),
                    span_id: span.span_id.clone(),
                    time_unix_nano: time,
                    service_name: service.clone(),
                    fingerprint: fp,
                });
            }
        }
    }
    events
}

pub fn derive_from_logs(logs: &LogsData) -> Vec<ErrorEvent> {
    let mut events = Vec::new();
    for rl in &logs.resource_logs {
        let service = rl
            .resource
            .as_ref()
            .and_then(|r| attr(&r.attributes, "service.name"))
            .unwrap_or("unknown")
            .to_string();
        for sl in &rl.scope_logs {
            for rec in &sl.log_records {
                let has_exception_attrs = attr(&rec.attributes, "exception.type").is_some()
                    || attr(&rec.attributes, "exception.message").is_some();
                let is_error_severity = rec.severity_number >= SEVERITY_ERROR
                    || matches!(rec.severity_text.as_str(), "ERROR" | "FATAL");
                if !is_error_severity && !has_exception_attrs {
                    continue;
                }
                let (source, error_type, message, stacktrace) = if has_exception_attrs {
                    (
                        ErrorSource::LogException,
                        attr(&rec.attributes, "exception.type").unwrap_or("unknown").to_string(),
                        attr(&rec.attributes, "exception.message")
                            .or_else(|| rec.body.as_ref().and_then(|b| b.as_str()))
                            .unwrap_or("")
                            .to_string(),
                        attr(&rec.attributes, "exception.stacktrace").map(str::to_string),
                    )
                } else {
                    (
                        ErrorSource::LogRecord,
                        "log_error".to_string(),
                        rec.body.as_ref().and_then(|b| b.as_str()).unwrap_or("").to_string(),
                        None,
                    )
                };
                let fp = fingerprint(&error_type, &message, stacktrace.as_deref());
                events.push(ErrorEvent {
                    source,
                    error_type,
                    message,
                    stacktrace,
                    trace_id: rec.trace_id.clone(),
                    span_id: rec.span_id.clone(),
                    time_unix_nano: rec.time_unix_nano.clone(),
                    service_name: service.clone(),
                    fingerprint: fp,
                });
            }
        }
    }
    events
}
