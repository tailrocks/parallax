use super::*;

fn snapshot() -> PruneSnapshot {
    PruneSnapshot {
        config_generation: "config-a".into(),
        protection_generation: "pins-a".into(),
        catalog_fingerprint: "catalog-a".into(),
    }
}

fn item(store: PruneStore, class: PruneClass, target: &str) -> PruneItem {
    PruneItem {
        store,
        class,
        target: target.into(),
        cutoff_nanos: 100,
        estimate: PruneEstimate {
            rows: Some(2),
            objects: None,
            bytes: Some(32),
        },
        exclusions: Vec::new(),
        warnings: Vec::new(),
    }
}

#[test]
fn equivalent_inputs_have_one_canonical_order_and_identity() {
    let greptime = item(
        PruneStore::Greptime,
        PruneClass::RawLogs,
        "opentelemetry_logs",
    );
    let turso = item(PruneStore::Turso, PruneClass::Issues, "issues");

    let left = PrunePlan::build(
        100,
        snapshot(),
        vec![turso.clone(), greptime.clone()],
        PrunePlanLimits::default(),
    )
    .expect("valid plan");
    let right = PrunePlan::build(
        100,
        snapshot(),
        vec![greptime, turso],
        PrunePlanLimits::default(),
    )
    .expect("valid plan");

    assert_eq!(left, right);
    assert_eq!(left.items[0].store, PruneStore::Greptime);
    assert_eq!(left.items[1].store, PruneStore::Turso);
    assert_eq!(
        left.plan_id,
        "6e953fddfb2026501f08f31627ebb3d22d770278bcdd7d2e58945ef502a310a0"
    );
}

#[test]
fn equivalent_annotation_sets_have_one_canonical_identity() {
    let mut left_item = item(PruneStore::Turso, PruneClass::Issues, "issues");
    left_item.exclusions = vec![
        PruneExclusion {
            kind: PruneExclusionKind::Pinned,
            count: 1,
        },
        PruneExclusion {
            kind: PruneExclusionKind::Unresolved,
            count: 2,
        },
    ];
    left_item.warnings = vec!["z-warning".into(), "a-warning".into()];
    let mut right_item = left_item.clone();
    right_item.exclusions.reverse();
    right_item.warnings.reverse();

    let left = PrunePlan::build(100, snapshot(), vec![left_item], PrunePlanLimits::default())
        .expect("valid plan");
    let right = PrunePlan::build(
        100,
        snapshot(),
        vec![right_item],
        PrunePlanLimits::default(),
    )
    .expect("valid plan");

    assert_eq!(left, right);
}

#[test]
fn persisted_plan_decoding_revalidates_version_identity_and_bounds() {
    let plan = PrunePlan::build(
        100,
        snapshot(),
        vec![item(PruneStore::Turso, PruneClass::Issues, "issues")],
        PrunePlanLimits::default(),
    )
    .expect("valid plan");
    let encoded = serde_json::to_string(&plan).expect("encode plan");
    assert_eq!(
        PrunePlan::decode(&encoded, PrunePlanLimits::default()).expect("decode plan"),
        plan
    );

    let mut tampered: serde_json::Value = serde_json::from_str(&encoded).expect("parse plan");
    tampered["items"][0]["estimate"]["rows"] = serde_json::json!(99);
    assert!(matches!(
        PrunePlan::decode(
            &serde_json::to_string(&tampered).expect("encode tampered plan"),
            PrunePlanLimits::default()
        ),
        Err(PrunePlanError::PlanIntegrityMismatch)
    ));

    tampered = serde_json::from_str(&encoded).expect("parse plan");
    tampered["contract_version"] = serde_json::json!(2);
    assert!(matches!(
        PrunePlan::decode(
            &serde_json::to_string(&tampered).expect("encode future plan"),
            PrunePlanLimits::default()
        ),
        Err(PrunePlanError::UnsupportedContractVersion(2))
    ));

    assert!(matches!(
        PrunePlan::decode(
            &encoded,
            PrunePlanLimits {
                max_items: 0,
                ..PrunePlanLimits::default()
            }
        ),
        Err(PrunePlanError::TooManyItems {
            actual: 1,
            limit: 0
        })
    ));
}

