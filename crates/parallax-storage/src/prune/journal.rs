use serde::{Deserialize, Serialize};

use super::{PruneItem, PrunePlan, PrunePlanLimits};
use crate::metadata::MetadataResult;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PruneJournalStepState {
    Planned,
    Executing,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PruneStepStart {
    Execute,
    AlreadyComplete,
}

pub const PRUNE_JOURNAL_ERROR_MAX_BYTES: usize = 1_024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PruneJournalStep {
    pub step_index: u32,
    pub item: PruneItem,
    pub state: PruneJournalStepState,
    pub last_error: Option<String>,
    pub completed_at_nanos: Option<u128>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PruneJournal {
    pub plan: PrunePlan,
    pub created_at_nanos: u128,
    pub updated_at_nanos: u128,
    pub completed_at_nanos: Option<u128>,
    pub steps: Vec<PruneJournalStep>,
}

#[async_trait::async_trait]
pub trait MetadataPruneJournalStore: Send + Sync {
    /// Atomically create the immutable plan journal and every ordered step.
    /// Repeating the same plan is idempotent; conflicting bytes fail closed.
    async fn create_prune_journal(
        &self,
        plan: &PrunePlan,
        now_nanos: u128,
        limits: PrunePlanLimits,
    ) -> MetadataResult<PruneJournal>;

    /// Recover a journal only after validating the persisted plan identity and
    /// current safety bounds.
    async fn prune_journal(
        &self,
        plan_id: &str,
        limits: PrunePlanLimits,
    ) -> MetadataResult<Option<PruneJournal>>;

    /// Start or retry a step. Completed steps return `AlreadyComplete` and
    /// must never execute again.
    async fn begin_prune_step(
        &self,
        plan_id: &str,
        step_index: u32,
        now_nanos: u128,
    ) -> MetadataResult<PruneStepStart>;

    /// Keep a failed step resumable in `executing` state with bounded evidence.
    async fn record_prune_step_failure(
        &self,
        plan_id: &str,
        step_index: u32,
        error: &str,
        now_nanos: u128,
    ) -> MetadataResult<()>;

    /// Complete an executing step idempotently. The journal completes only
    /// after no planned/executing steps remain.
    async fn complete_prune_step(
        &self,
        plan_id: &str,
        step_index: u32,
        now_nanos: u128,
    ) -> MetadataResult<()>;
}
