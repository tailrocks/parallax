#![expect(
    clippy::excessive_nesting,
    reason = "measured invocation resolver flow"
)]

//! GraphQL CLI-invocation domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::{HashMap, HashSet};

use crate::{
    ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string, retained_recent_range, saturate_i32,
};

use crate::resolvers::issues::Issue;

pub(crate) struct ObservedInvocation(pub(crate) parallax_storage::adapter::ObservedInvocation);

#[graphql_object(context = ApiContext)]
impl ObservedInvocation {
    fn invocation_id(&self) -> &str {
        &self.0.invocation_id
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    /// Latest `cli.command.name` seen on this invocation's root spans.
    fn last_command(&self) -> Option<&str> {
        self.0.last_command.as_deref()
    }
    /// `app.mode` seen on this invocation's root spans.
    fn app_mode(&self) -> Option<&str> {
        self.0.app_mode.as_deref()
    }
    fn first_nanos(&self) -> String {
        nanos_string(self.0.first_nanos)
    }
    fn last_nanos(&self) -> String {
        nanos_string(self.0.last_nanos)
    }
    fn span_count(&self) -> i32 {
        i32::try_from(self.0.span_count).unwrap_or(i32::MAX)
    }
    fn log_count(&self) -> i32 {
        i32::try_from(self.0.log_count).unwrap_or(i32::MAX)
    }
}

pub(crate) struct Invocation {
    record: model::InvocationRecord,
    /// Trace ids + error events of this run, fetched once however many of
    /// the derived fields a query selects. Prefetched on list paths.
    stats: tokio::sync::OnceCell<InvocationStats>,
}

/// One error occurrence surfaced on the invocation hub (journey beats).
#[derive(juniper::GraphQLObject)]
#[graphql(context = ApiContext)]
struct InvocationErrorEvent {
    ts_nanos: String,
    title: String,
    fingerprint: String,
    trace_id: Option<String>,
}

struct InvocationStats {
    trace_ids: Vec<String>,
    events: Vec<model::ErrorEventRow>,
    last_span_nanos: u128,
    /// End + outcome of the latest completed root `cli.command` span, for
    /// external invocations whose lifecycle was never wrapper-registered.
    command_completion: Option<(u128, String)>,
}

/// An unfinished invocation with no signal newer than this is `stale`.
const STALE_AFTER_NANOS: u128 = 5 * 60 * 1_000_000_000;

impl Invocation {
    fn new(record: model::InvocationRecord) -> Self {
        Self {
            record,
            stats: tokio::sync::OnceCell::new(),
        }
    }

    fn with_stats(record: model::InvocationRecord, stats: InvocationStats) -> Self {
        Self {
            record,
            stats: stats.into(),
        }
    }

