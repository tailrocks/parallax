use super::super::*;
use parallax_model::IssueOccurrence;
use parallax_storage::{PruneClass, PruneExclusionKind, PruneStore};

fn issue_occurrence<'a>(
    fingerprint: &'a str,
    attributes: &'a serde_json::Value,
) -> IssueOccurrence<'a> {
    IssueOccurrence {
        occurrence_id: format!("{fingerprint}:1").into(),
        fingerprint,
        title: fingerprint.to_string(),
        error_type: "Error",
        culprit: None,
        service: "svc",
        ts_nanos: 1_000_000,
        trace_id: None,
        attributes,
    }
}

#[tokio::test]
async fn issue_discovery_uses_persisted_resolution_time_and_preserves_open_issues() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = TursoMetadataStore::open(directory.path().join("metadata.db"))
        .await
        .expect("open metadata");
    let attributes = serde_json::json!({});
    for fingerprint in ["eligible", "not-expired", "open"] {
        store
            .upsert_issue_occurrence(&issue_occurrence(fingerprint, &attributes))
            .await
            .expect("seed issue");
    }
    store
        .set_issue_status("eligible", "resolved", 20_000_000)
        .await
        .expect("resolve eligible");
    store
        .set_issue_status("not-expired", "resolved", 30_000_000)
        .await
        .expect("resolve recent");

    let item = store
        .issue_prune_item(20_000_000)
        .await
        .expect("discover issue candidates");

    assert_eq!(item.store, PruneStore::Turso);
    assert_eq!(item.class, PruneClass::Issues);
    assert_eq!(item.estimate.rows, Some(1));
    assert_eq!(item.exclusions[0].kind, PruneExclusionKind::Unresolved);
    assert_eq!(item.exclusions[0].count, 1);
    assert_eq!(item.exclusions[1].kind, PruneExclusionKind::NotExpired);
    assert_eq!(item.exclusions[1].count, 1);

    let dependents = store
        .issue_dependent_prune_items(20_000_000)
        .await
        .expect("discover issue-owned candidates");
    assert_eq!(dependents.len(), 2);
    assert_eq!(dependents[0].class, PruneClass::IssueBuckets);
    assert_eq!(dependents[0].estimate.rows, Some(1));
    assert_eq!(dependents[1].class, PruneClass::IssueOccurrences);
    assert_eq!(dependents[1].estimate.rows, Some(1));
    assert!(
        dependents
            .iter()
            .all(|item| item.warnings == ["deleted only through the eligible issue owner cascade"])
    );

    store
        .set_issue_status("eligible", "open", 40_000_000)
        .await
        .expect("reopen eligible");
    let reopened = store
        .issue_prune_item(40_000_000)
        .await
        .expect("rediscover issue candidates");
    assert_eq!(reopened.estimate.rows, Some(1));
    assert_eq!(reopened.exclusions[0].count, 2);
}

#[tokio::test]
async fn invocation_discovery_counts_eligible_active_and_not_expired_rows() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = TursoMetadataStore::open(directory.path().join("metadata.db"))
        .await
        .expect("open metadata");

    store
        .start_invocation("eligible", None, None, 1_000_000)
        .await
        .expect("start eligible");
    store
        .finish_invocation("eligible", 20_000_000, 0, Some("success"))
        .await
        .expect("finish eligible");
    store
        .start_invocation("not-expired", None, None, 2_000_000)
        .await
        .expect("start recent");
    store
        .finish_invocation("not-expired", 30_000_000, 0, Some("success"))
        .await
        .expect("finish recent");
    store
        .start_invocation("active", None, None, 3_000_000)
        .await
        .expect("start active");

    let item = store
        .invocation_prune_item(20_000_000)
        .await
        .expect("discover invocation candidates");

    assert_eq!(item.store, PruneStore::Turso);
    assert_eq!(item.class, PruneClass::Invocations);
    assert_eq!(item.target, "invocations");
    assert_eq!(item.cutoff_nanos, 20_000_000);
    assert_eq!(item.estimate.rows, Some(1));
    assert_eq!(item.estimate.objects, None);
    assert_eq!(item.estimate.bytes, None);
    assert_eq!(item.exclusions.len(), 2);
    assert_eq!(item.exclusions[0].kind, PruneExclusionKind::Active);
    assert_eq!(item.exclusions[0].count, 1);
    assert_eq!(item.exclusions[1].kind, PruneExclusionKind::NotExpired);
    assert_eq!(item.exclusions[1].count, 1);
    assert!(item.warnings.is_empty());
}
