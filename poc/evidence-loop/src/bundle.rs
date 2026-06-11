//! Evidence-bundle assembly, redaction, and canonical hashing.
//!
//! Schema here is `bundle-v0-poc`: a deliberately reduced shape of the real
//! contract in docs/research/architecture/evidence-bundle-schema.md, keeping
//! its mandatory ideas — anchor, typed nodes, typed edges with strength,
//! missing-evidence report, redaction report, canonical hash.
//!
//! Canonical hash: sorted-key compact JSON ("JCS-lite": this PoC has no float
//! values, so RFC 8785 number formatting edge cases do not arise), SHA-256,
//! computed with the `canonical_hash` field absent.

use crate::derive::{ErrorEvent, ErrorSource};
use crate::otlp::{LogsData, TraceData};
use crate::redact::{redact_string, RedactionReport};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const SCHEMA_VERSION: &str = "bundle-v0-poc";

#[derive(Debug, Serialize)]
pub struct Bundle {
    pub schema_version: String,
    pub bundle_id: String,
    /// Latest observed telemetry timestamp — not wall clock, so output is
    /// reproducible from fixtures alone.
    pub generated_at_unix_nano: String,
    pub generator: Generator,
    pub project: String,
    pub anchor: Anchor,
    pub trigger: Trigger,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub missing_evidence: Vec<String>,
    pub redaction_report: RedactionReport,
    pub canonical_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Generator {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Anchor {
    pub r#type: String,
    pub fingerprint: String,
    pub service_name: String,
}

#[derive(Debug, Serialize)]
pub struct Trigger {
    pub r#type: String,
    pub dispatch_eligible: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "node_type")]
pub enum Node {
    #[serde(rename = "error_event")]
    ErrorEvent {
        id: String,
        source: ErrorSource,
        error_type: String,
        message: String,
        stacktrace: Option<String>,
        trace_id: String,
        span_id: String,
        time_unix_nano: String,
    },
    #[serde(rename = "span")]
    Span {
        id: String,
        name: String,
        trace_id: String,
        span_id: String,
        parent_span_id: Option<String>,
        status_code: String,
        duration_nano: u128,
    },
    #[serde(rename = "log_window")]
    LogWindow {
        id: String,
        trace_id: String,
        lines: Vec<String>,
    },
}

#[derive(Debug, Serialize)]
pub struct Edge {
    pub r#type: String,
    pub from: String,
    pub to: String,
    pub strength: String,
}

/// Assemble one bundle per fingerprint that has at least one error event.
pub fn build_bundles(
    project: &str,
    trace: &TraceData,
    logs: &LogsData,
    error_events: &[ErrorEvent],
) -> Vec<Bundle> {
    let mut fingerprints: Vec<String> = error_events.iter().map(|e| e.fingerprint.clone()).collect();
    fingerprints.sort();
    fingerprints.dedup();

    fingerprints
        .iter()
        .map(|fp| build_bundle(project, trace, logs, error_events, fp))
        .collect()
}

fn build_bundle(
    project: &str,
    trace: &TraceData,
    logs: &LogsData,
    all_events: &[ErrorEvent],
    fp: &str,
) -> Bundle {
    let mut report = RedactionReport::new();
    let events: Vec<&ErrorEvent> = all_events.iter().filter(|e| e.fingerprint == fp).collect();
    let anchor_event = events[0];
    let trace_id = anchor_event.trace_id.clone();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Error-event nodes (redacted).
    let mut error_node_ids = Vec::new();
    for (i, ev) in events.iter().enumerate() {
        let id = format!("err_{fp}_{i}");
        nodes.push(Node::ErrorEvent {
            id: id.clone(),
            source: ev.source.clone(),
            error_type: ev.error_type.clone(),
            message: redact_string(&ev.message, &mut report),
            stacktrace: ev.stacktrace.as_deref().map(|s| redact_string(s, &mut report)),
            trace_id: ev.trace_id.clone(),
            span_id: ev.span_id.clone(),
            time_unix_nano: ev.time_unix_nano.clone(),
        });
        error_node_ids.push(id);
    }

    // Spans of the anchoring trace, plus error_in_span edges.
    for rs in &trace.resource_spans {
        for ss in &rs.scope_spans {
            for span in &ss.spans {
                if span.trace_id != trace_id {
                    continue;
                }
                let id = format!("span_{}", span.span_id);
                let start: u128 = span.start_time_unix_nano.parse().unwrap_or(0);
                let end: u128 = span.end_time_unix_nano.parse().unwrap_or(start);
                nodes.push(Node::Span {
                    id: id.clone(),
                    name: span.name.clone(),
                    trace_id: span.trace_id.clone(),
                    span_id: span.span_id.clone(),
                    parent_span_id: span.parent_span_id.clone(),
                    status_code: span
                        .status
                        .as_ref()
                        .map(|s| s.code.clone())
                        .unwrap_or_else(|| "STATUS_CODE_UNSET".to_string()),
                    duration_nano: end.saturating_sub(start),
                });
                for (ev, err_id) in events.iter().zip(&error_node_ids) {
                    if ev.span_id == span.span_id {
                        edges.push(Edge {
                            r#type: "error_in_span".to_string(),
                            from: err_id.clone(),
                            to: id.clone(),
                            strength: "strong".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Log window for the anchoring trace (redacted), strong edge via trace_id.
    let mut lines = Vec::new();
    for rl in &logs.resource_logs {
        for sl in &rl.scope_logs {
            for rec in &sl.log_records {
                if rec.trace_id != trace_id {
                    continue;
                }
                let body = rec.body.as_ref().and_then(|b| b.as_str()).unwrap_or("");
                lines.push(redact_string(
                    &format!("{} {} {}", rec.time_unix_nano, rec.severity_text, body),
                    &mut report,
                ));
            }
        }
    }
    if !lines.is_empty() {
        let id = format!("logwin_{trace_id}");
        nodes.push(Node::LogWindow { id: id.clone(), trace_id: trace_id.clone(), lines });
        edges.push(Edge {
            r#type: "log_in_trace".to_string(),
            from: id,
            to: format!("trace_{trace_id}"),
            strength: "strong".to_string(),
        });
    }

    // same_fingerprint edges between error events (both exception encodings
    // converging on one group is the point being proven).
    for pair in error_node_ids.windows(2) {
        edges.push(Edge {
            r#type: "same_fingerprint".to_string(),
            from: pair[0].clone(),
            to: pair[1].clone(),
            strength: "strong".to_string(),
        });
    }

    let generated_at = latest_timestamp(all_events);
    let mut bundle = Bundle {
        schema_version: SCHEMA_VERSION.to_string(),
        bundle_id: format!("bndl_poc_{fp}"),
        generated_at_unix_nano: generated_at,
        generator: Generator {
            name: "evidence-loop-poc".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        project: project.to_string(),
        anchor: Anchor {
            r#type: "issue".to_string(),
            fingerprint: fp.to_string(),
            service_name: anchor_event.service_name.clone(),
        },
        trigger: Trigger {
            // PoC baseline is empty, so every fingerprint is new.
            r#type: "new_fingerprint".to_string(),
            dispatch_eligible: true,
        },
        nodes,
        edges,
        missing_evidence: vec![
            "no metric windows in fixtures".to_string(),
            "no deploy event in fixtures (deploy_adjacent_regression not evaluable)".to_string(),
        ],
        redaction_report: report,
        canonical_hash: None,
    };

    let hash = canonical_hash(&bundle);
    bundle.canonical_hash = Some(hash);
    bundle
}

fn latest_timestamp(events: &[ErrorEvent]) -> String {
    events
        .iter()
        .map(|e| e.time_unix_nano.parse::<u128>().unwrap_or(0))
        .max()
        .unwrap_or(0)
        .to_string()
}

/// SHA-256 over sorted-key compact JSON with `canonical_hash` removed.
pub fn canonical_hash(bundle: &Bundle) -> String {
    let mut value = serde_json::to_value(bundle).expect("bundle serializes");
    if let Value::Object(map) = &mut value {
        map.remove("canonical_hash");
    }
    let canonical = to_canonical_json(&value);
    let digest = Sha256::digest(canonical.as_bytes());
    format!("sha256:{}", digest.iter().map(|b| format!("{b:02x}")).collect::<String>())
}

fn to_canonical_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let inner: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", serde_json::to_string(k).unwrap(), to_canonical_json(&map[k])))
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        Value::Array(items) => {
            let inner: Vec<String> = items.iter().map(to_canonical_json).collect();
            format!("[{}]", inner.join(","))
        }
        leaf => serde_json::to_string(leaf).unwrap(),
    }
}
