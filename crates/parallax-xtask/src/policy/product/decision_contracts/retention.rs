use std::{fs, path::Path};

use anyhow::Result;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::diagnostic::Finding;

use super::error;

const RECORD: &str = "docs/research/decisions/retention-and-prune-contract.md";
const FIXTURE: &str = "docs/research/decisions/retention-and-prune-contract.toml";

const DATA_CLASSES: &[&str] = &[
    "raw-traces",
    "raw-logs",
    "raw-metrics",
    "error-events",
    "invocation-metric-points",
    "metric-exemplars",
    "issues",
    "issue-buckets",
    "issue-occurrences",
    "invocations",
    "dashboards",
    "investigations",
    "saved-views",
    "alert-rules",
    "alert-rule-states",
    "alert-incidents",
    "alert-destinations",
    "alert-delivery-events",
    "alert-checks",
    "spool",
    "pinned-evidence",
];

const RECORD_MARKERS: &[&str] = &[
    "`opentelemetry_traces`",
    "`opentelemetry_logs`",
    "native per-metric tables",
    "`error_events`",
    "`invocation_metric_points`",
    "`metric_exemplars`",
    "`issues`",
    "`issue_buckets`",
    "`issue_occurrences`",
    "`invocations`",
    "`dashboards`",
    "`investigations`",
    "`saved_views`",
    "`alert_rules`",
    "`alert_rule_states`",
    "`alert_incidents`",
    "`alert_destinations`",
    "`alert_delivery_events`",
    "`alert_checks`",
    "Legal and user expectations",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    record_sha256: String,
    schema_version: u8,
    status: String,
    decision_date: String,
    approved_by: String,
    approval: String,
    data_classes: Vec<String>,
    raw_traces: String,
    raw_logs: String,
    raw_metrics: String,
    derived_extensions: String,
    mutable_issue_state: String,
    invocations: String,
    saved_state: String,
    alert_state: String,
    spool: String,
    pinned_evidence: String,
    legal_user_expectations: String,
    default_traces_ttl: String,
    default_logs_ttl: String,
    default_metrics_ttl: String,
    default_error_events_ttl: String,
    resolved_grace_days: u16,
    dry_run_default: bool,
    destructive_confirmation: String,
    cross_store_recovery: String,
    logical_reclaim: String,
    physical_reclaim: String,
    native_metric_ttl: String,
    compatibility: String,
}

pub(super) fn check(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let record = match fs::read(root.join(RECORD)) {
        Ok(content) => Some(content),
        Err(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            findings.push(record_error(
                "retention and prune decision record is missing",
            ));
            None
        }
        Err(io_error) => return Err(io_error.into()),
    };
    let fixture = match fs::read_to_string(root.join(FIXTURE)) {
        Ok(content) => content,
        Err(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            findings.push(fixture_error(
                "retention and prune decision fixture is missing",
            ));
            return Ok(());
        }
        Err(io_error) => return Err(io_error.into()),
    };
    let contract: Contract = match toml::from_str(&fixture) {
        Ok(contract) => contract,
        Err(parse_error) => {
            findings.push(fixture_error(&format!(
                "retention and prune decision fixture is invalid: {parse_error}"
            )));
            return Ok(());
        }
    };
    if let Some(record) = record {
        let actual = format!("{:x}", Sha256::digest(&record));
        if actual != contract.record_sha256 {
            findings.push(record_error(
                "retention and prune record differs from its approved fixture",
            ));
        }
        let markdown = String::from_utf8_lossy(&record);
        for marker in missing_record_markers(&markdown) {
            findings.push(record_error(&format!(
                "retention and prune record is incomplete: missing {marker}"
            )));
        }
    }
    for violation in violations(&contract) {
        findings.push(fixture_error(violation));
    }
    Ok(())
}

fn missing_record_markers(markdown: &str) -> Vec<&'static str> {
    RECORD_MARKERS
        .iter()
        .copied()
        .filter(|marker| !markdown.contains(marker))
        .collect()
}

fn fixture_error(reason: &str) -> Finding {
    error("product.retention-decision", Path::new(FIXTURE), reason)
}

fn record_error(reason: &str) -> Finding {
    error("product.retention-decision", Path::new(RECORD), reason)
}

fn violations(contract: &Contract) -> Vec<&'static str> {
    let mut violations = Vec::new();
    check_approval(contract, &mut violations);
    check_ownership(contract, &mut violations);
    check_execution(contract, &mut violations);
    violations
}

