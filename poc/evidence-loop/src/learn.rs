//! Learn: outcome rows adjust future evidence selection.
//!
//! Stage 6 of docs/research/architecture/autonomous-fix-loop.md. The fixer
//! outcome ledger only allows a "learns from outcomes" claim when outcome rows
//! demonstrably alter a policy, retrieval, or scoring decision in a dated run.
//! This module is the deterministic kernel of that loop for evidence
//! selection: per edge type, how often were fixes accepted when the fixer
//! cited that edge, relative to the base accept rate? The resulting lift
//! re-weights neighborhood expansion and bundle inclusion in the Bundler.
//!
//! Deterministic: Laplace-smoothed rates, fixed-precision serialization, a
//! report that references the exact outcome rows it was computed from.

use crate::budget::OutcomeRow;
use serde::Serialize;
use std::collections::BTreeMap;

pub const POLICY_VERSION: &str = "evidence-weights-v0";

/// Laplace smoothing constant: one virtual accepted and one virtual
/// not-accepted citation per edge type, so single-row evidence cannot saturate
/// a weight.
const ALPHA: f64 = 1.0;

#[derive(Debug, Clone, Serialize)]
pub struct EdgeWeight {
    /// Rows that cited this edge type.
    pub cited_in: usize,
    /// Smoothed accept rate among citing rows (accepted = 1, edited = 0.5).
    pub accept_rate_when_cited: String,
    /// accept_rate_when_cited / base_accept_rate. > 1.0 means citing this
    /// edge type predicted acceptance; the Bundler should prefer expanding it.
    pub lift: String,
}

#[derive(Debug, Serialize)]
pub struct LearnerReport {
    pub policy_version: String,
    pub base_accept_rate: String,
    pub weights: BTreeMap<String, EdgeWeight>,
    /// The dated-row rule: every adjustment references the outcomes that
    /// caused it.
    pub basis_outcome_ids: Vec<String>,
}

fn accept_credit(row: &OutcomeRow) -> f64 {
    match row.classification.as_str() {
        "accepted" => 1.0,
        "edited" => 0.5, // human edits are partial credit
        _ => 0.0,
    }
}

/// Compute evidence-selection weights from the outcome corpus.
pub fn compute_edge_weights(rows: &[OutcomeRow]) -> LearnerReport {
    let n = rows.len() as f64;
    let total_credit: f64 = rows.iter().map(accept_credit).sum();
    let base_rate = if n > 0.0 { (total_credit + ALPHA) / (n + 2.0 * ALPHA) } else { 0.5 };

    let mut by_edge: BTreeMap<String, (usize, f64)> = BTreeMap::new();
    for row in rows {
        let credit = accept_credit(row);
        for edge_type in &row.cited_edge_types {
            let entry = by_edge.entry(edge_type.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += credit;
        }
    }

    let weights = by_edge
        .into_iter()
        .map(|(edge_type, (cited_in, credit))| {
            let rate = (credit + ALPHA) / (cited_in as f64 + 2.0 * ALPHA);
            let lift = rate / base_rate;
            (
                edge_type,
                EdgeWeight {
                    cited_in,
                    accept_rate_when_cited: format!("{rate:.3}"),
                    lift: format!("{lift:.3}"),
                },
            )
        })
        .collect();

    LearnerReport {
        policy_version: POLICY_VERSION.to_string(),
        base_accept_rate: format!("{base_rate:.3}"),
        weights,
        basis_outcome_ids: rows.iter().map(|r| r.outcome_id.clone()).collect(),
    }
}
