use super::*;

fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("metadata.db");
    (directory, path)
}

const MINUTE_NANOS: u128 = 60 * 1_000_000_000;

fn rule(id: &str) -> AlertRuleRecord {
    AlertRuleRecord {
        id: id.to_string(),
        name: "High error rate".to_string(),
        enabled: true,
        signal_type: "error_rate".to_string(),
        services: "[\"checkout\"]".to_string(),
        exclude_services: "[]".to_string(),
        attribute_filters: "[]".to_string(),
        group_by: Some("service".to_string()),
        comparator: "gt".to_string(),
        threshold: 0.2,
        threshold_upper: None,
        window_minutes: 5,
        minimum_sample_count: 10,
        consecutive_breaches_required: 2,
        consecutive_healthy_required: 2,
        no_data_behavior: "skip".to_string(),
        severity: "critical".to_string(),
        renotify_interval_minutes: 30,
        destination_ids: "[\"dest-1\"]".to_string(),
        metric_name: None,
        metric_aggregation: None,
        created_at_nanos: MINUTE_NANOS,
        updated_at_nanos: MINUTE_NANOS,
    }
}

#[tokio::test]
async fn rule_round_trip_and_update_preserves_created_at() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    let original = rule("r1");
    store.alert_rule_save(&original).await.expect("save");
    let loaded = store.alert_rule("r1").await.expect("get").expect("some");
    assert_eq!(loaded, original);

    let mut updated = original.clone();
    updated.name = "Renamed".to_string();
    updated.threshold = 0.5;
    updated.created_at_nanos = 99 * MINUTE_NANOS; // must be ignored on update
    updated.updated_at_nanos = 2 * MINUTE_NANOS;
    store.alert_rule_save(&updated).await.expect("update");
    let reloaded = store.alert_rule("r1").await.expect("get").expect("some");
    assert_eq!(reloaded.name, "Renamed");
    assert!((reloaded.threshold - 0.5).abs() < f64::EPSILON);
    assert_eq!(reloaded.created_at_nanos, MINUTE_NANOS);
    assert_eq!(reloaded.updated_at_nanos, 2 * MINUTE_NANOS);
    assert_eq!(store.alert_rules().await.expect("list").len(), 1);
}

#[tokio::test]
async fn rule_claim_is_cas_and_respects_interval() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    store.alert_rule_save(&rule("r1")).await.expect("save");

    let t0 = 100 * MINUTE_NANOS;
    assert!(store.alert_rule_claim("r1", t0, 30).await.expect("claim"));
    // Re-claim within the interval fails.
    assert!(
        !store
            .alert_rule_claim("r1", t0 + MINUTE_NANOS / 6, 30)
            .await
            .expect("claim")
    );
    // After the interval it succeeds again.
    assert!(
        store
            .alert_rule_claim("r1", t0 + MINUTE_NANOS, 30)
            .await
            .expect("claim")
    );
    // Disabled rules are never claimable.
    store
        .alert_rule_set_enabled("r1", false)
        .await
        .expect("disable");
    assert!(
        !store
            .alert_rule_claim("r1", t0 + 10 * MINUTE_NANOS, 30)
            .await
            .expect("claim")
    );
}

#[tokio::test]
async fn state_upsert_round_trip() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    let state = AlertRuleStateRecord {
        rule_id: "r1".to_string(),
        group_key: "checkout".to_string(),
        consecutive_breaches: 2,
        consecutive_healthy: 0,
        incident_open: true,
        last_notified_at_nanos: Some(MINUTE_NANOS),
        last_status: Some("breach".to_string()),
        last_value: Some(0.42),
        last_sample_count: 120,
        last_evaluated_at_nanos: Some(2 * MINUTE_NANOS),
        last_error: None,
    };
    store.alert_rule_state_upsert(&state).await.expect("upsert");
    let loaded = store
        .alert_rule_state("r1", "checkout")
        .await
        .expect("get")
        .expect("some");
    assert_eq!(loaded, state);

    let mut healed = state.clone();
    healed.incident_open = false;
    healed.consecutive_healthy = 2;
    healed.consecutive_breaches = 0;
    healed.last_status = Some("healthy".to_string());
    store
        .alert_rule_state_upsert(&healed)
        .await
        .expect("upsert");
    let states = store.alert_rule_states("r1").await.expect("list");
    assert_eq!(states, vec![healed]);
}

