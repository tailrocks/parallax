//! Ranked hypotheses with citations — the explanation layer.
//!
//! The product's core output is not data but an evidence-backed explanation
//! (docs/research/architecture/causal-reconstruction.md): ranked hypotheses
//! where every material claim cites deterministic evidence, and an explicit
//! `insufficient_evidence` verdict when the graph supports nothing. No model
//! involved at this layer: rules read the typed evidence graph, so the same
//! bundle always yields the same ranked list. An LLM may later *narrate*
//! these; it never invents them.
//!
//! Citation format: `node:<id>` and `edge:<type>:<from>-><to>`.

use crate::bundle::{Bundle, Edge, Node};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Hypothesis {
    pub id: String,
    /// Rule that produced it: agent_change_regression | deploy_regression |
    /// dependency_failure | insufficient_evidence.
    pub kind: String,
    pub statement: String,
    /// high | medium | low — derived from the strength of the citing edges.
    pub confidence: String,
    /// Every hypothesis cites the evidence it stands on. Never empty.
    pub evidence_refs: Vec<String>,
}

fn edge_ref(e: &Edge) -> String {
    format!("edge:{}:{}->{}", e.r#type, e.from, e.to)
}

/// Populate `bundle.hypotheses` from the evidence graph and recompute the
/// canonical hash. Deterministic: rule order is fixed, refs are ordered.
pub fn attach_hypotheses(bundle: &mut Bundle) {
    let mut hypotheses = Vec::new();

    let first_error = bundle.nodes.iter().find_map(|n| match n {
        Node::ErrorEvent { id, error_type, message, .. } => {
            Some((id.clone(), error_type.clone(), message.clone()))
        }
        _ => None,
    });
    let deploy = bundle.nodes.iter().find_map(|n| match n {
        Node::Deploy { id, release, .. } => Some((id.clone(), release.clone())),
        _ => None,
    });

    // Rule 1 — agent_change_regression: an agent edited the failing file,
    // its session produced the shipped commit, and that deploy preceded the
    // issue. Three strong deterministic edges -> high confidence.
    let edited_failing: Vec<&Edge> = bundle
        .edges
        .iter()
        .filter(|e| e.r#type == "agent_edited_failing_file" && e.strength == "strong")
        .collect();
    let produced_change = bundle
        .edges
        .iter()
        .find(|e| e.r#type == "agent_session_produced_change" && e.strength == "strong");
    if let (Some(edited), Some(produced), Some((err_type, release))) = (
        edited_failing.first(),
        produced_change,
        first_error.as_ref().zip(deploy.as_ref()).map(|((_, t, _), (_, r))| (t.clone(), r.clone())),
    ) {
        let file = bundle
            .nodes
            .iter()
            .find_map(|n| match n {
                Node::AgentAction { id, target, .. } if *id == edited.from => Some(target.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "an edited file".to_string());
        let mut refs = vec![edge_ref(edited), edge_ref(produced)];
        refs.extend(
            bundle
                .edges
                .iter()
                .filter(|e| e.r#type == "deploy_preceded_issue")
                .map(edge_ref),
        );
        hypotheses.push(Hypothesis {
            id: format!("hyp_{}", hypotheses.len() + 1),
            kind: "agent_change_regression".to_string(),
            statement: format!(
                "An agent-authored change to {file}, shipped in release {release}, likely introduced {err_type}."
            ),
            confidence: "high".to_string(),
            evidence_refs: refs,
        });
    }

    // Rule 2 — deploy_regression: a deploy preceded the issue (strong only
    // when the deployed SHA matches the erroring service's revision).
    if let Some(deploy_edge) =
        bundle.edges.iter().find(|e| e.r#type == "deploy_preceded_issue")
    {
        if let (Some((deploy_node_id, release)), Some((err_id, err_type, _))) =
            (deploy.as_ref(), first_error.as_ref())
        {
            hypotheses.push(Hypothesis {
                id: format!("hyp_{}", hypotheses.len() + 1),
                kind: "deploy_regression".to_string(),
                statement: format!(
                    "Release {release} was deployed shortly before {err_type} first appeared; the release is a regression suspect."
                ),
                confidence: if deploy_edge.strength == "strong" { "medium".to_string() } else { "low".to_string() },
                evidence_refs: vec![
                    edge_ref(deploy_edge),
                    format!("node:{deploy_node_id}"),
                    format!("node:{err_id}"),
                ],
            });
        }
    }

    // Rule 3 — dependency_failure: the error text indicates a saturated or
    // unreachable downstream dependency (timeout / pool / connection).
    if let Some((err_id, err_type, message)) = first_error.as_ref() {
        let lowered = message.to_lowercase();
        if ["timed out", "timeout", "pool", "connection"].iter().any(|p| lowered.contains(p)) {
            let mut refs = vec![format!("node:{err_id}")];
            if let Some(Node::LogWindow { id, .. }) =
                bundle.nodes.iter().find(|n| matches!(n, Node::LogWindow { .. }))
            {
                refs.push(format!("node:{id}"));
            }
            hypotheses.push(Hypothesis {
                id: format!("hyp_{}", hypotheses.len() + 1),
                kind: "dependency_failure".to_string(),
                statement: format!(
                    "{err_type} indicates a downstream dependency timing out or saturated; check the dependency's capacity and latency in the error window."
                ),
                confidence: "medium".to_string(),
                evidence_refs: refs,
            });
        }
    }

    // Fallback — honest inconclusiveness, citing what exists.
    if hypotheses.is_empty() {
        let refs: Vec<String> = first_error
            .as_ref()
            .map(|(id, _, _)| vec![format!("node:{id}")])
            .unwrap_or_else(|| {
                bundle
                    .nodes
                    .first()
                    .map(|n| match n {
                        Node::ErrorEvent { id, .. }
                        | Node::Span { id, .. }
                        | Node::LogWindow { id, .. }
                        | Node::Deploy { id, .. }
                        | Node::CliInvocation { id, .. }
                        | Node::AgentSession { id, .. }
                        | Node::AgentAction { id, .. } => vec![format!("node:{id}")],
                    })
                    .unwrap_or_default()
            });
        hypotheses.push(Hypothesis {
            id: "hyp_1".to_string(),
            kind: "insufficient_evidence".to_string(),
            statement: "The evidence graph does not support a root-cause hypothesis; see missing_evidence for what to instrument next.".to_string(),
            confidence: "low".to_string(),
            evidence_refs: refs,
        });
    }

    bundle.hypotheses = hypotheses;
    let hash = crate::bundle::canonical_hash(bundle);
    bundle.canonical_hash = Some(hash);
}
