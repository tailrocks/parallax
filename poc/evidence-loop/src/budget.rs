//! Autonomy budget: convert outcome history into a permitted autonomy level.
//!
//! Implements the v0 promotion policy sketched in
//! docs/research/00-vision/north-star-autonomous-fix-loop.md §3: autonomy is a
//! function of the trailing outcome corpus per (project, failure_class), never
//! a configuration knob. The thresholds below are the first concrete numbers
//! (tunable policy, not schema):
//!
//! | Level | Requires |
//! | --- | --- |
//! | L1 diagnose | default — any class starts here |
//! | L2 propose_patch | n >= 5, accept rate >= 0.6, zero reverts, zero redaction failures |
//! | L3 draft_pr | n >= 10, accept rate >= 0.7, revert+recurrence rate <= 0.05, zero redaction failures |
//! | L4/L5 | never emitted by v0 policy (L3 must be earned first — fixer-boundary ADR) |
//!
//! Edited-before-merge outcomes count as half credit ("human edits are partial
//! credit, not agent fixed unaided" — fixer outcome ledger counting rules).
//! Any redaction failure in the window caps the class at L1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct OutcomesData {
    #[serde(default)]
    pub outcomes: Vec<OutcomeRow>,
}

/// Subset of the fixer outcome ledger row that the budget and learner consume.
#[derive(Debug, Clone, Deserialize)]
pub struct OutcomeRow {
    pub outcome_id: String,
    pub failure_class: String,
    pub autonomy_level: String,
    /// diagnosis_only | proposal | draft_pr | accepted | edited | rejected |
    /// reverted | inconclusive
    pub classification: String,
    pub merged: bool,
    /// yes | no | unknown
    pub recurrence: String,
    pub redaction_failure: bool,
    /// Edge types the fixer's diagnosis/PR body cited from the bundle
    /// (evidence-citation gate). Consumed by the learner.
    #[serde(default)]
    pub cited_edge_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AutonomyLevel {
    L1Diagnose,
    L2ProposePatch,
    L3DraftPr,
}

impl AutonomyLevel {
    pub fn as_contract_str(&self) -> &'static str {
        match self {
            AutonomyLevel::L1Diagnose => "L1_diagnose",
            AutonomyLevel::L2ProposePatch => "L2_propose_patch",
            AutonomyLevel::L3DraftPr => "L3_draft_pr",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AutonomyBudget {
    pub failure_class: String,
    pub max_level: String,
    pub basis: BudgetBasis,
}

#[derive(Debug, Clone, Serialize)]
pub struct BudgetBasis {
    pub runs: usize,
    /// accepted + 0.5 * edited, over runs; serialized with fixed precision so
    /// bundles stay byte-deterministic.
    pub accepted_rate: String,
    pub revert_rate: String,
    pub recurrence_rate: String,
    pub redaction_failures: usize,
    pub policy_version: String,
}

pub const POLICY_VERSION: &str = "autonomy-budget-v0";

const L2_MIN_RUNS: usize = 5;
const L2_MIN_ACCEPT: f64 = 0.6;
const L3_MIN_RUNS: usize = 10;
const L3_MIN_ACCEPT: f64 = 0.7;
const L3_MAX_REGRESSION: f64 = 0.05;

/// Compute the permitted autonomy level for one failure class.
pub fn compute_budget(rows: &[OutcomeRow], failure_class: &str) -> AutonomyBudget {
    let class_rows: Vec<&OutcomeRow> =
        rows.iter().filter(|r| r.failure_class == failure_class).collect();

    let n = class_rows.len();
    let accepted = class_rows.iter().filter(|r| r.classification == "accepted").count() as f64;
    let edited = class_rows.iter().filter(|r| r.classification == "edited").count() as f64;
    let reverted = class_rows.iter().filter(|r| r.classification == "reverted").count() as f64;
    let recurred = class_rows.iter().filter(|r| r.recurrence == "yes").count() as f64;
    let redaction_failures = class_rows.iter().filter(|r| r.redaction_failure).count();

    let accepted_rate = if n > 0 { (accepted + 0.5 * edited) / n as f64 } else { 0.0 };
    let revert_rate = if n > 0 { reverted / n as f64 } else { 0.0 };
    let recurrence_rate = if n > 0 { recurred / n as f64 } else { 0.0 };

    let level = if redaction_failures > 0 {
        AutonomyLevel::L1Diagnose
    } else if n >= L3_MIN_RUNS
        && accepted_rate >= L3_MIN_ACCEPT
        && revert_rate + recurrence_rate <= L3_MAX_REGRESSION
    {
        AutonomyLevel::L3DraftPr
    } else if n >= L2_MIN_RUNS && accepted_rate >= L2_MIN_ACCEPT && revert_rate == 0.0 {
        AutonomyLevel::L2ProposePatch
    } else {
        AutonomyLevel::L1Diagnose
    };

    AutonomyBudget {
        failure_class: failure_class.to_string(),
        max_level: level.as_contract_str().to_string(),
        basis: BudgetBasis {
            runs: n,
            accepted_rate: format!("{accepted_rate:.3}"),
            revert_rate: format!("{revert_rate:.3}"),
            recurrence_rate: format!("{recurrence_rate:.3}"),
            redaction_failures,
            policy_version: POLICY_VERSION.to_string(),
        },
    }
}
