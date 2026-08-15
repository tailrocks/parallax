//! Normalize Sentry event payloads into Parallax `ErrorEventRow` (plan 118).
//!
//! Pure mapping: no HTTP, spool, or store I/O. Fingerprints use the same
//! deterministic function as OTLP-derived errors so cross-source echoes can
//! share an issue when type/message/top-frame match.

use crate::fingerprint::fingerprint_with_operation;
use parallax_model::{ErrorEventRow, ErrorSource};
use serde_json::{Map, Value};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Map one accepted Sentry event object into a single error row.
///
/// Returns `None` only when the JSON is not an object (caller should have
/// already rejected non-objects at the envelope boundary).
#[must_use]
pub fn derive_from_sentry_event(event: &Value) -> Option<ErrorEventRow> {
    let object = event.as_object()?;
    let (error_type, message, stacktrace) = exception_fields(object);
    let explicit_fp = explicit_fingerprint(object);
    let operation = object
        .get("transaction")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    // Explicit SDK fingerprint is part of grouping when present and non-default.
    let operation = explicit_fp.or(operation);
    let fingerprint = fingerprint_with_operation(
        &error_type,
        &message,
        stacktrace.as_deref(),
        operation.as_deref(),
    );
    let service = service_name(object);
    let (trace_id, span_id) = trace_ids(object);
    let ts_nanos = timestamp_nanos(object);
    let attributes = bounded_attributes(object);

    Some(ErrorEventRow {
        ts_nanos,
        service,
        fingerprint,
        error_type,
        message,
        stacktrace,
        source: ErrorSource::SentryEnvelope,
        trace_id,
        span_id,
        attributes,
    })
}

fn exception_fields(object: &Map<String, Value>) -> (String, String, Option<String>) {
    if let Some(values) = object
        .get("exception")
        .and_then(|v| v.get("values"))
        .and_then(Value::as_array)
        && let Some(first) = values.first().and_then(Value::as_object)
    {
        let error_type = first
            .get("type")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or("unknown")
            .to_string();
        let message = first
            .get("value")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .or_else(|| message_field(object))
            .unwrap_or_default();
        let stacktrace = frames_to_stacktrace(first.get("stacktrace"));
        return (error_type, message, stacktrace);
    }

    let error_type = object
        .get("level")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("error")
        .to_string();
    let message = message_field(object).unwrap_or_default();
    (error_type, message, None)
}

fn message_field(object: &Map<String, Value>) -> Option<String> {
    match object.get("message") {
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }
        Some(Value::Object(map)) => map
            .get("formatted")
            .or_else(|| map.get("message"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        _ => None,
    }
}

/// Sentry frames are oldest → newest; Parallax top-frame is newest first.
fn frames_to_stacktrace(stacktrace: Option<&Value>) -> Option<String> {
    let frames = stacktrace
        .and_then(|v| v.get("frames"))
        .and_then(Value::as_array)?;
    if frames.is_empty() {
        return None;
    }
    let mut lines = Vec::with_capacity(frames.len());
    for frame in frames.iter().rev() {
        let Some(frame) = frame.as_object() else {
            continue;
        };
        let function = frame
            .get("function")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let filename = frame
            .get("filename")
            .or_else(|| frame.get("abs_path"))
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let line = frame
            .get("lineno")
            .and_then(Value::as_u64)
            .map(|n| format!(":{n}"))
            .unwrap_or_default();
        lines.push(format!("{function} ({filename}{line})"));
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn explicit_fingerprint(object: &Map<String, Value>) -> Option<String> {
    let values = object.get("fingerprint").and_then(Value::as_array)?;
    let parts: Vec<&str> = values
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty() && *v != "{{ default }}")
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("|"))
    }
}

