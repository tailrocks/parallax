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

use crate::deploy::{find_adjacent_deploy, DeployEvent, ADJACENCY_WINDOW_NANOS};
use crate::derive::{ErrorEvent, ErrorSource};
use crate::otlp::{attr, LogsData, TraceData};
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
    /// Ranked, evidence-cited hypotheses. Populated by
    /// `hypothesis::attach_hypotheses`; empty until then.
    pub hypotheses: Vec<crate::hypothesis::Hypothesis>,
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
        service_name: String,
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
    #[serde(rename = "deploy")]
    Deploy {
        id: String,
        release: String,
        vcs_sha: String,
        environment: String,
        finished_at_unix_nano: String,
    },
    #[serde(rename = "cli_invocation")]
    CliInvocation {
        id: String,
        service_name: String,
        command_line: String,
        exit_code: Option<String>,
        span_id: String,
    },
    #[serde(rename = "agent_session")]
    AgentSession {
        id: String,
        tool: String,
        repo: String,
        vcs_sha: String,
        started_at_unix_nano: String,
        ended_at_unix_nano: String,
    },
    #[serde(rename = "agent_action")]
    AgentAction {
        id: String,
        session_id: String,
        seq: u32,
        action_type: String,
        target: String,
        detail: String,
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
    deploys: &[DeployEvent],
    error_events: &[ErrorEvent],
) -> Vec<Bundle> {
    let mut fingerprints: Vec<String> = error_events.iter().map(|e| e.fingerprint.clone()).collect();
    fingerprints.sort();
    fingerprints.dedup();

    fingerprints
        .iter()
        .map(|fp| build_bundle(project, trace, logs, deploys, error_events, fp))
        .collect()
}

/// Push error-event nodes (redacted) for `events`, returning their node ids.
fn push_error_nodes(
    events: &[&ErrorEvent],
    id_prefix: &str,
    nodes: &mut Vec<Node>,
    report: &mut RedactionReport,
) -> Vec<String> {
    let mut error_node_ids = Vec::new();
    for (i, ev) in events.iter().enumerate() {
        let id = format!("err_{id_prefix}_{i}");
        nodes.push(Node::ErrorEvent {
            id: id.clone(),
            source: ev.source.clone(),
            error_type: ev.error_type.clone(),
            message: redact_string(&ev.message, report),
            stacktrace: ev.stacktrace.as_deref().map(|s| redact_string(s, report)),
            trace_id: ev.trace_id.clone(),
            span_id: ev.span_id.clone(),
            time_unix_nano: ev.time_unix_nano.clone(),
        });
        error_node_ids.push(id);
    }
    error_node_ids
}

