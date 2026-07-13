//! Parallax GraphQL API — the V1 surface from the implementation spec §8,
//! served by **Juniper** (operator instruction, 2026-06-12: the library he
//! uses in his own services). Every client (CLI, UI, agents) goes through
//! this schema; none touch storage directly.
//!
//! Juniper notes (per the spec's dependency table): GraphQL `Int` is i32 —
//! counts saturate; nanosecond timestamps cross as strings; field names are
//! auto-camelCased; cost limits are resolver-level caps in V1.
//!
//! Resolver implementation modules are not a supported public contract:
//!
//! ```compile_fail
//! use parallax_api::resolvers::Trace;
//! ```

mod api_errors;
mod query_limits;
mod resolvers;
mod schema;

pub(crate) use resolvers::helpers::{
    parse_range, retained_recent_range, step_nanos, validate_investigation_name,
    validate_investigation_state, validate_metric_group_label, validate_metric_name,
    validate_saved_view_name, validate_saved_view_page,
};

use juniper::{FieldError, FieldResult, graphql_object};
use parallax_storage::{adapter::TelemetryStore, metadata::MetadataStore, model};
use std::{collections::HashMap, sync::Arc};

use resolvers::{
    AgentSessionOut, AttributeCompareRow, BundleOut, CriticalPath, Dashboard, EvidenceGap,
    FieldKey, FieldStats, Investigation, Issue, IssueList, IssueSort, LogRecord, MetricExemplar,
    ObservedRun, Overview, Point, ReleaseWindow, Run, RuntimeMetric, SavedView, Series,
    ServiceCatalogRow, ServiceMap, ServiceOverview, ServiceSummary, SignalKind, SpanRed,
    SqlResultOut, StoryBeat, Trace, TraceDiff, TraceEventsOut, TraceList, TraceSort, TraceSummary,
    TrendPoint,
};

mod memo;
pub use memo::RequestMemo;

/// Request context: shared storage adapters plus a per-request memo layer.
/// Constructed once per GraphQL request in the server handler — do not put a
/// long-lived `RequestMemo` behind a shared `Arc` (stale-data risk).
#[expect(missing_debug_implementations, reason = "opaque storage handle")]
pub struct ApiContext {
    pub store: Arc<dyn TelemetryStore>,
    pub metadata: Arc<dyn MetadataStore>,
    pub otlp_grpc_port: u16,
    pub memo: RequestMemo,
}

impl juniper::Context for ApiContext {}

pub(crate) fn field_err(error: impl std::fmt::Display) -> FieldError {
    api_errors::invalid(error)
}

