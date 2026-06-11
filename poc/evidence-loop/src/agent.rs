//! Agent sessions as evidence: "what did the agent do before this incident?"
//!
//! The fourth evidence class (services, CLI runs, deploys, agent sessions).
//! Sessions arrive as normalized records per
//! docs/research/capture/agent-cli-tracing.md — tool, repo, produced commit,
//! and an ordered action log (files read/edited, commands, tests, patches).
//! `attach_agent_evidence` joins them into a bundle when a session ended
//! within the relevance window before the first error, producing the chain
//! that answers causality questions: agent edit → deploy carrying the
//! session's commit (strong, SHA equality) → error. The sharpest edge is
//! `agent_edited_failing_file`: the action's target path appears in the
//! error's stacktrace.

use crate::bundle::{canonical_hash, Bundle, Edge, Node};
use crate::redact::redact_string;
use serde::{Deserialize, Serialize};

/// Sessions older than this relative to the first error are not attached.
pub const SESSION_RELEVANCE_WINDOW_NANOS: u128 = 24 * 60 * 60 * 1_000_000_000;

#[derive(Debug, Deserialize)]
pub struct AgentSessionsData {
    #[serde(default)]
    pub sessions: Vec<AgentSessionRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentSessionRecord {
    pub session_id: String,
    pub tool: String,
    pub started_at_unix_nano: String,
    pub ended_at_unix_nano: String,
    pub repo: String,
    pub vcs_sha: String,
    #[serde(default)]
    pub actions: Vec<AgentActionRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentActionRecord {
    pub seq: u32,
    pub action_type: String,
    pub target: String,
    pub detail: String,
}

fn first_error_nanos(bundle: &Bundle) -> Option<u128> {
    bundle
        .nodes
        .iter()
        .filter_map(|n| match n {
            Node::ErrorEvent { time_unix_nano, .. } => time_unix_nano.parse::<u128>().ok(),
            _ => None,
        })
        .min()
}

/// Attach relevant agent sessions to a bundle. Returns how many were
/// attached. Recomputes the canonical hash when anything changed.
pub fn attach_agent_evidence(
    bundle: &mut Bundle,
    sessions: &[AgentSessionRecord],
    window_nanos: u128,
) -> usize {
    let Some(first_error) = first_error_nanos(bundle) else {
        return 0;
    };
    let deploy_shas: Vec<String> = bundle
        .nodes
        .iter()
        .filter_map(|n| match n {
            Node::Deploy { vcs_sha, .. } => Some(vcs_sha.clone()),
            _ => None,
        })
        .collect();
    let stacktraces: Vec<String> = bundle
        .nodes
        .iter()
        .filter_map(|n| match n {
            Node::ErrorEvent { stacktrace: Some(s), .. } => Some(s.clone()),
            _ => None,
        })
        .collect();
    let anchor_error_id = bundle.nodes.iter().find_map(|n| match n {
        Node::ErrorEvent { id, .. } => Some(id.clone()),
        _ => None,
    });

    let mut attached = 0;
    for session in sessions {
        let ended: u128 = session.ended_at_unix_nano.parse().unwrap_or(0);
        let in_window = ended <= first_error && first_error - ended <= window_nanos;
        if !in_window {
            continue;
        }
        attached += 1;
        let session_node_id = format!("ags_{}", session.session_id);
        bundle.nodes.push(Node::AgentSession {
            id: session_node_id.clone(),
            tool: session.tool.clone(),
            repo: session.repo.clone(),
            vcs_sha: session.vcs_sha.clone(),
            started_at_unix_nano: session.started_at_unix_nano.clone(),
            ended_at_unix_nano: session.ended_at_unix_nano.clone(),
        });

        // Temporal + scope edge to the anchoring error (medium by design:
        // time adjacency is suggestive, not proof).
        if let Some(err_id) = &anchor_error_id {
            bundle.edges.push(Edge {
                r#type: "agent_session_preceded_issue".to_string(),
                from: session_node_id.clone(),
                to: err_id.clone(),
                strength: "medium".to_string(),
            });
        }

        // SHA equality with a deploy in the bundle: the session produced the
        // change that shipped. Deterministic key — strong.
        if deploy_shas.iter().any(|sha| sha == &session.vcs_sha) {
            bundle.edges.push(Edge {
                r#type: "agent_session_produced_change".to_string(),
                from: session_node_id.clone(),
                to: format!("deploy_sha_{}", &session.vcs_sha[..12.min(session.vcs_sha.len())]),
                strength: "strong".to_string(),
            });
        }

        for action in &session.actions {
            let action_node_id = format!("aga_{}_{}", session.session_id, action.seq);
            bundle.nodes.push(Node::AgentAction {
                id: action_node_id.clone(),
                session_id: session.session_id.clone(),
                seq: action.seq,
                action_type: action.action_type.clone(),
                target: action.target.clone(),
                detail: redact_string(&action.detail, &mut bundle.redaction_report),
            });
            bundle.edges.push(Edge {
                r#type: "action_in_session".to_string(),
                from: action_node_id.clone(),
                to: session_node_id.clone(),
                strength: "strong".to_string(),
            });

            // The sharpest suspicion edge: the agent edited a file that
            // appears in the failing stacktrace.
            if action.action_type == "file_edit"
                && stacktraces.iter().any(|st| st.contains(&action.target))
            {
                if let Some(err_id) = &anchor_error_id {
                    bundle.edges.push(Edge {
                        r#type: "agent_edited_failing_file".to_string(),
                        from: action_node_id,
                        to: err_id.clone(),
                        strength: "strong".to_string(),
                    });
                }
            }
        }
    }

    if attached > 0 {
        let hash = canonical_hash(bundle);
        bundle.canonical_hash = Some(hash);
    }
    attached
}
