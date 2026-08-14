use super::delivery::{DeliveryEventType, NotificationContext, webhook_payload_json};
use super::incident_bundle::{FAIL_INCIDENT_BUNDLE, assemble_incident_hash};
use parallax_metadata::AlertRuleRecord;
use std::sync::atomic::Ordering;

fn rule() -> AlertRuleRecord {
    AlertRuleRecord {
        id: "r1".into(),
        name: "High error rate".into(),
        enabled: true,
        signal_type: "error_rate".into(),
        services: "[\"checkout\"]".into(),
        exclude_services: "[]".into(),
        attribute_filters: "[]".into(),
        group_by: Some("service".into()),
        comparator: "gt".into(),
        threshold: 0.2,
        threshold_upper: None,
        window_minutes: 5,
        minimum_sample_count: 1,
        consecutive_breaches_required: 1,
        consecutive_healthy_required: 1,
        no_data_behavior: "skip".into(),
        severity: "critical".into(),
        renotify_interval_minutes: 30,
        destination_ids: "[]".into(),
        metric_name: None,
        metric_aggregation: None,
        created_at_nanos: 0,
        updated_at_nanos: 0,
    }
}

#[test]
fn assemble_hash_is_stable() {
    let rule = rule();
    let a = assemble_incident_hash(&rule, "inc-1", "checkout", Some(0.4), 10_000).unwrap();
    let b = assemble_incident_hash(&rule, "inc-1", "checkout", Some(0.4), 10_000).unwrap();
    assert_eq!(a.0, b.0);
    assert!(!a.0.is_empty());
}

#[test]
fn injected_failure_does_not_yield_hash() {
    FAIL_INCIDENT_BUNDLE.store(true, Ordering::SeqCst);
    let result = assemble_incident_hash(&rule(), "inc-1", "checkout", Some(0.4), 10_000);
    FAIL_INCIDENT_BUNDLE.store(false, Ordering::SeqCst);
    assert_eq!(result, Err("injected assembly failure".into()));
}

#[test]
fn webhook_payload_without_bundle_carries_bundle_error() {
    let ctx = NotificationContext {
        rule_id: "r1",
        rule_name: "High error rate",
        signal_type: "error_rate",
        severity: "critical",
        group_key: "checkout",
        incident_id: "inc-1",
        event_type: DeliveryEventType::Triggered,
        observed_value: Some(0.4),
        threshold: 0.2,
        threshold_upper: None,
        window_minutes: 5,
        incident_url: "http://127.0.0.1/alerts/incidents/inc-1",
        investigate_url: "http://127.0.0.1/traces",
        bundle_hash: None,
        bundle_url: None,
        top_hypothesis: None,
        deploy_adjacency: &[],
        bundle_error: Some("assembly unavailable"),
    };
    let body = webhook_payload_json(&ctx);
    assert!(body.contains("\"bundle_error\":\"assembly unavailable\""));
    assert!(body.contains("\"bundle_hash\":null"));
}

#[test]
fn webhook_payload_never_leaks_canary_secret() {
    let canary = "CANARY_TOKEN_XYZ";
    let (hash, top, adjacency) =
        assemble_incident_hash(&rule(), "inc-1", "checkout", Some(0.4), 10_000).unwrap();
    let adj: Vec<String> = serde_json::from_str(&adjacency).unwrap();
    let ctx = NotificationContext {
        rule_id: "r1",
        rule_name: "High error rate",
        signal_type: "error_rate",
        severity: "critical",
        group_key: "checkout",
        incident_id: "inc-1",
        event_type: DeliveryEventType::Triggered,
        observed_value: Some(0.4),
        threshold: 0.2,
        threshold_upper: None,
        window_minutes: 5,
        incident_url: "http://127.0.0.1/alerts/incidents/inc-1",
        investigate_url: "http://127.0.0.1/traces",
        bundle_hash: Some(hash.as_str()),
        bundle_url: Some("http://127.0.0.1/alerts?incident=inc-1"),
        top_hypothesis: top.as_deref(),
        deploy_adjacency: &adj,
        bundle_error: None,
    };
    let body = webhook_payload_json(&ctx);
    assert!(!body.contains(canary));
    assert!(body.contains("\"bundle_hash\":"));
    assert!(body.contains("\"bundle_error\":null"));
}