#[test]
fn execution_fails_closed_when_pin_protection_is_unavailable() {
    let unavailable = PruneSnapshot {
        protection_generation: "pins:none".into(),
        ..snapshot()
    };
    let plan = PrunePlan::build(
        100,
        unavailable.clone(),
        vec![item(PruneStore::Turso, PruneClass::Issues, "issues")],
        PrunePlanLimits::default(),
    )
    .expect("dry-run plan remains available");

    assert_eq!(
        plan.authorize(
            &PruneExecutionRequest::dry_run(plan.plan_id().to_string()),
            &unavailable
        )
        .expect("dry run remains safe"),
        PruneAuthorization::DryRun
    );
    let execute = PruneExecutionRequest::execute(plan.plan_id().to_string(), true)
        .expect("confirmed request");
    assert!(matches!(
        plan.authorize(&execute, &unavailable),
        Err(PrunePlanError::ProtectionUnavailable)
    ));
}

#[test]
fn construction_fails_closed_when_item_cap_is_exceeded() {
    let result = PrunePlan::build(
        100,
        snapshot(),
        vec![
            item(PruneStore::Greptime, PruneClass::RawLogs, "logs-a"),
            item(PruneStore::Greptime, PruneClass::RawLogs, "logs-b"),
        ],
        PrunePlanLimits {
            max_items: 1,
            ..PrunePlanLimits::default()
        },
    );

    assert!(matches!(
        result,
        Err(PrunePlanError::TooManyItems {
            actual: 2,
            limit: 1
        })
    ));
}

#[test]
fn construction_rejects_stale_or_ambiguous_plan_inputs() {
    let mut missing_generation = snapshot();
    missing_generation.protection_generation.clear();
    assert!(matches!(
        PrunePlan::build(
            100,
            missing_generation,
            Vec::new(),
            PrunePlanLimits::default()
        ),
        Err(PrunePlanError::EmptySnapshotField("protection_generation"))
    ));

    let mut wrong_cutoff = item(PruneStore::Turso, PruneClass::Issues, "issues");
    wrong_cutoff.cutoff_nanos = 99;
    assert!(matches!(
        PrunePlan::build(
            100,
            snapshot(),
            vec![wrong_cutoff],
            PrunePlanLimits::default()
        ),
        Err(PrunePlanError::CutoffMismatch {
            item: 99,
            plan: 100
        })
    ));

    let mut missing_estimate = item(PruneStore::Turso, PruneClass::Issues, "issues");
    missing_estimate.estimate = PruneEstimate {
        rows: None,
        objects: None,
        bytes: None,
    };
    assert!(matches!(
        PrunePlan::build(
            100,
            snapshot(),
            vec![missing_estimate],
            PrunePlanLimits::default()
        ),
        Err(PrunePlanError::MissingEstimate { .. })
    ));

    let mut byte_only = item(PruneStore::LocalDisk, PruneClass::Spool, "spool");
    byte_only.estimate = PruneEstimate {
        rows: None,
        objects: None,
        bytes: Some(32),
    };
    assert!(matches!(
        PrunePlan::build(100, snapshot(), vec![byte_only], PrunePlanLimits::default()),
        Err(PrunePlanError::MissingEstimate { .. })
    ));
}

#[test]
fn execution_guard_rejects_changed_configuration_protection_or_catalog() {
    let plan = PrunePlan::build(
        100,
        snapshot(),
        vec![item(PruneStore::Turso, PruneClass::Issues, "issues")],
        PrunePlanLimits::default(),
    )
    .expect("valid plan");
    plan.validate_snapshot(&snapshot())
        .expect("matching snapshot validates");

    for (field, changed) in [
        (
            "config_generation",
            PruneSnapshot {
                config_generation: "config-b".into(),
                ..snapshot()
            },
        ),
        (
            "protection_generation",
            PruneSnapshot {
                protection_generation: "pins-b".into(),
                ..snapshot()
            },
        ),
        (
            "catalog_fingerprint",
            PruneSnapshot {
                catalog_fingerprint: "catalog-b".into(),
                ..snapshot()
            },
        ),
    ] {
        assert!(matches!(
            plan.validate_snapshot(&changed),
            Err(PrunePlanError::StaleSnapshot { field: actual }) if actual == field
        ));
    }
}

