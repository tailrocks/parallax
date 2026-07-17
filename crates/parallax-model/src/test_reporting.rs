//! Query-neutral test-reporting identities, outcomes, and state transitions.

mod identity;

pub use identity::*;

use crate::TraceId;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestStatus {
    Passed,
    Failed,
    Broken,
    Skipped,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestParameter {
    pub name: String,
    pub value: String,
    pub excluded: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestConfiguration {
    pub dimensions: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestCaseRecord {
    pub key: TestCaseKey,
    pub identity_source: TestCaseIdentitySource,
    pub explicit_id: Option<String>,
    pub code_reference: Option<String>,
    pub suite_path: Vec<String>,
    pub name: String,
    pub first_seen_nanos: u128,
    pub last_seen_nanos: u128,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestVariantRecord {
    pub key: TestVariantKey,
    pub case_key: TestCaseKey,
    pub parameters: Vec<TestParameter>,
    pub first_seen_nanos: u128,
    pub last_seen_nanos: u128,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct TestAttempt(NonZeroU32);

impl TestAttempt {
    pub fn new(value: u32) -> Result<Self, TestAttemptError> {
        NonZeroU32::new(value).map(Self).ok_or(TestAttemptError)
    }

    #[must_use]
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl<'de> Deserialize<'de> for TestAttempt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TestAttemptError;

impl fmt::Display for TestAttemptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("test attempt must be one or greater")
    }
}

impl std::error::Error for TestAttemptError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestResultKey {
    pub variant_key: TestVariantKey,
    pub invocation_id: String,
    pub attempt: TestAttempt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestResultRecord {
    pub key: TestResultKey,
    pub status: TestStatus,
    pub trace_id: TraceId,
    pub span_id: String,
    pub started_at_nanos: u128,
    pub ended_at_nanos: u128,
    pub service: String,
    pub service_version: Option<String>,
    pub vcs_head_revision: Option<String>,
    pub configuration: TestConfiguration,
    pub failure_fingerprint: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptRollup {
    Passed,
    FlakyPass,
    Failed,
    Broken,
    Skipped,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptChain(Vec<TestResultRecord>);

impl AttemptChain {
    pub fn new(mut results: Vec<TestResultRecord>) -> Result<Self, AttemptChainError> {
        let first = results.first().ok_or(AttemptChainError::Empty)?;
        let variant = first.key.variant_key.clone();
        let invocation = first.key.invocation_id.clone();
        if results
            .iter()
            .any(|result| result.key.variant_key != variant)
        {
            return Err(AttemptChainError::MixedVariant);
        }
        if results
            .iter()
            .any(|result| result.key.invocation_id != invocation)
        {
            return Err(AttemptChainError::MixedInvocation);
        }
        results.sort_by_key(|result| result.key.attempt);
        if results
            .windows(2)
            .any(|pair| pair[0].key.attempt == pair[1].key.attempt)
        {
            return Err(AttemptChainError::DuplicateAttempt);
        }
        Ok(Self(results))
    }

    #[must_use]
    pub fn results(&self) -> &[TestResultRecord] {
        &self.0
    }

    #[must_use]
    pub fn rollup(&self) -> AttemptRollup {
        let mut prior_failure = false;
        for result in &self.0 {
            match result.status {
                TestStatus::Passed if prior_failure => return AttemptRollup::FlakyPass,
                TestStatus::Failed | TestStatus::Broken => prior_failure = true,
                _ => {}
            }
        }
        if self
            .0
            .iter()
            .any(|result| result.status == TestStatus::Failed)
        {
            AttemptRollup::Failed
        } else if self
            .0
            .iter()
            .any(|result| result.status == TestStatus::Broken)
        {
            AttemptRollup::Broken
        } else if self
            .0
            .iter()
            .any(|result| result.status == TestStatus::Passed)
        {
            AttemptRollup::Passed
        } else if self
            .0
            .iter()
            .all(|result| result.status == TestStatus::Skipped)
        {
            AttemptRollup::Skipped
        } else {
            AttemptRollup::Unknown
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptChainError {
    Empty,
    MixedVariant,
    MixedInvocation,
    DuplicateAttempt,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlakyState {
    Healthy,
    Flaky,
    Fixed,
    Broken,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FlakyEvidence {
    pub same_commit_divergence: bool,
    pub intra_invocation_mix: bool,
    pub window_transition_count: u32,
    pub consecutive_passes: u32,
    pub consistently_failing: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestFlakyStateRecord {
    pub variant_key: TestVariantKey,
    pub state: FlakyState,
    pub evidence: FlakyEvidence,
    pub updated_at_nanos: u128,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TestExplorerQuery {
    pub query: Option<String>,
    pub suite: Option<String>,
    pub service: Option<String>,
    pub service_version: Option<String>,
    pub status: Option<AttemptRollup>,
    pub flaky_state: Option<FlakyState>,
    pub configuration: Option<TestConfigurationFilter>,
    pub from_nanos: Option<u128>,
    pub to_nanos: Option<u128>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestConfigurationFilter {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TestExplorerSort {
    #[default]
    LastSeen,
    Name,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestExplorerRow {
    pub case: TestCaseRecord,
    pub variant: TestVariantRecord,
    pub invocation_id: String,
    pub rollup: AttemptRollup,
    pub attempt_count: u32,
    pub last_result: TestResultRecord,
    pub flaky: Option<TestFlakyStateRecord>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TestExplorerPage {
    pub items: Vec<TestExplorerRow>,
    pub has_more: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlakyPolicy {
    pub transition_threshold: NonZeroU32,
    pub recovery_passes: NonZeroU32,
}

impl FlakyState {
    #[must_use]
    pub fn transition(self, evidence: FlakyEvidence, policy: FlakyPolicy) -> Self {
        if evidence.consistently_failing {
            return Self::Broken;
        }
        if evidence.same_commit_divergence
            || evidence.intra_invocation_mix
            || evidence.window_transition_count >= policy.transition_threshold.get()
        {
            return Self::Flaky;
        }
        if self == Self::Flaky && evidence.consecutive_passes >= policy.recovery_passes.get() {
            return Self::Fixed;
        }
        self
    }
}

#[cfg(test)]
mod tests;
