use super::*;
use parallax_model::{
    FlakyPolicy, FlakyState, TestAttempt, TestConfiguration, TestResultKey, TestStatus,
    TestVariantKey, TraceId,
};
use std::num::NonZeroU32;
use std::str::FromStr;

fn result(invocation: &str, attempt: u32, status: TestStatus, ended: u128) -> TestResultRecord {
    TestResultRecord {
        key: TestResultKey {
            variant_key: TestVariantKey::from_str(&format!("tv1:{}", "a".repeat(64)))
                .expect("variant"),
            invocation_id: invocation.into(),
            attempt: TestAttempt::new(attempt).expect("attempt"),
        },
        status,
        trace_id: TraceId::from_str("abababababababababababababababab").expect("trace"),
        span_id: "cdcdcdcdcdcdcdcd".into(),
        started_at_nanos: ended.saturating_sub(1),
        ended_at_nanos: ended,
        service: "checkout".into(),
        service_version: None,
        vcs_head_revision: Some("abc".into()),
        configuration: TestConfiguration::default(),
        failure_fingerprint: None,
    }
}

#[test]
fn retry_mix_is_not_a_cross_invocation_transition() {
    let rows = vec![
        result("inv", 2, TestStatus::Passed, 20),
        result("inv", 1, TestStatus::Failed, 10),
    ];
    let evidence = evaluate_flaky_evidence(rows.clone(), 20, 20, 2).expect("evidence");
    assert!(evidence.intra_invocation_mix);
    assert!(!evidence.same_commit_divergence);
    assert_eq!(evidence.window_transition_count, 0);
    let reversed = rows.into_iter().rev().collect();
    assert_eq!(
        evidence,
        evaluate_flaky_evidence(reversed, 20, 20, 2).expect("evidence")
    );
}

#[test]
fn cross_invocation_signals_and_window_boundary_are_exact() {
    let rows = vec![
        result("old", 1, TestStatus::Failed, 9),
        result("fail", 1, TestStatus::Failed, 10),
        result("pass", 1, TestStatus::Passed, 20),
    ];
    let evidence = evaluate_flaky_evidence(rows, 20, 10, 2).expect("evidence");
    assert!(evidence.same_commit_divergence);
    assert_eq!(evidence.window_transition_count, 1);
    assert_eq!(evidence.consecutive_passes, 1);
}

#[test]
fn missing_revision_degrades_and_consistent_broken_precedes_flaky() {
    let mut first = result("one", 1, TestStatus::Broken, 10);
    first.vcs_head_revision = None;
    let mut second = result("two", 1, TestStatus::Failed, 20);
    second.vcs_head_revision = Some(" ".into());
    let evidence = evaluate_flaky_evidence(vec![first, second], 20, 20, 2).expect("evidence");
    assert!(!evidence.same_commit_divergence);
    assert!(evidence.consistently_failing);
    assert_eq!(
        FlakyState::Healthy.transition(
            evidence,
            FlakyPolicy {
                transition_threshold: NonZeroU32::new(1).expect("threshold"),
                recovery_passes: NonZeroU32::new(30).expect("passes"),
            },
        ),
        FlakyState::Broken
    );
}

#[test]
fn invalid_bounds_and_duplicate_attempts_fail_closed() {
    assert_eq!(
        evaluate_flaky_evidence(Vec::new(), 1, 0, 2),
        Err(FlakyEvaluationError::ZeroWindow)
    );
    assert_eq!(
        evaluate_flaky_evidence(Vec::new(), 1, 1, 0),
        Err(FlakyEvaluationError::ZeroFailureThreshold)
    );
    let row = result("dup", 1, TestStatus::Failed, 1);
    assert_eq!(
        evaluate_flaky_evidence(vec![row.clone(), row], 1, 1, 2),
        Err(FlakyEvaluationError::DuplicateAttempt)
    );
}

#[test]
fn propose_update_marks_flaky_on_intra_invocation_mix() {
    use crate::test_flakiness::{
        FlakyEvaluationConfig, default_flaky_policy, propose_flaky_state_update,
    };
    use parallax_model::FlakyState;

    let rows = vec![
        result("inv-1", 1, TestStatus::Failed, 10),
        result("inv-1", 2, TestStatus::Passed, 11),
    ];
    let variant = TestVariantKey::from_str(&format!("tv1:{}", "a".repeat(64))).expect("variant");
    let policy = default_flaky_policy();
    assert_eq!(policy.recovery_passes.get(), 30);
    let update = propose_flaky_state_update(
        variant,
        rows,
        None,
        FlakyEvaluationConfig {
            evaluated_at_nanos: 20,
            window_nanos: 20,
            policy,
            min_consistent_failures: 2,
        },
    )
    .expect("update");
    assert!(update.changed);
    assert_eq!(update.record.state, FlakyState::Flaky);
    assert!(update.record.evidence.intra_invocation_mix);
    assert_eq!(update.previous, None);
}
