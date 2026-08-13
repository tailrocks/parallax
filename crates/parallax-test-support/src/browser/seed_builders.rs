//! Dataset-specific row builders for browser contract seeds.

use anyhow::{Context, Result};
use parallax_metadata::{
    AlertDestinationRecord, AlertIncidentRecord, AlertRuleRecord, TursoMetadataStore,
};
use parallax_model::{HistogramRow, MetricPointRow, SpanRow};

use super::datasets::{
    ALERT_DEST_PILOT_ID, ALERT_DEST_PILOT_NAME, ALERT_INCIDENT_PILOT_ID, ALERT_RULE_PILOT_ID,
    ALERT_RULE_PILOT_NAME, CONTRACTS_TS_NANOS, DASHBOARD_PILOT_ID, DASHBOARD_PILOT_NAME,
    DASHBOARD_PILOT_WIDGET, INVESTIGATION_PILOT_ID, INVESTIGATION_PILOT_NAME, LOGS_PILOT_BODY,
    LOGS_PILOT_SERVICE_A, LOGS_PILOT_SERVICE_B, METRICS_PILOT_GAUGE, METRICS_PILOT_HISTOGRAM,
    TRACES_PILOT_CHILD_NAME, TRACES_PILOT_ERROR_NAME, TRACES_PILOT_ROOT_NAME,
    TRACES_PILOT_TRACE_ID, pilot_investigation_state_json,
};
use crate::builders::{MemoryStore, log_row, span};

pub(super) async fn seed_investigations_pilot(
    store: &MemoryStore,
    metadata: &TursoMetadataStore,
) -> Result<()> {
    store.push_spans(vec![span(
        "checkout",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bbbbbbbbbbbbbbbb",
        CONTRACTS_TS_NANOS,
        12_000_000,
    )]);
    metadata
        .investigation_save(
            INVESTIGATION_PILOT_ID,
            INVESTIGATION_PILOT_NAME,
            &pilot_investigation_state_json(),
            CONTRACTS_TS_NANOS,
        )
        .await
        .context("seed pilot investigation")
}

