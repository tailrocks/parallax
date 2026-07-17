use super::*;

fn valid_fixture() -> &'static str {
    r#"
schema_version = 1
record_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
status = "approved"
decision_date = "2026-07-17"
approval = "operator-directive-2026-07-17"
window = "explicit-inclusive"
eligible_samples = ["gauge", "sum", "explicit-histogram"]
non_finite = "exclude"
histogram_count = "count-row-once"
trend_bucket_limit = 120
trend_default_buckets = 60
trend_min_step_seconds = 1
bucket_boundaries = "left-closed-right-open-final-inclusive"
bucket_timestamp = "start"
empty_buckets = "zero"
step_rounding = "up"
canonical_name = "native-public-table-base"
histogram_family = "complete-family-only"
alias_resolution = "exactly-one-match"
lossy_reverse = "forbidden"
native_name_collision = "error"
metric_only_services = "finite-sample-in-window"
cli = "metrics-invocation"
graphql_compatibility = "preserve-v1"
"#
}

fn parse_fixture(source: &str) -> MetricSummaryContract {
    toml::from_str(source).expect("valid contract fixture")
}

#[test]
fn approved_complete_contract_passes() {
    assert!(violations(&parse_fixture(valid_fixture())).is_empty());
}

#[test]
fn checked_in_record_matches_approved_fixture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut findings = Vec::new();
    check(&root, &mut findings).expect("checked-in contract check");
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn missing_record_and_fixture_fail_closed() {
    let root = tempfile::tempdir().expect("temporary policy root");
    let mut findings = Vec::new();
    check(root.path(), &mut findings).expect("missing contract check");
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.rule_id == "product.metric-decision")
            .count(),
        2
    );
}

#[test]
fn every_contract_decision_fails_closed_when_changed() {
    let replacements = [
        (
            "record_sha256 = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
            "record_sha256 = \"invalid\"",
        ),
        ("schema_version = 1", "schema_version = 2"),
        ("status = \"approved\"", "status = \"draft\""),
        (
            "decision_date = \"2026-07-17\"",
            "decision_date = \"9999-99-99\"",
        ),
        (
            "approval = \"operator-directive-2026-07-17\"",
            "approval = \"pending\"",
        ),
        ("window = \"explicit-inclusive\"", "window = \"lifetime\""),
        (
            "eligible_samples = [\"gauge\", \"sum\", \"explicit-histogram\"]",
            "eligible_samples = [\"gauge\"]",
        ),
        ("non_finite = \"exclude\"", "non_finite = \"include\""),
        (
            "histogram_count = \"count-row-once\"",
            "histogram_count = \"all-rows\"",
        ),
        ("trend_bucket_limit = 120", "trend_bucket_limit = 121"),
        ("trend_default_buckets = 60", "trend_default_buckets = 61"),
        ("trend_min_step_seconds = 1", "trend_min_step_seconds = 0"),
        (
            "bucket_boundaries = \"left-closed-right-open-final-inclusive\"",
            "bucket_boundaries = \"closed\"",
        ),
        ("bucket_timestamp = \"start\"", "bucket_timestamp = \"end\""),
        ("empty_buckets = \"zero\"", "empty_buckets = \"omit\""),
        ("step_rounding = \"up\"", "step_rounding = \"down\""),
        (
            "canonical_name = \"native-public-table-base\"",
            "canonical_name = \"reverse-dots\"",
        ),
        (
            "histogram_family = \"complete-family-only\"",
            "histogram_family = \"suffix-only\"",
        ),
        (
            "alias_resolution = \"exactly-one-match\"",
            "alias_resolution = \"first\"",
        ),
        (
            "lossy_reverse = \"forbidden\"",
            "lossy_reverse = \"allowed\"",
        ),
        (
            "native_name_collision = \"error\"",
            "native_name_collision = \"first\"",
        ),
        (
            "metric_only_services = \"finite-sample-in-window\"",
            "metric_only_services = \"ignore\"",
        ),
        ("cli = \"metrics-invocation\"", "cli = \"metrics-run\""),
        (
            "graphql_compatibility = \"preserve-v1\"",
            "graphql_compatibility = \"breaking\"",
        ),
    ];

    for (approved, rejected) in replacements {
        let changed = valid_fixture().replacen(approved, rejected, 1);
        assert_eq!(
            violations(&parse_fixture(&changed)).len(),
            1,
            "changed decision was accepted: {rejected}"
        );
    }
}

#[test]
fn malformed_or_incomplete_fixture_fails_closed() {
    let unknown = format!("{}\nunknown = true", valid_fixture());
    for source in [
        "status = [",
        "schema_version = 1\nstatus = \"approved\"\n",
        &unknown,
    ] {
        toml::from_str::<MetricSummaryContract>(source).unwrap_err();
    }
}
