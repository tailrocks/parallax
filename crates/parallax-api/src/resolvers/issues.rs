//! GraphQL issues domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::HashSet;
use std::sync::Arc;

use crate::{retained_recent_range, ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string, saturate_i32};

use parallax_core::semconv;
use parallax_storage::model::MetricAgg;

pub struct Issue(pub(crate) model::Issue);

#[graphql_object(context = ApiContext)]
impl Issue {
    fn fingerprint(&self) -> &str {
        &self.0.fingerprint
    }
    fn title(&self) -> &str {
        &self.0.title
    }
    fn error_type(&self) -> &str {
        &self.0.error_type
    }
    fn culprit(&self) -> Option<&str> {
        self.0.culprit.as_deref()
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn status(&self) -> &str {
        &self.0.status
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.0.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.last_seen_nanos)
    }
    fn event_count(&self) -> i32 {
        saturate_i32(self.0.event_count)
    }
    fn last_trace_id(&self) -> Option<&str> {
        self.0.last_trace_id.as_deref()
    }
    /// Bounded top-tag-values cache as JSON: `{key: {value: count}}`.
    fn tags(&self) -> &str {
        &self.0.tags
    }

    /// The last-24h occurrence sparkline (hourly buckets), oldest first.
    async fn trend(&self, context: &ApiContext) -> FieldResult<Vec<TrendPoint>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(field_err)?
            .as_nanos();
        let since = now.saturating_sub(24 * 3_600_000_000_000);
        let points = context
            .metadata
            .issue_trend(&self.0.fingerprint, since, 3600)
            .await
            .map_err(field_err)?;
        Ok(points.into_iter().map(TrendPoint).collect())
    }

    /// The most recent stored occurrence.
    async fn latest_event(&self, context: &ApiContext) -> FieldResult<Option<ErrorEvent>> {
        let events = context
            .store
            .error_events_by_fingerprint(&self.0.fingerprint, 0..=u128::MAX, 1)
            .await
            .map_err(field_err)?;
        Ok(events.into_iter().next().map(ErrorEvent))
    }

    /// Recent occurrences of this issue, newest first, optionally
    /// range-bounded (`fromNanos`/`toNanos`).
    async fn events(
        &self,
        context: &ApiContext,
        limit: Option<i32>,
        from_nanos: Option<String>,
        to_nanos: Option<String>,
    ) -> FieldResult<Vec<ErrorEvent>> {
        let from = match from_nanos {
            Some(s) => s.parse().map_err(|_| field_err("invalid fromNanos"))?,
            None => 0,
        };
        let to = match to_nanos {
            Some(s) => s.parse().map_err(|_| field_err("invalid toNanos"))?,
            None => u128::MAX,
        };
        let events = context
            .store
            .error_events_by_fingerprint(&self.0.fingerprint, from..=to, clamp_limit(limit, 50))
            .await
            .map_err(field_err)?;
        Ok(events.into_iter().map(ErrorEvent).collect())
    }
}

/// Page of issues plus the (scan-capped) total for pagination.
pub struct IssueList {
    items: Vec<model::Issue>,
    total: usize,
}

#[graphql_object(context = ApiContext)]
impl IssueList {
    fn items(&self) -> Vec<Issue> {
        self.items.iter().cloned().map(Issue).collect()
    }
    /// Matching issues before paging — exact up to the 1000-row scan window.
    fn total(&self) -> i32 {
        i32::try_from(self.total).unwrap_or(i32::MAX)
    }
}

/// How `issues` lists are ordered. TREND = last-24h occurrence sum.
#[derive(juniper::GraphQLEnum, Clone, Copy)]
pub enum IssueSort {
    LastSeen,
    FirstSeen,
    Events,
    Trend,
}

impl IssueSort {
    fn key(self) -> model::IssueSortKey {
        match self {
            Self::LastSeen => model::IssueSortKey::LastSeen,
            Self::FirstSeen => model::IssueSortKey::FirstSeen,
            Self::Events => model::IssueSortKey::Events,
            Self::Trend => model::IssueSortKey::Trend,
        }
    }
}

pub struct ErrorEvent(pub(crate) model::ErrorEventRow);

#[graphql_object(context = ApiContext)]
impl ErrorEvent {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn fingerprint(&self) -> &str {
        &self.0.fingerprint
    }
    fn error_type(&self) -> &str {
        &self.0.error_type
    }
    fn message(&self) -> &str {
        &self.0.message
    }
    fn stacktrace(&self) -> Option<&str> {
        self.0.stacktrace.as_deref()
    }
    fn source(&self) -> String {
        serde_json::to_string(&self.0.source)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string()
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
    }
}

pub struct TrendPoint(pub(crate) model::TrendPoint);

#[graphql_object(context = ApiContext)]
impl TrendPoint {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn count(&self) -> i32 {
        i32::try_from(self.0.count).unwrap_or(i32::MAX)
    }
}

