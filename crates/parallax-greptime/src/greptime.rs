//! GreptimeDB `TelemetryStore` adapter: SQL over the HTTP API, DDL from the
//! implementation spec §5. All engine-specific SQL lives in this module.

use crate::adapter::{
    ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT, ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow,
    FIELD_KEYS_CAP, FIELD_TOP_VALUES_CAP, FieldKey, FieldSource, FieldStats, FieldValueCount,
    MAX_ROWS, MetricAnalyticsStore, MetricStore, OverviewTotals, ReleaseWindow,
    RuntimeMetricSeries, SERVICE_MAP_TRACE_CAP, ServiceCatalogRow, ServiceEdge, ServiceSummary,
    SignalKind, SpanRed, attribute_compare_key_allowed, attribute_compare_score,
    attribute_compare_value_allowed, field_key_identifier_like, field_key_namespace,
    field_value_display, metric_group_label_allowed, runtime_metric_family, runtime_metric_unit,
    span_field_key_allowed,
};
use crate::greptime_sql::{
    METRIC_BOOKKEEPING_COLUMNS, canonical_metric_display_name, escape, escape_ident,
    log_service_name_expr, metric_name_sql_filter, metric_table_candidates, quoted_ident,
    resource_attr_ident, runtime_display_name, wire_attr_ident,
};
use crate::model::*;
use parallax_proto::semconv;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

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
    r#""ts", "service", "name", "value", "trace_id", "span_id", "run_id", "attributes""#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExemplarMigrationState {
    Fresh,
    MigrateCanonical,
    ResumeFromLegacy,
    CleanupLegacy,
    Complete,
    UnknownCanonical,
}

fn exemplar_migration_state(
    canonical_key: Option<&[String]>,
    legacy_exists: bool,
) -> ExemplarMigrationState {
    const CURRENT: &[&str] = &["service", "name"];
    const LEGACY: &[&str] = &["service", "name", "trace_id", "span_id"];
    match canonical_key {
        Some(key) if key.iter().map(String::as_str).eq(CURRENT.iter().copied()) => {
            if legacy_exists {
                ExemplarMigrationState::CleanupLegacy
            } else {
                ExemplarMigrationState::Complete
            }
        }
        Some(key) if key.iter().map(String::as_str).eq(LEGACY.iter().copied()) => {
            ExemplarMigrationState::MigrateCanonical
        }
        Some(_) => ExemplarMigrationState::UnknownCanonical,
        None if legacy_exists => ExemplarMigrationState::ResumeFromLegacy,
        None => ExemplarMigrationState::Fresh,
    }
}

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

impl GreptimeStore {
    /// Pure SQL builder for `traces_search` (golden-tested).
    fn traces_search_sql(
        scan_where: &str,
        participation: &str,
        rep_where: &str,
        order: &str,
        limit: usize,
        offset: usize,
    ) -> (String, String) {
        let listed = format!(
            r#"SELECT "root"."trace_id", "root"."span_name", "root"."service_name",
                      "root"."ts_nanos", "root"."dur", "agg"."span_count",
                      "agg"."has_error"
               FROM (
                 SELECT "trace_id", "span_name", "service_name",
                        CAST("timestamp" AS BIGINT) AS "ts_nanos",
                        CAST("duration_nano" AS BIGINT) AS "dur",
                        ROW_NUMBER() OVER (
                          PARTITION BY "trace_id"
                          ORDER BY (CASE WHEN "parent_span_id" IS NULL OR "parent_span_id" = ''
                                         THEN 0 ELSE 1 END) ASC,
                                   "timestamp" ASC
                        ) AS "rn"
                 FROM opentelemetry_traces
                 WHERE {scan_where}{participation}
               ) AS "root"
               JOIN (
                 SELECT "trace_id", COUNT(*) AS "span_count",
                        MAX(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
                        AS "has_error"
                 FROM opentelemetry_traces
                 WHERE {scan_where}
                 GROUP BY "trace_id"
               ) AS "agg" ON "root"."trace_id" = "agg"."trace_id"
               WHERE {rep_where}"#
        );
        // Single-pass page + total (plan 075 Step 2): window function avoids a
        // second HTTP round-trip on the happy path.
        let page = format!(
            r#"SELECT *, COUNT(*) OVER () AS "total" FROM ({listed}) ORDER BY {order} LIMIT {limit} OFFSET {offset}"#
        );
        (listed, page)
    }

    fn histogram_count_series_sql(
        count_table: &str,
        step_secs: u128,
        range_start_ms: u128,
        range_end_ms: u128,
        service_clause: &str,
    ) -> String {
        format!(
            r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", SUM("greptime_value") AS "samples"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
            escape_ident(count_table),
            sql_ts(range_start_ms),
            sql_ts(range_end_ms),
        )
    }

    /// Server-side windowed merge for cumulative histogram buckets (plan 085).
    /// `MAX(greptime_value)` per (window, le) = latest cumulative sample in window.
    fn histogram_quantile_bucket_sql(
        bucket_table: &str,
        step_secs: u128,
        range_start_ms: u128,
        range_end_ms: u128,
        service_clause: &str,
    ) -> String {
        format!(
            r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms",
                          CAST("le" AS DOUBLE) AS "le",
                          MAX("greptime_value") AS "cum"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "bucket_ms", "le"
                   ORDER BY "bucket_ms""#,
            escape_ident(bucket_table),
            sql_ts(range_start_ms),
            sql_ts(range_end_ms),
        )
    }

    fn service_names_sql(range: &RangeInclusive<u128>) -> String {
        format!(
            r#"SELECT DISTINCT "svc" FROM (
                 SELECT "service_name" AS "svc" FROM opentelemetry_traces
                 WHERE "timestamp" >= {} AND "timestamp" <= {}
                 UNION ALL
                 SELECT {} AS "svc" FROM opentelemetry_logs
                 WHERE "timestamp" >= {} AND "timestamp" <= {}
                 UNION ALL
                 SELECT "service" AS "svc" FROM run_metric_points
                 WHERE "ts" >= {} AND "ts" <= {}
               ) WHERE "svc" IS NOT NULL AND "svc" != ''
               ORDER BY "svc""#,
            sql_ts(*range.start()),
            sql_ts(*range.end()),
            log_service_name_expr(),
            sql_ts(*range.start()),
            sql_ts(*range.end()),
            sql_ts(*range.start()),
            sql_ts(*range.end()),
        )
    }

    fn span_field_stats_sample_sql(value_expr: &str, sample_where: &str) -> String {
        format!(
            r#"SELECT {value_expr} AS "value"
               FROM opentelemetry_traces
               WHERE {sample_where}
               ORDER BY "timestamp" DESC
               LIMIT {MAX_ROWS}"#
        )
    }

    fn span_field_stats_top_values_sql(sample_sql: &str) -> String {
        format!(
            r#"WITH "field_sample" AS ({sample_sql})
               SELECT "value", COUNT(*) AS "count",
                      (SELECT COUNT(DISTINCT "value") FROM "field_sample") AS "distinct_count"
               FROM "field_sample"
               GROUP BY "value"
               ORDER BY "count" DESC, "value" ASC
               LIMIT {FIELD_TOP_VALUES_CAP}"#
        )
    }

    fn service_map_edges_sql(id_list: &str, range: &RangeInclusive<u128>) -> String {
        format!(
            r#"SELECT "parent"."service_name" AS "source",
                      "child"."service_name" AS "target",
                      COUNT(*) AS "call_count",
                      SUM(CASE WHEN "child"."span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
                        AS "error_count",
                      approx_percentile_cont("child"."duration_nano", 0.50) AS "p50_ns",
                      approx_percentile_cont("child"."duration_nano", 0.95) AS "p95_ns"
               FROM opentelemetry_traces AS "child"
               JOIN opentelemetry_traces AS "parent"
                 ON "child"."trace_id" = "parent"."trace_id"
                AND "child"."parent_span_id" = "parent"."span_id"
               WHERE "child"."trace_id" IN ({id_list})
                 AND "child"."timestamp" >= {}
                 AND "child"."timestamp" <= {}
                 AND "child"."span_kind" = 'SPAN_KIND_SERVER'
                 AND "child"."service_name" != "parent"."service_name"
               GROUP BY "parent"."service_name", "child"."service_name""#,
            sql_ts(*range.start()),
            sql_ts(*range.end()),
        )
    }

    fn select_spans_sql(where_clause: &str, order: &str, limit_clause: &str) -> String {
        format!(r#"SELECT * FROM opentelemetry_traces WHERE {where_clause}{order}{limit_clause}"#)
    }

    fn select_logs_sql(where_clause: &str, order: &str, limit_clause: &str) -> String {
        format!(
            r#"SELECT CAST("timestamp" AS BIGINT) AS "ts_nanos",
                          {} AS "service",
                          "severity_number", "severity_text", "body", "trace_id", "span_id",
                          {}, "scope_name",
                          "log_attributes",
                          "resource_attributes",
                          json_get_string("log_attributes", '{}') AS "event_name",
                          json_get_int("log_attributes", '{}') AS "observed_ts_nanos"
                   FROM opentelemetry_logs WHERE {where_clause}{order}{limit_clause}"#,
            log_service_name_expr(),
            wire_attr_ident(semconv::PARALLAX_RUN_ID),
            semconv::resource_json_path(semconv::EVENT_NAME),
            semconv::resource_json_path(semconv::LOG_OBSERVED_TS_NANOS),
        )
    }

    fn span_attribute_counts_sql(
        key: &str,
        range: &RangeInclusive<u128>,
        service: Option<&str>,
        error_only: bool,
    ) -> String {
        let column = format!(r#""span_attributes.{}""#, escape_ident(key));
        let value_expr = format!("CAST({column} AS STRING)");
        let mut clauses = vec![
            format!(
                r#""timestamp" >= {} AND "timestamp" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ),
            format!("{column} IS NOT NULL"),
        ];
        if let Some(service) = service {
            clauses.push(format!(r#""service_name" = '{}'"#, escape(service)));
        }
        if error_only {
            clauses.push(r#""span_status_code" = 'STATUS_CODE_ERROR'"#.to_string());
        }
        format!(
            r#"SELECT {value_expr} AS "value", COUNT(*) AS "n"
                   FROM opentelemetry_traces
                   WHERE {}
                   GROUP BY {value_expr}
                   ORDER BY "n" DESC
                   LIMIT 512"#,
            clauses.join(" AND ")
        )
    }
}

fn raw_sql_read_only(query: &str) -> bool {
    let trimmed = query.trim();
    let lowered = trimmed.to_ascii_lowercase();
    let read_only = [
        "select", "with", "show", "describe", "desc", "explain", "tql",
    ]
    .iter()
    .any(|prefix| lowered.starts_with(prefix));
    read_only
        && !trimmed.trim_end_matches(';').contains(';')
        && !(lowered.starts_with("explain") && lowered.contains("analyze"))
}

/// True when a SQL error is GreptimeDB reporting that the target table does not
/// exist yet. Native OTLP tables auto-create on the first forward, so any read
/// before the matching signal has arrived must read as empty rather than fail.
/// Matches GreptimeDB's "Table not found" plan error (code 4001).
fn is_missing_table(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .to_ascii_lowercase()
        .contains("table not found")
}

fn is_missing_column(error: &anyhow::Error) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    (message.contains("column") || message.contains("field"))
        && (message.contains("not found") || message.contains("not exist"))
}

fn json_literal(value: &serde_json::Value) -> String {
    format!("parse_json('{}')", escape(&value.to_string()))
}

fn opt_literal(value: &Option<String>) -> String {
    match value {
        Some(s) => format!("'{}'", escape(s)),
        None => "NULL".to_string(),
    }
}

impl GreptimeStore {
    pub async fn connect(
        base_url: &str,
        traces_ttl: &str,
        logs_ttl: &str,
        metrics_ttl: &str,
    ) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(HTTP_CLIENT_TIMEOUT)
            .build()?;
        let store = Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            traces_ttl: traces_ttl.to_string(),
            logs_ttl: logs_ttl.to_string(),
            metrics_ttl: metrics_ttl.to_string(),
            traces_deviations_done: AtomicBool::new(false),
            logs_deviations_done: AtomicBool::new(false),
            metric_table_cache: Arc::new(RwLock::new(HashMap::new())),
        };
        // Liveness probe before DDL.
        store
            .client
            .get(format!("{}/health", store.base_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(store)
    }

    /// Create extension tables + pre-create the native logs schema (idempotent),
    /// apply repair ALTERs, and reconcile TTLs from config.
    pub async fn bootstrap(&self, metrics_ttl: &str, error_events_ttl: &str) -> anyhow::Result<()> {
        // Pre-create opentelemetry_logs so extract-keys cannot promote high-card
        // attributes into the PRIMARY KEY (Plan 084). Schema matches GreptimeDB
        // v1.1.2 native OTLP logs + deliberate FIELD/TAG deviations.
        let logs_create = format!(
            r#"CREATE TABLE IF NOT EXISTS "opentelemetry_logs" (
                   "timestamp" TIMESTAMP(9) NOT NULL,
                   "trace_id" STRING NULL SKIPPING INDEX,
                   "span_id" STRING NULL,
                   "severity_text" STRING NULL,
                   "severity_number" INT NULL,
                   "body" STRING NULL FULLTEXT INDEX WITH(
                       analyzer = 'English',
                       backend = 'bloom',
                       case_sensitive = 'false',
                       false_positive_rate = '0.01',
                       granularity = '10240'
                   ),
                   "log_attributes" JSON NULL,
                   "trace_flags" INT UNSIGNED NULL,
                   "scope_name" STRING NULL,
                   "scope_version" STRING NULL,
                   "scope_attributes" JSON NULL,
                   "scope_schema_url" STRING NULL,
                   "resource_attributes" JSON NULL,
                   "resource_schema_url" STRING NULL,
                   "service.name" STRING NULL,
                   {} STRING NULL SKIPPING INDEX,
                   {} STRING NULL,
                   {} BIGINT NULL,
                   TIME INDEX ("timestamp"),
                   PRIMARY KEY ("scope_name", "service.name")
                 )
                 ENGINE=mito
                 WITH(
                   append_mode = 'true',
                   'greptime.semantic.signal_type' = 'log',
                   'greptime.semantic.source' = 'opentelemetry',
                   ttl = '{}'
                 )"#,
            wire_attr_ident(semconv::PARALLAX_RUN_ID),
            wire_attr_ident(semconv::EVENT_NAME),
            wire_attr_ident(semconv::LOG_OBSERVED_TS_NANOS),
            escape(&self.logs_ttl),
        );
        self.sql(&logs_create).await?;

        let statements = [
            format!(
                r#"CREATE TABLE IF NOT EXISTS error_events (
                   "ts" TIMESTAMP(9) NOT NULL, "service" STRING, "fingerprint" STRING,
                   "error_type" STRING, "message" STRING, "stacktrace" STRING, "source" STRING,
                   "trace_id" STRING, "span_id" STRING, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "fingerprint")
                 ) WITH (ttl = '{error_events_ttl}')"#
            ),
            format!(
                r#"CREATE TABLE IF NOT EXISTS run_metric_points (
                   "ts" TIMESTAMP(9) NOT NULL, "run_id" STRING SKIPPING INDEX,
                   "service" STRING, "name" STRING, "value" DOUBLE, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
                 ) WITH (append_mode = 'true', ttl = '{metrics_ttl}')"#
            ),
        ];
        for statement in statements {
            self.sql(&statement).await?;
        }
        self.migrate_metric_exemplars(metrics_ttl).await?;
        self.try_traces_deviations().await;
        self.try_logs_deviations().await;
        self.reconcile_ttls(metrics_ttl, error_events_ttl).await;
        Ok(())
    }

    fn metric_exemplars_ddl(table: &str, metrics_ttl: &str) -> String {
        format!(
            r#"CREATE TABLE IF NOT EXISTS {table} (
                   "ts" TIMESTAMP(9) NOT NULL,
                   "service" STRING, "name" STRING, "value" DOUBLE,
                   "trace_id" STRING SKIPPING INDEX, "span_id" STRING,
                   "run_id" STRING SKIPPING INDEX, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
                 ) WITH (append_mode = 'true', ttl = '{}')"#,
            escape(metrics_ttl)
        )
    }

    async fn table_exists(&self, table: &str) -> anyhow::Result<bool> {
        let rows = self
            .sql(&format!(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '{}'",
                escape(table)
            ))
            .await?;
        Ok(rows.first().map(|row| u128_at(row, 0)).unwrap_or(0) == 1)
    }

    async fn table_primary_key(&self, table: &str) -> anyhow::Result<Vec<String>> {
        Ok(self
            .sql(&format!("DESCRIBE {}", quoted_ident(table)))
            .await?
            .iter()
            .filter(|row| str_at(row, 2) == "PRI" && str_at(row, 5) == "TAG")
            .map(|row| str_at(row, 0))
            .collect())
    }

    async fn table_count(&self, table: &str) -> anyhow::Result<u128> {
        let rows = self
            .sql(&format!("SELECT COUNT(*) FROM {}", quoted_ident(table)))
            .await?;
        Ok(rows.first().map(|row| u128_at(row, 0)).unwrap_or(0))
    }

    async fn verify_exemplar_copy(&self, source: &str, destination: &str) -> anyhow::Result<()> {
        let source_count = self.table_count(source).await?;
        let destination_count = self.table_count(destination).await?;
        anyhow::ensure!(
            source_count == destination_count,
            "metric exemplar migration row-count mismatch: {source}={source_count}, {destination}={destination_count}"
        );
        let mismatches = self
            .sql(&format!(
                r#"SELECT COUNT(*) FROM (
                       (SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {source} EXCEPT
                        SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {destination})
                       UNION ALL
                       (SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {destination} EXCEPT
                        SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {source})
                   ) AS differences"#,
                source = quoted_ident(source),
                destination = quoted_ident(destination),
            ))
            .await?;
        anyhow::ensure!(
            mismatches.first().map(|row| u128_at(row, 0)).unwrap_or(0) == 0,
            "metric exemplar migration changed values"
        );
        Ok(())
    }

