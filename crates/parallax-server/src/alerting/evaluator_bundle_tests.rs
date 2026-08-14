use super::evaluator::{MeasurementSource, tick_once};
use super::incident_bundle::FAIL_INCIDENT_BUNDLE;
use parallax_metadata::{AlertDestinationRecord, AlertRuleRecord, TursoMetadataStore};
use std::sync::atomic::Ordering;

const SEC: u128 = 1_000_000_000;
const MIN: u128 = 60 * SEC;

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
    ) -> anyhow::Result<Vec<super::evaluator::GroupMeasurement>> {
        let mut values = self.values.lock().expect("lock");
        let value = if values.is_empty() {
            None
        } else {
            values.remove(0)
        };
        Ok(vec![super::evaluator::GroupMeasurement {
            group_key: "checkout".to_string(),
            measurement: super::AlertMeasurement {
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
    let directory = tempfile::TempDir::new().expect("temporary directory");
    let path = directory.path().join("metadata.db");
    (directory, path)
}

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

fn dest() -> AlertDestinationRecord {
    AlertDestinationRecord {
        id: "d1".into(),
        name: "hook".into(),
        kind: "webhook".into(),
        config: "{\"url\":\"http://127.0.0.1/hook\"}".into(),
        created_at_nanos: MIN,
        updated_at_nanos: MIN,
    }
}

#[tokio::test]
async fn open_persists_bundle_hash() {
    let (_dir, path) = temp_store();
    let store = TursoMetadataStore::open(path).await.expect("open");
    store.alert_rule_save(&rule()).await.expect("save");
    store.alert_destination_save(&dest()).await.expect("dest");
    let source = StubSource::new(vec![Some(0.5), Some(0.6)], 50);
    let mut now = 100 * MIN;
    tick_once(&store, &source, now, 30).await.expect("tick1");
    now += MIN;
    let report = tick_once(&store, &source, now, 30).await.expect("tick2");
    assert_eq!(report.incidents_opened, 1);
    assert_eq!(report.deliveries_enqueued, 1);
    let open = store
        .alert_incidents(Some("open"), Some("r1"), 10)
        .await
        .expect("list");
    assert!(open[0].bundle_hash.is_some());
    assert!(open[0].bundle_error.is_none());
}

#[tokio::test]
async fn open_survives_injected_assembly_failure() {
    FAIL_INCIDENT_BUNDLE.store(true, Ordering::SeqCst);
    let (_dir, path) = temp_store();
    let store = TursoMetadataStore::open(path).await.expect("open");
    store.alert_rule_save(&rule()).await.expect("save");
    store.alert_destination_save(&dest()).await.expect("dest");
    let source = StubSource::new(vec![Some(0.5), Some(0.6)], 50);
    let mut now = 100 * MIN;
    tick_once(&store, &source, now, 30).await.expect("tick1");
    now += MIN;
    let report = tick_once(&store, &source, now, 30).await.expect("tick2");
    FAIL_INCIDENT_BUNDLE.store(false, Ordering::SeqCst);
    assert_eq!(report.incidents_opened, 1);
    assert_eq!(report.deliveries_enqueued, 1);
    let open = store
        .alert_incidents(Some("open"), Some("r1"), 10)
        .await
        .expect("list");
    assert!(open[0].bundle_hash.is_none());
    assert_eq!(
        open[0].bundle_error.as_deref(),
        Some("injected assembly failure")
    );
}
