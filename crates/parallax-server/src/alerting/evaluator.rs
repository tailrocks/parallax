//! Evaluator tick (plan 167 step 2, preliminary).
//!
//! `tick_once` drives one synchronous evaluation pass: CAS-claim each enabled
//! rule, measure it via a [`MeasurementSource`], run the pure state machine,
//! persist state + audit rows, open/resolve/touch incidents, and enqueue
//! outbox deliveries. Measurement itself is behind the trait so the GreptimeDB
//! query implementation (peer-owned) plugs in without touching this flow; the
//! tokio interval loop wrapping `tick_once` is also peer-owned.

use std::collections::BTreeSet;

use parallax_metadata::{
    AlertCheckRecord, AlertRuleRecord, AlertRuleStateRecord, TursoMetadataStore,
};

use super::{AlertComparator, AlertMeasurement, NoDataBehavior, RuleEvalConfig, RuleEvalState};

#[path = "evaluator_transitions.rs"]
mod transitions;

/// One measured group within a rule's scope.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GroupMeasurement {
    /// Empty string = the rule's single ungrouped scope.
    pub group_key: String,
    pub measurement: AlertMeasurement,
}

/// Source of windowed measurements per rule. The GreptimeDB implementation
/// (error rate / percentiles / throughput / log count / metric aggregate)
/// is peer-owned; tests use stubs.
#[async_trait::async_trait]
pub(crate) trait MeasurementSource: Send + Sync {
    async fn measure(
        &self,
        rule: &AlertRuleRecord,
        from_nanos: u128,
        to_nanos: u128,
    ) -> anyhow::Result<Vec<GroupMeasurement>>;
}

/// Parse a stored rule's evaluation parameters into the pure config.
pub(crate) fn eval_config(rule: &AlertRuleRecord) -> anyhow::Result<RuleEvalConfig> {
    let comparator = match rule.comparator.as_str() {
        "gt" => AlertComparator::Gt,
        "gte" => AlertComparator::Gte,
        "lt" => AlertComparator::Lt,
        "lte" => AlertComparator::Lte,
        "between" => AlertComparator::Between,
        "not_between" => AlertComparator::NotBetween,
        other => anyhow::bail!("unknown comparator: {other}"),
    };
    let no_data_behavior = match rule.no_data_behavior.as_str() {
        "skip" => NoDataBehavior::Skip,
        "zero" => NoDataBehavior::Zero,
        other => anyhow::bail!("unknown no_data_behavior: {other}"),
    };
    let severity = match rule.severity.as_str() {
        "critical" => super::AlertSeverity::Critical,
        _ => super::AlertSeverity::Warning,
    };
    Ok(RuleEvalConfig {
        comparator,
        threshold: rule.threshold,
        threshold_upper: rule.threshold_upper,
        consecutive_breaches_required: rule.consecutive_breaches_required,
        consecutive_healthy_required: rule.consecutive_healthy_required,
        minimum_sample_count: rule.minimum_sample_count,
        no_data_behavior,
        renotify_interval_minutes: rule.renotify_interval_minutes,
        severity,
    })
}

const NANOS_PER_SEC: u128 = 1_000_000_000;

pub(super) fn nanos_to_unix_secs(nanos: u128) -> i64 {
    i64::try_from(nanos / NANOS_PER_SEC).unwrap_or(i64::MAX)
}

pub(super) fn state_from_record(record: &AlertRuleStateRecord) -> RuleEvalState {
    RuleEvalState {
        consecutive_breaches: record.consecutive_breaches,
        consecutive_healthy: record.consecutive_healthy,
        incident_open: record.incident_open,
        last_notified_at: record.last_notified_at_nanos.map(nanos_to_unix_secs),
        last_value: record.last_value,
        last_sample_count: record.last_sample_count,
    }
}

pub(super) fn record_from_state(
    rule_id: &str,
    group_key: &str,
    state: &RuleEvalState,
    status: &str,
    now_nanos: u128,
    error: Option<String>,
) -> AlertRuleStateRecord {
    AlertRuleStateRecord {
        rule_id: rule_id.to_string(),
        group_key: group_key.to_string(),
        consecutive_breaches: state.consecutive_breaches,
        consecutive_healthy: state.consecutive_healthy,
        incident_open: state.incident_open,
        last_notified_at_nanos: state
            .last_notified_at
            .map(|secs| u128::try_from(secs.max(0)).unwrap_or(0) * NANOS_PER_SEC),
        last_status: Some(status.to_string()),
        last_value: state.last_value,
        last_sample_count: state.last_sample_count,
        last_evaluated_at_nanos: Some(now_nanos),
        last_error: error,
    }
}