    async fn migrate_metric_exemplars(&self, metrics_ttl: &str) -> anyhow::Result<()> {
        let canonical_exists = self.table_exists(METRIC_EXEMPLARS_TABLE).await?;
        let legacy_exists = self.table_exists(METRIC_EXEMPLARS_LEGACY).await?;
        let canonical_key = if canonical_exists {
            Some(self.table_primary_key(METRIC_EXEMPLARS_TABLE).await?)
        } else {
            None
        };
        let state = exemplar_migration_state(canonical_key.as_deref(), legacy_exists);

        match state {
            ExemplarMigrationState::Complete => {
                if self.table_exists(METRIC_EXEMPLARS_REPLACEMENT).await? {
                    self.sql(&format!(
                        "DROP TABLE {}",
                        quoted_ident(METRIC_EXEMPLARS_REPLACEMENT)
                    ))
                    .await?;
                }
                return Ok(());
            }
            ExemplarMigrationState::CleanupLegacy => {
                self.verify_exemplar_copy(METRIC_EXEMPLARS_LEGACY, METRIC_EXEMPLARS_TABLE)
                    .await?;
                self.sql(&format!(
                    "DROP TABLE {}",
                    quoted_ident(METRIC_EXEMPLARS_LEGACY)
                ))
                .await?;
                if self.table_exists(METRIC_EXEMPLARS_REPLACEMENT).await? {
                    self.sql(&format!(
                        "DROP TABLE {}",
                        quoted_ident(METRIC_EXEMPLARS_REPLACEMENT)
                    ))
                    .await?;
                }
                return Ok(());
            }
            ExemplarMigrationState::Fresh => {
                self.sql(&Self::metric_exemplars_ddl(
                    METRIC_EXEMPLARS_TABLE,
                    metrics_ttl,
                ))
                .await?;
                return Ok(());
            }
            ExemplarMigrationState::UnknownCanonical => {
                anyhow::bail!("metric_exemplars has an unknown primary-key shape")
            }
            ExemplarMigrationState::MigrateCanonical | ExemplarMigrationState::ResumeFromLegacy => {
            }
        }

        let source = if state == ExemplarMigrationState::MigrateCanonical {
            METRIC_EXEMPLARS_TABLE
        } else {
            METRIC_EXEMPLARS_LEGACY
        };

        if self.table_exists(METRIC_EXEMPLARS_REPLACEMENT).await? {
            self.sql(&format!(
                "DROP TABLE {}",
                quoted_ident(METRIC_EXEMPLARS_REPLACEMENT)
            ))
            .await?;
        }
        self.sql(&Self::metric_exemplars_ddl(
            METRIC_EXEMPLARS_REPLACEMENT,
            metrics_ttl,
        ))
        .await?;
        self.sql(&format!(
            "INSERT INTO {} ({METRIC_EXEMPLAR_COLUMNS}) SELECT {METRIC_EXEMPLAR_COLUMNS} FROM {}",
            quoted_ident(METRIC_EXEMPLARS_REPLACEMENT),
            quoted_ident(source)
        ))
        .await?;
        self.verify_exemplar_copy(source, METRIC_EXEMPLARS_REPLACEMENT)
            .await?;

        if source == METRIC_EXEMPLARS_TABLE {
            self.sql(&format!(
                "ALTER TABLE {} RENAME {}",
                quoted_ident(METRIC_EXEMPLARS_TABLE),
                quoted_ident(METRIC_EXEMPLARS_LEGACY)
            ))
            .await?;
        }
        self.sql(&format!(
            "ALTER TABLE {} RENAME {}",
            quoted_ident(METRIC_EXEMPLARS_REPLACEMENT),
            quoted_ident(METRIC_EXEMPLARS_TABLE)
        ))
        .await?;
        self.verify_exemplar_copy(METRIC_EXEMPLARS_LEGACY, METRIC_EXEMPLARS_TABLE)
            .await?;
        self.sql(&format!(
            "DROP TABLE {}",
            quoted_ident(METRIC_EXEMPLARS_LEGACY)
        ))
        .await?;
        Ok(())
    }

    /// Apply configured retention TTLs via `ALTER TABLE … SET 'ttl'`.
    /// Per-metric native tables are excluded (TTL rides creation hints only).
    async fn reconcile_ttls(&self, metrics_ttl: &str, error_events_ttl: &str) {
        let targets = [
            ("opentelemetry_traces", self.traces_ttl.as_str()),
            ("opentelemetry_logs", self.logs_ttl.as_str()),
            ("error_events", error_events_ttl),
            ("run_metric_points", metrics_ttl),
            (METRIC_EXEMPLARS_TABLE, metrics_ttl),
        ];
        for (table, ttl) in targets {
            let sql = format!("ALTER TABLE {table} SET 'ttl' = '{}'", escape(ttl));
            if let Err(error) = self.sql(&sql).await {
                let text = error.to_string().to_ascii_lowercase();
                if !text.contains("not found")
                    && !text.contains("exist")
                    && !text.contains("unknown table")
                {
                    tracing::warn!("ttl reconcile for {table} failed: {error:#}");
                }
            }
        }
    }

    /// Run a batch of idempotent post-create ALTERs, swallowing the benign
    /// "already exists" / "not found" outcomes (the table may not exist yet, or
    /// the deviation may already be applied from a prior run).
    async fn try_deviations<I, S>(&self, statements: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for statement in statements {
            if let Err(error) = self.sql(statement.as_ref()).await {
                let text = error.to_string().to_ascii_lowercase();
                if !text.contains("exist")
                    && !text.contains("duplicate")
                    && !text.contains("not found")
                    && !text.contains("already")
                {
                    tracing::warn!("native deviation failed: {error:#}");
                }
            }
        }
    }

    /// Traces deviation: a `fingerprint` column for cross-signal correlation.
    async fn try_traces_deviations(&self) {
        self.try_deviations([
            // Contract and migration ownership:
            // plans/125-native-trace-fingerprint-deviation.md.
            r#"ALTER TABLE opentelemetry_traces ADD COLUMN "fingerprint" STRING"#,
        ])
        .await;
    }