pub struct BundleOut {
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
    inputs: &parallax_core::bundle::BundleInputs,
) -> FieldResult<Vec<parallax_core::bundle::MetricWindow>> {
    use parallax_core::bundle::{BundleAnchor, MetricWindow};
    const PAD_NANOS: u128 = 5 * 60 * 1_000_000_000;
    let (from, to, step_seconds, run_scope, service) = match &inputs.anchor {
        BundleAnchor::Run { run, .. } => {
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
            let start = run.started_at_nanos;
            let end = run
                .ended_at_nanos
                .into_iter()
                .chain(last_activity)
                .max()
                .unwrap_or(start);
            (
                start.saturating_sub(5_000_000_000),
                end + 30_000_000_000,
                5u32,
                Some(run.run_id.clone()),
                None,
            )
        }
        _ => {
            let anchor_ts = inputs
                .events
                .first()
                .map(|e| e.ts_nanos)
                .or_else(|| inputs.trace_spans.first().map(|s| s.ts_nanos));
            let Some(anchor_ts) = anchor_ts else {
                return Ok(Vec::new());
            };
            let run_id = inputs.trace_spans.iter().find_map(|s| s.run_id.clone());
            let service = inputs
                .trace_spans
                .first()
                .map(|s| s.service.clone())
                .or_else(|| inputs.events.first().map(|e| e.service.clone()));
            (
                anchor_ts.saturating_sub(PAD_NANOS),
                anchor_ts + PAD_NANOS,
                30u32,
                run_id,
                service,
            )
        }
    };
    let scope = if run_scope.is_some() {
        "run"
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
    .map_err(field_err)?;
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

#[allow(clippy::too_many_arguments)]
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
        .map_err(field_err)?;
    Ok(IssueList { items, total })
}

pub(crate) async fn issue(context: &ApiContext, fingerprint: String) -> FieldResult<Option<Issue>> {
    Ok(context
        .metadata
        .issue(&fingerprint)
        .await
        .map_err(field_err)?
        .map(Issue))
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
        .map_err(field_err)?
        .as_nanos();
    let since = now.saturating_sub(u128::from(hours) * 3_600_000_000_000);
    let points = context
        .metadata
        .issue_trend(&fingerprint, since, step)
        .await
        .map_err(field_err)?;
    Ok(points.into_iter().map(TrendPoint).collect())
}

pub(crate) async fn bundle(
    context: &ApiContext,
    fingerprint: Option<String>,
    run_id: Option<String>,
    trace_id: Option<String>,
    max_tokens: Option<i32>,
) -> FieldResult<Option<BundleOut>> {
    use parallax_core::bundle::{BundleAnchor, BundleInputs};
    let max_tokens = usize::try_from(max_tokens.unwrap_or(10_000).max(500)).unwrap_or(10_000);
    let anchors = [fingerprint.is_some(), run_id.is_some(), trace_id.is_some()];
    if anchors.iter().filter(|present| **present).count() != 1 {
        return Err(field_err(
            "bundle takes exactly one anchor: fingerprint, runId, or traceId",
        ));
    }

    let mut inputs = if let Some(fingerprint) = fingerprint {
        let Some(issue) = context
            .metadata
            .issue(&fingerprint)
            .await
            .map_err(field_err)?
        else {
            return Ok(None);
        };
        let events = context
            .store
            .error_events_by_fingerprint(&fingerprint, 0..=u128::MAX, 5)
            .await
            .map_err(field_err)?;
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
    } else if let Some(run_id) = run_id {
        let Some(run) = context.metadata.run(&run_id).await.map_err(field_err)? else {
            return Ok(None);
        };
        let spans = context
            .store
            .spans_by_run(&run_id, MAX_ROWS, retained_recent_range())
            .await
            .map_err(field_err)?;
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
            .map_err(field_err)?;
        let mut fingerprints: Vec<String> = Vec::new();
        let mut seen_fingerprints = HashSet::new();
        for event in &events {
            let fingerprint = event.fingerprint.clone();
            if seen_fingerprints.insert(fingerprint.clone()) {
                fingerprints.push(fingerprint);
            }
        }
        let issues = context
            .metadata
            .issues_by_fingerprints(&fingerprints)
            .await
            .map_err(field_err)?;
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
            .logs_by_run(&run_id, 200)
            .await
            .map_err(field_err)?;
        BundleInputs {
            anchor: BundleAnchor::Run {
                run: Box::new(run),
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
            .map_err(field_err)?;
        let mut fingerprints: Vec<String> = Vec::new();
        let mut seen_fingerprints = HashSet::new();
        for event in &events {
            let fingerprint = event.fingerprint.clone();
            if seen_fingerprints.insert(fingerprint.clone()) {
                fingerprints.push(fingerprint);
            }
        }
        let issues = context
            .metadata
            .issues_by_fingerprints(&fingerprints)
            .await
            .map_err(field_err)?;
        BundleInputs {
            anchor: BundleAnchor::Trace { trace_id, issues },
            events,
            trace_spans: Arc::unwrap_or_clone(trace_spans),
            trace_logs: Arc::unwrap_or_clone(trace_logs),
            metric_windows: Vec::new(),
        }
    };

    inputs.metric_windows = bundle_metric_windows(context, &inputs).await?;
    let bundle = parallax_core::bundle::assemble(inputs, max_tokens);
    let markdown = parallax_core::bundle::to_markdown(&bundle);
    let canonical_hash = bundle.canonical_hash.clone().unwrap_or_default();
    let json = serde_json::to_string_pretty(&bundle).map_err(field_err)?;
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
        .map_err(field_err)?;
    context
        .metadata
        .issue(&fingerprint)
        .await
        .map_err(field_err)?
        .map(Issue)
        .ok_or_else(|| field_err(format!("issue {fingerprint} not found")))
}
