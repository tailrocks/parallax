//! Bounded, replay-safe flaky-state scan tick.

use parallax_analysis::test_flakiness::{FlakyEvaluationConfig, propose_flaky_state_update};
use parallax_storage::metadata::{MetadataResult, MetadataStore};
use parallax_storage::model::{FlakyPolicy, TestFlakyCursor};
use std::num::{NonZeroU32, NonZeroU128, NonZeroUsize};

const FLAKY_WINDOW_NANOS: u128 = 30 * 24 * 60 * 60 * 1_000_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FlakyJobPolicy {
    pub window_nanos: NonZeroU128,
    pub state_policy: FlakyPolicy,
    pub min_consistent_failures: NonZeroU32,
    pub candidate_limit: NonZeroUsize,
    pub result_limit: NonZeroUsize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FlakyTickReport {
    pub candidates_seen: usize,
    pub states_upserted: usize,
    pub truncated_histories: usize,
    pub evaluation_errors: usize,
    pub next_cursor: Option<TestFlakyCursor>,
}

#[must_use]
pub(crate) fn preliminary_job_policy() -> FlakyJobPolicy {
    FlakyJobPolicy {
        window_nanos: NonZeroU128::new(FLAKY_WINDOW_NANOS).unwrap_or(NonZeroU128::MIN),
        state_policy: parallax_analysis::test_flakiness::default_flaky_policy(),
        min_consistent_failures: NonZeroU32::new(3).unwrap_or(NonZeroU32::MIN),
        candidate_limit: NonZeroUsize::new(200).unwrap_or(NonZeroUsize::MIN),
        result_limit: NonZeroUsize::new(500).unwrap_or(NonZeroUsize::MIN),
    }
}

pub(crate) async fn tick_once(
    metadata: &dyn MetadataStore,
    now_nanos: u128,
    policy: FlakyJobPolicy,
    cursor: Option<&TestFlakyCursor>,
) -> MetadataResult<FlakyTickReport> {
    let from_nanos = now_nanos.saturating_sub(policy.window_nanos.get());
    let page = metadata
        .test_flaky_candidates(from_nanos, now_nanos, cursor, policy.candidate_limit.get())
        .await?;
    let mut report = FlakyTickReport {
        candidates_seen: page.items.len(),
        ..FlakyTickReport::default()
    };

    for candidate in &page.items {
        let history = metadata
            .test_results_for_variant_window(
                candidate.variant_key.as_str(),
                from_nanos,
                now_nanos,
                policy.result_limit.get(),
            )
            .await?;
        if history.truncated {
            report.truncated_histories += 1;
            continue;
        }
        let previous = metadata
            .test_flaky_state(candidate.variant_key.as_str())
            .await?;
        let update = propose_flaky_state_update(
            candidate.variant_key.clone(),
            history.items,
            previous.as_ref(),
            FlakyEvaluationConfig {
                evaluated_at_nanos: now_nanos,
                window_nanos: policy.window_nanos.get(),
                policy: policy.state_policy,
                min_consistent_failures: policy.min_consistent_failures.get(),
            },
        );
        let Ok(update) = update else {
            report.evaluation_errors += 1;
            continue;
        };
        metadata.upsert_test_flaky_state(&update.record).await?;
        report.states_upserted += 1;
    }

    report.next_cursor = page
        .has_more
        .then_some(page.items.last())
        .flatten()
        .map(|last| TestFlakyCursor {
            last_ended_nanos: last.last_ended_nanos,
            variant_key: last.variant_key.clone(),
        });
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_analysis::test_flakiness::default_flaky_policy;
    use parallax_metadata::TursoMetadataStore;
    use parallax_storage::model::{
        FlakyState, TestAttempt, TestConfiguration, TestResultKey, TestResultRecord, TestStatus,
        TestVariantKey, TraceId,
    };
    use std::str::FromStr;

    fn policy(candidate_limit: usize, result_limit: usize) -> FlakyJobPolicy {
        FlakyJobPolicy {
            window_nanos: NonZeroU128::new(100).unwrap_or(NonZeroU128::MIN),
            state_policy: default_flaky_policy(),
            min_consistent_failures: NonZeroU32::new(2).unwrap_or(NonZeroU32::MIN),
            candidate_limit: NonZeroUsize::new(candidate_limit).unwrap_or(NonZeroUsize::MIN),
            result_limit: NonZeroUsize::new(result_limit).unwrap_or(NonZeroUsize::MIN),
        }
    }

    fn result(variant: &TestVariantKey, attempt: u32, status: TestStatus) -> TestResultRecord {
        TestResultRecord {
            key: TestResultKey {
                variant_key: variant.clone(),
                invocation_id: "inv".into(),
                attempt: TestAttempt::new(attempt).expect("attempt"),
            },
            status,
            trace_id: TraceId::from_str("abababababababababababababababab").expect("trace"),
            span_id: format!("{attempt:016x}"),
            started_at_nanos: u128::from(attempt) * 10,
            ended_at_nanos: u128::from(attempt) * 10 + 1,
            service: "checkout".into(),
            service_version: None,
            vcs_head_revision: Some("deadbeef".into()),
            configuration: TestConfiguration::default(),
            failure_fingerprint: None,
        }
    }

    #[tokio::test]
    async fn tick_persists_complete_retry_mix_and_replays_idempotently() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("metadata.db"))
            .await
            .expect("store");
        let variant =
            TestVariantKey::from_str(&format!("tv1:{}", "a".repeat(64))).expect("variant");
        for row in [
            result(&variant, 1, TestStatus::Failed),
            result(&variant, 2, TestStatus::Passed),
        ] {
            MetadataStore::upsert_test_result(&store, &row)
                .await
                .expect("result");
        }

        let first = tick_once(&store, 100, policy(10, 10), None)
            .await
            .expect("tick");
        assert_eq!(first.candidates_seen, 1);
        assert_eq!(first.states_upserted, 1);
        assert_eq!(first.next_cursor, None);
        let state = MetadataStore::test_flaky_state(&store, variant.as_str())
            .await
            .expect("state")
            .expect("record");
        assert_eq!(state.state, FlakyState::Flaky);
        assert!(state.evidence.intra_invocation_mix);

        let replay = tick_once(&store, 100, policy(10, 10), None)
            .await
            .expect("replay");
        assert_eq!(replay.states_upserted, 1);
        assert_eq!(
            MetadataStore::test_flaky_state(&store, variant.as_str())
                .await
                .expect("state"),
            Some(state)
        );
    }

    #[tokio::test]
    async fn truncated_history_preserves_prior_state() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("metadata.db"))
            .await
            .expect("store");
        let variant =
            TestVariantKey::from_str(&format!("tv1:{}", "b".repeat(64))).expect("variant");
        for row in [
            result(&variant, 1, TestStatus::Failed),
            result(&variant, 2, TestStatus::Passed),
        ] {
            MetadataStore::upsert_test_result(&store, &row)
                .await
                .expect("result");
        }
        let report = tick_once(&store, 100, policy(10, 1), None)
            .await
            .expect("tick");
        assert_eq!(report.truncated_histories, 1);
        assert_eq!(report.states_upserted, 0);
        assert!(
            MetadataStore::test_flaky_state(&store, variant.as_str())
                .await
                .expect("state")
                .is_none()
        );
    }

    #[tokio::test]
    async fn cursor_pages_complete_one_frozen_sweep() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("metadata.db"))
            .await
            .expect("store");
        let first_variant =
            TestVariantKey::from_str(&format!("tv1:{}", "c".repeat(64))).expect("variant");
        let second_variant =
            TestVariantKey::from_str(&format!("tv1:{}", "d".repeat(64))).expect("variant");
        for row in [
            result(&first_variant, 1, TestStatus::Passed),
            result(&second_variant, 1, TestStatus::Passed),
        ] {
            MetadataStore::upsert_test_result(&store, &row)
                .await
                .expect("result");
        }

        let first = tick_once(&store, 100, policy(1, 10), None)
            .await
            .expect("first page");
        assert_eq!(first.states_upserted, 1);
        let cursor = first.next_cursor.expect("cursor");
        let second = tick_once(&store, 100, policy(1, 10), Some(&cursor))
            .await
            .expect("second page");
        assert_eq!(second.states_upserted, 1);
        assert_eq!(second.next_cursor, None);
        for variant in [first_variant, second_variant] {
            assert!(
                MetadataStore::test_flaky_state(&store, variant.as_str())
                    .await
                    .expect("state")
                    .is_some()
            );
        }
    }
}
