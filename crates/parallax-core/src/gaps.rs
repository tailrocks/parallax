//! Deterministic detection of missing telemetry evidence.

use parallax_storage::model::{LogRow, SpanRow};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceGap {
    pub kind: String,
    pub subject: String,
    pub detail: String,
}

pub fn detect_gaps(spans: &[SpanRow], logs: &[LogRow]) -> Vec<EvidenceGap> {
    let span_ids: HashSet<&str> = spans.iter().map(|span| span.span_id.as_str()).collect();
    let span_keys: HashSet<(&str, &str)> = spans
        .iter()
        .map(|span| (span.trace_id.as_str(), span.span_id.as_str()))
        .collect();
    let mut gaps = Vec::new();

    for span in spans {
        if let Some(parent_id) = span.parent_span_id.as_deref().filter(|id| !id.is_empty())
            && !span_ids.contains(parent_id)
        {
            gaps.push(EvidenceGap {
                kind: "orphan_span".to_string(),
                subject: span.span_id.clone(),
                detail: "parent span missing from fetched evidence; this may be a legitimate cross-service root".to_string(),
            });
        }

        if span.kind == "SPAN_KIND_PRODUCER" && producer_target_missing(span, &span_keys) {
            gaps.push(EvidenceGap {
                kind: "producer_without_consumer".to_string(),
                subject: span.span_id.clone(),
                detail: "producer span has no linked consumer in fetched evidence".to_string(),
            });
        }

        if is_browser_or_client_span(span) && !has_child_server(span, spans) {
            gaps.push(EvidenceGap {
                kind: "browser_without_backend".to_string(),
                subject: span.span_id.clone(),
                detail: "browser/client span has no child server span in fetched evidence"
                    .to_string(),
            });
        }
    }

    for (index, log) in logs.iter().enumerate() {
        if !valid_trace_id(&log.trace_id) {
            gaps.push(EvidenceGap {
                kind: "log_without_trace".to_string(),
                subject: format!("log:{}:{index}", log.ts_nanos),
                detail: "log record has no valid trace id".to_string(),
            });
        }
    }

    gaps
}

fn valid_trace_id(trace_id: &str) -> bool {
    !trace_id.is_empty() && trace_id.chars().any(|c| c != '0')
}

fn producer_target_missing(span: &SpanRow, span_keys: &HashSet<(&str, &str)>) -> bool {
    let Some(links) = span.links.as_array() else {
        return true;
    };
    if links.is_empty() {
        return true;
    }
    links.iter().any(|link| {
        let trace_id = link
            .get("traceId")
            .or_else(|| link.get("trace_id"))
            .and_then(Value::as_str)
            .unwrap_or(span.trace_id.as_str());
        let span_id = link
            .get("spanId")
            .or_else(|| link.get("span_id"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        span_id.is_empty() || !span_keys.contains(&(trace_id, span_id))
    })
}

fn has_child_server(span: &SpanRow, spans: &[SpanRow]) -> bool {
    spans.iter().any(|candidate| {
        candidate.parent_span_id.as_deref() == Some(span.span_id.as_str())
            && candidate.kind == "SPAN_KIND_SERVER"
    })
}

fn is_browser_or_client_span(span: &SpanRow) -> bool {
    let resource_language = span
        .resource
        .get("telemetry.sdk.language")
        .and_then(Value::as_str);
    if resource_language == Some("webjs") {
        return true;
    }
    span.kind == "SPAN_KIND_CLIENT"
        && (has_key_prefix(&span.attributes, "http") || has_key_prefix(&span.attributes, "url"))
}

fn has_key_prefix(attributes: &Value, prefix: &str) -> bool {
    attributes
        .as_object()
        .is_some_and(|attrs| attrs.keys().any(|key| key.starts_with(prefix)))
}

#[cfg(test)]
mod tests;
