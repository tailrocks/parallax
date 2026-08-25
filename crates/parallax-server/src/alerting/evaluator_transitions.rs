//! Incident open/resolve/renotify side effects for one evaluated group.

use parallax_metadata::{
    AlertCheckRecord, AlertDeliveryEventRecord, AlertIncidentRecord, AlertRuleRecord,
    TursoMetadataStore,
};

use super::{
    GroupMeasurement, TickReport, nanos_to_unix_secs, record_from_state, state_from_record,
};
use crate::alerting::{
    AlertTransition, DeliveryEventType, EvaluationOutcome, RuleEvalConfig, evaluate_rule,
    unique_delivery_key,
};

use super::super::incident_bundle::IncidentAssembly;

/// Shared state for one evaluation tick: the metadata store handle, the tick
/// timestamp, and the test-only bundle failure injection flag.
pub(super) struct TickScope<'a> {
    pub(super) store: &'a TursoMetadataStore,
    pub(super) now_nanos: u128,
    pub(super) fail_bundle_assembly: bool,
}

fn destination_ids(rule: &AlertRuleRecord) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(&rule.destination_ids).unwrap_or_default()
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

pub(super) async fn evaluate_group(
    scope: &TickScope<'_>,
    rule: &AlertRuleRecord,
    config: &RuleEvalConfig,
    group: GroupMeasurement,
    report: &mut TickReport,
) -> anyhow::Result<()> {
    let store = scope.store;
    let now_nanos = scope.now_nanos;
    report.groups_evaluated += 1;
    let now_secs = nanos_to_unix_secs(now_nanos);
    let prev = store
        .alert_rule_state(&rule.id, &group.group_key)
        .await?
        .as_ref()
        .map_or_else(crate::alerting::RuleEvalState::default, state_from_record);
    let outcome = evaluate_rule(config, &prev, group.measurement, now_secs);
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
    apply_transition(scope, rule, &group.group_key, &outcome, report).await
}

fn new_incident(
    rule: &AlertRuleRecord,
    group_key: &str,
    outcome: &EvaluationOutcome,
    now_nanos: u128,
) -> AlertIncidentRecord {
    AlertIncidentRecord {
        id: format!(
            "inc-{}-{}-{}",
            rule.id,
            group_key,
            nanos_to_unix_secs(now_nanos)
        ),
        rule_id: rule.id.clone(),
        group_key: group_key.to_string(),
        status: "open".to_string(),
        severity: rule.severity.clone(),
        first_triggered_at_nanos: now_nanos,
        last_triggered_at_nanos: now_nanos,
        resolved_at_nanos: None,
        last_value: outcome.effective_value,
        last_notified_at_nanos: Some(now_nanos),
        bundle_hash: None,
        bundle_assembled_at_nanos: None,
        bundle_top_hypothesis: None,
        bundle_deploy_adjacency: None,
        bundle_error: None,
    }
}

async fn open_incident(
    scope: &TickScope<'_>,
    rule: &AlertRuleRecord,
    group_key: &str,
    outcome: &EvaluationOutcome,
    report: &mut TickReport,
) -> anyhow::Result<()> {
    let store = scope.store;
    let now_nanos = scope.now_nanos;
    let incident = new_incident(rule, group_key, outcome, now_nanos);
    if store.alert_incident_open(&incident).await? {
        report.incidents_opened += 1;
        enqueue_deliveries(
            store,
            rule,
            &incident.id,
            DeliveryEventType::Triggered,
            now_nanos,
            report,
        )
        .await?;
        super::super::incident_bundle::persist_incident_bundle(
            store,
            IncidentAssembly {
                rule,
                incident_id: &incident.id,
                group_key,
                last_value: outcome.effective_value,
                now_nanos,
                fail_assembly: scope.fail_bundle_assembly,
            },
        )
        .await;
    }
    Ok(())
}

async fn apply_transition(
    scope: &TickScope<'_>,
    rule: &AlertRuleRecord,
    group_key: &str,
    outcome: &EvaluationOutcome,
    report: &mut TickReport,
) -> anyhow::Result<()> {
    let store = scope.store;
    let now_nanos = scope.now_nanos;
    match outcome.transition {
        AlertTransition::None => Ok(()),
        AlertTransition::OpenIncident => {
            open_incident(scope, rule, group_key, outcome, report).await
        }
        AlertTransition::ResolveIncident => {
            if let Some(incident_id) = store
                .alert_incident_resolve(
                    rule.id.as_str(),
                    group_key,
                    now_nanos,
                    outcome.effective_value,
                )
                .await?
            {
                report.incidents_resolved += 1;
                enqueue_deliveries(
                    store,
                    rule,
                    &incident_id,
                    DeliveryEventType::Resolved,
                    now_nanos,
                    report,
                )
                .await?;
            }
            Ok(())
        }
        AlertTransition::Renotify => {
            if let Some(incident) = store
                .alert_incident_open_for(rule.id.as_str(), group_key)
                .await?
            {
                report.renotifies += 1;
                store
                    .alert_incident_touch(&incident.id, now_nanos, outcome.effective_value, true)
                    .await?;
                if !super::super::incident_bundle::should_reuse_bundle(&incident) {
                    super::super::incident_bundle::persist_incident_bundle(
                        store,
                        IncidentAssembly {
                            rule,
                            incident_id: &incident.id,
                            group_key,
                            last_value: outcome.effective_value,
                            now_nanos,
                            fail_assembly: scope.fail_bundle_assembly,
                        },
                    )
                    .await;
                }
                enqueue_deliveries(
                    store,
                    rule,
                    &incident.id,
                    DeliveryEventType::Renotify,
                    now_nanos,
                    report,
                )
                .await?;
            }
            Ok(())
        }
    }
}
