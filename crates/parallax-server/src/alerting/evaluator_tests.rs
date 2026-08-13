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
