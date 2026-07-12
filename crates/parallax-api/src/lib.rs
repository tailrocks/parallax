#![cfg_attr(test, allow(clippy::unwrap_used, reason = "fixture assertions"))]
//! Parallax GraphQL API — the V1 surface from the implementation spec §8,
//! served by **Juniper** (operator instruction, 2026-06-12: the library he
//! uses in his own services). Every client (CLI, UI, agents) goes through
//! this schema; none touch storage directly.
//! Juniper notes (per the spec's dependency table): GraphQL `Int` is i32 —
//! counts saturate; nanosecond timestamps cross as strings; field names are
//! auto-camelCased; cost limits are resolver-level caps in V1.
//!
//! Resolver implementation modules are not a supported public contract:
//!
//! ```compile_fail
//! use parallax_api::resolvers::Trace;
//! ```

mod query_limits;
mod resolvers;
mod schema;

pub(crate) use resolvers::helpers::{
    parse_range, retained_recent_range, step_nanos, validate_investigation_name,
    validate_investigation_state, validate_metric_group_label, validate_metric_name,
    validate_saved_view_name, validate_saved_view_page,
};

use juniper::{FieldError, FieldResult, graphql_object};
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model;
use std::{collections::HashMap, sync::Arc};

use resolvers::{
    AgentSessionOut, AttributeCompareRow, BundleOut, CriticalPath, Dashboard, EvidenceGap,
    FieldKey, FieldStats, Investigation, Issue, IssueList, IssueSort, LogRecord, MetricExemplar,
    ObservedRun, Overview, Point, ReleaseWindow, Run, RuntimeMetric, SavedView, Series,
    ServiceCatalogRow, ServiceMap, ServiceOverview, ServiceSummary, SignalKind, SpanRed,
    SqlResultOut, StoryBeat, Trace, TraceDiff, TraceEventsOut, TraceList, TraceSort, TraceSummary,
    TrendPoint,
};

/// Request-scoped memo for the highest-fan-in anchored reads. Built fresh on
/// every GraphQL request so sibling fields share one store round-trip per
/// (trace_id) without caching across requests.
#[derive(Default)]
pub struct RequestMemo {
    spans: tokio::sync::Mutex<HashMap<String, Arc<Vec<model::SpanRow>>>>,
    logs: tokio::sync::Mutex<HashMap<String, Arc<Vec<model::LogRow>>>>,
}

/// Request context: shared storage adapters plus a per-request memo layer.
/// Constructed once per GraphQL request in the server handler — do not put a
/// long-lived `RequestMemo` behind a shared `Arc` (stale-data risk).
pub struct ApiContext {
    pub store: Arc<dyn TelemetryStore>,
    pub metadata: Arc<MetadataStore>,
    pub otlp_grpc_port: u16,
    pub memo: RequestMemo,
}

impl juniper::Context for ApiContext {}

impl ApiContext {
    pub async fn spans_for(&self, trace_id: &str) -> FieldResult<Arc<Vec<model::SpanRow>>> {
        {
            let cache = self.memo.spans.lock().await;
            if let Some(rows) = cache.get(trace_id) {
                return Ok(Arc::clone(rows));
            }
        }
        let mut rows = self
            .store
            .spans_by_trace(trace_id)
            .await
            .map_err(field_err)?;
        if rows.len() > MAX_ROWS {
            tracing::warn!(
                trace_id,
                fetched = rows.len(),
                cap = MAX_ROWS,
                "anchored spans truncated to MAX_ROWS"
            );
            rows.truncate(MAX_ROWS);
        }
        let rows = Arc::new(rows);
        let mut cache = self.memo.spans.lock().await;
        Ok(Arc::clone(
            cache
                .entry(trace_id.to_string())
                .or_insert_with(|| Arc::clone(&rows)),
        ))
    }

