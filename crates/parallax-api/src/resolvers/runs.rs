//! GraphQL runs domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;
use std::collections::{HashMap, HashSet};

use crate::{retained_recent_range, ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string, saturate_i32};

use crate::resolvers::issues::Issue;

pub struct ObservedRun(pub(crate) parallax_storage::adapter::ObservedRun);

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

pub struct Run {
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
        let cell = tokio::sync::OnceCell::new();
        let _ = cell.set(stats);
        Self {
            record,
            stats: cell,
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
        Ok(issues.into_iter().map(Issue).collect())
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
mod tests {
    use super::*;
    use crate::resolvers::test_support::*;
    use crate::{RequestMemo, build_schema, execute};
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;

    use parallax_storage::model::{ErrorEventRow, ErrorSource};
    use std::sync::Arc;

    #[tokio::test]
    async fn memo_helper_truncates_and_reuses_spans_for_same_trace() {
        let store = Arc::new(MemoryStore::new());
        let mut spans = Vec::new();
        for i in 0..(MAX_ROWS + 25) {
            spans.push(span(
                "api",
                "big-trace",
                &format!("s{i}"),
                1_000_000_000 + i as u128,
                1_000,
            ));
        }
        store.push_spans(spans);
        let context = context_with_memory(store).await;
        let first = context.spans_for("big-trace").await.unwrap();
        let second = context.spans_for("big-trace").await.unwrap();
        assert_eq!(first.len(), MAX_ROWS);
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[tokio::test]
    async fn runs_list_stats_match_single_run() {
        let store = Arc::new(MemoryStore::new());
        let mut spans = Vec::new();
        for (run, traces) in [
            ("run-a", &["ta1", "ta2"][..]),
            ("run-b", &["tb1"][..]),
            ("run-c", &["tc1", "tc2", "tc3"][..]),
        ] {
            for (i, trace) in traces.iter().enumerate() {
                let mut row = span(
                    "api",
                    trace,
                    &format!("{run}-s{i}"),
                    1_000_000_000 + i as u128,
                    5_000,
                );
                row.run_id = Some(run.into());
                spans.push(row);
            }
        }
        store.push_spans(spans);
        store
            .write_error_events(vec![
                ErrorEventRow {
                    ts_nanos: 2_000_000_000,
                    service: "api".into(),
                    fingerprint: "fp-a".into(),
                    error_type: "Error".into(),
                    message: "boom-a".into(),
                    stacktrace: None,
                    source: ErrorSource::SpanStatus,
                    trace_id: "ta1".into(),
                    span_id: "run-a-s0".into(),
                    attributes: serde_json::Value::Null,
                },
                ErrorEventRow {
                    ts_nanos: 2_100_000_000,
                    service: "api".into(),
                    fingerprint: "fp-c".into(),
                    error_type: "Error".into(),
                    message: "boom-c".into(),
                    stacktrace: None,
                    source: ErrorSource::SpanStatus,
                    trace_id: "tc2".into(),
                    span_id: "run-c-s1".into(),
                    attributes: serde_json::Value::Null,
                },
            ])
            .await
            .unwrap();
        let context = context_with_memory(store).await;
        for (run_id, command) in [("run-a", "a"), ("run-b", "b"), ("run-c", "c")] {
            context
                .metadata
                .start_run(run_id, Some(command), 1_000_000_000)
                .await
                .unwrap();
        }
        let schema = build_schema();
        let list = juniper::http::GraphQLRequest::new(
            r#"{ runs { runId errorCount traceCount } }"#.into(),
            None,
            None,
        );
        let list_json = serde_json::to_value(execute(&schema, &context, list).await).unwrap();
        assert!(error_messages(&list_json).is_empty(), "{list_json}");
        let mut by_id = std::collections::BTreeMap::new();
        for row in list_json
            .pointer("/data/runs")
            .and_then(|v| v.as_array())
            .unwrap()
        {
            by_id.insert(
                row["runId"].as_str().unwrap().to_string(),
                (
                    row["errorCount"].as_i64().unwrap(),
                    row["traceCount"].as_i64().unwrap(),
                ),
            );
        }
        assert_eq!(by_id["run-a"], (1, 2));
        assert_eq!(by_id["run-b"], (0, 1));
        assert_eq!(by_id["run-c"], (1, 3));
        for run_id in ["run-a", "run-b", "run-c"] {
            let single_ctx = ApiContext {
                store: context.store.clone(),
                metadata: context.metadata.clone(),
                otlp_grpc_port: 4317,
                memo: RequestMemo::default(),
            };
            let q = juniper::http::GraphQLRequest::new(
                format!(r#"{{ run(runId: "{run_id}") {{ errorCount traceCount }} }}"#),
                None,
                None,
            );
            let single = serde_json::to_value(execute(&schema, &single_ctx, q).await).unwrap();
            assert_eq!(
                (
                    single
                        .pointer("/data/run/errorCount")
                        .and_then(|v| v.as_i64())
                        .unwrap(),
                    single
                        .pointer("/data/run/traceCount")
                        .and_then(|v| v.as_i64())
                        .unwrap(),
                ),
                by_id[run_id],
            );
        }
    }
}