#[test]
fn machine_output_preserves_typed_exclusions_estimates_and_warnings() {
    let mut issues = item(PruneStore::Turso, PruneClass::Issues, "issues");
    issues.exclusions = vec![
        PruneExclusion {
            kind: PruneExclusionKind::Unresolved,
            count: 3,
        },
        PruneExclusion {
            kind: PruneExclusionKind::Pinned,
            count: 1,
        },
    ];
    issues.warnings = vec!["physical bytes remain pending compaction".into()];
    let plan = PrunePlan::build(100, snapshot(), vec![issues], PrunePlanLimits::default())
        .expect("valid plan");

    let json = serde_json::to_value(plan).expect("machine-readable plan");
    assert_eq!(json["contract_version"], 1);
    assert_eq!(json["cutoff_nanos"], "100");
    assert_eq!(json["items"][0]["cutoff_nanos"], "100");
    assert_eq!(json["items"][0]["estimate"]["rows"], 2);
    assert_eq!(json["items"][0]["exclusions"][0]["kind"], "unresolved");
    assert_eq!(json["items"][0]["exclusions"][1]["kind"], "pinned");
    assert_eq!(
        json["items"][0]["warnings"][0],
        "physical bytes remain pending compaction"
    );
}

#[test]
fn construction_bounds_annotations_text_and_duplicate_targets() {
    let mut annotated = item(PruneStore::Turso, PruneClass::Issues, "issues");
    annotated.exclusions = vec![
        PruneExclusion {
            kind: PruneExclusionKind::Active,
            count: 1,
        },
        PruneExclusion {
            kind: PruneExclusionKind::Pinned,
            count: 1,
        },
    ];
    let tight = PrunePlanLimits {
        max_annotations_per_item: 1,
        ..PrunePlanLimits::default()
    };
    assert!(matches!(
        PrunePlan::build(100, snapshot(), vec![annotated], tight),
        Err(PrunePlanError::TooManyAnnotations {
            actual: 2,
            limit: 1,
            ..
        })
    ));

    let too_long = item(PruneStore::Turso, PruneClass::Issues, "1234567890");
    assert!(matches!(
        PrunePlan::build(
            100,
            snapshot(),
            vec![too_long],
            PrunePlanLimits {
                max_text_bytes: 9,
                ..PrunePlanLimits::default()
            }
        ),
        Err(PrunePlanError::TextTooLong {
            field: "target",
            ..
        })
    ));

    let duplicate = item(PruneStore::Turso, PruneClass::Issues, "issues");
    assert!(matches!(
        PrunePlan::build(
            100,
            snapshot(),
            vec![duplicate.clone(), duplicate],
            PrunePlanLimits::default()
        ),
        Err(PrunePlanError::DuplicateItem { .. })
    ));
}

#[test]
fn execution_requires_exact_plan_identity_fresh_snapshot_and_confirmation() {
    let plan = PrunePlan::build(
        100,
        snapshot(),
        vec![item(PruneStore::LocalDisk, PruneClass::Spool, "spool")],
        PrunePlanLimits::default(),
    )
    .expect("valid plan");

    let dry_run = PruneExecutionRequest::dry_run(plan.plan_id.clone());
    assert_eq!(
        plan.authorize(&dry_run, &snapshot()).expect("dry run"),
        PruneAuthorization::DryRun
    );

    assert!(matches!(
        PruneExecutionRequest::execute(plan.plan_id.clone(), false),
        Err(PrunePlanError::ConfirmationRequired)
    ));

    let wrong_plan = PruneExecutionRequest::execute("different-plan".into(), true)
        .expect("explicit confirmation");
    assert!(matches!(
        plan.authorize(&wrong_plan, &snapshot()),
        Err(PrunePlanError::PlanIdentityMismatch)
    ));

    let confirmed =
        PruneExecutionRequest::execute(plan.plan_id.clone(), true).expect("explicit confirmation");
    assert_eq!(
        plan.authorize(&confirmed, &snapshot())
            .expect("confirmed execution"),
        PruneAuthorization::Execute
    );

    let mut forged = plan.clone();
    forged.items[0].target = "different-target".into();
    assert!(matches!(
        forged.authorize(&confirmed, &snapshot()),
        Err(PrunePlanError::PlanIntegrityMismatch)
    ));
}
