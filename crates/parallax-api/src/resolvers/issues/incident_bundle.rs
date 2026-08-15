//! Assemble a bundle for an alert incident (plan 173).

use crate::{ApiContext, field_err};
use juniper::FieldResult;
use parallax_evidence::bundle::{
    BundleAnchor, BundleInputs, IncidentAnchor, MetricWindow, incident_bundle_window,
};

use super::BundleOut;

pub(super) async fn bundle_from_incident(
    context: &ApiContext,
    incident_id: &str,
    max_tokens: usize,
) -> FieldResult<Option<BundleOut>> {
    let store = context
        .alerts
        .as_ref()
        .ok_or_else(|| field_err("alerting storage is not available on this server"))?;
    let Some(incident) = store.alert_incident(incident_id).await.map_err(field_err)? else {
        return Ok(None);
    };
    let rule = store
        .alert_rule(&incident.rule_id)
        .await
        .map_err(field_err)?;
    let window_minutes = rule.as_ref().map_or(5, |rule| rule.window_minutes);
    let (from, to) = incident_bundle_window(incident.last_triggered_at_nanos, window_minutes);
    let service = (!incident.group_key.is_empty()).then_some(incident.group_key.as_str());
    let logs = context
        .store
        .logs_search(service, from..=to, None, None, None, &[], 50)
        .await
        .unwrap_or_default();
    let metric_windows = incident
        .last_value
        .and_then(|value| {
            MetricWindow::from_points(
                rule.as_ref()
                    .map(|rule| rule.signal_type.clone())
                    .unwrap_or_else(|| "signal".into()),
                "service",
                from,
                to,
                window_minutes.saturating_mul(60).max(1),
                vec![(incident.last_triggered_at_nanos, value)],
            )
        })
        .into_iter()
        .collect();
    let inputs = BundleInputs {
        anchor: BundleAnchor::Incident(Box::new(IncidentAnchor {
            incident_id: incident.id.clone(),
            rule_name: rule
                .as_ref()
                .map(|rule| rule.name.clone())
                .unwrap_or_default(),
            signal_type: rule
                .as_ref()
                .map(|rule| rule.signal_type.clone())
                .unwrap_or_default(),
            severity: incident.severity.clone(),
            group_key: incident.group_key.clone(),
            window_minutes,
            last_value: incident.last_value,
        })),
        events: Vec::new(),
        trace_spans: Vec::new(),
        trace_logs: logs,
        metric_windows,
        ci_adjacency: Vec::new(),
        deploy_adjacency: Vec::new(),
    };
    finish_bundle(inputs, max_tokens).await
}

pub(super) async fn finish_bundle(
    inputs: BundleInputs,
    max_tokens: usize,
) -> FieldResult<Option<BundleOut>> {
    let bundle = parallax_evidence::bundle::assemble(inputs, max_tokens);
    let markdown = parallax_evidence::bundle::to_markdown(&bundle);
    // Envelope hash covers `generated_at`. A wall-clock stamp made sequential
    // CLI / HTTP / MCP reads of the same evidence byte-diverge. Stamp from
    // the evidence window so the three projections stay identical.
    let window_nanos = bundle_window_nanos(&bundle).unwrap_or((0, 0));
    let generated_at_nanos = window_nanos.1;
    let envelope_inputs = parallax_evidence::bundle::EnvelopeInputs {
        bundle_id: bundle
            .canonical_hash
            .clone()
            .unwrap_or_else(|| format!("bundle-{generated_at_nanos}")),
        project: Some("local".to_string()),
        window_nanos: Some(window_nanos),
        generated_at_nanos,
    };
    let v1_hash = bundle.canonical_hash.clone().unwrap_or_default();
    let envelope = parallax_evidence::bundle::envelope_v1(bundle, envelope_inputs)
        .map_err(crate::internal_field_err)?;
    let canonical_hash = envelope.canonical_hash.clone().unwrap_or(v1_hash);
    let json = serde_json::to_string_pretty(&envelope).map_err(crate::internal_field_err)?;
    Ok(Some(BundleOut {
        json,
        markdown,
        canonical_hash,
    }))
}

fn bundle_window_nanos(bundle: &parallax_evidence::bundle::Bundle) -> Option<(u128, u128)> {
    let mut times = Vec::new();
    if let Some(issue) = &bundle.issue {
        if let Ok(v) = issue.first_seen_nanos.parse::<u128>() {
            times.push(v);
        }
        if let Ok(v) = issue.last_seen_nanos.parse::<u128>() {
            times.push(v);
        }
    }
    if let Some(invocation) = &bundle.invocation {
        if let Ok(v) = invocation.started_at_nanos.parse::<u128>() {
            times.push(v);
        }
        if let Some(ended) = &invocation.ended_at_nanos
            && let Ok(v) = ended.parse::<u128>()
        {
            times.push(v);
        }
    }
    let from = *times.iter().min()?;
    let to = *times.iter().max()?;
    Some((from, to))
}
