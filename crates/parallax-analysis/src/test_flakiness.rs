//! Pure flaky-evidence evaluation over completed test invocation chains.

use parallax_model::{
    AttemptChain, AttemptChainError, AttemptRollup, FlakyEvidence, TestResultRecord, TestVariantKey,
};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlakyEvaluationError {
    ZeroWindow,
    ZeroFailureThreshold,
    MixedVariant,
    DuplicateAttempt,
}

impl fmt::Display for FlakyEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroWindow => "flaky evaluation window must be nonzero",
            Self::ZeroFailureThreshold => "consistent-failure threshold must be nonzero",
            Self::MixedVariant => "flaky evaluation contains multiple test variants",
            Self::DuplicateAttempt => "test invocation contains a duplicate attempt",
        })
    }
}

impl std::error::Error for FlakyEvaluationError {}

struct InvocationEvidence {
    id: String,
    completed_at_nanos: u128,
    revision: Option<String>,
    rollup: AttemptRollup,
}

/// Evaluate one variant inside the inclusive `[evaluated_at - window, evaluated_at]` window.
///
/// Attempt rows are grouped into invocation chains before any cross-run signal
/// is counted. Thus a fail-then-pass retry produces `intra_invocation_mix`
/// without fabricating a window transition or same-commit divergence.
pub fn evaluate_flaky_evidence(
    results: Vec<TestResultRecord>,
    evaluated_at_nanos: u128,
    window_nanos: u128,
    min_consistent_failures: u32,
) -> Result<FlakyEvidence, FlakyEvaluationError> {
    if window_nanos == 0 {
        return Err(FlakyEvaluationError::ZeroWindow);
    }
    if min_consistent_failures == 0 {
        return Err(FlakyEvaluationError::ZeroFailureThreshold);
    }
    let since = evaluated_at_nanos.saturating_sub(window_nanos);
    let mut variant: Option<TestVariantKey> = None;
    let mut grouped = BTreeMap::<String, Vec<TestResultRecord>>::new();
    for result in results {
        if result.ended_at_nanos < since || result.ended_at_nanos > evaluated_at_nanos {
            continue;
        }
        if variant
            .as_ref()
            .is_some_and(|expected| expected != &result.key.variant_key)
        {
            return Err(FlakyEvaluationError::MixedVariant);
        }
        variant.get_or_insert_with(|| result.key.variant_key.clone());
        grouped
            .entry(result.key.invocation_id.clone())
            .or_default()
            .push(result);
    }

    let mut invocations = Vec::with_capacity(grouped.len());
    for (id, attempts) in grouped {
        let completed_at_nanos = attempts
            .iter()
            .map(|attempt| attempt.ended_at_nanos)
            .max()
            .unwrap_or(0);
        let revision = consistent_revision(&attempts);
        let chain = AttemptChain::new(attempts).map_err(map_chain_error)?;
        invocations.push(InvocationEvidence {
            id,
            completed_at_nanos,
            revision,
            rollup: chain.rollup(),
        });
    }
    invocations.sort_by(|left, right| {
        left.completed_at_nanos
            .cmp(&right.completed_at_nanos)
            .then_with(|| left.id.cmp(&right.id))
    });

    let intra_invocation_mix = invocations
        .iter()
        .any(|invocation| invocation.rollup == AttemptRollup::FlakyPass);
    let mut revisions = BTreeMap::<&str, (bool, bool)>::new();
    for invocation in &invocations {
        let Some(revision) = invocation.revision.as_deref() else {
            continue;
        };
        let flags = revisions.entry(revision).or_default();
        match invocation.rollup {
            AttemptRollup::Passed => flags.0 = true,
            AttemptRollup::Failed => flags.1 = true,
            _ => {}
        }
    }
    let same_commit_divergence = revisions.values().any(|(pass, fail)| *pass && *fail);
    let decisive: Vec<_> = invocations
        .iter()
        .filter_map(|invocation| match invocation.rollup {
            AttemptRollup::Passed | AttemptRollup::Failed => Some(invocation.rollup),
            _ => None,
        })
        .collect();
    let window_transition_count = decisive
        .windows(2)
        .filter(|pair| pair[0] != pair[1])
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    let consecutive_passes = invocations
        .iter()
        .rev()
        .take_while(|invocation| invocation.rollup == AttemptRollup::Passed)
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    let consistent_failure_count = invocations
        .iter()
        .rev()
        .take_while(|invocation| {
            matches!(
                invocation.rollup,
                AttemptRollup::Failed | AttemptRollup::Broken
            )
        })
        .count();

    Ok(FlakyEvidence {
        same_commit_divergence,
        intra_invocation_mix,
        window_transition_count,
        consecutive_passes,
        consistently_failing: consistent_failure_count
            >= usize::try_from(min_consistent_failures).unwrap_or(usize::MAX),
    })
}

fn consistent_revision(results: &[TestResultRecord]) -> Option<String> {
    let mut revisions = results
        .iter()
        .filter_map(|result| result.vcs_head_revision.as_deref())
        .map(str::trim)
        .filter(|revision| !revision.is_empty());
    let first = revisions.next()?;
    revisions
        .all(|revision| revision == first)
        .then(|| first.to_owned())
}

fn map_chain_error(error: AttemptChainError) -> FlakyEvaluationError {
    match error {
        AttemptChainError::DuplicateAttempt => FlakyEvaluationError::DuplicateAttempt,
        AttemptChainError::MixedVariant | AttemptChainError::MixedInvocation => {
            FlakyEvaluationError::MixedVariant
        }
        AttemptChainError::Empty => unreachable!("grouped invocation is nonempty"),
    }
}

pub mod scan;

pub use scan::{
    FlakyEvaluationConfig, FlakyStateUpdate, default_flaky_policy, propose_flaky_state_update,
};

#[cfg(test)]
#[path = "test_flakiness/tests.rs"]
mod tests;
