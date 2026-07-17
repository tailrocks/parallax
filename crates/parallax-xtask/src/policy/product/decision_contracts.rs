use std::{fs, path::Path};

use anyhow::Result;
use serde::Deserialize;

use crate::diagnostic::Finding;

use super::error;

const METRIC_RECORD: &str = "docs/research/decisions/metric-summary-contract.md";
const METRIC_FIXTURE: &str = "docs/research/decisions/metric-summary-contract.toml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetricSummaryContract {
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
    native_name_collision: String,
    metric_only_services: String,
    cli: String,
    graphql_compatibility: String,
}

pub(super) fn check(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    if !root.join(METRIC_RECORD).is_file() {
        findings.push(decision_error("metric summary decision record is missing"));
    }

    let fixture = match fs::read_to_string(root.join(METRIC_FIXTURE)) {
        Ok(content) => content,
        Err(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => {
            findings.push(decision_error("metric summary decision fixture is missing"));
            return Ok(());
        }
        Err(io_error) => return Err(io_error.into()),
    };
    let contract: MetricSummaryContract = match toml::from_str(&fixture) {
        Ok(contract) => contract,
        Err(parse_error) => {
            findings.push(decision_error(&format!(
                "metric summary decision fixture is invalid: {parse_error}"
            )));
            return Ok(());
        }
    };
    for violation in violations(&contract) {
        findings.push(decision_error(violation));
    }
    Ok(())
}

fn decision_error(reason: &str) -> Finding {
    error("product.metric-decision", Path::new(METRIC_FIXTURE), reason)
}

fn violations(contract: &MetricSummaryContract) -> Vec<&'static str> {
    let mut violations = Vec::new();
    require(contract.schema_version == 1, "schema_version must be 1", &mut violations);
    require(contract.status == "approved", "status must be approved", &mut violations);
    require(contract.decision_date == "2026-07-17", "decision date must match the operator directive", &mut violations);
    require(contract.approval == "operator-directive-2026-07-17", "approval must identify the operator directive", &mut violations);
    require(contract.window == "explicit-inclusive", "window must be explicit-inclusive", &mut violations);
    require(contract.eligible_samples == ["gauge", "sum", "explicit-histogram"], "eligible samples must be gauge, sum, and explicit-histogram", &mut violations);
    require(contract.non_finite == "exclude", "non-finite samples must be excluded", &mut violations);
    require(contract.histogram_count == "count-row-once", "histograms must count the count row once", &mut violations);
    require(contract.trend_bucket_limit == 120, "trend bucket limit must be 120", &mut violations);
    require(contract.trend_default_buckets == 60, "default trend bucket count must be 60", &mut violations);
    require(contract.trend_min_step_seconds == 1, "minimum trend step must be one second", &mut violations);
    require(contract.native_name_collision == "error", "native-name collisions must error", &mut violations);
    require(contract.metric_only_services == "finite-sample-in-window", "metric-only services require a finite sample in-window", &mut violations);
    require(contract.cli == "metrics-invocation", "CLI must retain metrics --invocation", &mut violations);
    require(contract.graphql_compatibility == "preserve-v1", "GraphQL compatibility must preserve V1", &mut violations);
    violations
}

fn require(condition: bool, violation: &'static str, violations: &mut Vec<&'static str>) {
    if !condition {
        violations.push(violation);
    }
}

#[cfg(test)]
mod tests;
