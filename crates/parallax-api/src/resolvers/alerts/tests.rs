//! Alert resolver round-trip and validation tests (plan 167 Step 4).

use crate::resolvers::test_support::*;
use crate::{build_schema, execute};

use parallax_test_support::builders::MemoryStore;

use std::sync::Arc;

async fn run(
    schema: &crate::Schema,
    context: &crate::ApiContext,
    query: impl Into<String>,
) -> serde_json::Value {
    let request = juniper::http::GraphQLRequest::new(query.into(), None, None);
    serde_json::to_value(execute(schema, context, request).await).unwrap()
}

const RULE_SAVE: &str = r#"
    mutation {
      alertRuleSave(input: {
        name: "High error rate",
        signalType: "error_rate",
        services: ["checkout"],
        comparator: "gt",
        threshold: 0.2,
        windowMinutes: 5,
        severity: "critical"
      }) {
        id name enabled signalType services excludeServices comparator
        threshold thresholdUpper windowMinutes minimumSampleCount
        consecutiveBreachesRequired consecutiveHealthyRequired
        noDataBehavior severity renotifyIntervalMinutes destinationIds
      }
    }
    "#;

#[tokio::test]
async fn alert_rule_round_trip_defaults_toggle_delete() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;

    let json = run(&schema, &context, RULE_SAVE).await;
    assert!(error_messages(&json).is_empty(), "alertRuleSave: {json}");
    let rule = json.pointer("/data/alertRuleSave").unwrap();
    let id = rule["id"].as_str().unwrap().to_string();
    assert_eq!(rule["enabled"], serde_json::json!(true));
    assert_eq!(rule["services"], serde_json::json!("[\"checkout\"]"));
    assert_eq!(rule["excludeServices"], serde_json::json!("[]"));
    assert_eq!(rule["minimumSampleCount"], serde_json::json!(1));
    assert_eq!(rule["consecutiveBreachesRequired"], serde_json::json!(2));
    assert_eq!(rule["consecutiveHealthyRequired"], serde_json::json!(2));
    assert_eq!(rule["noDataBehavior"], serde_json::json!("skip"));
    assert_eq!(rule["renotifyIntervalMinutes"], serde_json::json!(30));

    let json = run(&schema, &context, r#"{ alertRules { id name } }"#).await;
    assert!(error_messages(&json).is_empty(), "alertRules: {json}");
    assert_eq!(
        json.pointer("/data/alertRules/0/id"),
        Some(&serde_json::json!(id.as_str()))
    );

    // Update keeps id + created_at, changes threshold.
    let json = run(
        &schema,
        &context,
        format!(
            r#"mutation {{ alertRuleSave(input: {{
                 id: "{id}", name: "High error rate",
                 signalType: "error_rate", comparator: "gt",
                 threshold: 0.5, windowMinutes: 5, severity: "warning"
               }}) {{ id threshold severity }} }}"#
        ),
    )
    .await;
    assert!(error_messages(&json).is_empty(), "update: {json}");
    assert_eq!(
        json.pointer("/data/alertRuleSave/threshold"),
        Some(&serde_json::json!(0.5))
    );

    let json = run(
        &schema,
        &context,
        format!(
            r#"mutation {{ alertRuleSetEnabled(id: "{id}", enabled: false) {{ id enabled }} }}"#
        ),
    )
    .await;
    assert!(error_messages(&json).is_empty(), "setEnabled: {json}");
    assert_eq!(
        json.pointer("/data/alertRuleSetEnabled/enabled"),
        Some(&serde_json::json!(false))
    );

    let json = run(
        &schema,
        &context,
        format!(r#"mutation {{ alertRuleDelete(id: "{id}") }}"#),
    )
    .await;
    assert_eq!(
        json.pointer("/data/alertRuleDelete"),
        Some(&serde_json::json!(true))
    );
    let json = run(&schema, &context, r#"{ alertRules { id } }"#).await;
    assert_eq!(
        json.pointer("/data/alertRules"),
        Some(&serde_json::json!([]))
    );
}

#[tokio::test]
async fn alert_rule_validation_rejects_bad_input() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let cases = [
        (
            // between without thresholdUpper
            r#"mutation { alertRuleSave(input: { name: "r", signalType: "throughput",
                comparator: "between", threshold: 1, windowMinutes: 5,
                severity: "warning" }) { id } }"#,
            "thresholdUpper",
        ),
        (
            // metric without metricName
            r#"mutation { alertRuleSave(input: { name: "r", signalType: "metric",
                comparator: "gt", threshold: 1, windowMinutes: 5,
                severity: "warning" }) { id } }"#,
            "metricName",
        ),
        (
            // error_rate outside [0, 1]
            r#"mutation { alertRuleSave(input: { name: "r", signalType: "error_rate",
                comparator: "gt", threshold: 40, windowMinutes: 5,
                severity: "warning" }) { id } }"#,
            "fraction",
        ),
        (
            // unknown comparator
            r#"mutation { alertRuleSave(input: { name: "r", signalType: "throughput",
                comparator: "eq", threshold: 1, windowMinutes: 5,
                severity: "warning" }) { id } }"#,
            "comparator",
        ),
        (
            // thresholdUpper without a range comparator
            r#"mutation { alertRuleSave(input: { name: "r", signalType: "throughput",
                comparator: "gt", threshold: 1, thresholdUpper: 2, windowMinutes: 5,
                severity: "warning" }) { id } }"#,
            "between",
        ),
    ];
    for (mutation, expected) in cases {
        let json = run(&schema, &context, mutation).await;
        let errors = error_messages(&json);
        assert!(
            errors.iter().any(|message| message.contains(expected)),
            "expected {expected:?} in {errors:?}"
        );
    }
    let json = run(&schema, &context, r#"{ alertRules { id } }"#).await;
    assert_eq!(
        json.pointer("/data/alertRules"),
        Some(&serde_json::json!([])),
        "rejected rules must not persist"
    );
}

