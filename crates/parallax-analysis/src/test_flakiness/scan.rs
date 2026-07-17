//! One pure flaky-job step: evaluate a variant window and propose a state write.

use super::{FlakyEvaluationError, evaluate_flaky_evidence};
use parallax_model::{
    FlakyPolicy, FlakyState, TestFlakyStateRecord, TestResultRecord, TestVariantKey,
};
use std::num::NonZeroU32;

/// Default flaky policy for the V1 scan job (plan 155).
#[must_use]
pub fn default_flaky_policy() -> FlakyPolicy {
    FlakyPolicy {
        transition_threshold: NonZeroU32::new(2).expect("nonzero"),
        recovery_passes: NonZeroU32::new(3).expect("nonzero"),
    }
}

/// Outcome of evaluating one candidate variant inside a time window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlakyStateUpdate {
    pub record: TestFlakyStateRecord,
    pub previous: Option<FlakyState>,
    pub changed: bool,
}

/// Pure: map prior state + result window → next flaky state record.
pub fn propose_flaky_state_update(
    variant_key: TestVariantKey,
    results: Vec<TestResultRecord>,
    previous: Option<&TestFlakyStateRecord>,
    evaluated_at_nanos: u128,
    window_nanos: u128,
    policy: FlakyPolicy,
    min_consistent_failures: u32,
) -> Result<FlakyStateUpdate, FlakyEvaluationError> {
    let evidence = evaluate_flaky_evidence(
        results,
        evaluated_at_nanos,
        window_nanos,
        min_consistent_failures,
    )?;
    let prior_state = previous.map(|row| row.state).unwrap_or(FlakyState::Healthy);
    let next_state = prior_state.transition(evidence, policy);
    let record = TestFlakyStateRecord {
        variant_key,
        state: next_state,
        evidence,
        updated_at_nanos: evaluated_at_nanos,
    };
    Ok(FlakyStateUpdate {
        previous: previous.map(|row| row.state),
        changed: previous.is_none_or(|row| row.state != next_state || row.evidence != evidence),
        record,
    })
}
