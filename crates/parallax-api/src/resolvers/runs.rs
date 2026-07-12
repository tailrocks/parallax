#![expect(clippy::excessive_nesting, reason = "measured run resolver flow")]

//! GraphQL runs domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::{HashMap, HashSet};

use crate::{
    ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string, retained_recent_range, saturate_i32,
};

use crate::resolvers::issues::Issue;

pub(crate) struct ObservedRun(pub(crate) parallax_storage::adapter::ObservedRun);

#[graphql_object(context = ApiContext)]
impl ObservedRun {
    fn run_id(&self) -> &str {
        &self.0.run_id
    }
    fn service(&self) -> &str {
        &self.0.service
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

pub(crate) struct Run {
    record: model::RunRecord,
    /// Trace ids + error events of this run, fetched once however many of
    /// the derived fields a query selects. Prefetched on list paths.
    stats: tokio::sync::OnceCell<RunStats>,
}

struct RunStats {
    trace_ids: Vec<String>,
    events: Vec<model::ErrorEventRow>,
}

impl Run {
    fn new(record: model::RunRecord) -> Self {
        Self {
            record,
            stats: tokio::sync::OnceCell::new(),
        }
    }

    fn with_stats(record: model::RunRecord, stats: RunStats) -> Self {
        Self {
            record,
            stats: stats.into(),
        }
    }

    async fn stats(&self, context: &ApiContext) -> FieldResult<&RunStats> {
        self.stats
            .get_or_try_init(|| async {
                let spans = context
                    .store
                    .spans_by_run(&self.record.run_id, MAX_ROWS, retained_recent_range())
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
                    .error_events_by_traces(&trace_ids, MAX_ROWS)
                    .await
                    .map_err(field_err)?;
                Ok(RunStats { trace_ids, events })
            })
            .await
    }
}

fn run_stats_from_spans(
    spans: &[model::SpanRow],
    events_by_trace: &HashMap<String, Vec<model::ErrorEventRow>>,
) -> RunStats {
    let mut trace_ids: Vec<String> = Vec::new();
    let mut seen_trace_ids = HashSet::new();
    for span in spans {
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
    RunStats { trace_ids, events }
}

#[graphql_object(context = ApiContext)]
impl Run {
    fn run_id(&self) -> &str {
        &self.record.run_id
    }
    fn command(&self) -> Option<&str> {
        self.record.command.as_deref()
    }
    fn started_at_nanos(&self) -> String {
        nanos_string(self.record.started_at_nanos)
    }
    fn ended_at_nanos(&self) -> Option<String> {
        self.record.ended_at_nanos.map(nanos_string)
    }
    fn exit_code(&self) -> Option<i32> {
        self.record.exit_code
    }
    /// running | finished | external (auto-registered from telemetry).
    fn status(&self) -> &str {
        &self.record.status
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
            .map_err(field_err)?;
        Ok(Issue::from_rows(issues))
    }
}

pub(crate) async fn run(context: &ApiContext, run_id: String) -> FieldResult<Option<Run>> {
    Ok(context
        .metadata
        .run(&run_id)
        .await
        .map_err(field_err)?
        .map(Run::new))
}

pub(crate) async fn observed_runs(
    context: &ApiContext,
    limit: Option<i32>,
) -> FieldResult<Vec<ObservedRun>> {
    let runs = context
        .store
        .observed_runs(clamp_limit(limit, 50), retained_recent_range())
        .await
        .map_err(field_err)?;
    Ok(runs.into_iter().map(ObservedRun).collect())
}

pub(crate) async fn runs(context: &ApiContext, limit: Option<i32>) -> FieldResult<Vec<Run>> {
    let runs = context
        .metadata
        .runs(clamp_limit(limit, 50))
        .await
        .map_err(field_err)?;
    if runs.is_empty() {
        return Ok(Vec::new());
    }
    let run_ids: Vec<String> = runs.iter().map(|run| run.run_id.clone()).collect();
    let spans_by_run = context
        .store
        .spans_by_runs(&run_ids, MAX_ROWS)
        .await
        .map_err(field_err)?;
    let mut all_trace_ids: Vec<String> = Vec::new();
    let mut seen_trace_ids = HashSet::new();
    for spans in spans_by_run.values() {
        for span in spans {
            if seen_trace_ids.insert(span.trace_id.clone()) {
                all_trace_ids.push(span.trace_id.clone());
            }
        }
    }
    let event_limit = MAX_ROWS.saturating_mul(run_ids.len().max(1));
    let events = context
        .store
        .error_events_by_traces(&all_trace_ids, event_limit)
        .await
        .map_err(field_err)?;
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
            let spans = spans_by_run
                .get(&record.run_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let stats = run_stats_from_spans(spans, &events_by_trace);
            Run::with_stats(record, stats)
        })
        .collect())
}

pub(crate) async fn run_start(
    context: &ApiContext,
    run_id: String,
    command: Option<String>,
    started_at_nanos: String,
) -> FieldResult<bool> {
    let nanos: u128 = started_at_nanos
        .parse()
        .map_err(|_| field_err("invalid nanos"))?;
    context
        .metadata
        .start_run(&run_id, command.as_deref(), nanos)
        .await
        .map_err(field_err)?;
    Ok(true)
}

pub(crate) async fn run_finish(
    context: &ApiContext,
    run_id: String,
    ended_at_nanos: String,
    exit_code: i32,
) -> FieldResult<bool> {
    let nanos: u128 = ended_at_nanos
        .parse()
        .map_err(|_| field_err("invalid nanos"))?;
    context
        .metadata
        .finish_run(&run_id, nanos, exit_code)
        .await
        .map_err(field_err)?;
    Ok(true)
}

#[cfg(test)]
mod tests;
