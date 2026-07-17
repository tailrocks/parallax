//! GreptimeDB `TelemetryStore` adapter: SQL over the HTTP API, DDL from the
//! implementation spec §5. All engine-specific SQL lives in this module.

use crate::adapter::{
    ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT, ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow,
    FIELD_KEYS_CAP, FIELD_TOP_VALUES_CAP, FieldKey, FieldSource, FieldStats, FieldValueCount,
    MAX_ROWS, MetricAnalyticsStore, MetricStore, OverviewTotals, ReleaseWindow,
    RuntimeMetricSeries, ServiceCatalogRow, ServiceEdge, ServiceSummary,
    SignalKind, SpanRed, StorageError, StorageResult, attribute_compare_key_allowed,
    attribute_compare_score, attribute_compare_value_allowed, field_key_identifier_like,
    field_key_namespace, field_value_display, metric_group_label_allowed, runtime_metric_family,
    runtime_metric_unit, span_field_key_allowed,
};
use crate::greptime_sql::{
    METRIC_BOOKKEEPING_COLUMNS, canonical_metric_display_name, escape, escape_ident,
    log_service_name_expr, metric_name_sql_filter, metric_table_candidates, quoted_ident,
    resource_attr_ident, runtime_display_name, span_attr_ident, trace_attr_expr, wire_attr_ident,
};
use crate::model::*;
use parallax_semconv as semconv;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

mod analytics_helpers;
mod ingest;
mod invocation_store;
mod lifecycle;
mod metric_analytics;
mod query_sql;
mod row_decode;
mod service_analytics;
mod signal_analytics;
mod signal_queries;
mod trace_analytics;
mod trace_store;
mod transport;

use analytics_helpers::*;
use query_sql::*;
use row_decode::*;

/// Client-side HTTP deadline for all GreptimeDB requests (reads + OTLP forwards).
/// Slightly above the SQL `X-Greptime-Timeout` so the engine can return a
/// structured timeout before reqwest aborts the socket.
type MetricTableCache = Arc<RwLock<HashMap<(String, Option<String>), String>>>;

const HTTP_CLIENT_TIMEOUT: Duration = Duration::from_secs(70);
/// Server-side query deadline sent on SQL reads only (not on OTLP forwards).
const SQL_QUERY_TIMEOUT_HEADER: &str = "60s";
const METRIC_EXEMPLARS_TABLE: &str = "metric_exemplars";
const METRIC_EXEMPLARS_REPLACEMENT: &str = "metric_exemplars_v2";
const METRIC_EXEMPLARS_LEGACY: &str = "metric_exemplars_v1_legacy";
const METRIC_EXEMPLAR_COLUMNS: &str =
    r#""ts", "service", "name", "value", "trace_id", "span_id", "invocation_id", "attributes""#;

#[derive(Debug)]
pub struct GreptimeStore {
    base_url: String,
    client: reqwest::Client,
    /// Retention applied to forwarded native OTLP tables via `x-greptime-hints`.
    traces_ttl: String,
    logs_ttl: String,
    metrics_ttl: String,
    /// Guards the one-shot lazy per-signal deviations applied after that
    /// signal's first forward — each native OTLP table auto-creates on its own
    /// first ingest, so its post-create ALTERs can only land once *that* table
    /// exists. A single shared guard would be consumed by whichever signal
    /// forwards first (e.g. traces), permanently skipping the logs deviations.
    traces_deviations_done: AtomicBool,
    logs_deviations_done: AtomicBool,
    /// Positive-only metric name → table cache (plan 075/085).
    metric_table_cache: MetricTableCache,
}

#[cfg(test)]
mod tests;