fn build_bundle(
    project: &str,
    trace: &TraceData,
    logs: &LogsData,
    deploys: &[DeployEvent],
    all_events: &[ErrorEvent],
    fp: &str,
) -> Bundle {
    let mut report = RedactionReport::new();
    let events: Vec<&ErrorEvent> = all_events.iter().filter(|e| e.fingerprint == fp).collect();
    let anchor_event = events[0];
    let trace_id = anchor_event.trace_id.clone();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let error_node_ids = push_error_nodes(&events, fp, &mut nodes, &mut report);

    // Spans of the anchoring trace — across every emitting service/tier —
    // plus error_in_span edges and span_child_of topology edges. Cross-tier
    // reconstruction works because the browser and the backend share one
    // trace_id; the bundle simply includes both resources' spans.
    let mut span_ids_in_bundle: Vec<(String, Option<String>)> = Vec::new();
    for rs in &trace.resource_spans {
        let span_service = rs
            .resource
            .as_ref()
            .and_then(|r| attr(&r.attributes, "service.name"))
            .unwrap_or("unknown")
            .to_string();
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
                    service_name: span_service.clone(),
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
                span_ids_in_bundle.push((span.span_id.clone(), span.parent_span_id.clone()));
                // CLI invocations are first-class execution evidence: a span
                // carrying process.command_line becomes a cli_invocation node.
                if let Some(command_line) = attr(&span.attributes, "process.command_line") {
                    let inv_id = format!("cli_{}", span.span_id);
                    nodes.push(Node::CliInvocation {
                        id: inv_id.clone(),
                        service_name: span_service.clone(),
                        command_line: command_line.to_string(),
                        exit_code: attr(&span.attributes, "process.exit_code").map(str::to_string),
                        span_id: span.span_id.clone(),
                    });
                    for (ev, err_id) in events.iter().zip(&error_node_ids) {
                        if ev.span_id == span.span_id {
                            edges.push(Edge {
                                r#type: "error_in_invocation".to_string(),
                                from: err_id.clone(),
                                to: inv_id.clone(),
                                strength: "strong".to_string(),
                            });
                        }
                    }
                }
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
    for (span_id, parent) in &span_ids_in_bundle {
        if let Some(parent_id) = parent {
            if span_ids_in_bundle.iter().any(|(s, _)| s == parent_id) {
                edges.push(Edge {
                    r#type: "span_child_of".to_string(),
                    from: format!("span_{span_id}"),
                    to: format!("span_{parent_id}"),
                    strength: "strong".to_string(),
                });
            }
        }
    }

    // Log window for the anchoring trace (redacted), strong edge via
    // trace_id. Lines are service-tagged so a cross-tier window reads
    // coherently: browser breadcrumbs and backend logs interleave.
    let mut lines = Vec::new();
    for rl in &logs.resource_logs {
        let log_service = rl
            .resource
            .as_ref()
            .and_then(|r| attr(&r.attributes, "service.name"))
            .unwrap_or("unknown");
        for sl in &rl.scope_logs {
            for rec in &sl.log_records {
                if rec.trace_id != trace_id {
                    continue;
                }
                let body = rec.body.as_ref().and_then(|b| b.as_str()).unwrap_or("");
                lines.push(redact_string(
                    &format!("{} {} [{}] {}", rec.time_unix_nano, rec.severity_text, log_service, body),
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

    // Deploy adjacency: escalates the trigger and adds a deploy node. Edge is
    // strong when the deployed SHA matches the erroring service's
    // vcs.ref.head.revision resource attribute, medium on time adjacency alone
    // (strength tiers per docs/research/capture/deploy-change-context.md).
    let first_error_nanos = events
        .iter()
        .map(|e| e.time_unix_nano.parse::<u128>().unwrap_or(0))
        .min()
        .unwrap_or(0);
    let adjacent_deploy = find_adjacent_deploy(deploys, first_error_nanos, ADJACENCY_WINDOW_NANOS);
    let mut missing_evidence = vec!["no metric windows in fixtures".to_string()];
    if let Some(deploy) = adjacent_deploy {
        let id = format!("deploy_{}", deploy.release);
        let resource_revision = trace
            .resource_spans
            .first()
            .and_then(|rs| rs.resource.as_ref())
            .and_then(|r| attr(&r.attributes, "vcs.ref.head.revision"));
        let strength = if resource_revision == Some(deploy.vcs_sha.as_str()) {
            "strong"
        } else {
            "medium"
        };
        nodes.push(Node::Deploy {
            id: id.clone(),
            release: deploy.release.clone(),
            vcs_sha: deploy.vcs_sha.clone(),
            environment: deploy.environment.clone(),
            finished_at_unix_nano: deploy.finished_at_unix_nano.clone(),
        });
        edges.push(Edge {
            r#type: "deploy_preceded_issue".to_string(),
            from: id,
            to: error_node_ids[0].clone(),
            strength: strength.to_string(),
        });
    } else {
        missing_evidence
            .push("no deploy event within adjacency window (deploy_adjacent_regression not evaluable)".to_string());
    }
    let trigger_type = if adjacent_deploy.is_some() {
        "deploy_adjacent_regression"
    } else {
        "new_fingerprint"
    };

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
            // PoC baseline is empty, so every fingerprint is at least new;
            // deploy adjacency escalates it.
            r#type: trigger_type.to_string(),
            dispatch_eligible: true,
        },
        nodes,
        edges,
        hypotheses: Vec::new(),
        missing_evidence,
        redaction_report: report,
        canonical_hash: None,
    };

    let hash = canonical_hash(&bundle);
    bundle.canonical_hash = Some(hash);
    bundle
}

/// Run-anchored bundle: everything one `parallax.run.id` produced — the
/// local-first `parallax run inspect` shape. Anchored on the run, not a
/// fingerprint; trigger is `manual` (human/agent-requested) and never
/// dispatch-eligible. Returns None when the run id is unknown.
pub fn build_run_bundle(
    project: &str,
    trace: &TraceData,
    logs: &LogsData,
    all_events: &[ErrorEvent],
    run_id: &str,
) -> Option<Bundle> {
    use std::collections::BTreeSet;
    let mut report = RedactionReport::new();

    // Trace ids and anchoring service under this run id (resource-tagged).
    let mut run_trace_ids: BTreeSet<String> = BTreeSet::new();
    let mut service = "unknown".to_string();
    for rs in &trace.resource_spans {
        let tagged = rs
            .resource
            .as_ref()
            .and_then(|r| attr(&r.attributes, "parallax.run.id"))
            .is_some_and(|id| id == run_id);
        if !tagged {
            continue;
        }
        if let Some(name) = rs.resource.as_ref().and_then(|r| attr(&r.attributes, "service.name")) {
            service = name.to_string();
        }
        for ss in &rs.scope_spans {
            for span in &ss.spans {
                run_trace_ids.insert(span.trace_id.clone());
            }
        }
    }
    if run_trace_ids.is_empty() {
        return None;
    }

    let events: Vec<&ErrorEvent> =
        all_events.iter().filter(|e| run_trace_ids.contains(&e.trace_id)).collect();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let error_node_ids = push_error_nodes(&events, run_id, &mut nodes, &mut report);

    let mut span_ids_in_bundle: Vec<(String, Option<String>)> = Vec::new();
    for rs in &trace.resource_spans {
        let span_service = rs
            .resource
            .as_ref()
            .and_then(|r| attr(&r.attributes, "service.name"))
            .unwrap_or("unknown")
            .to_string();
        for ss in &rs.scope_spans {
            for span in &ss.spans {
                if !run_trace_ids.contains(&span.trace_id) {
                    continue;
                }
                let id = format!("span_{}", span.span_id);
                let start: u128 = span.start_time_unix_nano.parse().unwrap_or(0);
                let end: u128 = span.end_time_unix_nano.parse().unwrap_or(start);
                nodes.push(Node::Span {
                    id: id.clone(),
                    name: span.name.clone(),
                    service_name: span_service.clone(),
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
                span_ids_in_bundle.push((span.span_id.clone(), span.parent_span_id.clone()));
                if let Some(command_line) = attr(&span.attributes, "process.command_line") {
                    let inv_id = format!("cli_{}", span.span_id);
                    nodes.push(Node::CliInvocation {
                        id: inv_id.clone(),
                        service_name: span_service.clone(),
                        command_line: command_line.to_string(),
                        exit_code: attr(&span.attributes, "process.exit_code").map(str::to_string),
                        span_id: span.span_id.clone(),
                    });
                    for (ev, err_id) in events.iter().zip(&error_node_ids) {
                        if ev.span_id == span.span_id {
                            edges.push(Edge {
                                r#type: "error_in_invocation".to_string(),
                                from: err_id.clone(),
                                to: inv_id.clone(),
                                strength: "strong".to_string(),
                            });
                        }
                    }
                }
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
    for (span_id, parent) in &span_ids_in_bundle {
        if let Some(parent_id) = parent {
            if span_ids_in_bundle.iter().any(|(s, _)| s == parent_id) {
                edges.push(Edge {
                    r#type: "span_child_of".to_string(),
                    from: format!("span_{span_id}"),
                    to: format!("span_{parent_id}"),
                    strength: "strong".to_string(),
                });
            }
        }
    }

    let mut lines = Vec::new();
    for rl in &logs.resource_logs {
        let log_service = rl
            .resource
            .as_ref()
            .and_then(|r| attr(&r.attributes, "service.name"))
            .unwrap_or("unknown");
        for sl in &rl.scope_logs {
            for rec in &sl.log_records {
                if !run_trace_ids.contains(&rec.trace_id) {
                    continue;
                }
                let body = rec.body.as_ref().and_then(|b| b.as_str()).unwrap_or("");
                lines.push(redact_string(
                    &format!("{} {} [{}] {}", rec.time_unix_nano, rec.severity_text, log_service, body),
                    &mut report,
                ));
            }
        }
    }
    if !lines.is_empty() {
        let first_trace = run_trace_ids.iter().next().cloned().unwrap_or_default();
        let id = format!("logwin_{run_id}");
        nodes.push(Node::LogWindow { id: id.clone(), trace_id: first_trace.clone(), lines });
        edges.push(Edge {
            r#type: "log_in_trace".to_string(),
            from: id,
            to: format!("trace_{first_trace}"),
            strength: "strong".to_string(),
        });
    }

    let generated_at = latest_timestamp(all_events);
    let mut bundle = Bundle {
        schema_version: SCHEMA_VERSION.to_string(),
        bundle_id: format!("bndl_{run_id}"),
        generated_at_unix_nano: generated_at,
        generator: Generator {
            name: "evidence-loop-poc".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        project: project.to_string(),
        anchor: Anchor {
            r#type: "run".to_string(),
            fingerprint: run_id.to_string(),
            service_name: service,
        },
        trigger: Trigger { r#type: "manual".to_string(), dispatch_eligible: false },
        nodes,
        edges,
        hypotheses: Vec::new(),
        missing_evidence: vec![
            "no metric windows in fixtures".to_string(),
            "deploy adjacency not evaluated for run-anchored bundles".to_string(),
        ],
        redaction_report: report,
        canonical_hash: None,
    };
    let hash = canonical_hash(&bundle);
    bundle.canonical_hash = Some(hash);
    Some(bundle)
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