pub(crate) fn internal_field_err(error: impl std::error::Error + 'static) -> FieldError {
    api_errors::internal(error)
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

#[derive(Debug)]
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
    async fn overview(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Overview> { resolvers::services::overview(context, from_nanos, to_nanos).await }

    /// Per-signal count series for overview trend charts.
    async fn signal_count_series(context: &ApiContext, kind: SignalKind, service: Option<String>, from_nanos: String, to_nanos: String, step_seconds: Option<i32>,) -> FieldResult<Vec<Point>> { resolvers::services::signal_count_series(context, kind, service, from_nanos, to_nanos, step_seconds).await }

    /// Service summary rows for the services index.
    async fn service_list(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<ServiceSummary>> { resolvers::services::service_list(context, from_nanos, to_nanos).await }

    /// Per-version service release windows in the selected time range.
    async fn releases(context: &ApiContext, service: String, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<ReleaseWindow>> { resolvers::services::releases(context, service, from_nanos, to_nanos).await }

    /// Resource-identity catalog rows for services in the selected window.
    async fn service_catalog(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<ServiceCatalogRow>> { resolvers::services::service_catalog(context, from_nanos, to_nanos).await }

    /// Trace-path service graph over a bounded set of traces in the window.
    async fn service_map(context: &ApiContext, from_nanos: String, to_nanos: String, max_traces: Option<i32>,) -> FieldResult<ServiceMap> { resolvers::services::service_map(context, from_nanos, to_nanos, max_traces).await }

    /// Trace-derived RED analytics; works even when a service emits no metrics.
    async fn service_red(context: &ApiContext, service: Option<String>, from_nanos: String, to_nanos: String, step_seconds: Option<i32>,) -> FieldResult<SpanRed> { resolvers::services::service_red(context, service, from_nanos, to_nanos, step_seconds).await }

    /// Grouped errors: filtered, sorted, paged (spec §8 `issues`). The
    /// `query` argument substring-matches title, error type, and fingerprint;
    /// `fromNanos`/`toNanos` window on last-seen; `tagKey`+`tagValue` filter
    /// on the cached tags.
    #[expect(clippy::too_many_arguments, reason = "GraphQL issue filters are the public query contract")]
    async fn issues(context: &ApiContext, service: Option<String>, status: Option<String>, query: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, tag_key: Option<String>, tag_value: Option<String>, sort: Option<IssueSort>, limit: Option<i32>, offset: Option<i32>,) -> FieldResult<IssueList> { resolvers::issues::issues(context, service, status, query, from_nanos, to_nanos, tag_key, tag_value, sort, limit, offset).await }

    async fn issue(context: &ApiContext, fingerprint: String) -> FieldResult<Option<Issue>> { resolvers::issues::issue(context, fingerprint).await }

    /// Occurrence counts per bucket for one issue's sparkline, oldest
    /// first. Defaults: the last 24 hours in one-hour buckets.
    async fn issue_trend(context: &ApiContext, fingerprint: String, hours: Option<i32>, step_seconds: Option<i32>,) -> FieldResult<Vec<TrendPoint>> { resolvers::issues::issue_trend(context, fingerprint, hours, step_seconds).await }

    /// Every span of one trace, start-time ascending (cross-service).
    async fn trace(context: &ApiContext, trace_id: String) -> FieldResult<Option<Trace>> { resolvers::traces::trace(context, trace_id).await }

    /// Parsed span events across one trace, time ascending. `namePrefix`
    /// filters by event name (for example "rpc.message" or "exception").
    async fn trace_events(context: &ApiContext, trace_id: String, name_prefix: Option<String>, limit: Option<i32>,) -> FieldResult<TraceEventsOut> { resolvers::traces::trace_events(context, trace_id, name_prefix, limit).await }

    /// Summaries for traces referenced by this trace's span links.
    async fn linked_traces(context: &ApiContext, trace_id: String,) -> FieldResult<Vec<TraceSummary>> { resolvers::traces::linked_traces(context, trace_id).await }

    /// Critical-path hops that gate one trace's latency.
    async fn trace_critical_path(context: &ApiContext, trace_id: String,) -> FieldResult<CriticalPath> { resolvers::traces::trace_critical_path(context, trace_id).await }

    /// Structural diff between two traces' span trees.
    async fn trace_compare(context: &ApiContext, trace_id_a: String, trace_id_b: String,) -> FieldResult<TraceDiff> { resolvers::traces::trace_compare(context, trace_id_a, trace_id_b).await }

    /// Logs correlated to one trace, time ascending.
    async fn logs_by_trace(context: &ApiContext, trace_id: String) -> FieldResult<Vec<LogRecord>> { resolvers::logs::logs_by_trace(context, trace_id).await }

    /// Traces produced by one run, summarized (root span + aggregates),
    /// newest first. Open one via `trace(traceId:)`.
    async fn traces_by_run(context: &ApiContext, run_id: String, limit: Option<i32>,) -> FieldResult<Vec<TraceSummary>> { resolvers::traces::traces_by_run(context, run_id, limit).await }

    /// Logs produced by one run.
    async fn logs_by_run(context: &ApiContext, run_id: String, limit: Option<i32>,) -> FieldResult<Vec<LogRecord>> { resolvers::logs::logs_by_run(context, run_id, limit).await }

    /// Agent-session projection for one run when `gen_ai` producer spans exist.
    async fn agent_session(context: &ApiContext, run_id: String,) -> FieldResult<Option<AgentSessionOut>> { resolvers::story::agent_session(context, run_id).await }

    /// Deterministic story timeline for exactly one trace or run anchor.
    async fn story(context: &ApiContext, trace_id: Option<String>, run_id: Option<String>,) -> FieldResult<Vec<StoryBeat>> { resolvers::story::story(context, trace_id, run_id).await }

    /// Missing-evidence detector for exactly one trace or run anchor.
    async fn evidence_gaps(context: &ApiContext, trace_id: Option<String>, run_id: Option<String>,) -> FieldResult<Vec<EvidenceGap>> { resolvers::fields::evidence_gaps(context, trace_id, run_id).await }

    /// Span-attribute overrepresentation in selected vs baseline windows.
    #[expect(clippy::too_many_arguments, reason = "GraphQL field filters are the public query contract")]
    async fn attribute_compare(context: &ApiContext, selected_from_nanos: String, selected_to_nanos: String, baseline_from_nanos: String, baseline_to_nanos: String, service: Option<String>, error_only: Option<bool>, keys: Option<Vec<String>>, top_n: Option<i32>,) -> FieldResult<Vec<AttributeCompareRow>> { resolvers::fields::attribute_compare(context, selected_from_nanos, selected_to_nanos, baseline_from_nanos, baseline_to_nanos, service, error_only, keys, top_n).await }

    /// Scalar span/resource attribute keys in a bounded time window.
    async fn field_keys(context: &ApiContext, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<FieldKey>> { resolvers::fields::field_keys(context, from_nanos, to_nanos).await }

    /// Bounded coverage/cardinality/top-values stats for one field key.
    async fn field_stats(context: &ApiContext, key: String, from_nanos: String, to_nanos: String, service: Option<String>,) -> FieldResult<FieldStats> { resolvers::fields::field_stats(context, key, from_nanos, to_nanos, service).await }

    /// Unified log browse (spec §8 `logs`): every filter optional, newest
    /// first. `query` substring-matches the body; trace/run scoping
    /// composes with the other filters.
    #[expect(clippy::too_many_arguments, reason = "GraphQL log filters are the public query contract")]
    async fn logs(context: &ApiContext, trace_id: Option<String>, run_id: Option<String>, service: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, severity_min: Option<i32>, severity_max: Option<i32>, query: Option<String>, limit: Option<i32>,) -> FieldResult<Vec<LogRecord>> { resolvers::logs::logs(context, trace_id, run_id, service, from_nanos, to_nanos, severity_min, severity_max, query, limit).await }

    /// Logs surrounding one anchor timestamp, ascending.
    async fn logs_around(context: &ApiContext, anchor_nanos: String, window_seconds: Option<i32>, service: Option<String>, trace_id: Option<String>, limit: Option<i32>,) -> FieldResult<Vec<LogRecord>> { resolvers::logs::logs_around(context, anchor_nanos, window_seconds, service, trace_id, limit).await }

    /// Raw read-only SQL against the telemetry engine (`GreptimeDB`) — the
    /// engine's full query power over logs, traces, and metrics tables.
    /// SELECT-shaped single statements only.
    async fn sql(context: &ApiContext, query: String) -> FieldResult<SqlResultOut> { resolvers::sql::sql(context, query).await }

    /// Log counts per time bucket under the same filters as `logs` — the
    /// Discover-style histogram above the log table.
    #[expect(clippy::too_many_arguments, reason = "GraphQL log filters are the public query contract")]
    async fn log_count_series(context: &ApiContext, from_nanos: String, to_nanos: String, service: Option<String>, severity_min: Option<i32>, severity_max: Option<i32>, query: Option<String>, step_seconds: Option<i32>,) -> FieldResult<Vec<Point>> { resolvers::logs::log_count_series(context, from_nanos, to_nanos, service, severity_min, severity_max, query, step_seconds).await }

    /// One run by id (wrapper-registered or auto-registered external).
    async fn run(context: &ApiContext, run_id: String) -> FieldResult<Option<Run>> { resolvers::runs::run(context, run_id).await }

    /// One saved dashboard by id.
    async fn dashboard(context: &ApiContext, id: String) -> FieldResult<Option<Dashboard>> { resolvers::dashboards::dashboard(context, id).await }

    /// One saved investigation by id.
    async fn investigation(context: &ApiContext, id: String) -> FieldResult<Option<Investigation>> { resolvers::investigations::investigation(context, id).await }

    /// The predefined service overview (spec §8): CPU, memory, request rate,
    /// latency percentiles, error rate from well-known metric names, with
    /// graceful absence.
    async fn service_overview(context: &ApiContext, service: String, from_nanos: String, to_nanos: String, step_seconds: Option<i32>,) -> FieldResult<ServiceOverview> { resolvers::services::service_overview(context, service, from_nanos, to_nanos, step_seconds).await }

    /// Run ids observed in telemetry (any tool exporting `parallax.run.id`
    /// — e.g. jackin'), newest activity first. Independent of wrapper
    /// registration: this is how external runs appear in the UI.
    async fn observed_runs(context: &ApiContext, limit: Option<i32>,) -> FieldResult<Vec<ObservedRun>> { resolvers::runs::observed_runs(context, limit).await }

    /// Recent traces (root span + aggregates), newest first.
    async fn recent_traces(context: &ApiContext, limit: Option<i32>,) -> FieldResult<Vec<TraceSummary>> { resolvers::traces::recent_traces(context, limit).await }

    /// Filtered trace browse (UI Traces page / `parallax traces`): every
    /// filter optional; filters hit the root span except `errorOnly`,
    /// which looks at the whole trace.
    #[expect(clippy::too_many_arguments, reason = "GraphQL trace filters are the public query contract")]
    async fn traces(context: &ApiContext, service: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, min_duration_ms: Option<f64>, max_duration_ms: Option<f64>, error_only: Option<bool>, query: Option<String>, limit: Option<i32>, offset: Option<i32>, sort: Option<TraceSort>,) -> FieldResult<Vec<TraceSummary>> { resolvers::traces::traces(context, service, from_nanos, to_nanos, min_duration_ms, max_duration_ms, error_only, query, limit, offset, sort).await }

    /// Filtered, sorted, paged trace browse with total count for redesigned
    /// trace list clients.
    #[expect(clippy::too_many_arguments, reason = "GraphQL trace filters are the public query contract")]
    async fn traces_page(context: &ApiContext, service: Option<String>, from_nanos: Option<String>, to_nanos: Option<String>, min_duration_ms: Option<f64>, max_duration_ms: Option<f64>, error_only: Option<bool>, query: Option<String>, limit: Option<i32>, offset: Option<i32>, sort: Option<TraceSort>,) -> FieldResult<TraceList> { resolvers::traces::traces_page(context, service, from_nanos, to_nanos, min_duration_ms, max_duration_ms, error_only, query, limit, offset, sort).await }

    /// The bounded, redacted, hypothesis-ranked evidence bundle — the agent
    /// handoff artifact assembling trace + logs + metric windows together.
    /// Exactly one anchor: `fingerprint` (issue), `runId`, or `traceId`
    /// (spec §8). Null when the anchor does not exist.
    async fn bundle(context: &ApiContext, fingerprint: Option<String>, run_id: Option<String>, trace_id: Option<String>, max_tokens: Option<i32>,) -> FieldResult<Option<BundleOut>> { resolvers::issues::bundle(context, fingerprint, run_id, trace_id, max_tokens).await }

    /// Distinct metric names seen by the store (drives the dashboard
    /// builder), optionally prefix-filtered.
    async fn metric_names(context: &ApiContext, prefix: Option<String>,) -> FieldResult<Vec<String>> { resolvers::metrics::metric_names(context, prefix).await }

    /// Groupable label/tag keys for one metric.
    async fn metric_labels(context: &ApiContext, name: String) -> FieldResult<Vec<String>> { resolvers::metrics::metric_labels(context, name).await }

    /// Distinct values for one metric label inside a time window.
    async fn metric_label_values(context: &ApiContext, name: String, label: String, from_nanos: String, to_nanos: String,) -> FieldResult<Vec<String>> { resolvers::metrics::metric_label_values(context, name, label, from_nanos, to_nanos).await }

    /// Distinct service names (drives the service-overview selector).
    async fn services(context: &ApiContext) -> FieldResult<Vec<String>> { resolvers::metrics::services(context).await }

    /// Runtime metric lanes, scoped to exactly one service or run.
    async fn runtime_snapshot(context: &ApiContext, service: Option<String>, run_id: Option<String>, from_nanos: String, to_nanos: String, step_seconds: i32,) -> FieldResult<Vec<RuntimeMetric>> { resolvers::metrics::runtime_snapshot(context, service, run_id, from_nanos, to_nanos, step_seconds).await }

    /// Aggregated series for a point metric (gauge/sum); agg one of
    /// avg|min|max|sum|rate. With `groupBy` (an attribute key) one series
    /// per value; without it a single series with a null `groupValue`
    /// (spec §8 `metricSeries`). `runId` scopes to points whose resource
    /// carried `parallax.run.id` (run-anchored cross-analytics).
    #[expect(clippy::too_many_arguments, reason = "GraphQL metric filters are the public query contract")]
    async fn metric_series(context: &ApiContext, name: String, from_nanos: String, to_nanos: String, service: Option<String>, run_id: Option<String>, group_by: Option<String>, step_seconds: Option<i32>, agg: Option<String>,) -> FieldResult<Vec<Series>> { resolvers::metrics::metric_series(context, name, from_nanos, to_nanos, service, run_id, group_by, step_seconds, agg).await }

    /// Approximate quantile series from a histogram metric (q in 0..=1).
    #[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
    async fn histogram_quantile(context: &ApiContext, name: String, from_nanos: String, to_nanos: String, q: f64, service: Option<String>, step_seconds: Option<i32>,) -> FieldResult<Vec<Point>> { resolvers::metrics::histogram_quantile(context, name, from_nanos, to_nanos, q, service, step_seconds).await }

    /// Trace-linked exemplars for one metric, newest first.
    async fn metric_exemplars(context: &ApiContext, name: String, from_nanos: String, to_nanos: String, service: Option<String>, limit: Option<i32>,) -> FieldResult<Vec<MetricExemplar>> { resolvers::metrics::metric_exemplars(context, name, from_nanos, to_nanos, service, limit).await }

    /// Saved user dashboards, most recently updated first.
    async fn dashboards(context: &ApiContext) -> FieldResult<Vec<Dashboard>> { resolvers::dashboards::dashboards(context).await }

    /// Saved investigations/cases, most recently updated first.
    async fn investigations(context: &ApiContext) -> FieldResult<Vec<Investigation>> { resolvers::investigations::investigations(context).await }

    /// Named saved page states, most recently updated first.
    async fn saved_views(context: &ApiContext, page: Option<String>,) -> FieldResult<Vec<SavedView>> { resolvers::investigations::saved_views(context, page).await }

    async fn runs(context: &ApiContext, limit: Option<i32>) -> FieldResult<Vec<Run>> { resolvers::runs::runs(context, limit).await }

}

#[derive(Debug)]
pub struct Mutation;

#[rustfmt::skip]
#[graphql_object(context = ApiContext)]
impl Mutation {
    /// Set an issue's workflow status (open | resolved); returns the updated
    /// issue (spec §8: `Issue!`).
    async fn issue_set_status(context: &ApiContext, fingerprint: String, status: String,) -> FieldResult<Issue> { resolvers::issues::issue_set_status(context, fingerprint, status).await }

    /// Register a run (the CLI wrapper calls this before launching).
    async fn run_start(context: &ApiContext, run_id: String, command: Option<String>, started_at_nanos: String,) -> FieldResult<bool> { resolvers::runs::run_start(context, run_id, command, started_at_nanos).await }

    /// Create or update a user dashboard; returns the saved dashboard
    /// (spec §8: `Dashboard!`).
    async fn dashboard_save(context: &ApiContext, name: String, layout: String, id: Option<String>,) -> FieldResult<Dashboard> { resolvers::dashboards::dashboard_save(context, name, layout, id).await }

    /// Delete a user dashboard.
    async fn dashboard_delete(context: &ApiContext, id: String) -> FieldResult<bool> { resolvers::dashboards::dashboard_delete(context, id).await }

    /// Create or update an investigation/case state.
    async fn investigation_save(context: &ApiContext, name: String, state: String, id: Option<String>,) -> FieldResult<Investigation> { resolvers::investigations::investigation_save(context, name, state, id).await }

    /// Delete an investigation/case.
    async fn investigation_delete(context: &ApiContext, id: String) -> FieldResult<bool> { resolvers::investigations::investigation_delete(context, id).await }

    /// Create or update a named saved page state.
    async fn saved_view_save(context: &ApiContext, name: String, page: String, state: String, id: Option<String>,) -> FieldResult<SavedView> { resolvers::investigations::saved_view_save(context, name, page, state, id).await }

    /// Delete a named saved page state.
    async fn saved_view_delete(context: &ApiContext, id: String) -> FieldResult<bool> { resolvers::investigations::saved_view_delete(context, id).await }

    /// Close a run with the wrapped command's exit code.
    async fn run_finish(context: &ApiContext, run_id: String, ended_at_nanos: String, exit_code: i32,) -> FieldResult<bool> { resolvers::runs::run_finish(context, run_id, ended_at_nanos, exit_code).await }

}

pub use query_limits::check_query_limits;
pub(crate) use query_limits::clamp_limit;
pub use schema::{Schema, build_schema, execute};

#[cfg(test)]
mod tests;
