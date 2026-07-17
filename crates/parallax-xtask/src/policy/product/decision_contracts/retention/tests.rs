use super::*;

fn valid_fixture() -> &'static str {
    include_str!("../../../../../../../docs/research/decisions/retention-and-prune-contract.toml")
}

fn parse_fixture(source: &str) -> Contract {
    toml::from_str(source).expect("valid retention contract fixture")
}

#[test]
fn approved_complete_contract_passes() {
    assert!(violations(&parse_fixture(valid_fixture())).is_empty());
}

#[test]
fn checked_in_record_matches_approved_fixture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut findings = Vec::new();
    check(&root, &mut findings).expect("checked-in retention contract check");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn missing_record_and_fixture_fail_closed() {
    let root = tempfile::tempdir().expect("temporary policy root");
    let mut findings = Vec::new();
    check(root.path(), &mut findings).expect("missing retention contract check");
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.rule_id == "product.retention-decision")
            .count(),
        2
    );
}

#[test]
fn record_inventory_fails_closed_when_a_required_class_is_missing() {
    let markdown = RECORD_MARKERS
        .iter()
        .copied()
        .filter(|marker| *marker != "`saved_views`")
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(missing_record_markers(&markdown), ["`saved_views`"]);
}

#[test]
fn every_contract_field_fails_closed_when_changed() {
    let approved_contract = parse_fixture(valid_fixture());
    let changed = valid_fixture().replacen(&approved_contract.record_sha256, "invalid", 1);
    assert_eq!(violations(&parse_fixture(&changed)).len(), 1);

    let replacements = [
        ("schema_version = 1", "schema_version = 2"),
        ("status = \"approved\"", "status = \"draft\""),
        (
            "decision_date = \"2026-07-17\"",
            "decision_date = \"9999-99-99\"",
        ),
        (
            "approved_by = \"alexey@chainargos.com\"",
            "approved_by = \"pending\"",
        ),
        (
            "approval = \"operator-unblock-directive-2026-07-17\"",
            "approval = \"pending\"",
        ),
        (
            "raw_traces = \"greptime-native-configured-ttl\"",
            "raw_traces = \"custom\"",
        ),
        (
            "raw_logs = \"greptime-native-configured-ttl\"",
            "raw_logs = \"custom\"",
        ),
        (
            "raw_metrics = \"greptime-native-configured-ttl\"",
            "raw_metrics = \"custom\"",
        ),
        (
            "derived_extensions = \"greptime-signal-matched-ttl\"",
            "derived_extensions = \"forever\"",
        ),
        (
            "mutable_issue_state = \"turso-unresolved-retained-resolved-plus-30d\"",
            "mutable_issue_state = \"delete-all\"",
        ),
        (
            "invocations = \"turso-active-retained-terminal-plus-30d\"",
            "invocations = \"delete-all\"",
        ),
        (
            "saved_state = \"turso-explicit-delete-only\"",
            "saved_state = \"ttl\"",
        ),
        (
            "alert_state = \"turso-owner-policy-no-normal-prune\"",
            "alert_state = \"ttl\"",
        ),
        (
            "spool = \"local-bounded-config-and-immediate-prune\"",
            "spool = \"forever\"",
        ),
        (
            "pinned_evidence = \"protect-reachable-until-unpinned-or-expired\"",
            "pinned_evidence = \"ignore\"",
        ),
        (
            "legal_user_expectations = \"no-surprise-delete-user-state-or-live-evidence\"",
            "legal_user_expectations = \"unspecified\"",
        ),
        ("default_traces_ttl = \"7d\"", "default_traces_ttl = \"8d\""),
        ("default_logs_ttl = \"7d\"", "default_logs_ttl = \"8d\""),
        (
            "default_metrics_ttl = \"14d\"",
            "default_metrics_ttl = \"15d\"",
        ),
        (
            "default_error_events_ttl = \"30d\"",
            "default_error_events_ttl = \"31d\"",
        ),
        ("resolved_grace_days = 30", "resolved_grace_days = 0"),
        ("dry_run_default = true", "dry_run_default = false"),
        (
            "destructive_confirmation = \"execute-plus-interactive-confirm-or-yes\"",
            "destructive_confirmation = \"none\"",
        ),
        (
            "cross_store_recovery = \"durable-resumable-journal\"",
            "cross_store_recovery = \"best-effort\"",
        ),
        (
            "logical_reclaim = \"required-before-success\"",
            "logical_reclaim = \"best-effort\"",
        ),
        (
            "physical_reclaim = \"measured-async-compaction-may-remain-pending\"",
            "physical_reclaim = \"immediate\"",
        ),
        (
            "native_metric_ttl = \"catalog-reconcile-existing-and-creation-hint-new\"",
            "native_metric_ttl = \"new-only\"",
        ),
        (
            "compatibility = \"replace-spool-only-prune-with-planned-all-class-prune\"",
            "compatibility = \"preserve-spool-only\"",
        ),
    ];

    for (approved, rejected) in replacements {
        let changed = valid_fixture().replacen(approved, rejected, 1);
        assert_eq!(
            violations(&parse_fixture(&changed)).len(),
            1,
            "changed retention decision was accepted: {rejected}"
        );
    }

    let changed = valid_fixture().replacen("data_classes = [", "data_classes = [\n  \"extra\",", 1);
    assert_eq!(violations(&parse_fixture(&changed)).len(), 1);
}

#[test]
fn malformed_rejected_or_incomplete_fixture_fails_closed() {
    let unknown = format!("{}\nunknown = true", valid_fixture());
    let sources = [
        "status = [".to_string(),
        valid_fixture().replacen("status = \"approved\"", "status = \"rejected\"", 1),
        "schema_version = 1\nstatus = \"approved\"\n".to_string(),
        unknown,
    ];
    for source in sources {
        match toml::from_str::<Contract>(&source) {
            Ok(contract) => assert!(
                !violations(&contract).is_empty(),
                "invalid contract accepted"
            ),
            Err(_) => {}
        }
    }
}