/// Summary of one tick, for tests and the serve ready-banner counters.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct TickReport {
    pub rules_seen: usize,
    pub rules_claimed: usize,
    pub groups_evaluated: usize,
    pub incidents_opened: usize,
    pub incidents_resolved: usize,
    pub renotifies: usize,
    pub deliveries_enqueued: usize,
    pub rule_errors: usize,
}

async fn record_rule_error(
    store: &TursoMetadataStore,
    rule_id: &str,
    now_nanos: u128,
    error: impl ToString,
    report: &mut TickReport,
) -> anyhow::Result<()> {
    report.rule_errors += 1;
    store
        .alert_check_insert(&AlertCheckRecord {
            rule_id: rule_id.to_string(),
            group_key: String::new(),
            checked_at_nanos: now_nanos,
            value: None,
            sample_count: 0,
            status: "error".to_string(),
            error: Some(error.to_string()),
        })
        .await?;
    Ok(())
}

fn empty_group(group_key: String) -> GroupMeasurement {
    GroupMeasurement {
        group_key,
        measurement: AlertMeasurement {
            value: None,
            sample_count: 0,
        },
    }
}

async fn pad_unmeasured_groups(
    store: &TursoMetadataStore,
    rule_id: &str,
    groups: &mut Vec<GroupMeasurement>,
) -> anyhow::Result<()> {
    // Groups with existing state but no measurement this window must still
    // tick (no-data handling / healthy resolution); a rule with no data at
    // all evaluates its single ungrouped scope.
    let measured: BTreeSet<String> = groups.iter().map(|g| g.group_key.clone()).collect();
    for state in store.alert_rule_states(rule_id).await? {
        if !measured.contains(&state.group_key) {
            groups.push(empty_group(state.group_key));
        }
    }
    if groups.is_empty() {
        groups.push(empty_group(String::new()));
    }
    Ok(())
}

/// Run one evaluation pass over every enabled rule. Idempotent under repeated
/// invocation within `claim_interval_secs` (the CAS claim skips re-claimed
/// rules), safe under concurrent server instances.
pub(crate) async fn tick_once(
    store: &TursoMetadataStore,
    source: &dyn MeasurementSource,
    now_nanos: u128,
    claim_interval_secs: u32,
) -> anyhow::Result<TickReport> {
    tick_once_inner(store, source, now_nanos, claim_interval_secs, false).await
}

#[cfg(test)]
pub(crate) async fn tick_once_with_bundle_failure(
    store: &TursoMetadataStore,
    source: &dyn MeasurementSource,
    now_nanos: u128,
    claim_interval_secs: u32,
    fail_bundle_assembly: bool,
) -> anyhow::Result<TickReport> {
    tick_once_inner(
        store,
        source,
        now_nanos,
        claim_interval_secs,
        fail_bundle_assembly,
    )
    .await
}

async fn tick_once_inner(
    store: &TursoMetadataStore,
    source: &dyn MeasurementSource,
    now_nanos: u128,
    claim_interval_secs: u32,
    fail_bundle_assembly: bool,
) -> anyhow::Result<TickReport> {
    let mut report = TickReport::default();
    for rule in store.alert_rules().await? {
        report.rules_seen += 1;
        if !rule.enabled {
            continue;
        }
        if !store
            .alert_rule_claim(&rule.id, now_nanos, claim_interval_secs)
            .await?
        {
            continue;
        }
        report.rules_claimed += 1;

        let config = match eval_config(&rule) {
            Ok(config) => config,
            Err(error) => {
                record_rule_error(store, &rule.id, now_nanos, error, &mut report).await?;
                continue;
            }
        };

        let window_nanos = u128::from(rule.window_minutes) * 60 * NANOS_PER_SEC;
        let from_nanos = now_nanos.saturating_sub(window_nanos);
        let mut groups = match source.measure(&rule, from_nanos, now_nanos).await {
            Ok(groups) => groups,
            Err(error) => {
                record_rule_error(store, &rule.id, now_nanos, error, &mut report).await?;
                continue;
            }
        };
        pad_unmeasured_groups(store, &rule.id, &mut groups).await?;

        let scope = transitions::TickScope {
            store,
            now_nanos,
            fail_bundle_assembly,
        };
        for group in groups {
            transitions::evaluate_group(&scope, &rule, &config, group, &mut report).await?;
        }
    }
    Ok(report)
}

#[cfg(test)]
#[path = "evaluator_tests.rs"]
mod tests;