    pub async fn logs_for(&self, trace_id: &str) -> FieldResult<Arc<Vec<model::LogRow>>> {
        {
            let cache = self.memo.logs.lock().await;
            if let Some(rows) = cache.get(trace_id) {
                return Ok(Arc::clone(rows));
            }
        }
        let mut rows = self
            .store
            .logs_by_trace(trace_id)
            .await
            .map_err(field_err)?;
        if rows.len() > MAX_ROWS {
            tracing::warn!(
                trace_id,
                fetched = rows.len(),
                cap = MAX_ROWS,
                "anchored logs truncated to MAX_ROWS"
            );
            rows.truncate(MAX_ROWS);
        }
        let rows = Arc::new(rows);
        let mut cache = self.memo.logs.lock().await;
        Ok(Arc::clone(
            cache
                .entry(trace_id.to_string())
                .or_insert_with(|| Arc::clone(&rows)),
        ))
    }
}

pub(crate) fn field_err(e: impl std::fmt::Display) -> FieldError {
    FieldError::from(e.to_string())
}

pub(crate) fn nanos_string(nanos: u128) -> String {
    nanos.to_string()
}

pub(crate) fn saturate_i32(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Resolver-level row cap (the spec's Juniper note: cost limits are
/// resolver-level in V1; query-cost middleware is M5 hardening).
pub(crate) const MAX_ROWS: usize = 500;
/// Raw SQL is the power surface, so it gets a larger cap than typed resolvers
/// while still bounding GraphQL response size.
pub(crate) const SQL_MAX_ROWS: usize = 2_000;
pub(crate) const SAVED_VIEW_NAME_MAX: usize = 120;
pub(crate) const SAVED_VIEWS_PER_PAGE: usize = 100;
pub(crate) const INVESTIGATION_NAME_MAX: usize = 120;
pub(crate) const INVESTIGATION_PIN_CAP: usize = 100;
pub(crate) const INVESTIGATION_NOTES_MAX_BYTES: usize = 64 * 1024;

pub(crate) fn clamp_limit(limit: Option<i32>, default: usize) -> usize {
    limit
        .map_or(default, |l| usize::try_from(l.max(0)).unwrap_or(default))
        .min(MAX_ROWS)
}

pub struct Query;

#[rustfmt::skip]
#[graphql_object(context = ApiContext)]
impl Query {
    fn health() -> &'static str {
        "ok"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn otlp_grpc_port(context: &ApiContext) -> i32 {
        i32::from(context.otlp_grpc_port)
    }

    /// Whole-system counters for an inclusive time window. Counts are strings
    /// so large telemetry volumes never saturate GraphQL Int.
    async fn overview(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Overview> { crate::resolvers::services::overview(context, from_nanos, to_nanos).await }

    /// Per-signal count series for overview trend charts.
    async fn signal_count_series(context: &ApiContext, kind: SignalKind, service: Option<String>, from_nanos: String, to_nanos: String, step_seconds: Option<i32>,) -> FieldResult<Vec<Point>> { crate::resolvers::services::signal_count_series(context, kind, service, from_nanos, to_nanos, step_seconds).await }

    /// Service summary rows for the services index.
    async fn service_list(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<ServiceSummary>> { crate::resolvers::services::service_list(context, from_nanos, to_nanos).await }

    /// Per-version service release windows in the selected time range.
    async fn releases(context: &ApiContext, service: String, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<ReleaseWindow>> { crate::resolvers::services::releases(context, service, from_nanos, to_nanos).await }

    /// Resource-identity catalog rows for services in the selected window.
    async fn service_catalog(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<ServiceCatalogRow>> { crate::resolvers::services::service_catalog(context, from_nanos, to_nanos).await }

    /// Trace-path service graph over a bounded set of traces in the window.
    async fn service_map(context: &ApiContext, from_nanos: String, to_nanos: String, max_traces: Option<i32>,) -> FieldResult<ServiceMap> { crate::resolvers::services::service_map(context, from_nanos, to_nanos, max_traces).await }

    /// Trace-derived RED analytics; works even when a service emits no metrics.
    async fn service_red(context: &ApiContext, service: Option<String>, from_nanos: String, to_nanos: String, step_seconds: Option<i32>,) -> FieldResult<SpanRed> { crate::resolvers::services::service_red(context, service, from_nanos, to_nanos, step_seconds).await }

    /// Grouped errors: filtered, sorted, paged (spec §8 `issues`). The
    /// `query` argument substring-matches title, error type, and fingerprint;
    /// `fromNanos`/`toNanos` window on last-seen; `tagKey`+`tagValue` filter
    /// on the cached tags.
    #[allow(clippy::too_many_arguments)]
    async fn issues(context: &ApiContext, service: Option<String>, status: Option<String>, query: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, tag_key: Option<String>, tag_value: Option<String>, sort: Option<IssueSort>, limit: Option<i32>, offset: Option<i32>,) -> FieldResult<IssueList> { crate::resolvers::issues::issues(context, service, status, query, from_nanos, to_nanos, tag_key, tag_value, sort, limit, offset).await }

    async fn issue(context: &ApiContext, fingerprint: String) -> FieldResult<Option<Issue>> { crate::resolvers::issues::issue(context, fingerprint).await }

    /// Occurrence counts per bucket for one issue's sparkline, oldest
    /// first. Defaults: the last 24 hours in one-hour buckets.
    async fn issue_trend(context: &ApiContext, fingerprint: String, hours: Option<i32>, step_seconds: Option<i32>,) -> FieldResult<Vec<TrendPoint>> { crate::resolvers::issues::issue_trend(context, fingerprint, hours, step_seconds).await }

    /// Every span of one trace, start-time ascending (cross-service).
    async fn trace(context: &ApiContext, trace_id: String) -> FieldResult<Option<Trace>> { crate::resolvers::traces::trace(context, trace_id).await }

    /// Parsed span events across one trace, time ascending. `namePrefix`
    /// filters by event name (for example "rpc.message" or "exception").
    async fn trace_events(context: &ApiContext, trace_id: String, name_prefix: Option<String>, limit: Option<i32>,) -> FieldResult<TraceEventsOut> { crate::resolvers::traces::trace_events(context, trace_id, name_prefix, limit).await }

    /// Summaries for traces referenced by this trace's span links.
    async fn linked_traces(context: &ApiContext, trace_id: String,) -> FieldResult<Vec<TraceSummary>> { crate::resolvers::traces::linked_traces(context, trace_id).await }

    /// Critical-path hops that gate one trace's latency.
    async fn trace_critical_path(context: &ApiContext, trace_id: String,) -> FieldResult<CriticalPath> { crate::resolvers::traces::trace_critical_path(context, trace_id).await }

    /// Structural diff between two traces' span trees.
    async fn trace_compare(context: &ApiContext, trace_id_a: String, trace_id_b: String,) -> FieldResult<TraceDiff> { crate::resolvers::traces::trace_compare(context, trace_id_a, trace_id_b).await }

    /// Logs correlated to one trace, time ascending.
    async fn logs_by_trace(context: &ApiContext, trace_id: String) -> FieldResult<Vec<LogRecord>> { crate::resolvers::logs::logs_by_trace(context, trace_id).await }

    /// Traces produced by one run, summarized (root span + aggregates),
    /// newest first. Open one via `trace(traceId:)`.
    async fn traces_by_run(context: &ApiContext, run_id: String, limit: Option<i32>,) -> FieldResult<Vec<TraceSummary>> { crate::resolvers::traces::traces_by_run(context, run_id, limit).await }

    /// Logs produced by one run.
    async fn logs_by_run(context: &ApiContext, run_id: String, limit: Option<i32>,) -> FieldResult<Vec<LogRecord>> { crate::resolvers::logs::logs_by_run(context, run_id, limit).await }

    /// Agent-session projection for one run when gen_ai producer spans exist.
    async fn agent_session(context: &ApiContext, run_id: String,) -> FieldResult<Option<AgentSessionOut>> { crate::resolvers::story::agent_session(context, run_id).await }

    /// Deterministic story timeline for exactly one trace or run anchor.
    async fn story(context: &ApiContext, trace_id: Option<String>, run_id: Option<String>,) -> FieldResult<Vec<StoryBeat>> { crate::resolvers::story::story(context, trace_id, run_id).await }

    /// Missing-evidence detector for exactly one trace or run anchor.
    async fn evidence_gaps(context: &ApiContext, trace_id: Option<String>, run_id: Option<String>,) -> FieldResult<Vec<EvidenceGap>> { crate::resolvers::fields::evidence_gaps(context, trace_id, run_id).await }

    /// Span-attribute overrepresentation in selected vs baseline windows.
    #[allow(clippy::too_many_arguments)]
    async fn attribute_compare(context: &ApiContext, selected_from_nanos: String, selected_to_nanos: String, baseline_from_nanos: String, baseline_to_nanos: String, service: Option<String>, error_only: Option<bool>, keys: Option<Vec<String>>, top_n: Option<i32>,) -> FieldResult<Vec<AttributeCompareRow>> { crate::resolvers::fields::attribute_compare(context, selected_from_nanos, selected_to_nanos, baseline_from_nanos, baseline_to_nanos, service, error_only, keys, top_n).await }

    /// Scalar span/resource attribute keys in a bounded time window.
    async fn field_keys(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<FieldKey>> { crate::resolvers::fields::field_keys(context, from_nanos, to_nanos).await }

    /// Bounded coverage/cardinality/top-values stats for one field key.
    async fn field_stats(context: &ApiContext, key: String, from_nanos: String, to_nanos: String, service: Option<String>,) -> FieldResult<FieldStats> { crate::resolvers::fields::field_stats(context, key, from_nanos, to_nanos, service).await }

    /// Unified log browse (spec §8 `logs`): every filter optional, newest
    /// first. `query` substring-matches the body; trace/run scoping
    /// composes with the other filters.
    #[allow(clippy::too_many_arguments)]
    async fn logs(context: &ApiContext, trace_id: Option<String>, run_id: Option<String>, service: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, severity_min: Option<i32>, severity_max: Option<i32>, query: Option<String>, limit: Option<i32>,) -> FieldResult<Vec<LogRecord>> { crate::resolvers::logs::logs(context, trace_id, run_id, service, from_nanos, to_nanos, severity_min, severity_max, query, limit).await }

    /// Logs surrounding one anchor timestamp, ascending.
    async fn logs_around(context: &ApiContext, anchor_nanos: String, window_seconds: Option<i32>, service: Option<String>, trace_id: Option<String>, limit: Option<i32>,) -> FieldResult<Vec<LogRecord>> { crate::resolvers::logs::logs_around(context, anchor_nanos, window_seconds, service, trace_id, limit).await }

    /// Raw read-only SQL against the telemetry engine (GreptimeDB) — the
    /// engine's full query power over logs, traces, and metrics tables.
    /// SELECT-shaped single statements only.
    async fn sql(context: &ApiContext, query: String) -> FieldResult<SqlResultOut> { crate::resolvers::sql::sql(context, query).await }

    /// Log counts per time bucket under the same filters as `logs` — the
    /// Discover-style histogram above the log table.
    #[allow(clippy::too_many_arguments)]
    async fn log_count_series(context: &ApiContext, from_nanos: String, to_nanos: String, service: Option<String>, severity_min: Option<i32>, severity_max: Option<i32>, query: Option<String>, step_seconds: Option<i32>,) -> FieldResult<Vec<Point>> { crate::resolvers::logs::log_count_series(context, from_nanos, to_nanos, service, severity_min, severity_max, query, step_seconds).await }

    /// One run by id (wrapper-registered or auto-registered external).
    async fn run(context: &ApiContext, run_id: String) -> FieldResult<Option<Run>> { crate::resolvers::runs::run(context, run_id).await }

    /// One saved dashboard by id.
    async fn dashboard(context: &ApiContext, id: String) -> FieldResult<Option<Dashboard>> { crate::resolvers::dashboards::dashboard(context, id).await }

    /// One saved investigation by id.
    async fn investigation(context: &ApiContext, id: String) -> FieldResult<Option<Investigation>> { crate::resolvers::investigations::investigation(context, id).await }

    /// The predefined service overview (spec §8): CPU, memory, request rate,
    /// latency percentiles, error rate from well-known metric names, with
    /// graceful absence.
    async fn service_overview(context: &ApiContext, service: String, from_nanos: String, to_nanos: String, step_seconds: Option<i32>,) -> FieldResult<ServiceOverview> { crate::resolvers::services::service_overview(context, service, from_nanos, to_nanos, step_seconds).await }

    /// Run ids observed in telemetry (any tool exporting `parallax.run.id`
    /// — e.g. jackin'), newest activity first. Independent of wrapper
    /// registration: this is how external runs appear in the UI.
    async fn observed_runs(context: &ApiContext, limit: Option<i32>,) -> FieldResult<Vec<ObservedRun>> { crate::resolvers::runs::observed_runs(context, limit).await }

    /// Recent traces (root span + aggregates), newest first.
    async fn recent_traces(context: &ApiContext, limit: Option<i32>,) -> FieldResult<Vec<TraceSummary>> { crate::resolvers::traces::recent_traces(context, limit).await }

    /// Filtered trace browse (UI Traces page / `parallax traces`): every
    /// filter optional; filters hit the root span except `errorOnly`,
    /// which looks at the whole trace.
    #[allow(clippy::too_many_arguments)]
    async fn traces(context: &ApiContext, service: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, min_duration_ms: Option<f64>, max_duration_ms: Option<f64>, error_only: Option<bool>, query: Option<String>, limit: Option<i32>, offset: Option<i32>, sort: Option<TraceSort>,) -> FieldResult<Vec<TraceSummary>> { crate::resolvers::traces::traces(context, service, from_nanos, to_nanos, min_duration_ms, max_duration_ms, error_only, query, limit, offset, sort).await }

    /// Filtered, sorted, paged trace browse with total count for redesigned
    /// trace list clients.
    #[allow(clippy::too_many_arguments)]
    async fn traces_page(context: &ApiContext, service: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, min_duration_ms: Option<f64>, max_duration_ms: Option<f64>, error_only: Option<bool>, query: Option<String>, limit: Option<i32>, offset: Option<i32>, sort: Option<TraceSort>,) -> FieldResult<TraceList> { crate::resolvers::traces::traces_page(context, service, from_nanos, to_nanos, min_duration_ms, max_duration_ms, error_only, query, limit, offset, sort).await }

    /// The bounded, redacted, hypothesis-ranked evidence bundle — the agent
    /// handoff artifact assembling trace + logs + metric windows together.
    /// Exactly one anchor: `fingerprint` (issue), `runId`, or `traceId`
    /// (spec §8). Null when the anchor does not exist.
    async fn bundle(context: &ApiContext, fingerprint: Option<String>, run_id: Option<String>, trace_id: Option<String>, max_tokens: Option<i32>,) -> FieldResult<Option<BundleOut>> { crate::resolvers::issues::bundle(context, fingerprint, run_id, trace_id, max_tokens).await }

    /// Distinct metric names seen by the store (drives the dashboard
    /// builder), optionally prefix-filtered.
    async fn metric_names(context: &ApiContext, prefix: Option<String>,) -> FieldResult<Vec<String>> { crate::resolvers::metrics::metric_names(context, prefix).await }

    /// Groupable label/tag keys for one metric.
    async fn metric_labels(context: &ApiContext, name: String) -> FieldResult<Vec<String>> { crate::resolvers::metrics::metric_labels(context, name).await }

    /// Distinct values for one metric label inside a time window.
    async fn metric_label_values(context: &ApiContext, name: String, label: String, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<String>> { crate::resolvers::metrics::metric_label_values(context, name, label, from_nanos, to_nanos).await }

    /// Distinct service names (drives the service-overview selector).
    async fn services(context: &ApiContext) -> FieldResult<Vec<String>> { crate::resolvers::metrics::services(context).await }

    /// Runtime metric lanes, scoped to exactly one service or run.
    async fn runtime_snapshot(context: &ApiContext, service: Option<String>, run_id: Option<String>, from_nanos: String, to_nanos: String, step_seconds: i32,) -> FieldResult<Vec<RuntimeMetric>> { crate::resolvers::metrics::runtime_snapshot(context, service, run_id, from_nanos, to_nanos, step_seconds).await }

    /// Aggregated series for a point metric (gauge/sum); agg one of
    /// avg|min|max|sum|rate. With `groupBy` (an attribute key) one series
    /// per value; without it a single series with a null `groupValue`
    /// (spec §8 `metricSeries`). `runId` scopes to points whose resource
    /// carried `parallax.run.id` (run-anchored cross-analytics).
    #[allow(clippy::too_many_arguments)]
    async fn metric_series(context: &ApiContext, name: String, from_nanos: String, to_nanos: String, service: Option<String>, run_id: Option<String>, group_by: Option<String>, step_seconds: Option<i32>, agg: Option<String>,) -> FieldResult<Vec<Series>> { crate::resolvers::metrics::metric_series(context, name, from_nanos, to_nanos, service, run_id, group_by, step_seconds, agg).await }

    /// Approximate quantile series from a histogram metric (q in 0..=1).
    async fn histogram_quantile(context: &ApiContext, name: String, from_nanos: String, to_nanos: String, q: f64, service: Option<String>, step_seconds: Option<i32>,) -> FieldResult<Vec<Point>> { crate::resolvers::metrics::histogram_quantile(context, name, from_nanos, to_nanos, q, service, step_seconds).await }

    /// Trace-linked exemplars for one metric, newest first.
    async fn metric_exemplars(context: &ApiContext, name: String, from_nanos: String, to_nanos: String, service: Option<String>, limit: Option<i32>,) -> FieldResult<Vec<MetricExemplar>> { crate::resolvers::metrics::metric_exemplars(context, name, from_nanos, to_nanos, service, limit).await }

    /// Saved user dashboards, most recently updated first.
    async fn dashboards(context: &ApiContext) -> FieldResult<Vec<Dashboard>> { crate::resolvers::dashboards::dashboards(context).await }

    /// Saved investigations/cases, most recently updated first.
    async fn investigations(context: &ApiContext) -> FieldResult<Vec<Investigation>> { crate::resolvers::investigations::investigations(context).await }

    /// Named saved page states, most recently updated first.
    async fn saved_views(context: &ApiContext, page: Option<String>,) -> FieldResult<Vec<SavedView>> { crate::resolvers::investigations::saved_views(context, page).await }

    async fn runs(context: &ApiContext, limit: Option<i32>) -> FieldResult<Vec<Run>> { crate::resolvers::runs::runs(context, limit).await }

}

pub struct Mutation;

#[rustfmt::skip]
#[graphql_object(context = ApiContext)]
impl Mutation {
    /// Set an issue's workflow status (open | resolved); returns the updated
    /// issue (spec §8: `Issue!`).
    async fn issue_set_status(context: &ApiContext, fingerprint: String, status: String,) -> FieldResult<Issue> { crate::resolvers::issues::issue_set_status(context, fingerprint, status).await }

    /// Register a run (the CLI wrapper calls this before launching).
    async fn run_start(context: &ApiContext, run_id: String, command: Option<String>, started_at_nanos: String,) -> FieldResult<bool> { crate::resolvers::runs::run_start(context, run_id, command, started_at_nanos).await }

    /// Create or update a user dashboard; returns the saved dashboard
    /// (spec §8: `Dashboard!`).
    async fn dashboard_save(context: &ApiContext, name: String, layout: String, id: Option<String>,) -> FieldResult<Dashboard> { crate::resolvers::dashboards::dashboard_save(context, name, layout, id).await }

    /// Delete a user dashboard.
    async fn dashboard_delete(context: &ApiContext, id: String) -> FieldResult<bool> { crate::resolvers::dashboards::dashboard_delete(context, id).await }

    /// Create or update an investigation/case state.
    async fn investigation_save(context: &ApiContext, name: String, state: String, id: Option<String>,) -> FieldResult<Investigation> { crate::resolvers::investigations::investigation_save(context, name, state, id).await }

    /// Delete an investigation/case.
    async fn investigation_delete(context: &ApiContext, id: String) -> FieldResult<bool> { crate::resolvers::investigations::investigation_delete(context, id).await }

    /// Create or update a named saved page state.
    async fn saved_view_save(context: &ApiContext, name: String, page: String, state: String, id: Option<String>,) -> FieldResult<SavedView> { crate::resolvers::investigations::saved_view_save(context, name, page, state, id).await }

    /// Delete a named saved page state.
    async fn saved_view_delete(context: &ApiContext, id: String) -> FieldResult<bool> { crate::resolvers::investigations::saved_view_delete(context, id).await }

    /// Close a run with the wrapped command's exit code.
    async fn run_finish(context: &ApiContext, run_id: String, ended_at_nanos: String, exit_code: i32,) -> FieldResult<bool> { crate::resolvers::runs::run_finish(context, run_id, ended_at_nanos, exit_code).await }

}

pub use query_limits::check_query_limits;
pub use schema::{Schema, build_schema, execute};

#[cfg(test)]
mod tests;
