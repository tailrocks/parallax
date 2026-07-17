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
    AlertCheckRecord, AlertDeliveryEventRecord, AlertIncidentRecord, AlertRuleRecord,
    AlertRuleStateRecord, TursoMetadataStore,
};

use super::{
    AlertComparator, AlertMeasurement, AlertTransition, DeliveryEventType, NoDataBehavior,
    RuleEvalConfig, RuleEvalState, evaluate_rule, unique_delivery_key,
};

/// One measured group within a rule's scope.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupMeasurement {
    /// Empty string = the rule's single ungrouped scope.
    pub group_key: String,
    pub measurement: AlertMeasurement,
}

/// Source of windowed measurements per rule. The GreptimeDB implementation
/// (error rate / percentiles / throughput / log count / metric aggregate)
/// is peer-owned; tests use stubs.
#[async_trait::async_trait]
pub trait MeasurementSource: Send + Sync {
    async fn measure(
        &self,
        rule: &AlertRuleRecord,
        from_nanos: u128,
        to_nanos: u128,
    ) -> anyhow::Result<Vec<GroupMeasurement>>;
}

/// Parse a stored rule's evaluation parameters into the pure config.
pub fn eval_config(rule: &AlertRuleRecord) -> anyhow::Result<RuleEvalConfig> {
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

fn nanos_to_unix_secs(nanos: u128) -> i64 {
    i64::try_from(nanos / NANOS_PER_SEC).unwrap_or(i64::MAX)
}

fn state_from_record(record: &AlertRuleStateRecord) -> RuleEvalState {
    RuleEvalState {
        consecutive_breaches: record.consecutive_breaches,
        consecutive_healthy: record.consecutive_healthy,
        incident_open: record.incident_open,
        last_notified_at: record.last_notified_at_nanos.map(nanos_to_unix_secs),
        last_value: record.last_value,
        last_sample_count: record.last_sample_count,
    }
}

fn record_from_state(
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

fn destination_ids(rule: &AlertRuleRecord) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(&rule.destination_ids).unwrap_or_default()
}

/// Summary of one tick, for tests and the serve ready-banner counters.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TickReport {
    pub rules_seen: usize,
    pub rules_claimed: usize,
    pub groups_evaluated: usize,
    pub incidents_opened: usize,
    pub incidents_resolved: usize,
    pub renotifies: usize,
    pub deliveries_enqueued: usize,
    pub rule_errors: usize,
}

async fn enqueue_deliveries(
    store: &TursoMetadataStore,
    rule: &AlertRuleRecord,
    incident_id: &str,
    event_type: DeliveryEventType,
    now_nanos: u128,
    report: &mut TickReport,
) -> anyhow::Result<()> {
    for destination in destination_ids(rule) {
        let base_key = unique_delivery_key(incident_id, &destination, event_type);
        // Renotify repeats for the same open incident, so its key carries the
        // notification second; triggered/resolved stay strictly once.
        let delivery_key = match event_type {
            DeliveryEventType::Renotify => {
                format!("{base_key}|{}", nanos_to_unix_secs(now_nanos))
            }
            _ => base_key,
        };
        let inserted = store
            .alert_delivery_enqueue(&AlertDeliveryEventRecord {
                id: format!("del-{delivery_key}"),
                incident_id: incident_id.to_string(),
                destination_id: destination,
                event_type: event_type.as_str().to_string(),
                status: "pending".to_string(),
                attempt_count: 0,
                next_attempt_at_nanos: now_nanos,
                claimed_by: None,
                claim_expires_at_nanos: None,
                delivered_at_nanos: None,
                last_error: None,
                delivery_key,
                created_at_nanos: now_nanos,
            })
            .await?;
        if inserted {
            report.deliveries_enqueued += 1;
        }
    }
    Ok(())
}

