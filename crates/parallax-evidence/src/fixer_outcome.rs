//! Offline fixer outcome state machine (plan 123 residual).
//!
//! Parallax core never owns checkout/patch/PR/merge/deploy. This module only
//! records request→outcome transitions with fail-closed success rules:
//! opened PRs are never success; success requires review + runtime recurrence
//! evidence.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const OUTCOME_SCHEMA_VERSION: &str = "fixer-outcome-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixerPhase {
    Requested,
    Bundled,
    AgentSession,
    PatchProposed,
    DraftPrOpened,
    CiObserved,
    HumanReviewed,
    Merged,
    Reverted,
    RuntimeRecurrence,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixerTerminal {
    /// Explicit non-success terminal states.
    Failed,
    Unmerged,
    Reverted,
    Recurred,
    /// Only after review evidence AND no recurrence in the observation window.
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixerOutcomeRecord {
    pub schema_version: String,
    pub request_id: String,
    pub bundle_id: Option<String>,
    pub phase: FixerPhase,
    pub terminal: Option<FixerTerminal>,
    pub draft_pr_opened: bool,
    pub human_review_ok: bool,
    pub runtime_recurrence: bool,
    pub immutable_hash: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixerTransitionError {
    TerminalAlreadySet,
    OptimisticPrSuccess,
    MissingReviewForSuccess,
    RecurrenceBlocksSuccess,
    IllegalPhaseOrder,
}

impl FixerTransitionError {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TerminalAlreadySet => "terminal outcome already set",
            Self::OptimisticPrSuccess => "opened PR is never fix success",
            Self::MissingReviewForSuccess => "success requires human review evidence",
            Self::RecurrenceBlocksSuccess => "runtime recurrence blocks success",
            Self::IllegalPhaseOrder => "illegal phase order",
        }
    }
}

/// Start a new offline request row (requested phase).
#[must_use]
pub fn request(request_id: impl Into<String>) -> FixerOutcomeRecord {
    let mut record = FixerOutcomeRecord {
        schema_version: OUTCOME_SCHEMA_VERSION.into(),
        request_id: request_id.into(),
        bundle_id: None,
        phase: FixerPhase::Requested,
        terminal: None,
        draft_pr_opened: false,
        human_review_ok: false,
        runtime_recurrence: false,
        immutable_hash: String::new(),
        notes: Vec::new(),
    };
    record.immutable_hash = hash_record(&record);
    record
}

/// Apply a phase transition with fail-closed success rules.
pub fn transition(
    mut record: FixerOutcomeRecord,
    next: FixerPhase,
    note: Option<String>,
) -> Result<FixerOutcomeRecord, FixerTransitionError> {
    if record.terminal.is_some() {
        return Err(FixerTransitionError::TerminalAlreadySet);
    }
    if !phase_allowed(record.phase, next) {
        return Err(FixerTransitionError::IllegalPhaseOrder);
    }
    record.phase = next;
    if let Some(note) = note {
        record.notes.push(note);
    }
    match next {
        FixerPhase::DraftPrOpened => {
            record.draft_pr_opened = true;
            // Never auto-success on PR open.
        }
        FixerPhase::HumanReviewed => {
            record.human_review_ok = true;
        }
        FixerPhase::Reverted => {
            record.terminal = Some(FixerTerminal::Reverted);
        }
        FixerPhase::RuntimeRecurrence => {
            record.runtime_recurrence = true;
            record.terminal = Some(FixerTerminal::Recurred);
        }
        FixerPhase::Closed => {
            // Closed without explicit success remains Failed/Unmerged.
            if record.draft_pr_opened && !record.human_review_ok {
                record.terminal = Some(FixerTerminal::Unmerged);
            } else if record.terminal.is_none() {
                record.terminal = Some(FixerTerminal::Failed);
            }
        }
        _ => {}
    }
    record.immutable_hash = hash_record(&record);
    Ok(record)
}

/// Mark success only when review passed and no recurrence was observed.
pub fn mark_success(
    mut record: FixerOutcomeRecord,
) -> Result<FixerOutcomeRecord, FixerTransitionError> {
    if record.terminal.is_some() {
        return Err(FixerTransitionError::TerminalAlreadySet);
    }
    if record.draft_pr_opened && !record.human_review_ok {
        return Err(FixerTransitionError::OptimisticPrSuccess);
    }
    if !record.human_review_ok {
        return Err(FixerTransitionError::MissingReviewForSuccess);
    }
    if record.runtime_recurrence {
        return Err(FixerTransitionError::RecurrenceBlocksSuccess);
    }
    record.terminal = Some(FixerTerminal::Success);
    record.phase = FixerPhase::Closed;
    record.immutable_hash = hash_record(&record);
    Ok(record)
}

