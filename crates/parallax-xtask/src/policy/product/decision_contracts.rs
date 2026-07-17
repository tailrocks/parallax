use std::{fs, path::Path};

use anyhow::Result;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::diagnostic::Finding;

use super::error;

const METRIC_RECORD: &str = "docs/research/decisions/metric-summary-contract.md";
const METRIC_FIXTURE: &str = "docs/research/decisions/metric-summary-contract.toml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetricSummaryContract {
    record_sha256: String,
    schema_version: u8,
    status: String,
    decision_date: String,
    approval: String,
    window: String,
    eligible_samples: Vec<String>,
    non_finite: String,
    histogram_count: String,
    trend_bucket_limit: u16,
    trend_default_buckets: u16,
    trend_min_step_seconds: u16,
    bucket_boundaries: String,
    bucket_timestamp: String,
    empty_buckets: String,
    step_rounding: String,
    canonical_name: String,
    histogram_family: String,
    alias_resolution: String,
    lossy_reverse: String,
    native_name_collision: String,
    metric_only_services: String,
    cli: String,
    graphql_compatibility: String,
}

pub(super) fn check(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let record = match fs::read(root.join(METRIC_RECORD)) {
        Ok(content) => Some(content),
        Err(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            findings.push(record_error("metric summary decision record is missing"));
            None
        }
        Err(io_error) => return Err(io_error.into()),
    };

    let fixture = match fs::read_to_string(root.join(METRIC_FIXTURE)) {
        Ok(content) => content,
        Err(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            findings.push(fixture_error("metric summary decision fixture is missing"));
            return Ok(());
        }
        Err(io_error) => return Err(io_error.into()),
    };
    let contract: MetricSummaryContract = match toml::from_str(&fixture) {
        Ok(contract) => contract,
        Err(parse_error) => {
            findings.push(fixture_error(&format!(
                "metric summary decision fixture is invalid: {parse_error}"
            )));
            return Ok(());
        }
    };
    if let Some(record) = record {
        let actual = format!("{:x}", Sha256::digest(record));
        if actual != contract.record_sha256 {
            findings.push(record_error(
                "metric summary decision record differs from its approved fixture",
            ));
        }
    }
    for violation in violations(&contract) {
        findings.push(fixture_error(violation));
    }
    Ok(())
}

fn fixture_error(reason: &str) -> Finding {
    error("product.metric-decision", Path::new(METRIC_FIXTURE), reason)
}

fn record_error(reason: &str) -> Finding {
    error("product.metric-decision", Path::new(METRIC_RECORD), reason)
}

fn violations(contract: &MetricSummaryContract) -> Vec<&'static str> {
    let mut violations = Vec::new();
    check_approval(contract, &mut violations);
    check_samples(contract, &mut violations);
    check_trends(contract, &mut violations);
    check_names(contract, &mut violations);
    check_consumers(contract, &mut violations);
    violations
}

fn check_approval(contract: &MetricSummaryContract, violations: &mut Vec<&'static str>) {
    require(
        contract.record_sha256.len() == 64
            && contract
                .record_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
        "record_sha256 must be a lowercase SHA-256 digest",
        violations,
    );
    require(
        contract.schema_version == 1,
        "schema_version must be 1",
        violations,
    );
    require(
        contract.status == "approved",
        "status must be approved",
        violations,
    );
    require(
        contract.decision_date == "2026-07-17",
        "decision date must match the operator directive",
        violations,
    );
}

fn check_samples(contract: &MetricSummaryContract, violations: &mut Vec<&'static str>) {
    require(
        contract.approval == "operator-directive-2026-07-17",
        "approval must identify the operator directive",
        violations,
    );
    require(
        contract.window == "explicit-inclusive",
        "window must be explicit-inclusive",
        violations,
    );
    require(
        contract.eligible_samples == ["gauge", "sum", "explicit-histogram"],
        "eligible samples must be gauge, sum, and explicit-histogram",
        violations,
    );
    require(
        contract.non_finite == "exclude",
        "non-finite samples must be excluded",
        violations,
    );
}

fn check_trends(contract: &MetricSummaryContract, violations: &mut Vec<&'static str>) {
    require(
        contract.histogram_count == "count-row-once",
        "histograms must count the count row once",
        violations,
    );
    require(
        contract.trend_bucket_limit == 120,
        "trend bucket limit must be 120",
        violations,
    );
    require(
        contract.trend_default_buckets == 60,
        "default trend bucket count must be 60",
        violations,
    );
    require(
        contract.trend_min_step_seconds == 1,
        "minimum trend step must be one second",
        violations,
    );
    require(
        contract.bucket_boundaries == "left-closed-right-open-final-inclusive",
        "bucket boundaries must be left-closed/right-open with final endpoint inclusive",
        violations,
    );
    require(
        contract.bucket_timestamp == "start",
        "bucket timestamps must be bucket starts",
        violations,
    );
    require(
        contract.empty_buckets == "zero",
        "empty buckets must be zero-filled",
        violations,
    );
}

fn check_names(contract: &MetricSummaryContract, violations: &mut Vec<&'static str>) {
    require(
        contract.step_rounding == "up",
        "trend steps must round up",
        violations,
    );
    require(
        contract.canonical_name == "native-public-table-base",
        "canonical names must be native public-table bases",
        violations,
    );
    require(
        contract.histogram_family == "complete-family-only",
        "histogram suffixes collapse only for complete families",
        violations,
    );
    require(
        contract.alias_resolution == "exactly-one-match",
        "metric aliases must resolve to exactly one family",
        violations,
    );
    require(
        contract.lossy_reverse == "forbidden",
        "lossy native-name reversal must be forbidden",
        violations,
    );
}

fn check_consumers(contract: &MetricSummaryContract, violations: &mut Vec<&'static str>) {
    require(
        contract.native_name_collision == "error",
        "native-name collisions must error",
        violations,
    );
    require(
        contract.metric_only_services == "finite-sample-in-window",
        "metric-only services require a finite sample in-window",
        violations,
    );
    require(
        contract.cli == "metrics-invocation",
        "CLI must retain metrics --invocation",
        violations,
    );
    require(
        contract.graphql_compatibility == "preserve-v1",
        "GraphQL compatibility must preserve V1",
        violations,
    );
}

fn require(condition: bool, violation: &'static str, violations: &mut Vec<&'static str>) {
    if !condition {
        violations.push(violation);
    }
}

#[cfg(test)]
mod tests;
