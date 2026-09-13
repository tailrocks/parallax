//! GraphQL issues domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::HashSet;
use std::sync::Arc;

use crate::{
    ApiContext, MAX_ROWS, clamp_limit, field_err, internal_field_err, nanos_string,
    retained_recent_range, saturate_i32,
};

mod incident_bundle;
mod metric_windows;
mod nested;
pub(crate) use metric_windows::bundle_metric_windows;
pub(crate) use nested::{Issue, IssueList, IssueSort, TrendPoint};

fn issue_keys(events: &[model::ErrorEventRow]) -> Vec<(String, String)> {
    let mut seen = HashSet::new();
    events
        .iter()
        .filter(|event| seen.insert((event.service.clone(), event.fingerprint.clone())))
        .map(|event| (event.service.clone(), event.fingerprint.clone()))
        .collect()
}

/// Linkage-only deploy/CI adjacency for bundles (plans 121/124). Never root-cause.
async fn load_evidence_adjacency(
    context: &ApiContext,
    inputs: &mut parallax_evidence::bundle::BundleInputs,
) {
    let Some(store) = context.alerts.as_ref() else {
        return;
    };
    if let Ok(deploys) = store.list_recent_deploy_deliveries(3).await {
        inputs.deploy_adjacency = deploys
            .into_iter()
            .map(|row| {
                let sha = row
                    .commit_sha
                    .as_deref()
                    .map(|value| &value[..value.len().min(12)])
                    .unwrap_or("no-sha");
                let env = row.environment.as_deref().unwrap_or("unknown-env");
                let repo = row.repo_full_name.as_deref().unwrap_or("unknown/unknown");
                format!(
                    "Deploy {} on {repo} ({env}, sha={sha}, strength={}) is adjacent timing evidence only — {}",
                    row.deployment_id,
                    row.edge_strength,
                    parallax_evidence::github_deploy::DEPLOY_ADJACENCY_CLAIM_WORDING
                )
            })
            .collect();
    }
    if let Ok(attempts) = store.list_recent_ci_attempts(3).await {
        inputs.ci_adjacency = attempts
            .into_iter()
            .map(|row| {
                let name = row.name.as_deref().unwrap_or("job");
                let conclusion = row.conclusion.as_deref().unwrap_or("unknown");
                format!(
                    "CI attempt `{name}` on {} (run {}, attempt {}, conclusion={conclusion}) is adjacent timing evidence only — {}",
                    row.repo_full_name,
                    row.workflow_run_id,
                    row.attempt,
                    parallax_evidence::github_actions::CI_ADJACENCY_CLAIM_WORDING
                )
            })
            .collect();
    }
}

pub(crate) struct BundleOut {
    json: String,
    markdown: String,
    canonical_hash: String,
}

#[graphql_object(context = ApiContext)]
impl BundleOut {
    /// The bundle as canonical JSON.
    fn json(&self) -> &str {
        &self.json
    }
    /// The agent-facing Markdown projection.
    fn markdown(&self) -> &str {
        &self.markdown
    }
    fn canonical_hash(&self) -> &str {
        &self.canonical_hash
    }
}

#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn issues(
    context: &ApiContext,
    service: Option<String>,
    status: Option<String>,
    query: Option<String>,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    tag_key: Option<String>,
    tag_value: Option<String>,
    environment: Option<String>,
    sort: Option<IssueSort>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> FieldResult<IssueList> {
    if let Some(status) = status.as_deref()
        && !matches!(status, "open" | "resolved" | "regressed")
    {
        return Err(field_err("status must be open, resolved, or regressed"));
    }
    let filter = model::IssueQuery {
        service,
        status,
        query,
        from_nanos: match from_nanos {
            Some(s) => Some(s.parse().map_err(|_| field_err("invalid fromNanos"))?),
            None => None,
        },
        to_nanos: match to_nanos {
            Some(s) => Some(s.parse().map_err(|_| field_err("invalid toNanos"))?),
            None => None,
        },
        tag_key,
        tag_value,
        environment,
    };
    let offset = usize::try_from(offset.unwrap_or(0).max(0)).unwrap_or(0);
    let (items, total) = context
        .metadata
        .issues_filtered(
            &filter,
            sort.unwrap_or(IssueSort::LastSeen).key(),
            clamp_limit(limit, 50),
            offset,
        )
        .await
        .map_err(internal_field_err)?;
    Ok(IssueList::new(items, total))
}