    /// Logs deviations: SKIPPING on trace_id; ADD COLUMN repair for extract-key
    /// fields. Body FULLTEXT is native-default on ≥1.1 (no ALTER).
    async fn try_logs_deviations(&self) {
        self.try_deviations([
            r#"ALTER TABLE opentelemetry_logs MODIFY COLUMN "trace_id" SET SKIPPING INDEX"#
                .to_string(),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::SERVICE_NAME)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::PARALLAX_RUN_ID)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} STRING",
                wire_attr_ident(semconv::EVENT_NAME)
            ),
            format!(
                "ALTER TABLE opentelemetry_logs ADD COLUMN {} BIGINT",
                wire_attr_ident(semconv::LOG_OBSERVED_TS_NANOS)
            ),
        ])
        .await;
        let sql = format!(
            "ALTER TABLE opentelemetry_logs SET 'ttl' = '{}'",
            escape(&self.logs_ttl)
        );
        crate::outcomes::warn_error(self.sql(&sql).await, "logs TTL reconcile");
    }

    /// Apply the traces deviations once per process, after the first traces
    /// forward has auto-created `opentelemetry_traces`.
    async fn ensure_traces_deviations(&self) {
        if self
            .traces_deviations_done
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.try_traces_deviations().await;
            let sql = format!(
                "ALTER TABLE opentelemetry_traces SET 'ttl' = '{}'",
                escape(&self.traces_ttl)
            );
            crate::outcomes::warn_error(self.sql(&sql).await, "traces TTL reconcile");
        }
    }

    /// Apply the logs deviations once per process, after the first logs forward
    /// has auto-created `opentelemetry_logs`.
    async fn ensure_logs_deviations(&self) {
        if self
            .logs_deviations_done
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.try_logs_deviations().await;
        }
    }

    /// Forward a raw OTLP/HTTP protobuf body to one of GreptimeDB's native
    /// `/v1/otlp/v1/...` endpoints. `headers` carries the per-signal pipeline /
    /// extract-keys / hints; the body is sent verbatim.
    async fn forward_otlp(
        &self,
        path: &str,
        headers: &[(&str, &str)],
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        let mut request = self
            .client
            .post(format!("{}/v1/otlp/{path}", self.base_url))
            .header("content-type", "application/x-protobuf");
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        request.body(raw).send().await?.error_for_status()?;
        Ok(())
    }

    /// Like [`Self::sql`], but tolerant of a not-yet-created native table: the
    /// native OTLP tables (`opentelemetry_traces`/`_logs`, the per-metric engine
    /// tables) only exist after the first forward, so a read issued before any
    /// matching signal has arrived must read as **empty**, not error. Used by the
    /// typed read paths; the raw-SQL surface keeps strict [`Self::sql`].
    async fn sql_lenient(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        match self.sql(sql).await {
            Err(error) if is_missing_table(&error) => Ok(Vec::new()),
            other => other,
        }
    }

    /// Arrow+zstd sibling of [`Self::sql_lenient`] for heavy typed reads (plan 091).
    async fn sql_arrow_lenient(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        match self.sql_arrow(sql).await {
            Err(error) if is_missing_table(&error) => Ok(Vec::new()),
            other => other,
        }
    }

    /// Run one SQL statement; return the first result set's rows.
    ///
    /// Uses `greptimedb_v1` JSON — keep for DDL/admin, `information_schema`,
    /// single-row counts, `LIMIT 0` schema probes, and other tiny results.
    /// Heavy page/series reads use [`Self::sql_arrow`].
    pub async fn sql(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        let response = self.sql_json_response(sql).await?;
        // Success responses carry `output` (no `code`); failures carry
        // `error` (+ a non-zero `code`).
        if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
            let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            // Char-boundary-safe: byte slice at 200 can panic on multi-byte UTF-8.
            let sql_prefix: String = sql.chars().take(200).collect();
            anyhow::bail!("greptime sql failed (code {code}): {error} — sql: {sql_prefix}");
        }
        anyhow::ensure!(
            response.get("output").is_some(),
            "greptime sql returned neither output nor error: {response}"
        );
        let rows = response
            .pointer("/output/0/records/rows")
            .and_then(|r| r.as_array())
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.as_array().cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(rows)
    }

    /// Heavy read path: HTTP `format=arrow&compression=zstd` → same row shape
    /// as [`Self::sql`] (plan 091; measured GO in plan 090).
    pub async fn sql_arrow(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        let result = self.sql_with_schema_arrow(sql).await?;
        Ok(result.rows)
    }

    /// [`Self::sql_with_schema`] with the not-yet-created-table tolerance of
    /// [`Self::sql_lenient`]: returns an empty result set instead of erroring
    /// when the native table has not auto-created yet.
    async fn sql_with_schema_lenient(
        &self,
        sql: &str,
    ) -> anyhow::Result<crate::adapter::SqlResult> {
        match self.sql_with_schema(sql).await {
            Err(error) if is_missing_table(&error) => Ok(crate::adapter::SqlResult {
                columns: Vec::new(),
                rows: Vec::new(),
            }),
            other => other,
        }
    }

    /// Arrow+zstd sibling of [`Self::sql_with_schema_lenient`].
    async fn sql_with_schema_arrow_lenient(
        &self,
        sql: &str,
    ) -> anyhow::Result<crate::adapter::SqlResult> {
        match self.sql_with_schema_arrow(sql).await {
            Err(error) if is_missing_table(&error) => Ok(crate::adapter::SqlResult {
                columns: Vec::new(),
                rows: Vec::new(),
            }),
            other => other,
        }
    }

    /// Like [`Self::sql`], but also returns the result-set column names
    /// (the raw-SQL surface needs a generic grid, not a fixed projection).
    pub async fn sql_with_schema(&self, sql: &str) -> anyhow::Result<crate::adapter::SqlResult> {
        let response = self.sql_json_response(sql).await?;
        if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
            let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            anyhow::bail!("greptime sql failed (code {code}): {error}");
        }
        let columns = response
            .pointer("/output/0/records/schema/column_schemas")
            .and_then(|c| c.as_array())
            .map(|cols| {
                cols.iter()
                    .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let rows = response
            .pointer("/output/0/records/rows")
            .and_then(|r| r.as_array())
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.as_array().cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(crate::adapter::SqlResult { columns, rows })
    }

    /// Arrow+zstd variant of [`Self::sql_with_schema`] for wide/tall result sets.
    pub async fn sql_with_schema_arrow(
        &self,
        sql: &str,
    ) -> anyhow::Result<crate::adapter::SqlResult> {
        let bytes = self
            .client
            .post(format!(
                "{}/v1/sql?db=public&format=arrow&compression=zstd",
                self.base_url
            ))
            .header("X-Greptime-Timeout", SQL_QUERY_TIMEOUT_HEADER)
            .form(&[("sql", sql)])
            .send()
            .await?
            .bytes()
            .await?;
        let (columns, rows) = crate::arrow_sql::decode_arrow_ipc(&bytes).map_err(|error| {
            let sql_prefix: String = sql.chars().take(200).collect();
            anyhow::anyhow!("{error} — sql: {sql_prefix}")
        })?;
        Ok(crate::adapter::SqlResult { columns, rows })
    }

    async fn sql_json_response(&self, sql: &str) -> anyhow::Result<serde_json::Value> {
        Ok(self
            .client
            .post(format!("{}/v1/sql?db=public", self.base_url))
            .header("X-Greptime-Timeout", SQL_QUERY_TIMEOUT_HEADER)
            .form(&[("sql", sql)])
            .send()
            .await?
            .json()
            .await?)
    }

    async fn metric_table_for_name(
        &self,
        name: &str,
        suffix: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
        let cache_key = (name.to_string(), suffix.map(str::to_string));
        {
            let cache = self.metric_table_cache.read().await;
            if let Some(table) = cache.get(&cache_key) {
                return Ok(Some(table.clone()));
            }
        }
        let candidates = metric_table_candidates(name, suffix);
        if candidates.is_empty() {
            return Ok(None);
        }
        let quoted = candidates
            .iter()
            .map(|candidate| format!("'{}'", escape(candidate)))
            .collect::<Vec<_>>()
            .join(",");
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT "table_name" FROM information_schema.tables
                   WHERE "table_schema" = 'public' AND "table_name" IN ({quoted})"#
            ))
            .await?;
        let existing = rows
            .iter()
            .map(|row| str_at(row, 0))
            .collect::<BTreeSet<_>>();
        let found = candidates
            .into_iter()
            .find(|candidate| existing.contains(candidate));
        if let Some(ref table) = found {
            self.metric_table_cache
                .write()
                .await
                .insert(cache_key, table.clone());
        }
        Ok(found)
    }

    async fn resolved_metric_table(
        &self,
        name: &str,
    ) -> anyhow::Result<Option<(String, Vec<String>)>> {
        let table = match self.metric_table_for_name(name, None).await? {
            Some(table) => table,
            None => match self.metric_table_for_name(name, Some("_bucket")).await? {
                Some(table) => table,
                None => return Ok(None),
            },
        };
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT "column_name" FROM information_schema.columns
                   WHERE "table_schema" = 'public' AND "table_name" = '{}'
                   ORDER BY "column_name""#,
                escape(&table),
            ))
            .await?;
        let labels = rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|column| {
                !METRIC_BOOKKEEPING_COLUMNS.contains(&column.as_str())
                    && metric_group_label_allowed(column)
            })
            .collect();
        Ok(Some((table, labels)))
    }

    async fn insert(&self, table: &str, columns: &str, values: Vec<String>) -> anyhow::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        let sql = format!(
            "INSERT INTO {table} ({columns}) VALUES {}",
            values.join(",")
        );
        self.sql(&sql).await?;
        Ok(())
    }

    /// Select spans from the native `opentelemetry_traces` table. `SELECT *` is
    /// used so the per-attribute columns (`span_attributes.*` /
    /// `resource_attributes.*`) — which auto-widen over time — are all present
    /// and can be folded back into the `attributes`/`resource` JSON maps.
    async fn select_spans(
        &self,
        where_clause: &str,
        order: &str,
        limit_clause: &str,
    ) -> anyhow::Result<Vec<SpanRow>> {
        let result = self
            .sql_with_schema_arrow_lenient(&Self::select_spans_sql(
                where_clause,
                order,
                limit_clause,
            ))
            .await?;
        let cols = ColumnIndex::new(&result.columns);
        Ok(result
            .rows
            .iter()
            .map(|row| {
                // native: `timestamp` is the span start TIME INDEX (ns);
                // `duration_nano` is the generated duration in ns.
                let (attributes, resource) = cols.reassemble_attrs(row);
                let events = match cols.json("span_events", row) {
                    serde_json::Value::Null => None,
                    value => Some(value.to_string()),
                };
                SpanRow {
                    ts_nanos: cols.u128("timestamp", row),
                    service: cols.string("service_name", row),
                    trace_id: cols.string("trace_id", row),
                    span_id: cols.string("span_id", row),
                    parent_span_id: cols.opt_string("parent_span_id", row),
                    name: cols.string("span_name", row),
                    kind: cols.string("span_kind", row),
                    status_code: cols.string("span_status_code", row),
                    status_message: cols.string("span_status_message", row),
                    duration_ns: cols.u128("duration_nano", row),
                    // Native run id flattens to a resource-attribute column.
                    run_id: cols
                        .opt_string(&semconv::resource_column(semconv::PARALLAX_RUN_ID), row),
                    scope_name: cols.string("scope_name", row),
                    events,
                    links: cols.json("span_links", row),
                    attributes,
                    resource,
                }
            })
            .collect())
    }

    /// Select logs from the native `opentelemetry_logs` table. Top-level OTLP
    /// log identity fields are mirrored into attributes before native forward
    /// because GreptimeDB does not map them to columns yet.
    async fn select_logs(
        &self,
        where_clause: &str,
        order: &str,
        limit_clause: &str,
    ) -> anyhow::Result<Vec<LogRow>> {
        let rows = self
            .sql_arrow_lenient(&Self::select_logs_sql(where_clause, order, limit_clause))
            .await?;
        Ok(rows.iter().map(|row| log_row_from_row(row)).collect())
    }
}

/// A row → `LogRow` projection for the fixed native log column order used by
/// [`GreptimeStore::select_logs`] and `logs_search`.
fn log_row_from_row(row: &[serde_json::Value]) -> LogRow {
    LogRow {
        ts_nanos: u128_at(row, 0),
        service: str_at(row, 1),
        severity_num: row.get(2).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        severity_text: str_at(row, 3),
        body: str_at(row, 4),
        trace_id: str_at(row, 5),
        span_id: str_at(row, 6),
        run_id: opt_str_at(row, 7),
        scope_name: str_at(row, 8),
        attributes: json_at(row, 9),
        resource: json_at(row, 10),
        event_name: str_at(row, 11),
        observed_ts_nanos: u128_at(row, 12),
    }
}

/// Maps native result-column names to their position in a row, so a `SELECT *`
/// (whose schema auto-widens with new attribute keys) can be read by name and
/// the `span_attributes.*` / `resource_attributes.*` columns folded back into
/// the `attributes` / `resource` JSON objects the model carries.
struct ColumnIndex<'a> {
    columns: &'a [String],
    by_name: HashMap<&'a str, usize>,
}

impl<'a> ColumnIndex<'a> {
    fn new(columns: &'a [String]) -> Self {
        let by_name = columns
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();
        Self { columns, by_name }
    }

    fn idx(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }

    fn string(&self, name: &str, row: &[serde_json::Value]) -> String {
        self.idx(name).map(|i| str_at(row, i)).unwrap_or_default()
    }

    fn opt_string(&self, name: &str, row: &[serde_json::Value]) -> Option<String> {
        self.idx(name)
            .and_then(|i| opt_str_at(row, i))
            .filter(|s| !s.is_empty())
    }

    fn u128(&self, name: &str, row: &[serde_json::Value]) -> u128 {
        self.idx(name).map(|i| u128_at(row, i)).unwrap_or(0)
    }

    fn json(&self, name: &str, row: &[serde_json::Value]) -> serde_json::Value {
        self.idx(name)
            .map(|i| json_at(row, i))
            .unwrap_or(serde_json::Value::Null)
    }

    /// Fold the flattened native attribute columns back into two JSON maps:
    /// `span_attributes.<k>` → attributes, `resource_attributes.<k>` → resource
    /// (the dotted prefix stripped). Non-null scalar values only.
    fn reassemble_attrs(
        &self,
        row: &[serde_json::Value],
    ) -> (serde_json::Value, serde_json::Value) {
        let mut attributes = serde_json::Map::new();
        let mut resource = serde_json::Map::new();
        for (i, name) in self.columns.iter().enumerate() {
            let Some(value) = row.get(i) else { continue };
            if value.is_null() {
                continue;
            }
            if let Some(key) = name.strip_prefix("span_attributes.") {
                attributes.insert(key.to_string(), value.clone());
            } else if let Some(key) = name.strip_prefix("resource_attributes.") {
                resource.insert(key.to_string(), value.clone());
            }
        }
        (
            serde_json::Value::Object(attributes),
            serde_json::Value::Object(resource),
        )
    }
}

#[derive(Debug, Clone)]
struct SpanFieldColumn {
    key: String,
    column: String,
    source: FieldSource,
}

fn span_field_column_from_name(column: &str) -> Option<SpanFieldColumn> {
    if let Some(key) = column.strip_prefix("span_attributes.") {
        let key = key.to_string();
        return span_field_key_allowed(&key).then_some(SpanFieldColumn {
            key,
            column: column.to_string(),
            source: FieldSource::Span,
        });
    }
    if let Some(key) = column.strip_prefix("resource_attributes.") {
        let key = format!("resource.{key}");
        return span_field_key_allowed(&key).then_some(SpanFieldColumn {
            key,
            column: column.to_string(),
            source: FieldSource::Resource,
        });
    }
    None
}

