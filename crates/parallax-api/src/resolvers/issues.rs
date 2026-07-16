//! GraphQL issues domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::HashSet;
use std::sync::Arc;

use crate::{
    ApiContext, MAX_ROWS, clamp_limit, field_err, internal_field_err, nanos_string,
    retained_recent_range, saturate_i32,
};

use parallax_analysis::semconv;
use parallax_storage::model::MetricAgg;

mod nested;
pub(crate) use nested::{Issue, IssueList, IssueSort, TrendPoint};

fn unique_fingerprints(events: &[model::ErrorEventRow]) -> Vec<String> {
    let mut seen = HashSet::new();
    events
        .iter()
        .filter(|event| seen.insert(event.fingerprint.clone()))
        .map(|event| event.fingerprint.clone())
        .collect()
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

pub(crate) async fn bundle_metric_windows(
    context: &ApiContext,
    inputs: &parallax_evidence::bundle::BundleInputs,
) -> FieldResult<Vec<parallax_evidence::bundle::MetricWindow>> {
    use parallax_evidence::bundle::{BundleAnchor, MetricWindow};
    const PAD_NANOS: u128 = 5 * 60 * 1_000_000_000;
    let (from, to, step_seconds, run_scope, service) =
        if let BundleAnchor::Invocation { invocation, .. } = &inputs.anchor {
            let last_activity = inputs
                .trace_logs
                .iter()
                .map(|l| l.ts_nanos)
                .chain(
                    inputs
                        .trace_spans
                        .iter()
                        .map(|s| s.ts_nanos + s.duration_ns),
                )
                .max();
            let start = invocation.started_at_nanos;
            let end = invocation
                .ended_at_nanos
                .into_iter()
                .chain(last_activity)
                .max()
                .unwrap_or(start);
            (
                start.saturating_sub(5_000_000_000),
                end + 30_000_000_000,
                5u32,
                Some(invocation.invocation_id.clone()),
                None,
            )
        } else {
            let anchor_ts = inputs
                .events
                .first()
                .map(|e| e.ts_nanos)
                .or_else(|| inputs.trace_spans.first().map(|s| s.ts_nanos));
            let Some(anchor_ts) = anchor_ts else {
                return Ok(Vec::new());
            };
            let invocation_id = inputs
                .trace_spans
                .iter()
                .find_map(|s| s.invocation_id.clone());
            let service = inputs
                .trace_spans
                .first()
                .map(|s| s.service.clone())
                .or_else(|| inputs.events.first().map(|e| e.service.clone()));
            (
                anchor_ts.saturating_sub(PAD_NANOS),
                anchor_ts + PAD_NANOS,
                30u32,
                invocation_id,
                service,
            )
        };
    let scope = if run_scope.is_some() {
        "invocation"
    } else {
        "service"
    };
    let step = u128::from(step_seconds) * 1_000_000_000;
    let (cpu, memory, tokio_tasks) = tokio::try_join!(
        context.store.metric_series(
            semconv::BUNDLE_WINDOW_METRICS[0],
            service.as_deref(),
            run_scope.as_deref(),
            from..=to,
            step,
            MetricAgg::Avg,
        ),
        context.store.metric_series(
            semconv::BUNDLE_WINDOW_METRICS[1],
            service.as_deref(),
            run_scope.as_deref(),
            from..=to,
            step,
            MetricAgg::Avg,
        ),
        context.store.metric_series(
            semconv::BUNDLE_WINDOW_METRICS[2],
            service.as_deref(),
            run_scope.as_deref(),
            from..=to,
            step,
            MetricAgg::Avg,
        ),
    )
    .map_err(internal_field_err)?;
    let mut windows = Vec::new();
    for (metric, points) in [
        (semconv::BUNDLE_WINDOW_METRICS[0], cpu),
        (semconv::BUNDLE_WINDOW_METRICS[1], memory),
        (semconv::BUNDLE_WINDOW_METRICS[2], tokio_tasks),
    ] {
        if let Some(window) = MetricWindow::from_points(
            metric,
            scope,
            from,
            to,
            step_seconds,
            points.into_iter().map(|p| (p.ts_nanos, p.value)).collect(),
        ) {
            windows.push(window);
        }
    }
    Ok(windows)
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
    sort: Option<IssueSort>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> FieldResult<IssueList> {
    if let Some(status) = status.as_deref()
        && !matches!(status, "open" | "resolved")
    {
        return Err(field_err("status must be open or resolved"));
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

pub(crate) async fn issue(context: &ApiContext, fingerprint: String) -> FieldResult<Option<Issue>> {
    Ok(context
        .metadata
        .issue(&fingerprint)
        .await
        .map_err(internal_field_err)?
        .map(Issue::single))
}

pub(crate) async fn issue_trend(
    context: &ApiContext,
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
        .issue_trend(&fingerprint, since, step)
        .await
        .map_err(internal_field_err)?;
    Ok(points.into_iter().map(TrendPoint).collect())
}

fn validate_bundle_anchors(
    fingerprint: bool,
    invocation_id: bool,
    trace_id: bool,
) -> FieldResult<()> {
    if [fingerprint, invocation_id, trace_id]
        .into_iter()
        .filter(|present| *present)
        .count()
        == 1
    {
        Ok(())
    } else {
        Err(field_err(
            "bundle takes exactly one anchor: fingerprint, invocationId, or traceId",
        ))
    }
}

pub(crate) async fn bundle(
    context: &ApiContext,
    fingerprint: Option<String>,
    invocation_id: Option<String>,
    trace_id: Option<String>,
    max_tokens: Option<i32>,
) -> FieldResult<Option<BundleOut>> {
    use parallax_evidence::bundle::{BundleAnchor, BundleInputs};
    let trace_id = crate::validate_optional_trace_id(trace_id)?;
    let max_tokens = usize::try_from(max_tokens.unwrap_or(10_000).max(500)).unwrap_or(10_000);
    validate_bundle_anchors(
        fingerprint.is_some(),
        invocation_id.is_some(),
        trace_id.is_some(),
    )?;

    let mut inputs = if let Some(fingerprint) = fingerprint {
        let Some(issue) = context
            .metadata
            .issue(&fingerprint)
            .await
            .map_err(internal_field_err)?
        else {
            return Ok(None);
        };
        let events = context
            .store
            .error_events_by_fingerprint(&fingerprint, 0..=u128::MAX, 5)
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
        let fingerprints = unique_fingerprints(&events);
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
        let fingerprints = unique_fingerprints(&events);
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
        }
    };

    inputs.metric_windows = bundle_metric_windows(context, &inputs).await?;
    let bundle = parallax_evidence::bundle::assemble(inputs, max_tokens);
    let markdown = parallax_evidence::bundle::to_markdown(&bundle);
    let canonical_hash = bundle.canonical_hash.clone().unwrap_or_default();
    let json = serde_json::to_string_pretty(&bundle).map_err(internal_field_err)?;
    Ok(Some(BundleOut {
        json,
        markdown,
        canonical_hash,
    }))
}

pub(crate) async fn issue_set_status(
    context: &ApiContext,
    fingerprint: String,
    status: String,
) -> FieldResult<Issue> {
    if !matches!(status.as_str(), "open" | "resolved") {
        return Err(field_err("status must be open or resolved"));
    }
    context
        .metadata
        .set_issue_status(&fingerprint, &status)
        .await
        .map_err(internal_field_err)?;
    context
        .metadata
        .issue(&fingerprint)
        .await
        .map_err(internal_field_err)?
        .map(Issue::single)
        .ok_or_else(|| field_err(format!("issue {fingerprint} not found")))
}

#[cfg(test)]
mod tests;
