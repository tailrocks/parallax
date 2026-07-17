use super::*;
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::str::FromStr;

fn key(prefix: &str, digit: char) -> String {
    format!("{prefix}:{}", digit.to_string().repeat(64))
}

fn result(variant: &str, invocation: &str, attempt: u32, status: TestStatus) -> TestResultRecord {
    TestResultRecord {
        key: TestResultKey {
            variant_key: TestVariantKey::from_str(variant).expect("variant key"),
            invocation_id: invocation.to_string(),
            attempt: TestAttempt::new(attempt).expect("attempt"),
        },
        status,
        trace_id: TraceId::from_str("abababababababababababababababab").expect("trace"),
        span_id: "cdcdcdcdcdcdcdcd".to_string(),
        started_at_nanos: 1,
        ended_at_nanos: 2,
        service: "checkout".to_string(),
        service_version: Some("1.2.3".to_string()),
        vcs_head_revision: Some("deadbeef".to_string()),
        configuration: TestConfiguration {
            dimensions: BTreeMap::from([(
                "test.configuration.os".to_string(),
                "linux".to_string(),
            )]),
        },
        failure_fingerprint: None,
    }
}

#[test]
fn keys_and_attempts_reject_noncanonical_wire_values() {
    let case = TestCaseKey::from_str(&key("tc1", 'a')).expect("case key");
    assert_eq!(case.as_str(), key("tc1", 'a'));
    TestCaseKey::from_str(&key("tv1", 'a')).expect_err("wrong prefix");
    TestCaseKey::from_str(&key("tc1", 'A')).expect_err("uppercase");
    TestCaseKey::from_str("tc1:abc").expect_err("short digest");
    TestAttempt::new(0).expect_err("zero attempt");
    assert_eq!(TestAttempt::new(1).expect("attempt").get(), 1);
    serde_json::from_str::<TestVariantKey>(&format!("\"{}\"", key("tv1", 'F')))
        .expect_err("serde uses validation");
}

#[test]
fn attempt_chain_preserves_attempts_and_never_masks_flaky_pass() {
    let variant = key("tv1", 'b');
    let chain = AttemptChain::new(vec![
        result(&variant, "inv-1", 2, TestStatus::Passed),
        result(&variant, "inv-1", 1, TestStatus::Failed),
    ])
    .expect("chain");
    assert_eq!(chain.results().len(), 2);
    assert_eq!(chain.results()[0].key.attempt.get(), 1);
    assert_eq!(chain.rollup(), AttemptRollup::FlakyPass);

    AttemptChain::new(Vec::new()).expect_err("empty");
    AttemptChain::new(vec![
        result(&variant, "inv-1", 1, TestStatus::Passed),
        result(&variant, "inv-1", 1, TestStatus::Failed),
    ])
    .expect_err("duplicate attempt");
    AttemptChain::new(vec![
        result(&variant, "inv-1", 1, TestStatus::Passed),
        result(&variant, "inv-2", 2, TestStatus::Passed),
    ])
    .expect_err("mixed invocation");
}

#[test]
fn flaky_state_requires_evidence_and_recovers_at_policy_threshold() {
    let policy = FlakyPolicy {
        transition_threshold: NonZeroU32::new(3).expect("nonzero"),
        recovery_passes: NonZeroU32::new(30).expect("nonzero"),
    };
    assert_eq!(
        FlakyState::Healthy.transition(FlakyEvidence::default(), policy),
        FlakyState::Healthy
    );
    assert_eq!(
        FlakyState::Healthy.transition(
            FlakyEvidence {
                intra_invocation_mix: true,
                ..FlakyEvidence::default()
            },
            policy,
        ),
        FlakyState::Flaky
    );
    assert_eq!(
        FlakyState::Flaky.transition(
            FlakyEvidence {
                consecutive_passes: 29,
                ..FlakyEvidence::default()
            },
            policy,
        ),
        FlakyState::Flaky
    );
    assert_eq!(
        FlakyState::Flaky.transition(
            FlakyEvidence {
                consecutive_passes: 30,
                ..FlakyEvidence::default()
            },
            policy,
        ),
        FlakyState::Fixed
    );
    assert_eq!(
        FlakyState::Healthy.transition(
            FlakyEvidence {
                consistently_failing: true,
                intra_invocation_mix: true,
                ..FlakyEvidence::default()
            },
            policy,
        ),
        FlakyState::Broken
    );
}
