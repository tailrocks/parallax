#![expect(clippy::excessive_nesting, reason = "measured legacy OTLP traversal")]

//! Error-event derivation from normalized rows — graduated from
//! `poc/evidence-loop/src/derive.rs` with identical rules: span `exception`
//! events, span ERROR status, ERROR/FATAL logs, and exception-attribute logs
//! (the post-Span-Events encoding). Producer-stated `error.type` and
//! `cli.command.name` attributes refine grouping when present.

use crate::fingerprint::fingerprint_with_operation;
use crate::semconv;
use parallax_model::{ErrorEventRow, ErrorSource, LogRow};
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};

pub const SEVERITY_ERROR: i32 = 17;

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn attr_str<'a>(attributes: &'a [KeyValue], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|item| item.key == key)
        .and_then(|item| match item.value.as_ref()?.value.as_ref()? {
            AnyValueEnum::StringValue(value) => Some(value.as_str()),
            _ => None,
        })
}

fn attributes_to_json(attributes: &[KeyValue]) -> serde_json::Value {
    serde_json::Value::Object(
        attributes
            .iter()
            .map(|item| {
                let value = item
                    .value
                    .as_ref()
                    .map_or(serde_json::Value::Null, any_value_to_json);
                (item.key.clone(), value)
            })
            .collect(),
    )
}

fn any_value_to_json(value: &AnyValue) -> serde_json::Value {
    match &value.value {
        Some(AnyValueEnum::StringValue(value)) => value.clone().into(),
        Some(AnyValueEnum::BoolValue(value)) => (*value).into(),
        Some(AnyValueEnum::IntValue(value)) => (*value).into(),
        Some(AnyValueEnum::DoubleValue(value)) => serde_json::Number::from_f64(*value)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        Some(AnyValueEnum::BytesValue(value)) => hex(value).into(),
        Some(AnyValueEnum::ArrayValue(value)) => {
            value.values.iter().map(any_value_to_json).collect()
        }
        Some(AnyValueEnum::KvlistValue(value)) => attributes_to_json(&value.values),
        Some(_) | None => serde_json::Value::Null,
    }
}

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
#[must_use]
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
                let is_error = span.status.as_ref().is_some_and(|s| s.code == 2);
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
                                        .or_else(|| {
                                            attr_str(&span.attributes, semconv::EXCEPTION_TYPE)
                                        })
                                        .unwrap_or("span_error")
                                        .to_string(),
                                    span.status
                                        .as_ref()
                                        .map(|s| s.message.clone())
                                        .filter(|m| !m.is_empty())
                                        .unwrap_or_else(|| span.name.clone()),
                                    attr_str(&span.attributes, semconv::EXCEPTION_STACKTRACE)
                                        .map(str::to_string),
                                    u128::from(span.end_time_unix_nano),
                                    attr_str(&span.attributes, semconv::CLI_COMMAND_NAME)
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
                                attr_str(&event.attributes, semconv::CLI_COMMAND_NAME)
                                    .or_else(|| {
                                        attr_str(&span.attributes, semconv::CLI_COMMAND_NAME)
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
        let operation = json_attr_str(&row.attributes, semconv::CLI_COMMAND_NAME);
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
                json_attr_str(&row.attributes, semconv::EXCEPTION_STACKTRACE).map(str::to_string),
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
/// ANSI escapes are stripped — colored CLI output titles like its plain form.
#[must_use]
pub fn issue_title(error_type: &str, message: &str) -> String {
    let clean = crate::fingerprint::strip_ansi(message);
    let head = clean.lines().next().unwrap_or("").trim();
    if head.is_empty() {
        error_type.to_string()
    } else {
        format!("{error_type}: {head}")
    }
}

/// Culprit: the top stack frame, when a stacktrace exists.
#[must_use]
pub fn culprit(stacktrace: Option<&str>) -> Option<String> {
    stacktrace
        .and_then(|s| s.lines().next())
        .map(|l| l.trim().to_string())
}

#[cfg(test)]
mod tests;