fn check_approval(contract: &Contract, violations: &mut Vec<&'static str>) {
    require(
        contract.record_sha256.len() == 64
            && contract
                .record_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
        "record_sha256 must be a lowercase SHA-256 digest",
        violations,
    );
    for (condition, violation) in [
        (contract.schema_version == 1, "schema_version must be 1"),
        (contract.status == "approved", "status must be approved"),
        (
            contract.decision_date == "2026-07-17",
            "decision date must match the operator directive",
        ),
        (
            contract.approved_by == "alexey@chainargos.com",
            "approved_by must identify the operator",
        ),
        (
            contract.approval == "operator-unblock-directive-2026-07-17",
            "approval must identify the unblock directive",
        ),
    ] {
        require(condition, violation, violations);
    }
}

fn check_ownership(contract: &Contract, violations: &mut Vec<&'static str>) {
    let expected_classes = DATA_CLASSES
        .iter()
        .map(|class| (*class).to_string())
        .collect::<Vec<_>>();
    require(
        contract.data_classes == expected_classes,
        "data_classes must enumerate every persisted lifecycle class in canonical order",
        violations,
    );
    for (condition, violation) in [
        (
            contract.raw_traces == "greptime-native-configured-ttl",
            "raw traces must use the configured native-table TTL",
        ),
        (
            contract.raw_logs == "greptime-native-configured-ttl",
            "raw logs must use the configured native-table TTL",
        ),
        (
            contract.raw_metrics == "greptime-native-configured-ttl",
            "raw metrics must use configured native-table TTLs",
        ),
        (
            contract.derived_extensions == "greptime-signal-matched-ttl",
            "derived extensions must follow their owning signal TTL",
        ),
        (
            contract.mutable_issue_state == "turso-unresolved-retained-resolved-plus-30d",
            "unresolved issue state must be retained and resolved state must receive 30 days",
        ),
        (
            contract.invocations == "turso-active-retained-terminal-plus-30d",
            "active invocations must be retained and terminal invocations must receive 30 days",
        ),
        (
            contract.saved_state == "turso-explicit-delete-only",
            "saved state must require explicit deletion",
        ),
        (
            contract.alert_state == "turso-owner-policy-no-normal-prune",
            "alert state must remain under its owner policy and outside normal prune",
        ),
        (
            contract.spool == "local-bounded-config-and-immediate-prune",
            "spool lifecycle must preserve configured bounds and immediate prune",
        ),
        (
            contract.pinned_evidence == "protect-reachable-until-unpinned-or-expired",
            "reachable pinned evidence must be protected",
        ),
        (
            contract.legal_user_expectations == "no-surprise-delete-user-state-or-live-evidence",
            "legal and user expectations must forbid surprise deletion",
        ),
    ] {
        require(condition, violation, violations);
    }
}

fn check_execution(contract: &Contract, violations: &mut Vec<&'static str>) {
    for (actual, expected, violation) in [
        (
            &contract.default_traces_ttl,
            "7d",
            "default trace TTL must remain 7d",
        ),
        (
            &contract.default_logs_ttl,
            "7d",
            "default log TTL must remain 7d",
        ),
        (
            &contract.default_metrics_ttl,
            "14d",
            "default metric TTL must remain 14d",
        ),
        (
            &contract.default_error_events_ttl,
            "30d",
            "default error-event TTL must remain 30d",
        ),
    ] {
        require(actual == expected, violation, violations);
    }
    for (condition, violation) in [
        (
            contract.resolved_grace_days == 30,
            "resolved issue and terminal invocation grace must be 30 days",
        ),
        (contract.dry_run_default, "prune must default to dry-run"),
        (
            contract.destructive_confirmation == "execute-plus-interactive-confirm-or-yes",
            "destructive execution must require explicit execution and confirmation",
        ),
        (
            contract.cross_store_recovery == "durable-resumable-journal",
            "cross-store work must use a durable resumable journal",
        ),
        (
            contract.logical_reclaim == "required-before-success",
            "logical deletion must complete before success",
        ),
        (
            contract.physical_reclaim == "measured-async-compaction-may-remain-pending",
            "physical reclaim must report asynchronous compaction honestly",
        ),
        (
            contract.native_metric_ttl == "catalog-reconcile-existing-and-creation-hint-new",
            "native metric TTL must cover existing and newly created tables",
        ),
        (
            contract.compatibility == "replace-spool-only-prune-with-planned-all-class-prune",
            "compatibility must replace spool-only prune with planned all-class prune",
        ),
    ] {
        require(condition, violation, violations);
    }
}

fn require(condition: bool, violation: &'static str, violations: &mut Vec<&'static str>) {
    if !condition {
        violations.push(violation);
    }
}

#[cfg(test)]
mod tests;