pub(crate) async fn issue(
    context: &ApiContext,
    service: String,
    fingerprint: String,
) -> FieldResult<Option<Issue>> {
    Ok(context
        .metadata
        .issue(&service, &fingerprint)
        .await
        .map_err(internal_field_err)?
        .map(Issue::single))
}

pub(crate) async fn issue_trend(
    context: &ApiContext,
    service: String,
    fingerprint: String,
    hours: Option<i32>,
    step_seconds: Option<i32>,
) -> FieldResult<Vec<TrendPoint>> {
    let hours = u64::try_from(hours.unwrap_or(24).clamp(1, 24 * 30)).unwrap_or(24);
    let step = u32::try_from(step_seconds.unwrap_or(3600).clamp(60, 86_400)).unwrap_or(3600);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(internal_field_err)?
        .as_nanos();
    let since = now.saturating_sub(u128::from(hours) * 3_600_000_000_000);
    let points = context
        .metadata
        .issue_trend(&service, &fingerprint, since, step)
        .await
        .map_err(internal_field_err)?;
    Ok(points.into_iter().map(TrendPoint).collect())
}

fn validate_bundle_anchors(present: usize) -> FieldResult<()> {
    if present == 1 {
        Ok(())
    } else {
        Err(field_err(
            "bundle takes exactly one anchor: fingerprint, invocationId, traceId, or alertIncidentId",
        ))
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one-of anchor arguments form the public GraphQL contract"
)]
pub(crate) async fn bundle(
    context: &ApiContext,
    service: Option<String>,
    fingerprint: Option<String>,
    invocation_id: Option<String>,
    trace_id: Option<String>,
    alert_incident_id: Option<String>,
    max_tokens: Option<i32>,
) -> FieldResult<Option<BundleOut>> {
    use parallax_evidence::bundle::{BundleAnchor, BundleInputs};
    let trace_id = crate::validate_optional_trace_id(trace_id)?;
    let max_tokens = usize::try_from(max_tokens.unwrap_or(10_000).max(500)).unwrap_or(10_000);
    validate_bundle_anchors(
        usize::from(fingerprint.is_some())
            + usize::from(invocation_id.is_some())
            + usize::from(trace_id.is_some())
            + usize::from(alert_incident_id.is_some()),
    )?;
    if let Some(incident_id) = alert_incident_id {
        return incident_bundle::bundle_from_incident(context, &incident_id, max_tokens).await;
    }

    let mut inputs = if let Some(fingerprint) = fingerprint {
        let Some(service) = service.as_deref() else {
            return Err(field_err(
                "service is required when bundling by fingerprint: issue identity is (service, fingerprint)",
            ));
        };
        let Some(issue) = context
            .metadata
            .issue(service, &fingerprint)
            .await
            .map_err(internal_field_err)?
        else {
            return Ok(None);
        };
        let events = context
            .store
            .error_events_by_fingerprint(service, &fingerprint, 0..=u128::MAX, 5, None)
            .await
            .map_err(internal_field_err)?;
        let (trace_spans, trace_logs) = match issue.last_trace_id.as_deref() {
            Some(trace_id) => {
                let (spans, logs) =
                    tokio::try_join!(context.spans_for(trace_id), context.logs_for(trace_id),)?;
                (Arc::unwrap_or_clone(spans), Arc::unwrap_or_clone(logs))
            }
            None => (Vec::new(), Vec::new()),
        };
        BundleInputs {
            anchor: BundleAnchor::Issue(Box::new(issue)),
            events,
            trace_spans,
            trace_logs,
            metric_windows: Vec::new(),
            ci_adjacency: Vec::new(),
            deploy_adjacency: Vec::new(),
        }
    } else if let Some(invocation_id) = invocation_id {
        let Some(run) = context
            .metadata
            .invocation(&invocation_id)
            .await
            .map_err(internal_field_err)?
        else {
            return Ok(None);
        };
        let spans = context
            .store
            .spans_by_invocation(&invocation_id, MAX_ROWS, retained_recent_range())
            .await
            .map_err(internal_field_err)?;
        let mut trace_ids: Vec<String> = Vec::new();
        let mut seen_trace_ids = HashSet::new();
        for span in &spans {
            let trace_id = span.trace_id.clone();
            if seen_trace_ids.insert(trace_id.clone()) {
                trace_ids.push(trace_id);
            }
        }
        let events = context
            .store
            .error_events_by_traces(&trace_ids, 50)
            .await
            .map_err(internal_field_err)?;
        let fingerprints = issue_keys(&events);
        let issues = context
            .metadata
            .issues_by_fingerprints(&fingerprints)
            .await
            .map_err(internal_field_err)?;
        // The trace behind the newest error carries the evidence; the
        // run's logs are the log section.
        let evidence_trace = events.first().map(|e| e.trace_id.clone());
        let trace_spans = match &evidence_trace {
            Some(trace_id) if !trace_id.is_empty() => spans
                .iter()
                .filter(|s| s.trace_id == *trace_id)
                .cloned()
                .collect(),
            _ => Vec::new(),
        };
        let trace_logs = context
            .store
            .logs_by_invocation(&invocation_id, 200)
            .await
            .map_err(internal_field_err)?;
        BundleInputs {
            anchor: BundleAnchor::Invocation {
                invocation: Box::new(run),
                issues,
            },
            events,
            trace_spans,
            trace_logs,
            metric_windows: Vec::new(),
            ci_adjacency: Vec::new(),
            deploy_adjacency: Vec::new(),
        }
    } else {
        let trace_id = trace_id.unwrap_or_default();
        let (trace_spans, trace_logs) =
            tokio::try_join!(context.spans_for(&trace_id), context.logs_for(&trace_id),)?;
        if trace_spans.is_empty() {
            return Ok(None);
        }
        let events = context
            .store
            .error_events_by_traces(std::slice::from_ref(&trace_id), 50)
            .await
            .map_err(internal_field_err)?;
        let fingerprints = issue_keys(&events);
        let issues = context
            .metadata
            .issues_by_fingerprints(&fingerprints)
            .await
            .map_err(internal_field_err)?;
        BundleInputs {
            anchor: BundleAnchor::Trace { trace_id, issues },
            events,
            trace_spans: Arc::unwrap_or_clone(trace_spans),
            trace_logs: Arc::unwrap_or_clone(trace_logs),
            metric_windows: Vec::new(),
            ci_adjacency: Vec::new(),
            deploy_adjacency: Vec::new(),
        }
    };

    inputs.metric_windows = bundle_metric_windows(context, &inputs).await?;
    load_evidence_adjacency(context, &mut inputs).await;
    incident_bundle::finish_bundle(inputs, max_tokens).await
}

pub(crate) async fn issue_set_status(
    context: &ApiContext,
    service: String,
    fingerprint: String,
    status: String,
) -> FieldResult<Issue> {
    if !matches!(status.as_str(), "open" | "resolved") {
        return Err(field_err("status must be open or resolved"));
    }
    let changed_at_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(internal_field_err)?
        .as_nanos();
    context
        .metadata
        .set_issue_status(&service, &fingerprint, &status, changed_at_nanos)
        .await
        .map_err(internal_field_err)?;
    context
        .metadata
        .issue(&service, &fingerprint)
        .await
        .map_err(internal_field_err)?
        .map(Issue::single)
        .ok_or_else(|| field_err(format!("issue {service}/{fingerprint} not found")))
}

#[cfg(test)]
mod tests;
