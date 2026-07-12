use super::*;
use crate::adapter::{TelemetryStore, TraceQuery, TraceSort};

fn span(trace: &str, span_id: &str, parent: Option<&str>, service: &str, ts: u128) -> SpanRow {
    SpanRow {
        ts_nanos: ts,
        service: service.into(),
        trace_id: trace.into(),
        span_id: span_id.into(),
        parent_span_id: parent.map(Into::into),
        name: format!("{service}-{span_id}"),
        kind: "SPAN_KIND_INTERNAL".into(),
        status_code: "STATUS_CODE_UNSET".into(),
        status_message: String::new(),
        duration_ns: 1_000,
        run_id: None,
        scope_name: String::new(),
        events: None,
        links: serde_json::Value::Null,
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

fn span_with_duration(
    trace: &str,
    span_id: &str,
    parent: Option<&str>,
    service: &str,
    ts: u128,
    duration_ns: u128,
) -> SpanRow {
    let mut row = span(trace, span_id, parent, service, ts);
    row.duration_ns = duration_ns;
    row
}

fn error_event(service: &str, ts: u128) -> ErrorEventRow {
    ErrorEventRow {
        ts_nanos: ts,
        service: service.into(),
        fingerprint: format!("{service}-fp"),
        error_type: "Error".into(),
        message: "boom".into(),
        stacktrace: None,
        source: ErrorSource::SpanStatus,
        trace_id: format!("{service}-trace"),
        span_id: format!("{service}-span"),
        attributes: serde_json::Value::Null,
    }
}

fn log(run_id: Option<&str>, ts: u128, severity_num: i32) -> LogRow {
    LogRow {
        ts_nanos: ts,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: "api".into(),
        severity_num,
        severity_text: format!("S{severity_num}"),
        body: format!("log-{ts}"),
        trace_id: format!("trace-{ts}"),
        span_id: format!("span-{ts}"),
        run_id: run_id.map(Into::into),
        scope_name: String::new(),
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

fn query(service: Option<&str>) -> TraceQuery {
    TraceQuery {
        service: service.map(Into::into),
        limit: 50,
        ..Default::default()
    }
}

fn span_with_attrs(trace: &str, span_id: &str, ts: u128, attrs: serde_json::Value) -> SpanRow {
    let mut row = span(trace, span_id, None, "checkout", ts);
    row.attributes = attrs;
    row
}

fn span_with_release(trace: &str, span_id: &str, ts: u128, version: &str) -> SpanRow {
    let mut row = span(trace, span_id, None, "checkout", ts);
    row.resource = serde_json::json!({ "service.version": version });
    row
}

fn span_with_resource(
    trace: &str,
    span_id: &str,
    service: &str,
    ts: u128,
    resource: serde_json::Value,
) -> SpanRow {
    let mut row = span(trace, span_id, None, service, ts);
    row.resource = resource;
    row
}

include!("tests/fields_metrics.rs");
include!("tests/traces_services.rs");