#[tokio::test]
async fn incident_lifecycle_never_double_opens() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    let incident = AlertIncidentRecord {
        id: "i1".to_string(),
        rule_id: "r1".to_string(),
        group_key: "checkout".to_string(),
        status: "open".to_string(),
        severity: "critical".to_string(),
        first_triggered_at_nanos: MINUTE_NANOS,
        last_triggered_at_nanos: MINUTE_NANOS,
        resolved_at_nanos: None,
        last_value: Some(0.4),
        last_notified_at_nanos: Some(MINUTE_NANOS),
        bundle_hash: None,
        bundle_assembled_at_nanos: None,
        bundle_top_hypothesis: None,
        bundle_deploy_adjacency: None,
        bundle_error: None,
    };
    assert!(store.alert_incident_open(&incident).await.expect("open"));
    // Second open for the same (rule, group) is a no-op.
    let mut duplicate = incident.clone();
    duplicate.id = "i2".to_string();
    assert!(!store.alert_incident_open(&duplicate).await.expect("open"));

    store
        .alert_incident_touch("i1", 2 * MINUTE_NANOS, Some(0.6), true)
        .await
        .expect("touch");
    let open = store
        .alert_incident_open_for("r1", "checkout")
        .await
        .expect("get")
        .expect("some");
    assert_eq!(open.id, "i1");
    assert_eq!(open.last_triggered_at_nanos, 2 * MINUTE_NANOS);
    assert_eq!(open.last_value, Some(0.6));
    assert_eq!(open.last_notified_at_nanos, Some(2 * MINUTE_NANOS));

    let resolved = store
        .alert_incident_resolve("r1", "checkout", 3 * MINUTE_NANOS, Some(0.01))
        .await
        .expect("resolve");
    assert_eq!(resolved.as_deref(), Some("i1"));
    // No open incident remains; resolving again is a no-op.
    assert!(
        store
            .alert_incident_open_for("r1", "checkout")
            .await
            .expect("get")
            .is_none()
    );
    assert!(
        store
            .alert_incident_resolve("r1", "checkout", 4 * MINUTE_NANOS, None)
            .await
            .expect("resolve")
            .is_none()
    );
    // A fresh breach can open a new incident after resolution.
    let mut second = incident.clone();
    second.id = "i3".to_string();
    assert!(store.alert_incident_open(&second).await.expect("open"));

    let all = store
        .alert_incidents(None, Some("r1"), 10)
        .await
        .expect("list");
    assert_eq!(all.len(), 2);
    let open_only = store
        .alert_incidents(Some("open"), None, 10)
        .await
        .expect("list");
    assert_eq!(open_only.len(), 1);
    assert_eq!(open_only[0].id, "i3");
}

#[tokio::test]
async fn destination_round_trip() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    let destination = AlertDestinationRecord {
        id: "d1".to_string(),
        name: "Ops webhook".to_string(),
        kind: "webhook".to_string(),
        config: "{\"url\":\"http://127.0.0.1:9000/hook\"}".to_string(),
        created_at_nanos: MINUTE_NANOS,
        updated_at_nanos: MINUTE_NANOS,
    };
    store
        .alert_destination_save(&destination)
        .await
        .expect("save");
    assert_eq!(
        store.alert_destination("d1").await.expect("get"),
        Some(destination)
    );
    assert!(store.alert_destination_delete("d1").await.expect("delete"));
    assert!(store.alert_destinations().await.expect("list").is_empty());
}

fn delivery(id: &str, key: &str, due_nanos: u128) -> AlertDeliveryEventRecord {
    AlertDeliveryEventRecord {
        id: id.to_string(),
        incident_id: "i1".to_string(),
        destination_id: "d1".to_string(),
        event_type: "triggered".to_string(),
        status: "pending".to_string(),
        attempt_count: 0,
        next_attempt_at_nanos: due_nanos,
        claimed_by: None,
        claim_expires_at_nanos: None,
        delivered_at_nanos: None,
        last_error: None,
        delivery_key: key.to_string(),
        created_at_nanos: due_nanos,
    }
}

#[tokio::test]
async fn delivery_enqueue_is_idempotent_and_lease_claims() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    let due = MINUTE_NANOS;
    assert!(
        store
            .alert_delivery_enqueue(&delivery("e1", "i1:d1:triggered:0", due))
            .await
            .expect("enqueue")
    );
    // Same delivery key: ignored.
    assert!(
        !store
            .alert_delivery_enqueue(&delivery("e2", "i1:d1:triggered:0", due))
            .await
            .expect("enqueue")
    );
    // Not yet due.
    assert!(
        store
            .alert_deliveries_claim("w1", due - 1_000_000_000, 60, 10)
            .await
            .expect("claim")
            .is_empty()
    );
    // Due: claimed with a lease.
    let claimed = store
        .alert_deliveries_claim("w1", due, 60, 10)
        .await
        .expect("claim");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].id, "e1");
    assert_eq!(claimed[0].claimed_by.as_deref(), Some("w1"));
    // While leased, a second worker gets nothing.
    assert!(
        store
            .alert_deliveries_claim("w2", due + 1_000_000_000, 60, 10)
            .await
            .expect("claim")
            .is_empty()
    );
    // After lease expiry another worker can take it over.
    let takeover = store
        .alert_deliveries_claim("w2", due + 2 * MINUTE_NANOS, 60, 10)
        .await
        .expect("claim");
    assert_eq!(takeover.len(), 1);
    assert_eq!(takeover[0].claimed_by.as_deref(), Some("w2"));
}

