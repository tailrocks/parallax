//! Minimal OTLP/JSON data model — only the subset this PoC consumes.
//!
//! Field names follow the OTLP JSON encoding (camelCase, nanos as strings).

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TraceData {
    #[serde(rename = "resourceSpans", default)]
    pub resource_spans: Vec<ResourceSpans>,
}

#[derive(Debug, Deserialize)]
pub struct LogsData {
    #[serde(rename = "resourceLogs", default)]
    pub resource_logs: Vec<ResourceLogs>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceSpans {
    pub resource: Option<Resource>,
    #[serde(rename = "scopeSpans", default)]
    pub scope_spans: Vec<ScopeSpans>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceLogs {
    pub resource: Option<Resource>,
    #[serde(rename = "scopeLogs", default)]
    pub scope_logs: Vec<ScopeLogs>,
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
}

#[derive(Debug, Deserialize)]
pub struct ScopeSpans {
    #[serde(default)]
    pub spans: Vec<Span>,
}

#[derive(Debug, Deserialize)]
pub struct ScopeLogs {
    #[serde(rename = "logRecords", default)]
    pub log_records: Vec<LogRecord>,
}

#[derive(Debug, Deserialize)]
pub struct Span {
    #[serde(rename = "traceId", default)]
    pub trace_id: String,
    #[serde(rename = "spanId", default)]
    pub span_id: String,
    #[serde(rename = "parentSpanId", default)]
    pub parent_span_id: Option<String>,
    pub name: String,
    #[serde(rename = "startTimeUnixNano", default)]
    pub start_time_unix_nano: String,
    #[serde(rename = "endTimeUnixNano", default)]
    pub end_time_unix_nano: String,
    pub status: Option<SpanStatus>,
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub events: Vec<SpanEvent>,
}

#[derive(Debug, Deserialize)]
pub struct SpanStatus {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    #[serde(rename = "timeUnixNano", default)]
    pub time_unix_nano: String,
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
}

#[derive(Debug, Deserialize)]
pub struct LogRecord {
    #[serde(rename = "timeUnixNano", default)]
    pub time_unix_nano: String,
    #[serde(rename = "severityNumber", default)]
    pub severity_number: i32,
    #[serde(rename = "severityText", default)]
    pub severity_text: String,
    pub body: Option<AnyValue>,
    #[serde(rename = "traceId", default)]
    pub trace_id: String,
    #[serde(rename = "spanId", default)]
    pub span_id: String,
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
}

#[derive(Debug, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: AnyValue,
}

#[derive(Debug, Deserialize)]
pub struct AnyValue {
    #[serde(rename = "stringValue")]
    pub string_value: Option<String>,
    #[serde(rename = "intValue")]
    pub int_value: Option<serde_json::Value>,
    #[serde(rename = "boolValue")]
    pub bool_value: Option<bool>,
}

impl AnyValue {
    pub fn as_str(&self) -> Option<&str> {
        self.string_value.as_deref()
    }
}

/// Look up a string attribute by key in an OTLP attribute list.
pub fn attr<'a>(attrs: &'a [KeyValue], key: &str) -> Option<&'a str> {
    attrs.iter().find(|kv| kv.key == key).and_then(|kv| kv.value.as_str())
}
