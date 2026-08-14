//! Preview evaluation cases (plan 171). Sibling of `preview.rs` so the impl
//! file stays below the inline-test-module ratchet.

use super::preview::{MINUTES_NS, preview_rule};
use super::{AlertMeasurement, GroupMeasurement, MeasurementSource};
use async_trait::async_trait;
use parallax_metadata::AlertRuleRecord;

struct ConstantSource(f64);

#[async_trait]
impl MeasurementSource for ConstantSource {
    async fn measure(
        &self,
        _rule: &AlertRuleRecord,
        _from: u128,
        _to: u128,
    ) -> anyhow::Result<Vec<GroupMeasurement>> {
        Ok(vec![GroupMeasurement {
            group_key: "checkout".into(),
            measurement: AlertMeasurement {
                value: Some(self.0),
                sample_count: 10,
            },
        }])
    }
}

fn draft(threshold: f64, breaches: u32) -> AlertRuleRecord {
    AlertRuleRecord {
        id: "preview".into(),
        name: "preview".into(),
        enabled: true,
        signal_type: "error_rate".into(),
        services: "[]".into(),
        exclude_services: "[]".into(),
        attribute_filters: "[]".into(),
        group_by: Some("service".into()),
        comparator: "gt".into(),
        threshold,
        threshold_upper: None,
        window_minutes: 12,
        minimum_sample_count: 1,
        consecutive_breaches_required: breaches,
        consecutive_healthy_required: 2,
        no_data_behavior: "skip".into(),
        severity: "warning".into(),
        renotify_interval_minutes: 30,
        destination_ids: "[]".into(),
        metric_name: None,
        metric_aggregation: None,
        created_at_nanos: 0,
        updated_at_nanos: 0,
    }
}

#[tokio::test]
async fn consecutive_breaches_mark_would_fire() {
    let rule = draft(0.1, 2);
    let preview = preview_rule(&ConstantSource(0.5), &rule, 12, 12 * MINUTES_NS)
        .await
        .unwrap();
    assert_eq!(preview.groups.len(), 1);
    let fires: Vec<bool> = preview.groups[0]
        .points
        .iter()
        .map(|point| point.would_fire)
        .collect();
    assert!(
        fires.iter().any(|fired| *fired),
        "expected a would-fire marker: {fires:?}"
    );
    assert!(preview.groups[0].samples_sufficient);
}

#[tokio::test]
async fn below_threshold_does_not_fire() {
    let rule = draft(0.9, 1);
    let preview = preview_rule(&ConstantSource(0.1), &rule, 6, 6 * MINUTES_NS)
        .await
        .unwrap();
    assert!(
        preview.groups[0]
            .points
            .iter()
            .all(|point| !point.would_fire),
        "quiet draft must not mark fire"
    );
}