#[tokio::test]
async fn concurrent_claimers_exactly_one_wins() {
    let (_dir, path) = temp_db();
    let store = std::sync::Arc::new(TursoMetadataStore::open(path).await.expect("open"));
    let due = MINUTE_NANOS;
    store
        .alert_delivery_enqueue(&delivery("e-conc", "k-conc", due))
        .await
        .expect("enqueue");
    let first = {
        let store = std::sync::Arc::clone(&store);
        tokio::spawn(async move { store.alert_deliveries_claim("w1", due, 60, 10).await })
    };
    let second = {
        let store = std::sync::Arc::clone(&store);
        tokio::spawn(async move { store.alert_deliveries_claim("w2", due, 60, 10).await })
    };
    let left = first.await.expect("join").expect("claim");
    let right = second.await.expect("join").expect("claim");
    assert_eq!(left.len() + right.len(), 1);
    let later = due + 2 * MINUTE_NANOS;
    let reclaim = store
        .alert_deliveries_claim("w3", later, 60, 10)
        .await
        .expect("reclaim");
    assert_eq!(reclaim.len(), 1);
    assert_eq!(reclaim[0].claimed_by.as_deref(), Some("w3"));
}

#[tokio::test]
async fn delivery_failure_backoff_and_dead_letter() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    let due = MINUTE_NANOS;
    store
        .alert_delivery_enqueue(&delivery("e1", "k1", due))
        .await
        .expect("enqueue");
    store
        .alert_deliveries_claim("w1", due, 60, 10)
        .await
        .expect("claim");
    store
        .alert_delivery_mark_failed("e1", "HTTP 500", due + 5 * MINUTE_NANOS, false)
        .await
        .expect("fail");
    let events = store
        .alert_deliveries_for_incident("i1")
        .await
        .expect("list");
    assert_eq!(events[0].attempt_count, 1);
    assert_eq!(events[0].status, "pending");
    assert_eq!(events[0].last_error.as_deref(), Some("HTTP 500"));
    assert_eq!(events[0].claimed_by, None);
    // Not claimable before the backed-off next attempt.
    assert!(
        store
            .alert_deliveries_claim("w1", due + MINUTE_NANOS, 60, 10)
            .await
            .expect("claim")
            .is_empty()
    );
    // Dead-letter removes it from the pending pool entirely.
    store
        .alert_delivery_mark_failed("e1", "HTTP 500", due + 6 * MINUTE_NANOS, true)
        .await
        .expect("fail");
    assert!(
        store
            .alert_deliveries_claim("w1", due + 10 * MINUTE_NANOS, 60, 10)
            .await
            .expect("claim")
            .is_empty()
    );
    // Success path on a fresh row.
    store
        .alert_delivery_enqueue(&delivery("e2", "k2", due))
        .await
        .expect("enqueue");
    store
        .alert_delivery_mark_delivered("e2", due + MINUTE_NANOS)
        .await
        .expect("deliver");
    let events = store
        .alert_deliveries_for_incident("i1")
        .await
        .expect("list");
    let delivered = events.iter().find(|e| e.id == "e2").expect("row");
    assert_eq!(delivered.status, "delivered");
    assert_eq!(delivered.delivered_at_nanos, Some(due + MINUTE_NANOS));
}

#[tokio::test]
async fn checks_append_and_prune_to_retention() {
    let (_dir, path) = temp_db();
    let store = TursoMetadataStore::open(path).await.expect("open");
    let total = ALERT_CHECKS_KEEP_PER_RULE + 25;
    for index in 0..total {
        store
            .alert_check_insert(&AlertCheckRecord {
                rule_id: "r1".to_string(),
                group_key: "checkout".to_string(),
                checked_at_nanos: (index as u128 + 1) * MINUTE_NANOS,
                value: Some(f64::from(u32::try_from(index).expect("small index")) / 100.0),
                sample_count: 10,
                status: "healthy".to_string(),
                error: None,
            })
            .await
            .expect("insert");
    }
    let checks = store.alert_checks("r1", total + 10).await.expect("list");
    assert_eq!(checks.len(), ALERT_CHECKS_KEEP_PER_RULE);
    // Newest first; oldest retained row is `total - keep + 1`.
    assert_eq!(checks[0].checked_at_nanos, total as u128 * MINUTE_NANOS);
    let oldest = checks.last().expect("row");
    assert_eq!(
        oldest.checked_at_nanos,
        (total - ALERT_CHECKS_KEEP_PER_RULE + 1) as u128 * MINUTE_NANOS
    );
    // Other rules are untouched by pruning.
    store
        .alert_check_insert(&AlertCheckRecord {
            rule_id: "r2".to_string(),
            group_key: String::new(),
            checked_at_nanos: MINUTE_NANOS,
            value: None,
            sample_count: 0,
            status: "no_data".to_string(),
            error: None,
        })
        .await
        .expect("insert");
    assert_eq!(store.alert_checks("r2", 10).await.expect("list").len(), 1);
}
