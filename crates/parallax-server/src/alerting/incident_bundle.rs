//! Fire-time incident bundle assembly (plan 173). Never blocks delivery:
//! enqueue happens first; this persist is best-effort with a hard timeout.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use parallax_evidence::bundle::{
    BundleAnchor, BundleInputs, IncidentAnchor, MetricWindow, assemble,
};
use parallax_metadata::{AlertIncidentRecord, AlertRuleRecord, TursoMetadataStore};

/// Injected assembly failure for evaluator tests. Never set in product.
pub(crate) static FAIL_INCIDENT_BUNDLE: AtomicBool = AtomicBool::new(false);

pub(crate) const ASSEMBLY_TIMEOUT: Duration = Duration::from_millis(250);

pub(crate) fn measured_series_window(
    rule: &AlertRuleRecord,
    last_value: Option<f64>,
    now_nanos: u128,
) -> Vec<MetricWindow> {
    let Some(value) = last_value else {
        return Vec::new();
    };
    let (from, to) =
        parallax_evidence::bundle::incident_bundle_window(now_nanos, rule.window_minutes);
    MetricWindow::from_points(
        rule.signal_type.clone(),
        "service",
        from,
        to,
        rule.window_minutes.saturating_mul(60).max(1),
        vec![(now_nanos, value)],
    )
    .into_iter()
    .collect()
}

pub(crate) fn assemble_incident_hash(
    rule: &AlertRuleRecord,
    incident_id: &str,
    group_key: &str,
    last_value: Option<f64>,
    now_nanos: u128,
) -> Result<(String, Option<String>, String), String> {
    if FAIL_INCIDENT_BUNDLE.load(Ordering::SeqCst) {
        return Err("injected assembly failure".into());
    }
    let inputs = BundleInputs {
        anchor: BundleAnchor::Incident(Box::new(IncidentAnchor {
            incident_id: incident_id.to_string(),
            rule_name: rule.name.clone(),
            signal_type: rule.signal_type.clone(),
            severity: rule.severity.clone(),
            group_key: group_key.to_string(),
            window_minutes: rule.window_minutes,
            last_value,
        })),
        events: Vec::new(),
        trace_spans: Vec::new(),
        trace_logs: Vec::new(),
        metric_windows: measured_series_window(rule, last_value, now_nanos),
        ci_adjacency: Vec::new(),
        deploy_adjacency: Vec::new(),
    };
    let bundle = assemble(inputs, 4_000);
    let hash = bundle
        .canonical_hash
        .ok_or_else(|| "assembly produced no hash".to_string())?;
    let top = bundle.hypotheses.first().map(|h| h.statement.clone());
    let adjacency = serde_json::to_string(
        &bundle
            .hypotheses
            .iter()
            .filter_map(|h| (h.kind == "deploy_adjacency").then_some(h.statement.clone()))
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".into());
    Ok((hash, top, adjacency))
}

/// Persist hash/error after the incident row exists. Errors are swallowed —
/// the tick and outbox enqueue must proceed.
pub(crate) async fn persist_incident_bundle(
    store: &TursoMetadataStore,
    rule: &AlertRuleRecord,
    incident_id: &str,
    group_key: &str,
    last_value: Option<f64>,
    now_nanos: u128,
) {
    let assembled = tokio::time::timeout(ASSEMBLY_TIMEOUT, async {
        assemble_incident_hash(rule, incident_id, group_key, last_value, now_nanos)
    })
    .await;
    let (hash, top, adjacency, error) = match assembled {
        Ok(Ok((hash, top, adjacency))) => (Some(hash), top, Some(adjacency), None),
        Ok(Err(message)) => (None, None, None, Some(message)),
        Err(_) => (None, None, None, Some("assembly timed out".into())),
    };
    drop(
        store
            .alert_incident_set_bundle(
                incident_id,
                parallax_metadata::IncidentBundleSnapshot {
                    hash: hash.as_deref(),
                    assembled_at_nanos: now_nanos,
                    top_hypothesis: top.as_deref(),
                    deploy_adjacency: adjacency.as_deref(),
                    error: error.as_deref(),
                },
            )
            .await,
    );
}

/// Renotify reuses the stored bundle unless the incident has none.
pub(crate) fn should_reuse_bundle(incident: &AlertIncidentRecord) -> bool {
    incident.bundle_hash.is_some() && incident.bundle_error.is_none()
}