fn phase_allowed(from: FixerPhase, to: FixerPhase) -> bool {
    matches!(
        (from, to),
        (FixerPhase::Requested, FixerPhase::Bundled)
            | (FixerPhase::Bundled, FixerPhase::AgentSession)
            | (FixerPhase::AgentSession, FixerPhase::PatchProposed)
            | (
                FixerPhase::PatchProposed,
                FixerPhase::DraftPrOpened | FixerPhase::Closed
            )
            | (
                FixerPhase::DraftPrOpened,
                FixerPhase::CiObserved | FixerPhase::HumanReviewed | FixerPhase::Closed
            )
            | (
                FixerPhase::CiObserved,
                FixerPhase::HumanReviewed | FixerPhase::Closed
            )
            | (
                FixerPhase::HumanReviewed,
                FixerPhase::Merged | FixerPhase::Closed
            )
            | (
                FixerPhase::Merged,
                FixerPhase::Reverted | FixerPhase::RuntimeRecurrence | FixerPhase::Closed
            )
            | (
                FixerPhase::Reverted | FixerPhase::RuntimeRecurrence,
                FixerPhase::Closed
            )
    )
}

fn hash_record(record: &FixerOutcomeRecord) -> String {
    let mut for_hash = record.clone();
    for_hash.immutable_hash.clear();
    let bytes = serde_json::to_vec(&for_hash).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_open_is_never_success() {
        let mut row = request("fixreq_1");
        row = transition(row, FixerPhase::Bundled, None).unwrap();
        row = transition(row, FixerPhase::AgentSession, None).unwrap();
        row = transition(row, FixerPhase::PatchProposed, None).unwrap();
        row = transition(row, FixerPhase::DraftPrOpened, Some("pr#1".into())).unwrap();
        assert!(row.draft_pr_opened);
        assert!(matches!(
            mark_success(row),
            Err(FixerTransitionError::OptimisticPrSuccess)
        ));
    }

    #[test]
    fn success_requires_review_and_no_recurrence() {
        let mut row = request("fixreq_2");
        for phase in [
            FixerPhase::Bundled,
            FixerPhase::AgentSession,
            FixerPhase::PatchProposed,
            FixerPhase::DraftPrOpened,
            FixerPhase::HumanReviewed,
        ] {
            row = transition(row, phase, None).unwrap();
        }
        let success = mark_success(row.clone()).unwrap();
        assert_eq!(success.terminal, Some(FixerTerminal::Success));

        let mut recurred = row;
        recurred = transition(recurred, FixerPhase::Merged, None).unwrap();
        recurred = transition(recurred, FixerPhase::RuntimeRecurrence, None).unwrap();
        assert_eq!(recurred.terminal, Some(FixerTerminal::Recurred));
        assert!(matches!(
            mark_success(recurred),
            Err(FixerTransitionError::TerminalAlreadySet)
        ));
    }

    #[test]
    fn offline_multi_arm_preserves_failures() {
        // Arm A: PR without review → unmerged
        let mut a = request("arm_a");
        for phase in [
            FixerPhase::Bundled,
            FixerPhase::AgentSession,
            FixerPhase::PatchProposed,
            FixerPhase::DraftPrOpened,
            FixerPhase::Closed,
        ] {
            a = transition(a, phase, None).unwrap();
        }
        assert_eq!(a.terminal, Some(FixerTerminal::Unmerged));

        // Arm B: reviewed success
        let mut b = request("arm_b");
        for phase in [
            FixerPhase::Bundled,
            FixerPhase::AgentSession,
            FixerPhase::PatchProposed,
            FixerPhase::DraftPrOpened,
            FixerPhase::HumanReviewed,
        ] {
            b = transition(b, phase, None).unwrap();
        }
        b = mark_success(b).unwrap();
        assert_eq!(b.terminal, Some(FixerTerminal::Success));
        assert_ne!(a.immutable_hash, b.immutable_hash);
    }
}