fn service_name(object: &Map<String, Value>) -> String {
    if let Some(tags) = object.get("tags").and_then(Value::as_object) {
        for key in ["service", "service.name", "server_name"] {
            if let Some(value) = tags
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
            {
                return value.to_string();
            }
        }
    }
    object
        .get("server_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

fn trace_ids(object: &Map<String, Value>) -> (String, String) {
    let trace = object
        .get("contexts")
        .and_then(|v| v.get("trace"))
        .and_then(Value::as_object);
    let trace_id = trace
        .and_then(|t| t.get("trace_id"))
        .and_then(Value::as_str)
        .map(normalize_hex_id)
        .unwrap_or_default();
    let span_id = trace
        .and_then(|t| t.get("span_id"))
        .and_then(Value::as_str)
        .map(normalize_hex_id)
        .filter(|id| !id.is_empty())
        .or_else(|| {
            // Sentry event_id is 32 hex; use as synthetic span_id when no OTel
            // context so occurrence_id stays stable and non-empty.
            object
                .get("event_id")
                .and_then(Value::as_str)
                .map(normalize_hex_id)
                .filter(|id| id.len() == 32)
        })
        .unwrap_or_default();
    // Prefer empty trace over fabricating one; worker skips empty as
    // non-linkable. Cross-source OTLP correlation requires real IDs.
    (trace_id, span_id)
}

fn normalize_hex_id(raw: &str) -> String {
    raw.chars()
        .filter(|c| *c != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn timestamp_nanos(object: &Map<String, Value>) -> u128 {
    match object.get("timestamp") {
        Some(Value::Number(n)) => {
            if let Some(f) = n.as_f64() {
                // Sentry timestamps are epoch seconds (possibly fractional).
                if f.is_finite() && f > 0.0 {
                    return seconds_to_nanos(f);
                }
            }
            if let Some(i) = n.as_u64() {
                // Heuristic: values that look like millis vs seconds.
                return if i > 10_000_000_000 {
                    u128::from(i) * 1_000_000
                } else {
                    u128::from(i) * 1_000_000_000
                };
            }
            0
        }
        Some(Value::String(s)) => parse_timestamp_string(s.trim()).unwrap_or(0),
        _ => 0,
    }
}

fn parse_timestamp_string(raw: &str) -> Option<u128> {
    if let Ok(secs) = raw.parse::<f64>()
        && secs.is_finite()
        && secs > 0.0
    {
        return Some(seconds_to_nanos(secs));
    }
    let parsed = OffsetDateTime::parse(raw, &Rfc3339).ok()?;
    let secs = u128::try_from(parsed.unix_timestamp()).ok()?;
    Some(secs.saturating_mul(1_000_000_000) + u128::from(parsed.nanosecond()))
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Sentry event timestamps are epoch seconds; nanos fit u128 for realistic ranges"
)]
fn seconds_to_nanos(secs: f64) -> u128 {
    (secs * 1_000_000_000.0) as u128
}

/// Keep only low-risk structured fields. No request bodies, cookies, or
/// user PII by default (plan 118 redaction gate still owns full fail-closed).
fn bounded_attributes(object: &Map<String, Value>) -> Value {
    let mut out = Map::new();
    out.insert(
        "parallax.source".into(),
        Value::String("sentry_envelope".into()),
    );
    if let Some(event_id) = object.get("event_id").and_then(Value::as_str) {
        out.insert(
            "sentry.event_id".into(),
            Value::String(normalize_hex_id(event_id)),
        );
    }
    for key in ["platform", "level", "release", "environment", "logger"] {
        if let Some(value) = object
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            out.insert(format!("sentry.{key}"), Value::String(value.to_string()));
        }
    }
    if let Some(tags) = object.get("tags").and_then(Value::as_object) {
        let tag_map = copy_safe_tags(tags);
        if !tag_map.is_empty() {
            out.insert("sentry.tags".into(), Value::Object(tag_map));
        }
    }
    Value::Object(out)
}

fn copy_safe_tags(tags: &Map<String, Value>) -> Map<String, Value> {
    let mut tag_map = Map::new();
    for (k, v) in tags.iter().take(32) {
        let Some(s) = v.as_str() else {
            continue;
        };
        // Bound tag values; never copy secret-shaped keys wholesale.
        if k.len() <= 64 && s.len() <= 256 && !looks_sensitive_key(k) {
            tag_map.insert(k.clone(), Value::String(s.to_string()));
        }
    }
    tag_map
}

fn looks_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("authorization")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("dsn")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn exception_event_maps_type_message_stack_and_trace() {
        let event = json!({
            "event_id": "9ec79c33ec9942ab8353589fcb2e04dc",
            "timestamp": 1_700_000_000.5,
            "level": "error",
            "platform": "native",
            "server_name": "payments",
            "exception": {
                "values": [{
                    "type": "Panic",
                    "value": "boom after 2000ms",
                    "stacktrace": {
                        "frames": [
                            {"function": "main", "filename": "src/main.rs", "lineno": 10},
                            {"function": "handle", "filename": "src/lib.rs", "lineno": 42}
                        ]
                    }
                }]
            },
            "contexts": {
                "trace": {
                    "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
                    "span_id": "00f067aa0ba902b7"
                }
            },
            "tags": {"service": "checkout"}
        });
        let row = derive_from_sentry_event(&event).expect("row");
        assert_eq!(row.error_type, "Panic");
        assert_eq!(row.message, "boom after 2000ms");
        assert_eq!(row.service, "checkout");
        assert_eq!(row.source, ErrorSource::SentryEnvelope);
        assert_eq!(row.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(row.span_id, "00f067aa0ba902b7");
        assert_eq!(row.ts_nanos, 1_700_000_000_500_000_000);
        let stack = row.stacktrace.expect("stack");
        // Newest frame first.
        assert!(stack.starts_with("handle (src/lib.rs:42)"), "{stack}");
        assert!(stack.contains("main (src/main.rs:10)"), "{stack}");
        // Same fingerprint function as OTLP path for cross-source grouping.
        let expected = fingerprint_with_operation("Panic", "boom after 2000ms", Some(&stack), None);
        assert_eq!(row.fingerprint, expected);
    }

    #[test]
    fn message_only_event_uses_level_as_type() {
        let event = json!({
            "event_id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "message": {"formatted": "hello world"},
            "level": "fatal",
            "platform": "native"
        });
        let row = derive_from_sentry_event(&event).expect("row");
        assert_eq!(row.error_type, "fatal");
        assert_eq!(row.message, "hello world");
        assert_eq!(row.span_id, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert!(row.stacktrace.is_none());
    }

    #[test]
    fn explicit_fingerprint_changes_grouping() {
        let base = json!({
            "event_id": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "message": "same",
            "level": "error"
        });
        let with_fp = json!({
            "event_id": "cccccccccccccccccccccccccccccccc",
            "message": "same",
            "level": "error",
            "fingerprint": ["payments", "timeout"]
        });
        let a = derive_from_sentry_event(&base).expect("a");
        let b = derive_from_sentry_event(&with_fp).expect("b");
        assert_ne!(a.fingerprint, b.fingerprint);
    }

    #[test]
    fn sensitive_tags_are_dropped() {
        let event = json!({
            "event_id": "dddddddddddddddddddddddddddddddd",
            "message": "x",
            "level": "error",
            "tags": {
                "env": "prod",
                "api_token": "should-not-appear",
                "dsn": "https://u:p@example.com/1"
            }
        });
        let row = derive_from_sentry_event(&event).expect("row");
        let tags = row.attributes.get("sentry.tags").expect("tags");
        assert_eq!(tags.get("env").and_then(Value::as_str), Some("prod"));
        assert!(tags.get("api_token").is_none());
        assert!(tags.get("dsn").is_none());
    }

    #[test]
    fn rfc3339_timestamp_from_java_sdk_maps_to_nanos() {
        let event = json!({"event_id":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","timestamp":"2023-11-15T12:00:00.500Z","message":"c8-java-sdk PaymentError","level":"error"});
        assert_eq!(
            derive_from_sentry_event(&event).expect("row").ts_nanos,
            1_700_049_600_500_000_000
        );
    }
}