    async fn stats(&self, context: &ApiContext) -> FieldResult<&InvocationStats> {
        self.stats
            .get_or_try_init(|| async {
                let spans = context
                    .store
                    .spans_by_invocation(
                        &self.record.invocation_id,
                        MAX_ROWS,
                        retained_recent_range(),
                    )
                    .await
                    .map_err(crate::internal_field_err)?;
                let mut trace_ids: Vec<String> = Vec::new();
                let mut seen_trace_ids = HashSet::new();
                let mut last_span_nanos = 0;
                for span in &spans {
                    last_span_nanos = last_span_nanos.max(span.ts_nanos);
                    let trace_id = span.trace_id.clone();
                    if seen_trace_ids.insert(trace_id.clone()) {
                        trace_ids.push(trace_id);
                    }
                }
                let command_completion = command_completion(&spans);
                let events = context
                    .store
                    .error_events_by_traces(&trace_ids, MAX_ROWS)
                    .await
                    .map_err(crate::internal_field_err)?;
                Ok(InvocationStats {
                    trace_ids,
                    events,
                    last_span_nanos,
                    command_completion,
                })
            })
            .await
    }
}

fn invocation_stats_from_spans(
    spans: &[model::SpanRow],
    events_by_trace: &HashMap<String, Vec<model::ErrorEventRow>>,
) -> InvocationStats {
    let mut trace_ids: Vec<String> = Vec::new();
    let mut seen_trace_ids = HashSet::new();
    let mut last_span_nanos = 0;
    for span in spans {
        last_span_nanos = last_span_nanos.max(span.ts_nanos);
        let trace_id = span.trace_id.clone();
        if seen_trace_ids.insert(trace_id.clone()) {
            trace_ids.push(trace_id);
        }
    }
    let mut events: Vec<model::ErrorEventRow> = Vec::new();
    for trace_id in &trace_ids {
        if let Some(trace_events) = events_by_trace.get(trace_id) {
            events.extend(trace_events.iter().cloned());
        }
    }
    events.sort_by_key(|event| std::cmp::Reverse(event.ts_nanos));
    events.truncate(MAX_ROWS);
    InvocationStats {
        trace_ids,
        events,
        last_span_nanos,
        command_completion: command_completion(spans),
    }
}

/// Latest completed root `cli.command` span carrying an `outcome` attribute:
/// the observed end of an external (non-wrapper-registered) invocation. The
/// outcome attribute is recorded at span completion, so its presence means
/// the command finished.
fn command_completion(spans: &[model::SpanRow]) -> Option<(u128, String)> {
    // A daemon's lifecycle is not command-derived: its capsule children
    // complete root command spans while the daemon keeps running, so any
    // daemon-mode signal disables the derivation (wrapper registration or
    // staleness closes daemons).
    let daemon = spans.iter().any(|span| {
        span.attributes
            .get(parallax_analysis::semconv::APP_MODE)
            .and_then(|value| value.as_str())
            == Some("daemon")
    });
    if daemon {
        return None;
    }
    spans
        .iter()
        .filter(|span| {
            span.name == parallax_analysis::semconv::CLI_COMMAND_SPAN_NAME
                && span.parent_span_id.as_deref().is_none_or(str::is_empty)
        })
        .filter_map(|span| {
            // A capsule child is wrapped INSIDE another invocation; its
            // completion never closes the surrounding invocation.
            if span
                .attributes
                .get(parallax_analysis::semconv::APP_MODE)
                .and_then(|value| value.as_str())
                == Some("capsule")
            {
                return None;
            }
            let outcome = span
                .attributes
                .get(parallax_analysis::semconv::OUTCOME)
                .and_then(|value| value.as_str())?;
            Some((span.ts_nanos + span.duration_ns, outcome.to_string()))
        })
        .max_by_key(|(end, _)| *end)
}

#[graphql_object(context = ApiContext)]
impl Invocation {
    fn invocation_id(&self) -> &str {
        &self.record.invocation_id
    }
    fn command(&self) -> Option<&str> {
        self.record.command.as_deref()
    }
    fn started_at_nanos(&self) -> String {
        nanos_string(self.record.started_at_nanos)
    }
    /// Wrapper-registered end, else the observed end of the latest completed
    /// root `cli.command` span (external invocations).
    async fn ended_at_nanos(&self, context: &ApiContext) -> FieldResult<Option<String>> {
        if let Some(ended) = self.record.ended_at_nanos {
            return Ok(Some(nanos_string(ended)));
        }
        Ok(self
            .stats(context)
            .await?
            .command_completion
            .as_ref()
            .map(|(end, _)| nanos_string(*end)))
    }
    fn exit_code(&self) -> Option<i32> {
        self.record.exit_code
    }
    /// cli (wrapper-registered) | external (auto-registered from telemetry).
    fn registration(&self) -> &str {
        if self.record.status == "external" {
            "external"
        } else {
            "cli"
        }
    }
    /// one_shot | interactive | daemon | capsule, when registered.
    fn app_mode(&self) -> Option<&str> {
        self.record.app_mode.as_deref()
    }
    /// Bounded result taxonomy (success | failure | error | timeout | skip |
    /// cancellation): wrapper-registered, else observed from the root
    /// `cli.command` span's outcome attribute.
    async fn outcome(&self, context: &ApiContext) -> FieldResult<Option<String>> {
        if let Some(outcome) = self.record.outcome.as_deref() {
            return Ok(Some(outcome.to_string()));
        }
        Ok(self
            .stats(context)
            .await?
            .command_completion
            .as_ref()
            .map(|(_, outcome)| outcome.clone()))
    }
    /// Derived lifecycle: running | finished | failed | stale. `failed` is a
    /// finished invocation with a non-zero exit code; `stale` is an unfinished
    /// invocation with no signal newer than five minutes.
    async fn status(&self, context: &ApiContext) -> FieldResult<String> {
        if self.record.ended_at_nanos.is_some() {
            return Ok(if self.record.exit_code.unwrap_or(0) != 0 {
                "failed".to_string()
            } else {
                "finished".to_string()
            });
        }
        // External invocations end when their observed root command span
        // completes (its outcome attribute is recorded at completion).
        if let Some((_, outcome)) = &self.stats(context).await?.command_completion {
            return Ok(
                if matches!(outcome.as_str(), "failure" | "error" | "timeout") {
                    "failed".to_string()
                } else {
                    "finished".to_string()
                },
            );
        }
        let now_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let stale_floor = now_nanos.saturating_sub(STALE_AFTER_NANOS);
        if self.record.started_at_nanos >= stale_floor {
            return Ok("running".to_string());
        }
        let last_signal = self
            .stats(context)
            .await?
            .last_span_nanos
            .max(self.record.started_at_nanos);
        Ok(if last_signal >= stale_floor {
            "running".to_string()
        } else {
            "stale".to_string()
        })
    }
    /// Error events derived inside this run's traces.
    async fn error_count(&self, context: &ApiContext) -> FieldResult<i32> {
        Ok(saturate_i32(self.stats(context).await?.events.len() as u64))
    }
    /// Distinct traces this run produced.
    async fn trace_count(&self, context: &ApiContext) -> FieldResult<i32> {
        Ok(saturate_i32(
            self.stats(context).await?.trace_ids.len() as u64
        ))
    }
    /// Interactive sessions observed inside this invocation.
    async fn session_count(&self, context: &ApiContext) -> FieldResult<i32> {
        let sessions = context
            .store
            .sessions_by_invocation(
                &self.record.invocation_id,
                retained_recent_range(),
                MAX_ROWS,
            )
            .await
            .map_err(crate::internal_field_err)?;
        Ok(saturate_i32(sessions.len() as u64))
    }
    /// Per-occurrence error events inside this run's traces, nanosecond
    /// timestamps intact — journey attribution needs each occurrence at its
    /// exact time, not the group's ms-truncated last-seen.
    async fn error_events(&self, context: &ApiContext) -> FieldResult<Vec<InvocationErrorEvent>> {
        Ok(self
            .stats(context)
            .await?
            .events
            .iter()
            .map(|event| InvocationErrorEvent {
                ts_nanos: nanos_string(event.ts_nanos),
                title: parallax_analysis::derive::issue_title(&event.error_type, &event.message),
                fingerprint: event.fingerprint.clone(),
                trace_id: (!event.trace_id.is_empty()).then(|| event.trace_id.clone()),
            })
            .collect())
    }
    /// Grouped issues whose events fell inside this run's traces.
    async fn issues(&self, context: &ApiContext) -> FieldResult<Vec<Issue>> {
        let stats = self.stats(context).await?;
        let mut fingerprints: Vec<String> = Vec::new();
        let mut seen_fingerprints = HashSet::new();
        for event in &stats.events {
            let fingerprint = event.fingerprint.clone();
            if seen_fingerprints.insert(fingerprint.clone()) {
                fingerprints.push(fingerprint);
            }
        }
        let issues = context
            .metadata
            .issues_by_fingerprints(&fingerprints)
            .await
            .map_err(crate::internal_field_err)?;
        Ok(Issue::from_rows(issues))
    }
}

pub(crate) async fn invocation(
    context: &ApiContext,
    invocation_id: String,
) -> FieldResult<Option<Invocation>> {
    Ok(context
        .metadata
        .invocation(&invocation_id)
        .await
        .map_err(crate::internal_field_err)?
        .map(Invocation::new))
}

pub(crate) async fn observed_invocations(
    context: &ApiContext,
    limit: Option<i32>,
) -> FieldResult<Vec<ObservedInvocation>> {
    let runs = context
        .store
        .observed_invocations(clamp_limit(limit, 50), retained_recent_range())
        .await
        .map_err(crate::internal_field_err)?;
    Ok(runs.into_iter().map(ObservedInvocation).collect())
}

pub(crate) async fn invocations(
    context: &ApiContext,
    limit: Option<i32>,
) -> FieldResult<Vec<Invocation>> {
    let runs = context
        .metadata
        .invocations(clamp_limit(limit, 50))
        .await
        .map_err(crate::internal_field_err)?;
    if runs.is_empty() {
        return Ok(Vec::new());
    }
    let invocation_ids: Vec<String> = runs.iter().map(|run| run.invocation_id.clone()).collect();
    let spans_by_invocation = context
        .store
        .spans_by_invocations(&invocation_ids, MAX_ROWS)
        .await
        .map_err(crate::internal_field_err)?;
    let mut all_trace_ids: Vec<String> = Vec::new();
    let mut seen_trace_ids = HashSet::new();
    for spans in spans_by_invocation.values() {
        for span in spans {
            if seen_trace_ids.insert(span.trace_id.clone()) {
                all_trace_ids.push(span.trace_id.clone());
            }
        }
    }
    let event_limit = MAX_ROWS.saturating_mul(invocation_ids.len().max(1));
    let events = context
        .store
        .error_events_by_traces(&all_trace_ids, event_limit)
        .await
        .map_err(crate::internal_field_err)?;
    let mut events_by_trace: HashMap<String, Vec<model::ErrorEventRow>> = HashMap::new();
    for event in events {
        events_by_trace
            .entry(event.trace_id.clone())
            .or_default()
            .push(event);
    }
    Ok(runs
        .into_iter()
        .map(|record| {
            let spans = spans_by_invocation
                .get(&record.invocation_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let stats = invocation_stats_from_spans(spans, &events_by_trace);
            Invocation::with_stats(record, stats)
        })
        .collect())
}

pub(crate) async fn invocation_start(
    context: &ApiContext,
    invocation_id: String,
    command: Option<String>,
    app_mode: Option<String>,
    started_at_nanos: String,
) -> FieldResult<bool> {
    let nanos: u128 = started_at_nanos
        .parse()
        .map_err(|_| field_err("invalid nanos"))?;
    context
        .metadata
        .start_invocation(
            &invocation_id,
            command.as_deref(),
            app_mode.as_deref(),
            nanos,
        )
        .await
        .map_err(crate::internal_field_err)?;
    Ok(true)
}

pub(crate) async fn invocation_finish(
    context: &ApiContext,
    invocation_id: String,
    ended_at_nanos: String,
    exit_code: i32,
    outcome: Option<String>,
) -> FieldResult<bool> {
    let nanos: u128 = ended_at_nanos
        .parse()
        .map_err(|_| field_err("invalid nanos"))?;
    context
        .metadata
        .finish_invocation(&invocation_id, nanos, exit_code, outcome.as_deref())
        .await
        .map_err(crate::internal_field_err)?;
    Ok(true)
}

#[cfg(test)]
mod tests;
