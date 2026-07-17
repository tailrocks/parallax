use super::super::*;
use parallax_storage::{MetadataPruneStore, PruneClass, PruneExclusionKind, PruneStore};

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