pub(super) fn seed_logs_pilot(store: &MemoryStore) {
    let mut rows = Vec::new();
    for (index, (service, severity_num, severity_text, body)) in [
        (LOGS_PILOT_SERVICE_A, 9, "INFO", "checkout started"),
        (LOGS_PILOT_SERVICE_A, 13, "WARN", "checkout retry"),
        (LOGS_PILOT_SERVICE_A, 17, "ERROR", LOGS_PILOT_BODY),
        (LOGS_PILOT_SERVICE_B, 9, "INFO", "billing posted"),
        (LOGS_PILOT_SERVICE_B, 13, "WARN", "billing delayed"),
        (LOGS_PILOT_SERVICE_B, 17, "ERROR", "billing declined"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut row = log_row(
            service,
            TRACES_PILOT_TRACE_ID,
            CONTRACTS_TS_NANOS + u128::try_from(index).unwrap_or(0) * 1_000,
            body,
        );
        row.severity_num = severity_num;
        row.severity_text = severity_text.into();
        rows.push(row);
    }
    store.push_logs(rows);
}

fn named_span(
    service: &str,
    name: &str,
    span_id: &str,
    parent: Option<&str>,
    error: bool,
    offset: u128,
) -> SpanRow {
    let mut row = span(
        service,
        TRACES_PILOT_TRACE_ID,
        span_id,
        CONTRACTS_TS_NANOS + offset,
        8_000_000,
    );
    row.name = name.into();
    row.parent_span_id = parent.map(str::to_string);
    if error {
        row.status_code = "STATUS_CODE_ERROR".into();
        row.status_message = "pay failed".into();
    }
    row
}

pub(super) fn seed_traces_pilot(store: &MemoryStore) {
    store.push_spans(vec![
        named_span(
            LOGS_PILOT_SERVICE_A,
            TRACES_PILOT_ROOT_NAME,
            "1111111111111111",
            None,
            false,
            0,
        ),
        named_span(
            LOGS_PILOT_SERVICE_A,
            TRACES_PILOT_CHILD_NAME,
            "2222222222222222",
            Some("1111111111111111"),
            false,
            1_000,
        ),
        named_span(
            LOGS_PILOT_SERVICE_A,
            TRACES_PILOT_ERROR_NAME,
            "3333333333333333",
            Some("1111111111111111"),
            true,
            2_000,
        ),
    ]);
}

pub(super) async fn seed_dashboards_pilot(
    store: &MemoryStore,
    metadata: &TursoMetadataStore,
) -> Result<()> {
    seed_metrics_pilot(store);
    let layout = serde_json::json!([{
        "metric": METRICS_PILOT_HISTOGRAM,
        "agg": "avg",
        "chart": "line",
        "title": DASHBOARD_PILOT_WIDGET,
        "w": 2
    }])
    .to_string();
    metadata
        .dashboard_save(
            DASHBOARD_PILOT_ID,
            DASHBOARD_PILOT_NAME,
            &layout,
            CONTRACTS_TS_NANOS,
        )
        .await
        .context("seed dashboard")
}

pub(super) fn seed_sql_pilot(store: &MemoryStore) {
    store.push_logs(vec![
        log_row(
            LOGS_PILOT_SERVICE_A,
            TRACES_PILOT_TRACE_ID,
            CONTRACTS_TS_NANOS,
            LOGS_PILOT_BODY,
        ),
        log_row(
            LOGS_PILOT_SERVICE_B,
            TRACES_PILOT_TRACE_ID,
            CONTRACTS_TS_NANOS + 1_000,
            "billing posted",
        ),
    ]);
}

pub(super) async fn seed_alerts_pilot(metadata: &TursoMetadataStore) -> Result<()> {
    metadata
        .alert_destination_save(&AlertDestinationRecord {
            id: ALERT_DEST_PILOT_ID.into(),
            name: ALERT_DEST_PILOT_NAME.into(),
            kind: "webhook".into(),
            config: r#"{"url":"https://example.test/hooks/parallax"}"#.into(),
            created_at_nanos: CONTRACTS_TS_NANOS,
            updated_at_nanos: CONTRACTS_TS_NANOS,
        })
        .await
        .context("seed destination")?;
    metadata
        .alert_rule_save(&AlertRuleRecord {
            id: ALERT_RULE_PILOT_ID.into(),
            name: ALERT_RULE_PILOT_NAME.into(),
            enabled: true,
            signal_type: "error_rate".into(),
            services: r#"["checkout"]"#.into(),
            exclude_services: "[]".into(),
            attribute_filters: "[]".into(),
            group_by: None,
            comparator: "gt".into(),
            threshold: 0.2,
            threshold_upper: None,
            window_minutes: 5,
            minimum_sample_count: 1,
            consecutive_breaches_required: 2,
            consecutive_healthy_required: 2,
            no_data_behavior: "skip".into(),
            severity: "critical".into(),
            renotify_interval_minutes: 30,
            destination_ids: format!(r#"["{ALERT_DEST_PILOT_ID}"]"#),
            metric_name: None,
            metric_aggregation: None,
            created_at_nanos: CONTRACTS_TS_NANOS,
            updated_at_nanos: CONTRACTS_TS_NANOS,
        })
        .await
        .context("seed rule")?;
    metadata
        .alert_incident_open(&AlertIncidentRecord {
            id: ALERT_INCIDENT_PILOT_ID.into(),
            rule_id: ALERT_RULE_PILOT_ID.into(),
            group_key: "checkout".into(),
            status: "open".into(),
            severity: "critical".into(),
            first_triggered_at_nanos: CONTRACTS_TS_NANOS,
            last_triggered_at_nanos: CONTRACTS_TS_NANOS,
            resolved_at_nanos: None,
            last_value: Some(0.4),
            last_notified_at_nanos: None,
        })
        .await
        .context("seed incident")?;
    metadata
        .alert_incident_resolve(
            ALERT_RULE_PILOT_ID,
            "checkout",
            CONTRACTS_TS_NANOS + 1_000,
            Some(0.1),
        )
        .await
        .context("resolve incident")?;
    Ok(())
}

pub(super) fn seed_metrics_pilot(store: &MemoryStore) {
    store.push_metrics(
        vec![MetricPointRow {
            ts_nanos: CONTRACTS_TS_NANOS,
            service: LOGS_PILOT_SERVICE_A.into(),
            name: METRICS_PILOT_GAUGE.into(),
            value: 4.0,
            is_monotonic: false,
            invocation_id: None,
            attributes: serde_json::json!({}),
        }],
        vec![HistogramRow {
            ts_nanos: CONTRACTS_TS_NANOS,
            service: LOGS_PILOT_SERVICE_A.into(),
            name: METRICS_PILOT_HISTOGRAM.into(),
            count: 4,
            sum: 80.0,
            bucket_counts: vec![1, 2, 1],
            bounds: vec![10.0, 50.0],
            attributes: serde_json::json!({}),
        }],
        vec![],
    );
}