#[tokio::test]
async fn alert_destination_round_trip_and_validation() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;

    let json = run(
        &schema,
        &context,
        r#"mutation { alertDestinationSave(name: "Ops hook", kind: "webhook",
             config: "{\"url\": \"http://127.0.0.1:9099/hook\"}") { id name kind config } }"#,
    )
    .await;
    assert!(error_messages(&json).is_empty(), "destinationSave: {json}");
    let id = json
        .pointer("/data/alertDestinationSave/id")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_string();

    // email is deferred in V1; non-http url rejected.
    for mutation in [
        r#"mutation { alertDestinationSave(name: "m", kind: "email",
             config: "{\"url\": \"http://x\"}") { id } }"#,
        r#"mutation { alertDestinationSave(name: "m", kind: "webhook",
             config: "{\"url\": \"file:///etc/passwd\"}") { id } }"#,
        r#"mutation { alertDestinationSave(name: "m", kind: "webhook",
             config: "not json") { id } }"#,
    ] {
        let json = run(&schema, &context, mutation).await;
        assert!(
            !error_messages(&json).is_empty(),
            "expected rejection: {json}"
        );
    }

    let json = run(&schema, &context, r#"{ alertDestinations { id } }"#).await;
    assert_eq!(
        json.pointer("/data/alertDestinations/0/id"),
        Some(&serde_json::json!(id.as_str()))
    );

    let json = run(
        &schema,
        &context,
        format!(r#"mutation {{ alertDestinationDelete(id: "{id}") }}"#),
    )
    .await;
    assert_eq!(
        json.pointer("/data/alertDestinationDelete"),
        Some(&serde_json::json!(true))
    );
}

#[tokio::test]
async fn alert_incidents_states_and_checks_read_paths() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;

    let json = run(&schema, &context, RULE_SAVE).await;
    let rule_id = json
        .pointer("/data/alertRuleSave/id")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_string();
    let store = context.alerts.as_ref().unwrap();
    let opened = store
        .alert_incident_open(&parallax_metadata::AlertIncidentRecord {
            id: "inc_1".to_string(),
            rule_id: rule_id.clone(),
            group_key: "checkout".to_string(),
            status: "open".to_string(),
            severity: "critical".to_string(),
            first_triggered_at_nanos: 60_000_000_000,
            last_triggered_at_nanos: 60_000_000_000,
            resolved_at_nanos: None,
            last_value: Some(0.4),
            last_notified_at_nanos: None,
            bundle_hash: None,
            bundle_assembled_at_nanos: None,
            bundle_top_hypothesis: None,
            bundle_deploy_adjacency: None,
            bundle_error: None,
        })
        .await
        .unwrap();
    assert!(opened);
    store
        .alert_check_insert(&parallax_metadata::AlertCheckRecord {
            rule_id: rule_id.clone(),
            group_key: "checkout".to_string(),
            checked_at_nanos: 60_000_000_000,
            value: Some(0.4),
            sample_count: 12,
            status: "breach".to_string(),
            error: None,
        })
        .await
        .unwrap();

    let json = run(
        &schema,
        &context,
        format!(
            r#"{{ alertIncidents(status: "open") {{
                 id ruleId groupKey status severity lastValue
                 rule {{ id name }}
               }}
               alertChecks(ruleId: "{rule_id}") {{ status value sampleCount }}
               alertRuleStates(ruleId: "{rule_id}") {{ ruleId }} }}"#
        ),
    )
    .await;
    assert!(error_messages(&json).is_empty(), "reads: {json}");
    assert_eq!(
        json.pointer("/data/alertIncidents/0/id"),
        Some(&serde_json::json!("inc_1"))
    );
    assert_eq!(
        json.pointer("/data/alertIncidents/0/rule/id"),
        Some(&serde_json::json!(rule_id.as_str()))
    );
    assert_eq!(
        json.pointer("/data/alertChecks/0/status"),
        Some(&serde_json::json!("breach"))
    );
    assert_eq!(
        json.pointer("/data/alertChecks/0/sampleCount"),
        Some(&serde_json::json!(12))
    );

    let json = run(
        &schema,
        &context,
        r#"{ alertIncidents(status: "bogus") { id } }"#,
    )
    .await;
    assert!(!error_messages(&json).is_empty(), "bad status: {json}");
}

#[tokio::test]
async fn alert_resolvers_report_unavailable_without_turso_handle() {
    let schema = build_schema();
    let mut context = context_with_memory(Arc::new(MemoryStore::new())).await;
    context.alerts = None;
    let json = run(&schema, &context, r#"{ alertRules { id } }"#).await;
    let errors = error_messages(&json);
    assert!(
        errors
            .iter()
            .any(|message| message.contains("not available")),
        "expected unavailable error: {errors:?}"
    );
}