/// Run one evaluation pass over every enabled rule. Idempotent under repeated
/// invocation within `claim_interval_secs` (the CAS claim skips re-claimed
/// rules), safe under concurrent server instances.
pub async fn tick_once(
    store: &TursoMetadataStore,
    source: &dyn MeasurementSource,
    now_nanos: u128,
    claim_interval_secs: u32,
) -> anyhow::Result<TickReport> {
    let mut report = TickReport::default();
    let now_secs = nanos_to_unix_secs(now_nanos);
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
                report.rule_errors += 1;
                store
                    .alert_check_insert(&AlertCheckRecord {
                        rule_id: rule.id.clone(),
                        group_key: String::new(),
                        checked_at_nanos: now_nanos,
                        value: None,
                        sample_count: 0,
                        status: "error".to_string(),
                        error: Some(error.to_string()),
                    })
                    .await?;
                continue;
            }
        };

        let window_nanos = u128::from(rule.window_minutes) * 60 * NANOS_PER_SEC;
        let from_nanos = now_nanos.saturating_sub(window_nanos);
        let mut groups = match source.measure(&rule, from_nanos, now_nanos).await {
            Ok(groups) => groups,
            Err(error) => {
                report.rule_errors += 1;
                store
                    .alert_check_insert(&AlertCheckRecord {
                        rule_id: rule.id.clone(),
                        group_key: String::new(),
                        checked_at_nanos: now_nanos,
                        value: None,
                        sample_count: 0,
                        status: "error".to_string(),
                        error: Some(error.to_string()),
                    })
                    .await?;
                continue;
            }
        };

        // Groups with existing state but no measurement this window must still
        // tick (no-data handling / healthy resolution); a rule with no data at
        // all evaluates its single ungrouped scope.
        let measured: BTreeSet<String> = groups.iter().map(|g| g.group_key.clone()).collect();
        for state in store.alert_rule_states(&rule.id).await? {
            if !measured.contains(&state.group_key) {
                groups.push(GroupMeasurement {
                    group_key: state.group_key,
                    measurement: AlertMeasurement {
                        value: None,
                        sample_count: 0,
                    },
                });
            }
        }
        if groups.is_empty() {
            groups.push(GroupMeasurement {
                group_key: String::new(),
                measurement: AlertMeasurement {
                    value: None,
                    sample_count: 0,
                },
            });
        }

        for group in groups {
            report.groups_evaluated += 1;
            let prev = store
                .alert_rule_state(&rule.id, &group.group_key)
                .await?
                .as_ref()
                .map_or_else(RuleEvalState::default, state_from_record);
            let outcome = evaluate_rule(&config, &prev, group.measurement, now_secs);
            let status = if outcome.effective_value.is_none() {
                "no_data"
            } else if outcome.is_breach {
                "breach"
            } else {
                "healthy"
            };
            store
                .alert_rule_state_upsert(&record_from_state(
                    &rule.id,
                    &group.group_key,
                    &outcome.state,
                    status,
                    now_nanos,
                    None,
                ))
                .await?;
            store
                .alert_check_insert(&AlertCheckRecord {
                    rule_id: rule.id.clone(),
                    group_key: group.group_key.clone(),
                    checked_at_nanos: now_nanos,
                    value: outcome.effective_value,
                    sample_count: group.measurement.sample_count,
                    status: status.to_string(),
                    error: None,
                })
                .await?;

            match outcome.transition {
                AlertTransition::None => {}
                AlertTransition::OpenIncident => {
                    let incident_id = format!("inc-{}-{}-{}", rule.id, group.group_key, now_secs);
                    let created = store
                        .alert_incident_open(&AlertIncidentRecord {
                            id: incident_id.clone(),
                            rule_id: rule.id.clone(),
                            group_key: group.group_key.clone(),
                            status: "open".to_string(),
                            severity: rule.severity.clone(),
                            first_triggered_at_nanos: now_nanos,
                            last_triggered_at_nanos: now_nanos,
                            resolved_at_nanos: None,
                            last_value: outcome.effective_value,
                            last_notified_at_nanos: Some(now_nanos),
                        })
                        .await?;
                    if created {
                        report.incidents_opened += 1;
                        enqueue_deliveries(
                            store,
                            &rule,
                            &incident_id,
                            DeliveryEventType::Triggered,
                            now_nanos,
                            &mut report,
                        )
                        .await?;
                    }
                }
                AlertTransition::ResolveIncident => {
                    if let Some(incident_id) = store
                        .alert_incident_resolve(
                            &rule.id,
                            &group.group_key,
                            now_nanos,
                            outcome.effective_value,
                        )
                        .await?
                    {
                        report.incidents_resolved += 1;
                        enqueue_deliveries(
                            store,
                            &rule,
                            &incident_id,
                            DeliveryEventType::Resolved,
                            now_nanos,
                            &mut report,
                        )
                        .await?;
                    }
                }
                AlertTransition::Renotify => {
                    if let Some(incident) = store
                        .alert_incident_open_for(&rule.id, &group.group_key)
                        .await?
                    {
                        report.renotifies += 1;
                        store
                            .alert_incident_touch(
                                &incident.id,
                                now_nanos,
                                outcome.effective_value,
                                true,
                            )
                            .await?;
                        enqueue_deliveries(
                            store,
                            &rule,
                            &incident.id,
                            DeliveryEventType::Renotify,
                            now_nanos,
                            &mut report,
                        )
                        .await?;
                    }
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubSource {
        values: std::sync::Mutex<Vec<Option<f64>>>,
        sample_count: u64,
    }

    impl StubSource {
        fn new(values: Vec<Option<f64>>, sample_count: u64) -> Self {
            Self {
                values: std::sync::Mutex::new(values),
                sample_count,
            }
        }
    }

    #[async_trait::async_trait]
    impl MeasurementSource for StubSource {
        async fn measure(
            &self,
            _rule: &AlertRuleRecord,
            _from_nanos: u128,
            _to_nanos: u128,
        ) -> anyhow::Result<Vec<GroupMeasurement>> {
            let mut values = self.values.lock().expect("lock");
            let value = if values.is_empty() {
                None
            } else {
                values.remove(0)
            };
            Ok(vec![GroupMeasurement {
                group_key: "checkout".to_string(),
                measurement: AlertMeasurement {
                    value,
                    sample_count: if value.is_some() {
                        self.sample_count
                    } else {
                        0
                    },
                },
            }])
        }
    }

    fn temp_store() -> (tempfile::TempDir, std::path::PathBuf) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("metadata.db");
        (directory, path)
    }

    const SEC: u128 = 1_000_000_000;
    const MIN: u128 = 60 * SEC;

    fn rule() -> AlertRuleRecord {
        AlertRuleRecord {
            id: "r1".to_string(),
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
            minimum_sample_count: 1,
            consecutive_breaches_required: 2,
            consecutive_healthy_required: 2,
            no_data_behavior: "skip".to_string(),
            severity: "critical".to_string(),
            renotify_interval_minutes: 30,
            destination_ids: "[\"d1\"]".to_string(),
            metric_name: None,
            metric_aggregation: None,
            created_at_nanos: MIN,
            updated_at_nanos: MIN,
        }
    }

    #[tokio::test]
    async fn breach_lifecycle_open_renotify_resolve() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        store.alert_rule_save(&rule()).await.expect("save");
        let source = StubSource::new(
            vec![
                Some(0.5), // breach 1 — no transition (hysteresis)
                Some(0.6), // breach 2 — open incident
                Some(0.7), // still breaching, before renotify interval — none
                Some(0.7), // breach after 31m — renotify
                Some(0.0), // healthy 1
                Some(0.0), // healthy 2 — resolve
            ],
            50,
        );

        let mut now = 100 * MIN;
        let r1 = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(r1.incidents_opened, 0);
        assert_eq!(r1.groups_evaluated, 1);

        now += MIN;
        let r2 = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(r2.incidents_opened, 1);
        assert_eq!(r2.deliveries_enqueued, 1);
        let open = store
            .alert_incidents(Some("open"), Some("r1"), 10)
            .await
            .expect("list");
        assert_eq!(open.len(), 1);
        let incident_id = open[0].id.clone();
        let deliveries = store
            .alert_deliveries_for_incident(&incident_id)
            .await
            .expect("deliveries");
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].event_type, "triggered");

        now += MIN;
        let r3 = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(r3.renotifies, 0);

        now += 31 * MIN;
        let r4 = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(r4.renotifies, 1);
        assert_eq!(r4.deliveries_enqueued, 1);

        now += MIN;
        tick_once(&store, &source, now, 30).await.expect("tick");
        now += MIN;
        let r6 = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(r6.incidents_resolved, 1);
        let deliveries = store
            .alert_deliveries_for_incident(&incident_id)
            .await
            .expect("deliveries");
        assert_eq!(deliveries.len(), 3);
        assert!(deliveries.iter().any(|d| d.event_type == "resolved"));
        assert!(
            store
                .alert_incidents(Some("open"), None, 10)
                .await
                .expect("list")
                .is_empty()
        );
        // Audit rows exist for every evaluated tick.
        let checks = store.alert_checks("r1", 100).await.expect("checks");
        assert_eq!(checks.len(), 6);
    }

    #[tokio::test]
    async fn tick_is_idempotent_within_claim_interval() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        store.alert_rule_save(&rule()).await.expect("save");
        let source = StubSource::new(vec![Some(0.9), Some(0.9)], 50);
        let now = 100 * MIN;
        let first = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(first.rules_claimed, 1);
        // Same instant: claim CAS refuses, nothing evaluated twice.
        let second = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(second.rules_claimed, 0);
        assert_eq!(second.groups_evaluated, 0);
        assert_eq!(store.alert_checks("r1", 10).await.expect("checks").len(), 1);
    }

    #[tokio::test]
    async fn no_data_skip_keeps_state_and_records_no_data() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        store.alert_rule_save(&rule()).await.expect("save");
        let source = StubSource::new(vec![Some(0.9), None], 50);
        let mut now = 100 * MIN;
        tick_once(&store, &source, now, 30).await.expect("tick");
        now += MIN;
        let report = tick_once(&store, &source, now, 30).await.expect("tick");
        assert_eq!(report.incidents_opened, 0);
        let state = store
            .alert_rule_state("r1", "checkout")
            .await
            .expect("state")
            .expect("some");
        // Skip preserves the breach counter from the first tick.
        assert_eq!(state.consecutive_breaches, 1);
        assert_eq!(state.last_status.as_deref(), Some("no_data"));
    }

    #[tokio::test]
    async fn bad_comparator_records_error_and_continues() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let mut bad = rule();
        bad.comparator = "wat".to_string();
        store.alert_rule_save(&bad).await.expect("save");
        let source = StubSource::new(vec![Some(0.9)], 50);
        let report = tick_once(&store, &source, 100 * MIN, 30)
            .await
            .expect("tick");
        assert_eq!(report.rule_errors, 1);
        assert_eq!(report.groups_evaluated, 0);
        let checks = store.alert_checks("r1", 10).await.expect("checks");
        assert_eq!(checks[0].status, "error");
        assert!(checks[0].error.as_deref().unwrap_or("").contains("wat"));
    }

    #[tokio::test]
    async fn disabled_rule_is_never_evaluated() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let mut off = rule();
        off.enabled = false;
        store.alert_rule_save(&off).await.expect("save");
        let source = StubSource::new(vec![Some(0.9)], 50);
        let report = tick_once(&store, &source, 100 * MIN, 30)
            .await
            .expect("tick");
        assert_eq!(report.rules_seen, 1);
        assert_eq!(report.rules_claimed, 0);
        assert!(
            store
                .alert_checks("r1", 10)
                .await
                .expect("checks")
                .is_empty()
        );
    }
}
