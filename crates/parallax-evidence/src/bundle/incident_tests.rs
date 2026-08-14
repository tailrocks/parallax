use super::*;

fn incident_anchor() -> IncidentAnchor {
    IncidentAnchor {
        incident_id: "inc-r1-checkout-1".into(),
        rule_name: "High error rate".into(),
        signal_type: "error_rate".into(),
        severity: "critical".into(),
        group_key: "checkout".into(),
        window_minutes: 5,
        last_value: Some(0.42),
    }
}

fn incident_inputs(logs: Vec<LogRow>, windows: Vec<MetricWindow>) -> BundleInputs {
    BundleInputs {
        anchor: BundleAnchor::Incident(Box::new(incident_anchor())),
        events: Vec::new(),
        trace_spans: Vec::new(),
        trace_logs: logs,
        metric_windows: windows,
        ci_adjacency: Vec::new(),
        deploy_adjacency: vec!["deploy checkout@abc adjacent to breach".into()],
    }
}

fn test_log(index: usize) -> LogRow {
    LogRow {
        ts_nanos: index as u128,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: "checkout".into(),
        severity_num: 9,
        severity_text: "INFO".into(),
        body: format!("line-{index} secret=CANARY_TOKEN_XYZ"),
        trace_id: String::new(),
        span_id: String::new(),
        invocation_id: None,
        session_id: None,
        scope_name: "test".into(),
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

#[test]
fn incident_bundle_includes_measured_series_and_trace_gap() {
    let window = MetricWindow::from_points(
        "error_rate",
        "service",
        0,
        10,
        60,
        vec![(1, 0.2), (2, 0.42)],
    )
    .expect("window");
    let bundle = assemble(incident_inputs(Vec::new(), vec![window]), 8_000);
    assert_eq!(bundle.anchor.kind, "alert_incident");
    assert_eq!(bundle.anchor.id, "inc-r1-checkout-1");
    assert_eq!(bundle.metric_windows.len(), 1);
    assert!(
        bundle
            .missing_evidence
            .iter()
            .any(|gap| gap.contains("no trace"))
    );
    assert!(
        bundle
            .missing_evidence
            .iter()
            .any(|gap| gap.contains("no stored error events in the incident window"))
    );
    assert!(
        bundle
            .hypotheses
            .iter()
            .any(|h| h.kind == "deploy_adjacency")
    );
}

#[test]
fn incident_bundle_hash_is_stable() {
    let window = MetricWindow::from_points("error_rate", "service", 0, 10, 60, vec![(2, 0.42)])
        .expect("window");
    let left = assemble(incident_inputs(Vec::new(), vec![window.clone()]), 8_000);
    let right = assemble(incident_inputs(Vec::new(), vec![window]), 8_000);
    assert_eq!(left.canonical_hash, right.canonical_hash);
    assert!(left.canonical_hash.is_some());
}

#[test]
fn incident_bundle_bounds_log_lines() {
    let logs: Vec<LogRow> = (0..80).map(test_log).collect();
    let bundle = assemble(incident_inputs(logs, Vec::new()), 400);
    assert!(bundle.bounded.dropped_log_lines > 0);
    assert!(bundle.bounded.estimated_tokens <= 400 + 50);
}

#[test]
fn incident_window_is_centered_and_clamped() {
    let (from, to) = incident_bundle_window(1_000_000_000_000, 5);
    assert_eq!(to - from, 10 * 60 * 1_000_000_000);
    let center = 200 * 60 * 1_000_000_000;
    let (from, to) = incident_bundle_window(center, 9_000);
    assert_eq!(to - from, 120 * 60 * 1_000_000_000);
}