fn quoted_field_column(column: &SpanFieldColumn) -> String {
    format!(r#""{}""#, escape_ident(&column.column))
}

fn str_at(row: &[serde_json::Value], index: usize) -> String {
    row.get(index)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn opt_str_at(row: &[serde_json::Value], index: usize) -> Option<String> {
    row.get(index).and_then(|v| v.as_str()).map(str::to_string)
}

fn opt_nonempty_str_at(row: &[serde_json::Value], index: usize) -> Option<String> {
    opt_str_at(row, index).filter(|value| !value.trim().is_empty())
}

/// Clamp a u128 time bound to what the engine's TIMESTAMP cast accepts
/// (i64); open-ended `..=u128::MAX` ranges otherwise fail query planning
/// ("Casting value to Timestamp is invalid").
fn sql_ts(bound: u128) -> i64 {
    i64::try_from(bound).unwrap_or(i64::MAX)
}

/// Shared WHERE clauses for `logs_search` and `log_count_series`.
///
/// Body search uses `matches_term` (FULLTEXT bloom): term match, not substring;
/// whitespace tokens AND-combined; double-quoted phrase; punctuation → LIKE.
/// Memory adapter stays substring (Plan 084 intentional divergence).
fn log_filter_clauses(
    service: Option<&str>,
    range: &RangeInclusive<u128>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    body_contains: Option<&str>,
) -> Vec<String> {
    let mut clauses = vec![format!(
        r#""timestamp" >= {} AND "timestamp" <= {}"#,
        sql_ts(*range.start()),
        sql_ts(*range.end())
    )];
    if let Some(service) = service {
        clauses.push(format!(
            r#"{} = '{}'"#,
            log_service_name_expr(),
            escape(service)
        ));
    }
    if let Some(min) = severity_min {
        clauses.push(format!(r#""severity_number" >= {min}"#));
    }
    if let Some(max) = severity_max {
        clauses.push(format!(r#""severity_number" <= {max}"#));
    }
    if let Some(needle) = body_contains {
        push_body_search_clause(&mut clauses, needle);
    }
    clauses
}

fn push_body_search_clause(clauses: &mut Vec<String>, needle: &str) {
    let needle = needle.trim();
    if needle.is_empty() {
        return;
    }
    if needle.len() >= 2 && needle.starts_with('"') && needle.ends_with('"') {
        let phrase = &needle[1..needle.len() - 1];
        clauses.push(format!(r#"matches_term("body", '{}')"#, escape(phrase)));
        return;
    }
    if !needle.chars().any(|c| c.is_alphanumeric()) {
        let escaped = escape(
            &needle
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_"),
        );
        clauses.push(format!(r#""body" LIKE '%{escaped}%' ESCAPE '\'"#));
        return;
    }
    for token in needle.split_whitespace() {
        if !token.is_empty() {
            clauses.push(format!(r#"matches_term("body", '{}')"#, escape(token)));
        }
    }
}

fn u128_at(row: &[serde_json::Value], index: usize) -> u128 {
    let Some(value) = row.get(index) else {
        return 0;
    };
    if let Some(n) = value.as_u64() {
        return u128::from(n);
    }
    if let Some(n) = value.as_i64()
        && n >= 0
    {
        return u128::try_from(n).unwrap_or(0);
    }
    if let Some(s) = value.as_str()
        && let Ok(n) = s.parse::<u128>()
    {
        tracing::warn!(
            target: "parallax_greptime",
            index,
            "u128_at decoded JSON string timestamp; prefer integer wire encoding"
        );
        return n;
    }
    if let Some(f) = value.as_f64()
        && f.is_finite()
        && f >= 0.0
    {
        tracing::warn!(
            target: "parallax_greptime",
            index,
            "u128_at decoded JSON float timestamp; prefer integer wire encoding"
        );
        return f.max(0.0) as u128;
    }
    0
}

fn absorb_observed_run(
    runs: &mut HashMap<String, crate::adapter::ObservedRun>,
    row: &[serde_json::Value],
    is_span: bool,
) -> Option<String> {
    let run_id = str_at(row, 0);
    if run_id.is_empty() {
        return None;
    }
    let first = u128_at(row, 1);
    let last = u128_at(row, 2);
    let count = u128_at(row, 3) as u64;
    let entry = runs
        .entry(run_id.clone())
        .or_insert_with(|| crate::adapter::ObservedRun {
            run_id: run_id.clone(),
            first_nanos: first,
            last_nanos: last,
            span_count: 0,
            log_count: 0,
            service: str_at(row, 4),
        });
    entry.first_nanos = entry.first_nanos.min(first);
    entry.last_nanos = entry.last_nanos.max(last);
    if is_span {
        entry.span_count += count;
    } else {
        entry.log_count += count;
    }
    Some(run_id)
}

fn f64_at(row: &[serde_json::Value], index: usize) -> f64 {
    row.get(index).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

fn trace_filter_clauses(service: Option<&str>, range: &RangeInclusive<u128>) -> Vec<String> {
    let mut clauses = vec![format!(
        r#""timestamp" >= {} AND "timestamp" <= {}"#,
        sql_ts(*range.start()),
        sql_ts(*range.end())
    )];
    if let Some(service) = service {
        clauses.push(format!(r#""service_name" = '{}'"#, escape(service)));
    }
    clauses
}

fn json_at(row: &[serde_json::Value], index: usize) -> serde_json::Value {
    match row.get(index) {
        Some(serde_json::Value::String(s)) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
        }
        Some(other) => other.clone(),
        None => serde_json::Value::Null,
    }
}

#[async_trait::async_trait]
impl crate::adapter::IngestStore for GreptimeStore {
    async fn ingest_traces(
        &self,
        _request: &parallax_proto::collector_trace::ExportTraceServiceRequest,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        // Forward the raw OTLP verbatim to the native traces endpoint; the
        // `greptime_trace_v1` pipeline auto-creates `opentelemetry_traces`. The
        // decoded spans are the worker's tee (errors/live/runs), not stored here.
        let hints = format!("ttl={},append_mode=true", self.traces_ttl);
        self.forward_otlp(
            "v1/traces",
            &[
                ("x-greptime-pipeline-name", "greptime_trace_v1"),
                ("x-greptime-hints", &hints),
            ],
            raw,
        )
        .await?;
        self.ensure_traces_deviations().await;
        Ok(())
    }

    async fn ingest_logs(
        &self,
        _request: &parallax_proto::collector_logs::ExportLogsServiceRequest,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        // The extract-keys header promotes run id and typed-log identity
        // attributes to native columns in opentelemetry_logs.
        let hints = format!("ttl={},append_mode=true", self.logs_ttl);
        let extract_keys = format!(
            "{},{},{},{}",
            semconv::SERVICE_NAME,
            semconv::PARALLAX_RUN_ID,
            semconv::EVENT_NAME,
            semconv::LOG_OBSERVED_TS_NANOS
        );
        self.forward_otlp(
            "v1/logs",
            &[
                ("x-greptime-log-extract-keys", &extract_keys),
                ("x-greptime-hints", &hints),
            ],
            raw,
        )
        .await?;
        self.ensure_logs_deviations().await;
        Ok(())
    }

    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        _histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        // Forward all metrics to the native metric engine (one table per metric
        // name; histograms split into `_bucket`/`_count`/`_sum`).
        let hints = format!("ttl={}", self.metrics_ttl);
        self.forward_otlp("v1/metrics", &[("x-greptime-hints", &hints)], raw)
            .await?;
        // Run-scoped points (Q6, Approach 2): the metric engine cannot hold a
        // high-card `run_id` tag, so persist those points to `run_metric_points`
        // where `run_id` is an indexed column.
        let values = points
            .iter()
            .filter(|p| p.run_id.as_deref().is_some_and(|id| !id.is_empty()))
            .map(|p| {
                format!(
                    "({},'{}','{}','{}',{},{})",
                    p.ts_nanos, // TIMESTAMP(9): nanos
                    escape(p.run_id.as_deref().unwrap_or_default()),
                    escape(&p.service),
                    escape(&p.name),
                    p.value,
                    json_literal(&p.attributes),
                )
            })
            .collect();
        self.insert(
            "run_metric_points",
            "\"ts\", \"run_id\", \"service\", \"name\", \"value\", \"attributes\"",
            values,
        )
        .await?;

        let values = exemplars
            .iter()
            .map(|r| {
                format!(
                    "({},'{}','{}',{},'{}','{}',{},{})",
                    r.ts_nanos,
                    escape(&r.service),
                    escape(&r.name),
                    r.value,
                    escape(&r.trace_id),
                    escape(&r.span_id),
                    opt_literal(&r.run_id),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(METRIC_EXEMPLARS_TABLE, METRIC_EXEMPLAR_COLUMNS, values)
            .await
    }

    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> anyhow::Result<()> {
        let values = rows
            .iter()
            .map(|r| {
                let source = serde_json::to_string(&r.source).unwrap_or_default();
                format!(
                    "({},'{}','{}','{}','{}',{},'{}','{}','{}',{})",
                    r.ts_nanos,
                    escape(&r.service),
                    escape(&r.fingerprint),
                    escape(&r.error_type),
                    escape(&r.message),
                    opt_literal(&r.stacktrace),
                    source.trim_matches('"'),
                    escape(&r.trace_id),
                    escape(&r.span_id),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(
            "error_events",
            "\"ts\", \"service\", \"fingerprint\", \"error_type\", \"message\", \"stacktrace\", \"source\", \"trace_id\", \"span_id\", \"attributes\"",
            values,
        )
        .await
    }
}

#[async_trait::async_trait]
impl crate::adapter::TraceStore for GreptimeStore {
    async fn spans_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<SpanRow>> {
        self.select_spans(
            &format!(r#""trace_id" = '{}'"#, escape(trace_id)),
            r#" ORDER BY "timestamp" ASC"#,
            "",
        )
        .await
    }

    async fn traces_by_ids(
        &self,
        trace_ids: &[String],
    ) -> anyhow::Result<Vec<crate::adapter::TraceSummary>> {
        // O(n) dedup preserving request order (MAX_ROWS still caps fan-out).
        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        for trace_id in trace_ids.iter().filter(|trace_id| !trace_id.is_empty()) {
            if !seen.insert(trace_id.as_str()) {
                continue;
            }
            ids.push(trace_id.clone());
            if ids.len() >= MAX_ROWS {
                break;
            }
        }
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_list = ids
            .iter()
            .map(|trace_id| format!("'{}'", escape(trace_id)))
            .collect::<Vec<_>>()
            .join(",");
        let spans = self
            .select_spans(
                &format!(r#""trace_id" IN ({id_list})"#),
                r#" ORDER BY "timestamp" ASC"#,
                "",
            )
            .await?;
        let mut grouped: BTreeMap<String, Vec<SpanRow>> = BTreeMap::new();
        for span in spans {
            grouped.entry(span.trace_id.clone()).or_default().push(span);
        }
        let mut by_id: BTreeMap<_, _> = grouped
            .into_iter()
            .filter_map(|(trace_id, spans)| {
                let root = spans.iter().min_by_key(|span| {
                    (
                        !span.parent_span_id.as_deref().is_none_or(str::is_empty),
                        span.ts_nanos,
                    )
                })?;
                Some((
                    trace_id.clone(),
                    crate::adapter::TraceSummary {
                        trace_id,
                        root_name: root.name.clone(),
                        service: root.service.clone(),
                        start_nanos: root.ts_nanos,
                        duration_ns: root.duration_ns,
                        span_count: spans.len() as u64,
                        has_error: spans
                            .iter()
                            .any(|span| span.status_code == "STATUS_CODE_ERROR"),
                    },
                ))
            })
            .collect();
        Ok(ids
            .into_iter()
            .filter_map(|trace_id| by_id.remove(&trace_id))
            .collect())
    }

    async fn spans_by_run(
        &self,
        run_id: &str,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<SpanRow>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let escaped_run_id = escape(run_id);
        let limit_clause = format!(" LIMIT {limit}");
        let trace_run_column = resource_attr_ident(semconv::PARALLAX_RUN_ID);
        let mut native_missing = false;
        let mut spans = match self
            .select_spans(
                &format!(r#"{trace_run_column} = '{escaped_run_id}'"#),
                r#" ORDER BY "timestamp" DESC"#,
                &limit_clause,
            )
            .await
        {
            Ok(spans) => spans,
            Err(error) if is_missing_column(&error) => {
                native_missing = true;
                Vec::new()
            }
            Err(error) => return Err(error),
        };
        if native_missing || spans.is_empty() {
            let via_logs = match self
                .select_spans(
                    &format!(
                        r#""trace_id" IN (
                    SELECT DISTINCT "trace_id" FROM opentelemetry_logs
                    WHERE {} = '{}'
                      AND "timestamp" >= {} AND "timestamp" <= {}
                  )"#,
                        wire_attr_ident(semconv::PARALLAX_RUN_ID),
                        escaped_run_id,
                        sql_ts(*range.start()),
                        sql_ts(*range.end()),
                    ),
                    r#" ORDER BY "timestamp" DESC"#,
                    &limit_clause,
                )
                .await
            {
                Ok(spans) => spans,
                Err(error) if is_missing_column(&error) => Vec::new(),
                Err(error) => return Err(error),
            };
            let mut seen: BTreeSet<(String, String)> = spans
                .iter()
                .map(|span| (span.trace_id.clone(), span.span_id.clone()))
                .collect();
            for span in via_logs {
                if seen.insert((span.trace_id.clone(), span.span_id.clone())) {
                    spans.push(span);
                }
            }
        }
        spans.sort_by_key(|span| span.ts_nanos);
        if spans.len() > limit {
            spans.drain(0..spans.len() - limit);
        }
        Ok(spans)
    }

    async fn spans_by_runs(
        &self,
        run_ids: &[String],
        limit_per_run: usize,
    ) -> anyhow::Result<HashMap<String, Vec<SpanRow>>> {
        let mut out: HashMap<String, Vec<SpanRow>> = HashMap::with_capacity(run_ids.len());
        for run_id in run_ids {
            out.entry(run_id.clone()).or_default();
        }
        if run_ids.is_empty() || limit_per_run == 0 {
            return Ok(out);
        }
        let escaped = run_ids
            .iter()
            .filter(|id| !id.is_empty())
            .map(|id| format!("'{}'", escape(id)))
            .collect::<Vec<_>>();
        if escaped.is_empty() {
            return Ok(out);
        }
        let id_list = escaped.join(",");
        let trace_run_column = resource_attr_ident(semconv::PARALLAX_RUN_ID);
        let sql = format!(
            r#"SELECT * FROM (
                 SELECT *, ROW_NUMBER() OVER (
                   PARTITION BY {trace_run_column}
                   ORDER BY "timestamp" DESC
                 ) AS "rn"
                 FROM opentelemetry_traces
                 WHERE {trace_run_column} IN ({id_list})
               ) WHERE "rn" <= {limit_per_run}
               ORDER BY "timestamp" ASC"#
        );
        let result = match self.sql_with_schema_arrow_lenient(&sql).await {
            Ok(result) => result,
            Err(error) if is_missing_column(&error) => {
                for run_id in run_ids {
                    out.insert(
                        run_id.clone(),
                        self.spans_by_run(run_id, limit_per_run, 0..=u128::MAX)
                            .await?,
                    );
                }
                return Ok(out);
            }
            Err(error) => return Err(error),
        };
        let cols = ColumnIndex::new(&result.columns);
        for row in &result.rows {
            let (attributes, resource) = cols.reassemble_attrs(row);
            let events = match cols.json("span_events", row) {
                serde_json::Value::Null => None,
                value => Some(value.to_string()),
            };
            let span = SpanRow {
                ts_nanos: cols.u128("timestamp", row),
                service: cols.string("service_name", row),
                trace_id: cols.string("trace_id", row),
                span_id: cols.string("span_id", row),
                parent_span_id: cols.opt_string("parent_span_id", row),
                name: cols.string("span_name", row),
                kind: cols.string("span_kind", row),
                status_code: cols.string("span_status_code", row),
                status_message: cols.string("span_status_message", row),
                duration_ns: cols.u128("duration_nano", row),
                run_id: cols.opt_string(&semconv::resource_column(semconv::PARALLAX_RUN_ID), row),
                scope_name: cols.string("scope_name", row),
                events,
                links: cols.json("span_links", row),
                attributes,
                resource,
            };
            if let Some(run_id) = span.run_id.clone() {
                out.entry(run_id).or_default().push(span);
            }
        }
        Ok(out)
    }
}

#[async_trait::async_trait]
impl crate::adapter::LogStore for GreptimeStore {
    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>> {
        let mut logs = self
            .select_logs(
                &format!(
                    r#"{} = '{}'"#,
                    wire_attr_ident(semconv::PARALLAX_RUN_ID),
                    escape(run_id)
                ),
                r#" ORDER BY "timestamp" DESC"#,
                &format!(" LIMIT {limit}"),
            )
            .await?;
        logs.reverse();
        Ok(logs)
    }

    async fn logs_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<LogRow>> {
        self.select_logs(
            &format!(r#""trace_id" = '{}'"#, escape(trace_id)),
            r#" ORDER BY "timestamp" ASC"#,
            "",
        )
        .await
    }
}

#[async_trait::async_trait]
impl MetricStore for GreptimeStore {
    async fn metric_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>> {
        Ok(self
            .discover_metric_names(&range)
            .await?
            .into_iter()
            .collect())
    }

    async fn metric_labels(&self, name: &str) -> anyhow::Result<Vec<String>> {
        Ok(self
            .resolved_metric_table(name)
            .await?
            .map(|(_, labels)| labels)
            .unwrap_or_default())
    }

    async fn metric_label_values(
        &self,
        name: &str,
        label: &str,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<String>> {
        anyhow::ensure!(
            metric_group_label_allowed(label),
            "high-cardinality identifier - filter, don't group"
        );
        let Some((table, labels)) = self.resolved_metric_table(name).await? else {
            return Ok(Vec::new());
        };
        anyhow::ensure!(
            labels.iter().any(|known| known == label),
            "unknown metric label"
        );
        let label_ident = format!(r#""{}""#, escape_ident(label));
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT CAST({label_ident} AS STRING) AS "value"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}
                     AND {label_ident} IS NOT NULL
                   ORDER BY "value" LIMIT 100"#,
                escape_ident(&table),
                sql_ts(range.start() / 1_000_000),
                sql_ts(range.end() / 1_000_000),
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|value| attribute_compare_value_allowed(value))
            .collect())
    }
}

#[async_trait::async_trait]
impl crate::adapter::ServiceAnalyticsStore for GreptimeStore {
    async fn service_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>> {
        let rows = self.sql_lenient(&Self::service_names_sql(&range)).await?;
        Ok(rows
            .iter()
            .map(|r| str_at(r, 0))
            .filter(|s| !s.is_empty())
            .collect())
    }

    async fn overview_totals(&self, range: RangeInclusive<u128>) -> anyhow::Result<OverviewTotals> {
        let start = sql_ts(*range.start());
        let end = sql_ts(*range.end());
        let log_svc = log_service_name_expr();
        let traces_sql = format!(
            r#"SELECT "t"."spans", "t"."traces", "t"."errors", "s"."svc"
                   FROM (
                     SELECT COUNT(*) AS "spans",
                            COUNT(DISTINCT "trace_id") AS "traces",
                            SUM(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
                              AS "errors"
                     FROM opentelemetry_traces
                     WHERE "timestamp" >= {start} AND "timestamp" <= {end}
                   ) AS "t"
                   LEFT JOIN (
                     SELECT DISTINCT "service_name" AS "svc"
                     FROM opentelemetry_traces
                     WHERE "timestamp" >= {start} AND "timestamp" <= {end}
                       AND "service_name" IS NOT NULL AND "service_name" != ''
                   ) AS "s" ON TRUE"#
        );
        let logs_sql = format!(
            r#"SELECT "t"."logs", "s"."svc"
                   FROM (
                     SELECT COUNT(*) AS "logs"
                     FROM opentelemetry_logs
                     WHERE "timestamp" >= {start} AND "timestamp" <= {end}
                   ) AS "t"
                   LEFT JOIN (
                     SELECT DISTINCT {log_svc} AS "svc"
                     FROM opentelemetry_logs
                     WHERE "timestamp" >= {start} AND "timestamp" <= {end}
                   ) AS "s" ON TRUE"#
        );
        let (trace_rows, log_rows) =
            tokio::try_join!(self.sql_lenient(&traces_sql), self.sql_lenient(&logs_sql))?;
        let mut services = BTreeSet::new();
        let span_count = trace_rows
            .first()
            .map(|r| u128_at(r, 0) as u64)
            .unwrap_or(0);
        let trace_count = trace_rows
            .first()
            .map(|r| u128_at(r, 1) as u64)
            .unwrap_or(0);
        let error_count = trace_rows
            .first()
            .map(|r| u128_at(r, 2) as u64)
            .unwrap_or(0);
        for row in &trace_rows {
            let svc = str_at(row, 3);
            if !svc.is_empty() {
                services.insert(svc);
            }
        }
        let log_count = log_rows.first().map(|r| u128_at(r, 0) as u64).unwrap_or(0);
        for row in &log_rows {
            let svc = str_at(row, 1);
            if !svc.is_empty() {
                services.insert(svc);
            }
        }
        Ok(OverviewTotals {
            span_count,
            trace_count,
            log_count,
            metric_point_count: 0,
            error_count,
            error_rate: if span_count == 0 {
                0.0
            } else {
                error_count as f64 / span_count as f64
            },
            active_services: services.len() as u64,
        })
    }

    async fn signal_count_series(
        &self,
        kind: SignalKind,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let rows = match kind {
            SignalKind::Spans | SignalKind::Traces => {
                let clauses = trace_filter_clauses(service, &range);
                let agg = if kind == SignalKind::Traces {
                    r#"COUNT(DISTINCT "trace_id")"#
                } else {
                    "COUNT(*)"
                };
                self.sql_arrow_lenient(&format!(
                    r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "timestamp") AS BIGINT)
                              AS "bucket_ns", {agg} AS "n"
                       FROM opentelemetry_traces WHERE {}
                       GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                    clauses.join(" AND "),
                ))
                .await?
            }
            SignalKind::Logs => {
                let clauses = log_filter_clauses(service, &range, None, None, None);
                self.sql_arrow_lenient(&format!(
                    r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "timestamp") AS BIGINT)
                              AS "bucket_ns", COUNT(*) AS "n"
                       FROM opentelemetry_logs WHERE {}
                       GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                    clauses.join(" AND "),
                ))
                .await?
            }
            SignalKind::Errors => {
                let mut clauses = vec![format!(
                    r#""ts" >= {} AND "ts" <= {}"#,
                    sql_ts(*range.start()),
                    sql_ts(*range.end())
                )];
                if let Some(service) = service {
                    clauses.push(format!(r#""service" = '{}'"#, escape(service)));
                }
                self.sql_arrow_lenient(&format!(
                    r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "ts") AS BIGINT)
                              AS "bucket_ns", COUNT(*) AS "n"
                       FROM error_events WHERE {}
                       GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                    clauses.join(" AND "),
                ))
                .await?
            }
            SignalKind::MetricPoints => Vec::new(),
        };
        Ok(rows
            .iter()
            .map(|row| SeriesPoint {
                ts_nanos: u128_at(row, 0),
                value: f64_at(row, 1),
            })
            .collect())
    }

    async fn service_summaries(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceSummary>> {
        // Latest stable GreptimeDB accepts approx_percentile_cont(col, q);
        // verified through Parallax raw SQL for trace duration percentiles.
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT "service_name", CAST(MAX("timestamp") AS BIGINT) AS "last_seen",
                          COUNT(*) AS "spans",
                          SUM(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
                          AS "errors",
                          approx_percentile_cont("duration_nano", 0.95) AS "p95_ns"
                   FROM opentelemetry_traces
                   WHERE "timestamp" >= {} AND "timestamp" <= {}
                   GROUP BY "service_name" ORDER BY "last_seen" DESC"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let name = str_at(row, 0);
                (!name.is_empty()).then(|| ServiceSummary {
                    name,
                    last_seen_nanos: u128_at(row, 1),
                    span_count: u128_at(row, 2) as u64,
                    error_count: u128_at(row, 3) as u64,
                    p95_ms: Some(f64_at(row, 4) / 1_000_000.0),
                })
            })
            .collect())
    }

    async fn release_windows(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ReleaseWindow>> {
        let version_column = resource_attr_ident(semconv::SERVICE_VERSION);
        let sql = format!(
            r#"SELECT {version_column} AS "version",
                      MIN(CAST("timestamp" AS BIGINT)) AS "first_seen_nanos",
                      MAX(CAST("timestamp" AS BIGINT)) AS "last_seen_nanos",
                      COUNT(*) AS "span_count"
               FROM opentelemetry_traces
               WHERE "service_name" = '{}'
                 AND "timestamp" >= {}
                 AND "timestamp" <= {}
                 AND {version_column} IS NOT NULL
                 AND {version_column} != ''
               GROUP BY {version_column}
               ORDER BY "first_seen_nanos" ASC, "version" ASC"#,
            escape(service),
            sql_ts(*range.start()),
            sql_ts(*range.end()),
        );
        let rows = match self.sql(&sql).await {
            Err(error) if is_missing_table(&error) || is_missing_column(&error) => Vec::new(),
            other => other?,
        };
        Ok(rows
            .iter()
            .filter_map(|row| {
                let version = str_at(row, 0);
                (!version.is_empty()).then(|| ReleaseWindow {
                    version,
                    first_seen_nanos: u128_at(row, 1),
                    last_seen_nanos: u128_at(row, 2),
                    span_count: u128_at(row, 3) as u64,
                })
            })
            .collect())
    }

    async fn service_catalog(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceCatalogRow>> {
        let schema = self
            .sql_with_schema_lenient("SELECT * FROM opentelemetry_traces LIMIT 0")
            .await?;
        let has_column = |name: &str| schema.columns.iter().any(|column| column == name);
        let latest_attr = |attr: &str, alias: &str| {
            let column = semconv::resource_column(attr);
            if has_column(&column) {
                format!(r#"MAX(t."{}") AS "{}""#, escape_ident(&column), alias)
            } else {
                format!(r#"NULL AS "{}""#, alias)
            }
        };
        let service_instance_column = semconv::resource_column(semconv::SERVICE_INSTANCE_ID);
        let instance_count = if has_column(&service_instance_column) {
            format!(
                "COUNT(DISTINCT {})",
                resource_attr_ident(semconv::SERVICE_INSTANCE_ID)
            )
        } else {
            "0".to_string()
        };
        // GreptimeDB 1.1 rejects `arg_max`, and aggregate `last_value` is not
        // timestamp-stable for native trace rows. Verified live on 2026-07-08:
        // use one max-timestamp CTE per service, join the matching row(s), then
        // aggregate duplicate latest rows with MAX.
        let sql = format!(
            r#"WITH latest AS (
                   SELECT "service_name", MAX("timestamp") AS "last_seen"
                   FROM opentelemetry_traces
                   WHERE "timestamp" >= {} AND "timestamp" <= {}
                   GROUP BY "service_name"
               ),
               counts AS (
                   SELECT "service_name", {instance_count} AS "instance_count"
                   FROM opentelemetry_traces
                   WHERE "timestamp" >= {} AND "timestamp" <= {}
                   GROUP BY "service_name"
               )
               SELECT l."service_name",
                      {},
                      {},
                      {},
                      {},
                      {},
                      {},
                      CAST(l."last_seen" AS BIGINT) AS "last_seen_nanos",
                      c."instance_count"
               FROM latest l
               JOIN opentelemetry_traces t
                 ON t."service_name" = l."service_name"
                AND t."timestamp" = l."last_seen"
               LEFT JOIN counts c ON c."service_name" = l."service_name"
               GROUP BY l."service_name", l."last_seen", c."instance_count"
               ORDER BY l."service_name" ASC
               LIMIT {}"#,
            sql_ts(*range.start()),
            sql_ts(*range.end()),
            sql_ts(*range.start()),
            sql_ts(*range.end()),
            latest_attr(semconv::SERVICE_VERSION, "service_version"),
            latest_attr(semconv::SERVICE_NAMESPACE, "service_namespace"),
            latest_attr(
                semconv::DEPLOYMENT_ENVIRONMENT_NAME,
                "deployment_environment"
            ),
            latest_attr(semconv::TELEMETRY_SDK_LANGUAGE, "telemetry_sdk_language"),
            latest_attr(semconv::TELEMETRY_SDK_NAME, "telemetry_sdk_name"),
            latest_attr(semconv::TELEMETRY_SDK_VERSION, "telemetry_sdk_version"),
            MAX_ROWS,
        );
        let rows = self.sql_lenient(&sql).await?;
        Ok(rows
            .iter()
            .map(|row| ServiceCatalogRow {
                name: str_at(row, 0),
                service_version: opt_nonempty_str_at(row, 1),
                service_namespace: opt_nonempty_str_at(row, 2),
                deployment_environment: opt_nonempty_str_at(row, 3),
                telemetry_sdk_language: opt_nonempty_str_at(row, 4),
                telemetry_sdk_name: opt_nonempty_str_at(row, 5),
                telemetry_sdk_version: opt_nonempty_str_at(row, 6),
                last_seen_nanos: u128_at(row, 7),
                instance_count: u128_at(row, 8) as u64,
            })
            .collect())
    }

    async fn span_red_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<SpanRed> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let clauses = trace_filter_clauses(service, &range);
        // Latest stable GreptimeDB accepts approx_percentile_cont(col, q);
        // verified through Parallax raw SQL for trace duration percentiles.
        let rows = self
            .sql_arrow_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "timestamp") AS BIGINT)
                          AS "bucket_ns",
                          COUNT(*) AS "spans",
                          SUM(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
                          AS "errors",
                          approx_percentile_cont("duration_nano", 0.50) AS "p50_ns",
                          approx_percentile_cont("duration_nano", 0.95) AS "p95_ns",
                          approx_percentile_cont("duration_nano", 0.99) AS "p99_ns"
                   FROM opentelemetry_traces WHERE {}
                   GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                clauses.join(" AND "),
            ))
            .await?;
        let mut red = SpanRed::default();
        for row in &rows {
            let ts_nanos = u128_at(row, 0);
            let spans = f64_at(row, 1);
            let errors = f64_at(row, 2);
            red.rate.push(SeriesPoint {
                ts_nanos,
                value: spans / step_secs as f64,
            });
            red.error_rate.push(SeriesPoint {
                ts_nanos,
                value: if spans == 0.0 { 0.0 } else { errors / spans },
            });
            red.p50.push(SeriesPoint {
                ts_nanos,
                value: f64_at(row, 3) / 1_000_000.0,
            });
            red.p95.push(SeriesPoint {
                ts_nanos,
                value: f64_at(row, 4) / 1_000_000.0,
            });
            red.p99.push(SeriesPoint {
                ts_nanos,
                value: f64_at(row, 5) / 1_000_000.0,
            });
        }
        Ok(red)
    }
}

#[async_trait::async_trait]
impl MetricAnalyticsStore for GreptimeStore {
    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
        run_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let sql_agg = match agg {
            MetricAgg::Avg => "avg",
            MetricAgg::Min => "min",
            MetricAgg::Max => "max",
            MetricAgg::Sum | MetricAgg::Rate => "sum",
        };
        // Run-scoped reads hit the `run_metric_points` extension table (ns time
        // index, `value` column); aggregate reads hit the per-metric native
        // table (ms `greptime_timestamp`, `greptime_value`, `service_name` tag).
        let rows = if let Some(run_id) = run_id {
            let service_clause = service
                .map(|svc| format!(r#" AND "service" = '{}'"#, escape(svc)))
                .unwrap_or_default();
            let name_filter = metric_name_sql_filter(r#""name""#, name);
            self.sql_arrow_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "ts") AS BIGINT)
                          AS "bucket_ns", {sql_agg}("value") AS "agg_value"
                   FROM run_metric_points
                   WHERE {name_filter} AND "run_id" = '{}'{service_clause}
                     AND "ts" >= {} AND "ts" <= {}
                   GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                escape(run_id),
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?
        } else {
            let Some(table) = self.metric_table_for_name(name, None).await? else {
                return Ok(Vec::new());
            };
            let service_clause = service
                .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
                .unwrap_or_default();
            self.sql_arrow_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", {sql_agg}("greptime_value") AS "agg_value"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
                escape_ident(&table),
                sql_ts(range.start() / 1_000_000),
                sql_ts(range.end() / 1_000_000),
            ))
            .await?
        };
        // Run-metric buckets are already nanos; native metric buckets are ms.
        let scale = if run_id.is_some() { 1 } else { 1_000_000 };
        let mut series: Vec<SeriesPoint> = rows
            .iter()
            .map(|row| SeriesPoint {
                ts_nanos: u128_at(row, 0) * scale,
                value: row.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
            .collect();
        if agg == MetricAgg::Rate {
            series = crate::adapter::rate_from_buckets(&series, step_secs * 1_000_000_000);
        }
        Ok(series)
    }

    async fn histogram_quantile(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        q: f64,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let series = self
            .histogram_quantiles(name, service, range, step_nanos, &[q])
            .await?;
        Ok(series.into_iter().next().unwrap_or_default())
    }

    async fn histogram_quantiles(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        quantiles: &[f64],
    ) -> anyhow::Result<Vec<Vec<SeriesPoint>>> {
        if quantiles.is_empty() {
            return Ok(Vec::new());
        }
        let Some(bucket_table) = self.metric_table_for_name(name, Some("_bucket")).await? else {
            return Ok(vec![Vec::new(); quantiles.len()]);
        };
        // Server-side date_bin + MAX per (window, le) = latest cumulative (plan 085).
        let service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let rows = self
            .sql_arrow_lenient(&Self::histogram_quantile_bucket_sql(
                &bucket_table,
                step_secs,
                range.start() / 1_000_000,
                range.end() / 1_000_000,
                &service_clause,
            ))
            .await?;
        let mut windows: BTreeMap<u128, BTreeMap<OrderedF64, f64>> = Default::default();
        for row in &rows {
            let ts_nanos = u128_at(row, 0) * 1_000_000;
            let le = row.get(1).and_then(|v| v.as_f64()).unwrap_or(f64::INFINITY);
            let cumulative = row.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);
            windows
                .entry(ts_nanos)
                .or_default()
                .insert(OrderedF64(le), cumulative);
        }
        let mut out = Vec::with_capacity(quantiles.len());
        for &q in quantiles {
            out.push(
                windows
                    .iter()
                    .map(|(ts_nanos, bounds)| SeriesPoint {
                        ts_nanos: *ts_nanos,
                        value: quantile_from_cumulative(bounds, q),
                    })
                    .collect(),
            );
        }
        Ok(out)
    }

    async fn metric_exemplars(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<MetricExemplarRow>> {
        let service_clause = service
            .map(|svc| format!(r#" AND "service" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT CAST("ts" AS BIGINT) AS "ts_nanos",
                          "service", "name", "value", "trace_id", "span_id", "run_id",
                          json_to_string("attributes")
                   FROM {METRIC_EXEMPLARS_TABLE}
                   WHERE "name" = '{}' AND "ts" >= {} AND "ts" <= {}{service_clause}
                   ORDER BY "ts" DESC LIMIT {}"#,
                escape(name),
                sql_ts(*range.start()),
                sql_ts(*range.end()),
                limit.min(MAX_ROWS)
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| MetricExemplarRow {
                ts_nanos: u128_at(row, 0),
                service: str_at(row, 1),
                name: str_at(row, 2),
                value: f64_at(row, 3),
                trace_id: str_at(row, 4),
                span_id: str_at(row, 5),
                run_id: opt_str_at(row, 6),
                attributes: json_at(row, 7),
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl crate::adapter::RunStore for GreptimeStore {
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>> {
        let rows = self
            .sql(&format!(
                r#"SELECT CAST("ts" AS BIGINT) AS "ts_nanos", "service", "fingerprint", "error_type",
                          "message", "stacktrace", "source", "trace_id", "span_id",
                          json_to_string("attributes")
                   FROM error_events WHERE "fingerprint" = '{}' AND "ts" >= {} AND "ts" <= {}
                   ORDER BY "ts" DESC LIMIT {limit}"#,
                escape(fingerprint),
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| ErrorEventRow {
                ts_nanos: u128_at(row, 0),
                service: str_at(row, 1),
                fingerprint: str_at(row, 2),
                error_type: str_at(row, 3),
                message: str_at(row, 4),
                stacktrace: opt_str_at(row, 5),
                source: serde_json::from_value(serde_json::Value::String(str_at(row, 6)))
                    .unwrap_or(ErrorSource::LogRecord),
                trace_id: str_at(row, 7),
                span_id: str_at(row, 8),
                attributes: json_at(row, 9),
            })
            .collect())
    }

    async fn observed_runs(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<crate::adapter::ObservedRun>> {
        let mut runs: HashMap<String, crate::adapter::ObservedRun> = HashMap::new();
        let start = sql_ts(*range.start());
        let end = sql_ts(*range.end());
        let trace_run_column = resource_attr_ident(semconv::PARALLAX_RUN_ID);
        let native_span_rows = match self
            .sql_lenient(&format!(
                r#"SELECT {trace_run_column} AS "run_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT "span_id") AS "n",
                          MAX("service_name") AS "svc"
                   FROM opentelemetry_traces
                   WHERE {trace_run_column} IS NOT NULL AND {trace_run_column} != ''
                     AND "timestamp" >= {start} AND "timestamp" <= {end}
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT {limit}"#
            ))
            .await
        {
            Ok(rows) => rows,
            Err(error) if is_missing_column(&error) => Vec::new(),
            Err(error) => return Err(error),
        };
        let mut native_span_run_ids = BTreeSet::new();
        for row in &native_span_rows {
            if let Some(run_id) = absorb_observed_run(&mut runs, row, true) {
                native_span_run_ids.insert(run_id);
            }
        }
        if native_span_rows.len() >= limit {
            let mut runs: Vec<_> = runs.into_values().collect();
            runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
            runs.truncate(limit);
            return Ok(runs);
        }
        let run_col = wire_attr_ident(semconv::PARALLAX_RUN_ID);
        let log_svc = log_service_name_expr();
        let sources = [
            (
                format!(
                    r#"SELECT l.{run_col} AS "run_id",
                          CAST(MIN(s."timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX(s."timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT s."span_id") AS "n",
                          MAX(s."service_name") AS "svc"
                   FROM opentelemetry_logs l
                   JOIN opentelemetry_traces s ON s."trace_id" = l."trace_id"
                   WHERE l.{run_col} IS NOT NULL
                     AND l.{run_col} != ''
                     AND l."timestamp" >= {start} AND l."timestamp" <= {end}
                     AND s."timestamp" >= {start} AND s."timestamp" <= {end}
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#
                ),
                true,
            ),
            (
                format!(
                    r#"SELECT {run_col} AS "run_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(*) AS "n",
                          MAX({log_svc}) AS "svc"
                   FROM opentelemetry_logs
                   WHERE {run_col} IS NOT NULL AND {run_col} != ''
                     AND "timestamp" >= {start} AND "timestamp" <= {end}
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#
                ),
                false,
            ),
        ];
        for (query, is_span) in sources {
            let rows = match self.sql_lenient(&format!("{query}{limit}")).await {
                Ok(rows) => rows,
                Err(error) if is_missing_column(&error) => Vec::new(),
                Err(error) => return Err(error),
            };
            for row in &rows {
                if is_span && native_span_run_ids.contains(&str_at(row, 0)) {
                    continue;
                }
                absorb_observed_run(&mut runs, row, is_span);
            }
        }
        let mut runs: Vec<_> = runs.into_values().collect();
        runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
        runs.truncate(limit);
        Ok(runs)
    }
}

#[async_trait::async_trait]
impl crate::adapter::TraceAnalyticsStore for GreptimeStore {
    async fn traces_search(
        &self,
        query: &crate::adapter::TraceQuery,
    ) -> anyhow::Result<crate::adapter::TraceList> {
        // One representative span per trace — its root (no parent), else the
        // earliest span when no root was stored (all-INTERNAL traces).
        //
        // Scan window — also bounds representative, participation, and
        // in-window span_count/has_error (plan 075, aligned with memory).
        let mut scan = Vec::new();
        if let Some(from) = query.from_nanos {
            scan.push(format!(r#""timestamp" >= {}"#, sql_ts(from)));
        }
        if let Some(to) = query.to_nanos {
            scan.push(format!(r#""timestamp" <= {}"#, sql_ts(to)));
        }
        let scan_where = if scan.is_empty() {
            "1 = 1".to_string()
        } else {
            scan.join(" AND ")
        };
        // `service` matches any in-window trace the service participates in.
        let participation = match &query.service {
            Some(service) => format!(
                r#" AND "trace_id" IN (SELECT "trace_id" FROM opentelemetry_traces WHERE "service_name" = '{}' AND {scan_where})"#,
                escape(service)
            ),
            None => String::new(),
        };
        // Representative-span filters, applied after the per-trace pick.
        let mut rep = vec!["\"rn\" = 1".to_string()];
        if let Some(min) = query.min_duration_ns {
            rep.push(format!(r#""dur" >= {}"#, u64::try_from(min)?));
        }
        if let Some(max) = query.max_duration_ns {
            rep.push(format!(r#""dur" <= {}"#, u64::try_from(max)?));
        }
        if let Some(needle) = &query.name_contains {
            let escaped = escape(needle).replace('%', r"\%").replace('_', r"\_");
            rep.push(format!(r#""span_name" LIKE '%{escaped}%' ESCAPE '\'"#));
        }
        if query.error_only {
            rep.push(r#""has_error" > 0"#.to_string());
        }
        let order = match query.sort {
            crate::adapter::TraceSort::StartDesc => r#""ts_nanos" DESC"#,
            crate::adapter::TraceSort::DurationDesc => r#""dur" DESC"#,
            crate::adapter::TraceSort::DurationAsc => r#""dur" ASC"#,
            crate::adapter::TraceSort::SpanCountDesc => r#""span_count" DESC"#,
        };
        let (listed, page_sql) = Self::traces_search_sql(
            &scan_where,
            &participation,
            &rep.join(" AND "),
            order,
            query.limit,
            query.offset,
        );
        // One happy-path execution: page rows + COUNT(*) OVER () as last column.
        // Empty page (offset past end) falls back to a count-only query.
        let roots = self.sql_arrow_lenient(&page_sql).await?;
        let total = if let Some(row) = roots.first() {
            // listed projects 7 columns; window total is column index 7.
            u128_at(row, 7) as u64
        } else {
            self.sql_lenient(&format!(r#"SELECT COUNT(*) AS "total" FROM ({listed})"#))
                .await?
                .first()
                .map(|r| u128_at(r, 0) as u64)
                .unwrap_or(0)
        };
        let traces: Vec<_> = roots
            .iter()
            .map(|row| crate::adapter::TraceSummary {
                trace_id: str_at(row, 0),
                root_name: str_at(row, 1),
                service: str_at(row, 2),
                start_nanos: u128_at(row, 3),
                duration_ns: u128_at(row, 4),
                span_count: u128_at(row, 5) as u64,
                has_error: u128_at(row, 6) > 0,
            })
            .collect();
        Ok(crate::adapter::TraceList {
            items: traces,
            total,
        })
    }

    async fn attribute_compare(
        &self,
        selected: RangeInclusive<u128>,
        baseline: RangeInclusive<u128>,
        service: Option<&str>,
        error_only: bool,
        keys: &[String],
        top_n: usize,
    ) -> anyhow::Result<Vec<AttributeCompareRow>> {
        let limit = top_n.min(ATTRIBUTE_COMPARE_TOP_N_CAP);
        if limit == 0 {
            return Ok(Vec::new());
        }

        let available = self.discover_span_attribute_keys().await?;
        let candidate_keys: Vec<String> = if keys.is_empty() {
            available
                .into_iter()
                .filter(|key| attribute_compare_key_allowed(key))
                .take(ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT)
                .collect()
        } else {
            keys.iter()
                .map(|key| key.trim())
                .filter(|key| attribute_compare_key_allowed(key))
                .filter(|key| available.contains(*key))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .take(ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT)
                .map(str::to_string)
                .collect()
        };

        // Concurrent per-key fan-out in chunks of 8 (plan 075 Step 3).
        // Each future runs selected+baseline pair with try_join!.
        let mut per_key: Vec<(
            String,
            u64,
            BTreeMap<String, u64>,
            u64,
            BTreeMap<String, u64>,
        )> = Vec::with_capacity(candidate_keys.len());
        for chunk in candidate_keys.chunks(8) {
            let futs =
                chunk.iter().map(|key| {
                    let key = key.clone();
                    let selected = selected.clone();
                    let baseline = baseline.clone();
                    async move {
                        let ((selected_total, selected_counts), (baseline_total, baseline_counts)) =
                            tokio::try_join!(
                                self.span_attribute_counts(&key, &selected, service, error_only),
                                self.span_attribute_counts(&key, &baseline, service, error_only),
                            )?;
                        Ok::<_, anyhow::Error>((
                            key,
                            selected_total,
                            selected_counts,
                            baseline_total,
                            baseline_counts,
                        ))
                    }
                });
            let chunk_results = futures_util::future::try_join_all(futs).await?;
            per_key.extend(chunk_results);
        }

        let mut rows = Vec::new();
        // Preserve original key order when assembling (sort reorders by score).
        for (key, selected_total, selected_counts, baseline_total, baseline_counts) in per_key {
            for (value, selected_count) in selected_counts {
                let baseline_count = baseline_counts.get(&value).copied().unwrap_or(0);
                let score = attribute_compare_score(
                    selected_count,
                    selected_total,
                    baseline_count,
                    baseline_total,
                );
                if score > 0.0 {
                    rows.push(AttributeCompareRow {
                        key: key.clone(),
                        value,
                        selected_count,
                        selected_total,
                        baseline_count,
                        baseline_total,
                        score,
                    });
                }
            }
        }

        rows.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| b.selected_count.cmp(&a.selected_count))
                .then_with(|| a.key.cmp(&b.key))
                .then_with(|| a.value.cmp(&b.value))
        });
        rows.truncate(limit);
        Ok(rows)
    }

    async fn span_field_keys(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<FieldKey>> {
        let mut columns = self.span_field_columns().await?;
        columns.truncate(FIELD_KEYS_CAP);
        if columns.is_empty() {
            return Ok(Vec::new());
        }

        let mut projections = vec!["COUNT(*) AS \"__total\"".to_string()];
        for (index, column) in columns.iter().enumerate() {
            projections.push(format!(
                r#"SUM(CASE WHEN {} IS NOT NULL THEN 1 ELSE 0 END) AS "k{index}""#,
                quoted_field_column(column)
            ));
        }
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT {}
                   FROM opentelemetry_traces
                   WHERE "timestamp" >= {} AND "timestamp" <= {}"#,
                projections.join(", "),
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ))
            .await?;
        let Some(row) = rows.first() else {
            return Ok(Vec::new());
        };
        let row_count = u128_at(row, 0) as u64;

        Ok(columns
            .into_iter()
            .enumerate()
            .filter_map(|(index, column)| {
                let non_null_count = u128_at(row, index + 1) as u64;
                (non_null_count > 0).then(|| FieldKey {
                    namespace: field_key_namespace(&column.key),
                    coverage: if row_count == 0 {
                        0.0
                    } else {
                        non_null_count as f64 / row_count as f64
                    },
                    is_identifier: field_key_identifier_like(&column.key),
                    key: column.key,
                    source: column.source,
                    row_count,
                    non_null_count,
                })
            })
            .collect())
    }

    async fn span_field_stats(
        &self,
        key: &str,
        range: RangeInclusive<u128>,
        service: Option<&str>,
    ) -> anyhow::Result<FieldStats> {
        anyhow::ensure!(span_field_key_allowed(key), "invalid field key");
        let Some(column) = self
            .span_field_columns()
            .await?
            .into_iter()
            .find(|column| column.key == key)
        else {
            anyhow::bail!("unknown span field key");
        };
        let column_ident = quoted_field_column(&column);
        let mut clauses = vec![format!(
            r#""timestamp" >= {} AND "timestamp" <= {}"#,
            sql_ts(*range.start()),
            sql_ts(*range.end())
        )];
        if let Some(service) = service {
            clauses.push(format!(r#""service_name" = '{}'"#, escape(service)));
        }
        let base_where = clauses.join(" AND ");
        let totals = self
            .sql_lenient(&format!(
                r#"SELECT COUNT(*) AS "total",
                          SUM(CASE WHEN {column_ident} IS NOT NULL THEN 1 ELSE 0 END) AS "non_null"
                   FROM opentelemetry_traces
                   WHERE {base_where}"#
            ))
            .await?;
        let row_count = totals
            .first()
            .map(|row| u128_at(row, 0) as u64)
            .unwrap_or(0);
        let non_null_count = totals
            .first()
            .map(|row| u128_at(row, 1) as u64)
            .unwrap_or(0);
        let value_expr = format!("CAST({column_ident} AS STRING)");
        let sample_where = format!("{base_where} AND {column_ident} IS NOT NULL");
        let sample_sql = Self::span_field_stats_sample_sql(&value_expr, &sample_where);
        let value_rows = self
            .sql_lenient(&Self::span_field_stats_top_values_sql(&sample_sql))
            .await?;
        let distinct_count = value_rows
            .first()
            .map(|row| u128_at(row, 2) as u64)
            .unwrap_or(0);
        let top_values = value_rows
            .iter()
            .filter_map(|row| {
                let value = field_value_display(&str_at(row, 0))?;
                Some(FieldValueCount {
                    value,
                    count: u128_at(row, 1) as u64,
                })
            })
            .collect();
        let sample_count = non_null_count.min(MAX_ROWS as u64);
        let is_identifier = field_key_identifier_like(key)
            || (sample_count >= 20 && distinct_count >= sample_count.saturating_sub(1));
        Ok(FieldStats {
            key: key.to_string(),
            namespace: field_key_namespace(key),
            source: column.source,
            row_count,
            non_null_count,
            distinct_count,
            coverage: if row_count == 0 {
                0.0
            } else {
                non_null_count as f64 / row_count as f64
            },
            capped: non_null_count > MAX_ROWS as u64,
            is_identifier,
            top_values,
        })
    }

    async fn service_map(
        &self,
        range: RangeInclusive<u128>,
        max_traces: usize,
    ) -> anyhow::Result<Vec<ServiceEdge>> {
        let trace_limit = max_traces.min(SERVICE_MAP_TRACE_CAP);
        if trace_limit == 0 {
            return Ok(Vec::new());
        }
        let trace_rows = self
            .sql_lenient(&format!(
                r#"SELECT "trace_id", MAX("timestamp") AS "last_seen"
                   FROM opentelemetry_traces
                   WHERE "timestamp" >= {} AND "timestamp" <= {}
                   GROUP BY "trace_id"
                   ORDER BY "last_seen" DESC, "trace_id" ASC
                   LIMIT {trace_limit}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ))
            .await?;
        let trace_ids: Vec<String> = trace_rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|trace_id| !trace_id.is_empty())
            .collect();
        if trace_ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_list = trace_ids
            .iter()
            .map(|trace_id| format!("'{}'", escape(trace_id)))
            .collect::<Vec<_>>()
            .join(",");
        let rows = self
            .sql_arrow_lenient(&Self::service_map_edges_sql(&id_list, &range))
            .await?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let source = str_at(row, 0);
                let target = str_at(row, 1);
                if source.is_empty() || target.is_empty() || source == target {
                    return None;
                }
                Some(ServiceEdge {
                    source,
                    target,
                    call_count: u128_at(row, 2) as u64,
                    error_count: u128_at(row, 3) as u64,
                    p50_ms: f64_at(row, 4) / 1_000_000.0,
                    p95_ms: f64_at(row, 5) / 1_000_000.0,
                })
            })
            .collect())
    }

    async fn error_events_by_traces(
        &self,
        trace_ids: &[String],
        limit: usize,
    ) -> anyhow::Result<Vec<ErrorEventRow>> {
        if trace_ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_list = trace_ids
            .iter()
            .map(|t| format!("'{}'", escape(t)))
            .collect::<Vec<_>>()
            .join(",");
        let rows = self
            .sql(&format!(
                r#"SELECT CAST("ts" AS BIGINT) AS "ts_nanos", "service", "fingerprint", "error_type",
                          "message", "stacktrace", "source", "trace_id", "span_id",
                          json_to_string("attributes")
                   FROM error_events WHERE "trace_id" IN ({id_list})
                   ORDER BY "ts" DESC LIMIT {limit}"#
            ))
            .await?;
        Ok(rows.iter().map(|row| error_event_from_row(row)).collect())
    }
}

#[async_trait::async_trait]
impl crate::adapter::LogAnalyticsStore for GreptimeStore {
    async fn logs_search(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<LogRow>> {
        let clauses =
            log_filter_clauses(service, &range, severity_min, severity_max, body_contains);
        self.select_logs(
            &clauses.join(" AND "),
            r#" ORDER BY "timestamp" DESC"#,
            &format!(" LIMIT {limit}"),
        )
        .await
    }
}

#[async_trait::async_trait]
impl crate::adapter::RuntimeMetricStore for GreptimeStore {
    async fn metric_series_grouped(
        &self,
        name: &str,
        service: Option<&str>,
        group_by: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<Vec<(String, Vec<SeriesPoint>)>> {
        anyhow::ensure!(
            metric_group_label_allowed(group_by),
            "high-cardinality identifier - filter, don't group"
        );
        let Some((table, labels)) = self.resolved_metric_table(name).await? else {
            return Ok(Vec::new());
        };
        anyhow::ensure!(
            labels.iter().any(|label| label == group_by),
            "unknown metric label"
        );
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let sql_agg = match agg {
            MetricAgg::Avg => "avg",
            MetricAgg::Min => "min",
            MetricAgg::Max => "max",
            MetricAgg::Sum | MetricAgg::Rate => "sum",
        };
        let service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        // native: metric-engine tags are real columns (resource attrs promoted
        // to tags); group on the quoted tag column, missing → "(none)".
        let group_col = format!(r#""{}""#, escape_ident(group_by));
        let rows = self
            .sql_arrow_lenient(&format!(
                r#"SELECT COALESCE(CAST({group_col} AS STRING), '(none)') AS "grp",
                          CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", {sql_agg}("greptime_value") AS "agg_value"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "grp", "bucket_ms" ORDER BY "grp", "bucket_ms""#,
                escape_ident(&table),
                sql_ts(range.start() / 1_000_000),
                sql_ts(range.end() / 1_000_000),
            ))
            .await?;
        let mut groups: BTreeMap<String, Vec<SeriesPoint>> = Default::default();
        for row in &rows {
            groups.entry(str_at(row, 0)).or_default().push(SeriesPoint {
                ts_nanos: u128_at(row, 1) * 1_000_000,
                value: row.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0),
            });
        }
        Ok(groups
            .into_iter()
            .map(|(group, series)| {
                let series = if agg == MetricAgg::Rate {
                    crate::adapter::rate_from_buckets(&series, step_secs * 1_000_000_000)
                } else {
                    series
                };
                (group, series)
            })
            .collect())
    }

    async fn runtime_snapshot(
        &self,
        service: Option<&str>,
        run_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<RuntimeMetricSeries>> {
        // Filter runtime families first, then fetch series concurrently
        // (plan 075 Step 3) in chunks of 8.
        let metrics: Vec<(String, &'static str)> = self
            .metric_names(range.clone())
            .await?
            .into_iter()
            .filter_map(|metric| runtime_metric_family(&metric).map(|family| (metric, family)))
            .collect();
        let mut rows = Vec::with_capacity(metrics.len());
        for chunk in metrics.chunks(8) {
            let futs = chunk.iter().map(|(metric, family)| {
                let metric = metric.clone();
                let family = *family;
                let range = range.clone();
                async move {
                    let points = self
                        .metric_series(&metric, service, run_id, range, step_nanos, MetricAgg::Avg)
                        .await?;
                    Ok::<_, anyhow::Error>((metric, family, points))
                }
            });
            let chunk_results = futures_util::future::try_join_all(futs).await?;
            for (metric, family, points) in chunk_results {
                if points.is_empty() {
                    continue;
                }
                rows.push(RuntimeMetricSeries {
                    family: family.to_string(),
                    metric: metric.clone(),
                    unit: runtime_metric_unit(&metric),
                    points,
                });
            }
        }
        rows.sort_by(|a, b| a.family.cmp(&b.family).then(a.metric.cmp(&b.metric)));
        Ok(rows)
    }

    async fn histogram_count_series(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        // Resolve the real count sibling table (dotted OTel names → underscore
        // native table + `_count` suffix), same as histogram_quantile.
        let Some(count_table) = self.metric_table_for_name(name, Some("_count")).await? else {
            return Ok(Vec::new());
        };
        // native: the `<name>_count` sibling table holds the per-sample count
        // as `greptime_value`; sum it per window for the request-rate numerator.
        let rows = self
            .sql_arrow_lenient(&Self::histogram_count_series_sql(
                &count_table,
                step_secs,
                range.start() / 1_000_000,
                range.end() / 1_000_000,
                &service_clause,
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| SeriesPoint {
                ts_nanos: u128_at(row, 0) * 1_000_000,
                value: row.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl crate::adapter::ErrorAnalyticsStore for GreptimeStore {
    async fn error_count_series(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let rows = self
            .sql(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "ts") AS BIGINT)
                          AS "bucket_ns", COUNT(*) AS "n"
                   FROM error_events
                   WHERE "service" = '{}' AND "ts" >= {} AND "ts" <= {}
                   GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                escape(service),
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| SeriesPoint {
                ts_nanos: u128_at(row, 0),
                value: row.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl crate::adapter::LogCountStore for GreptimeStore {
    async fn log_count_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let clauses =
            log_filter_clauses(service, &range, severity_min, severity_max, body_contains);
        let rows = self
            .sql_arrow_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "timestamp") AS BIGINT)
                          AS "bucket_ns", COUNT(*) AS "n"
                   FROM opentelemetry_logs WHERE {}
                   GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                clauses.join(" AND ")
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| SeriesPoint {
                ts_nanos: u128_at(row, 0),
                value: row.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
            .collect())
    }
}

#[async_trait::async_trait]
impl crate::adapter::RawSqlStore for GreptimeStore {
    async fn raw_sql(&self, query: &str) -> anyhow::Result<crate::adapter::SqlResult> {
        anyhow::ensure!(
            raw_sql_read_only(query),
            "raw_sql: read-only statements only"
        );
        self.sql_with_schema(query).await
    }
}

impl GreptimeStore {
    async fn span_field_columns(&self) -> anyhow::Result<Vec<SpanFieldColumn>> {
        let rows = self
            .sql(
                r#"SELECT "column_name" FROM information_schema.columns
                   WHERE "table_schema" = 'public'
                     AND "table_name" = 'opentelemetry_traces'
                     AND ("column_name" LIKE 'span_attributes.%'
                          OR "column_name" LIKE 'resource_attributes.%')
                   ORDER BY "column_name""#,
            )
            .await?;
        let mut columns = vec![SpanFieldColumn {
            key: format!("resource.{}", semconv::SERVICE_NAME),
            column: "service_name".to_string(),
            source: FieldSource::Resource,
        }];
        let service_name_key = format!("resource.{}", semconv::SERVICE_NAME);
        columns.extend(
            rows.iter()
                .filter_map(|row| span_field_column_from_name(&str_at(row, 0)))
                .filter(|column| column.key != service_name_key),
        );
        Ok(columns)
    }

    async fn discover_span_attribute_keys(&self) -> anyhow::Result<BTreeSet<String>> {
        let rows = self
            .sql(
                r#"SELECT "column_name" FROM information_schema.columns
                   WHERE "table_schema" = 'public'
                     AND "table_name" = 'opentelemetry_traces'
                     AND "column_name" LIKE 'span_attributes.%'
                   ORDER BY "column_name""#,
            )
            .await?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                str_at(row, 0)
                    .strip_prefix("span_attributes.")
                    .map(str::to_string)
            })
            .collect())
    }

    async fn span_attribute_counts(
        &self,
        key: &str,
        range: &RangeInclusive<u128>,
        service: Option<&str>,
        error_only: bool,
    ) -> anyhow::Result<(u64, BTreeMap<String, u64>)> {
        let rows = self
            .sql_lenient(&Self::span_attribute_counts_sql(
                key, range, service, error_only,
            ))
            .await?;

        let mut total = 0;
        let mut counts = BTreeMap::new();
        for row in &rows {
            let value = str_at(row, 0);
            if !attribute_compare_value_allowed(&value) {
                continue;
            }
            let count = u128_at(row, 1) as u64;
            total += count;
            counts.insert(value, count);
        }
        Ok((total, counts))
    }

    /// Discover the base metric names from the schema: every public table that
    /// is neither a native otel table, an extension table, the metric-engine
    /// physical table, nor a system table. Histogram siblings collapse to the
    /// base name (`<name>_bucket`/`_count`/`_sum` → `<name>`), sorted unique.
    async fn discover_metric_names(
        &self,
        range: &RangeInclusive<u128>,
    ) -> anyhow::Result<BTreeSet<String>> {
        const RESERVED: &[&str] = &[
            "opentelemetry_traces",
            "opentelemetry_traces_services",
            "opentelemetry_traces_operations",
            "opentelemetry_logs",
            "error_events",
            "run_metric_points",
            METRIC_EXEMPLARS_TABLE,
            "greptime_physical_table",
        ];
        let rows = self
            .sql(
                r#"SELECT "table_name" FROM information_schema.tables
                   WHERE "table_schema" = 'public'"#,
            )
            .await?;
        let tables = rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|table| {
                !table.is_empty()
                    && !RESERVED.contains(&table.as_str())
                    && !table.starts_with("opentelemetry_")
            })
            .collect::<Vec<_>>();
        let table_set = tables.iter().cloned().collect::<BTreeSet<_>>();
        let mut names = BTreeSet::new();
        for table in tables {
            let base = if let Some(base) = table.strip_suffix("_bucket") {
                base.to_string()
            } else if let Some(base) = table.strip_suffix("_count") {
                if table_set.contains(&format!("{base}_bucket")) {
                    base.to_string()
                } else {
                    table.clone()
                }
            } else if let Some(base) = table.strip_suffix("_sum") {
                if table_set.contains(&format!("{base}_bucket")) {
                    base.to_string()
                } else {
                    table.clone()
                }
            } else {
                table.clone()
            };
            let display = runtime_display_name(&base).unwrap_or_else(|| base.to_string());
            names.insert(canonical_metric_display_name(&display));
        }

        // Run-scoped extension rows keep the original OTLP metric name. Union
        // them so run dashboards can use dotted names even when native table
        // names are Prometheus-normalized.
        for row in self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT "name" FROM run_metric_points
                   WHERE "name" IS NOT NULL AND "name" != ''
                     AND "ts" >= {} AND "ts" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?
        {
            names.insert(canonical_metric_display_name(&str_at(&row, 0)));
        }
        Ok(names)
    }
}

/// A total-ordering wrapper for histogram bucket bounds (`le`), so they can key
/// a `BTreeMap`. NaN sorts last; bounds are well-formed finite values or +inf.
#[derive(PartialEq)]
struct OrderedF64(f64);

impl Eq for OrderedF64 {}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Greater)
    }
}

/// Linear-interpolated quantile from native cumulative `le`-bucket counts
/// (`bound → cumulative count ≤ bound`, ascending). Mirrors the explicit-bucket
/// math the in-memory store uses, adapted to native cumulative buckets.
fn quantile_from_cumulative(bounds: &BTreeMap<OrderedF64, f64>, q: f64) -> f64 {
    let Some((_, &total)) = bounds.iter().next_back() else {
        return 0.0;
    };
    if total <= 0.0 {
        return 0.0;
    }
    let target = q.clamp(0.0, 1.0) * total;
    let mut prev_bound = 0.0;
    let mut prev_cumulative = 0.0;
    for (OrderedF64(bound), &cumulative) in bounds {
        if cumulative >= target {
            let upper = if bound.is_finite() {
                *bound
            } else {
                prev_bound
            };
            let span = cumulative - prev_cumulative;
            let within = if span <= 0.0 {
                0.0
            } else {
                (target - prev_cumulative) / span
            };
            return prev_bound + (upper - prev_bound) * within;
        }
        prev_bound = if bound.is_finite() {
            *bound
        } else {
            prev_bound
        };
        prev_cumulative = cumulative;
    }
    prev_bound
}

/// Shared row → `ErrorEventRow` projection (fingerprint + trace-set reads).
fn error_event_from_row(row: &[serde_json::Value]) -> ErrorEventRow {
    ErrorEventRow {
        ts_nanos: u128_at(row, 0),
        service: str_at(row, 1),
        fingerprint: str_at(row, 2),
        error_type: str_at(row, 3),
        message: str_at(row, 4),
        stacktrace: opt_str_at(row, 5),
        source: serde_json::from_value(serde_json::Value::String(str_at(row, 6)))
            .unwrap_or(ErrorSource::LogRecord),
        trace_id: str_at(row, 7),
        span_id: str_at(row, 8),
        attributes: json_at(row, 9),
    }
}

#[cfg(test)]
mod tests;
