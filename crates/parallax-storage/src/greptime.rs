//! GreptimeDB `TelemetryStore` adapter: SQL over the HTTP API, DDL from the
//! implementation spec §5. All engine-specific SQL lives in this module.

use crate::adapter::{
    ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT, ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow,
    FIELD_KEYS_CAP, FIELD_TOP_VALUES_CAP, FieldKey, FieldSource, FieldStats, FieldValueCount,
    MAX_ROWS, OverviewTotals, ReleaseWindow, RuntimeMetricSeries, SERVICE_MAP_TRACE_CAP,
    ServiceCatalogRow, ServiceEdge, ServiceSummary, SignalKind, SpanRed, TelemetryStore,
    attribute_compare_key_allowed, attribute_compare_score, attribute_compare_value_allowed,
    field_key_identifier_like, field_key_namespace, field_value_display,
    metric_group_label_allowed, runtime_metric_family, runtime_metric_unit, span_field_key_allowed,
};
use crate::model::*;
use parallax_proto::semconv;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Client-side HTTP deadline for all GreptimeDB requests (reads + OTLP forwards).
/// Slightly above the SQL `X-Greptime-Timeout` so the engine can return a
/// structured timeout before reqwest aborts the socket.
const HTTP_CLIENT_TIMEOUT: Duration = Duration::from_secs(70);
/// Server-side query deadline sent on SQL reads only (not on OTLP forwards).
const SQL_QUERY_TIMEOUT_HEADER: &str = "60s";

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
}

fn escape(text: &str) -> String {
    text.replace('\'', "''")
}

/// Escape a value for inclusion inside a double-quoted SQL identifier.
fn escape_ident(text: &str) -> String {
    text.replace('"', "\"\"")
}

fn quoted_ident(text: &str) -> String {
    format!(r#""{}""#, escape_ident(text))
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
        let page = format!(
            r#"SELECT * FROM ({listed}) ORDER BY {order} LIMIT {limit} OFFSET {offset}"#
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

    fn histogram_quantile_bucket_sql(
        bucket_table: &str,
        range_start_ms: u128,
        range_end_ms: u128,
        service_clause: &str,
    ) -> String {
        format!(
            r#"SELECT CAST("greptime_timestamp" AS BIGINT) AS "ts_ms",
                          CAST("le" AS DOUBLE) AS "le", "greptime_value" AS "cumulative"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   ORDER BY "greptime_timestamp" ASC"#,
            escape_ident(bucket_table),
            sql_ts(range_start_ms),
            sql_ts(range_end_ms),
        )
    }

    fn select_spans_sql(where_clause: &str, order: &str, limit_clause: &str) -> String {
        format!(
            r#"SELECT * FROM opentelemetry_traces WHERE {where_clause}{order}{limit_clause}"#
        )
    }

    fn select_logs_sql(where_clause: &str, order: &str, limit_clause: &str) -> String {
        format!(
            r#"SELECT CAST("timestamp" AS BIGINT) AS "ts_nanos",
                          {} AS "service",
                          "severity_number", "severity_text", "body", "trace_id", "span_id",
                          {}, "scope_name",
                          json_to_string("log_attributes"),
                          json_to_string("resource_attributes"),
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
        range: &std::ops::RangeInclusive<u128>,
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

fn resource_attr_ident(attr: &str) -> String {
    quoted_ident(&semconv::resource_column(attr))
}

fn wire_attr_ident(attr: &str) -> String {
    quoted_ident(attr)
}

fn resource_json_get(attr: &str) -> String {
    format!(
        r#"json_get_string("resource_attributes", '{}')"#,
        semconv::resource_json_path(attr)
    )
}

/// Prefer extract-key column `"service.name"` (TAG); fall back to resource JSON
/// for pre-084 rows. extract-keys names columns after the OTLP attribute key.
fn log_service_name_expr() -> String {
    format!(
        r#"COALESCE("service.name", {})"#,
        resource_json_get(semconv::SERVICE_NAME)
    )
}

// Greptime metric-engine point tables discovered with live DESCRIBE expose
// `greptime_timestamp` and `greptime_value`; explicit histogram bucket tables
// add `le`. They are bookkeeping, not groupable metric labels.
const METRIC_BOOKKEEPING_COLUMNS: &[&str] = &["greptime_timestamp", "greptime_value", "le"];

fn native_metric_base(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn metric_table_candidates(name: &str, suffix: Option<&str>) -> Vec<String> {
    let suffix = suffix.unwrap_or_default();
    let mut bases = vec![name.to_string()];
    let native = native_metric_base(name);
    if native != name {
        bases.push(native.clone());
    }
    if !native.ends_with("_total") {
        bases.push(format!("{native}_total"));
    }
    for unit_suffix in ["_ratio", "_bytes", "_seconds", "_nanoseconds_total"] {
        if !native.ends_with(unit_suffix) {
            bases.push(format!("{native}{unit_suffix}"));
        }
    }

    let mut candidates = Vec::new();
    for base in bases {
        let candidate = format!("{base}{suffix}");
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn runtime_display_name(base: &str) -> Option<String> {
    const PREFIXES: &[(&str, &str)] = &[
        ("process_", "process."),
        ("system_", "system."),
        ("jvm_", "jvm."),
        ("container_", "container."),
        ("db_client_connection_", "db.client.connection."),
    ];
    if let Some(rest) = base.strip_prefix("tokio_runtime_") {
        return Some(format!("tokio.runtime.{rest}"));
    }
    PREFIXES.iter().find_map(|(native, display)| {
        base.strip_prefix(native)
            .map(|rest| format!("{display}{}", rest.replace('_', ".")))
    })
}

const METRIC_DISPLAY_ALIASES: &[(&str, &str)] = &[
    ("tokio.runtime.alive.tasks", "tokio.runtime.alive_tasks"),
    (
        "tokio.runtime.blocking.pool.depth",
        "tokio.runtime.blocking_pool_depth",
    ),
    (
        "tokio.runtime.global.queue.depth",
        "tokio.runtime.global_queue_depth",
    ),
    (
        "tokio.runtime.total.busy.duration.ms",
        "tokio.runtime.total_busy_duration_ms",
    ),
    (
        "tokio.runtime.total.park.count",
        "tokio.runtime.total_park_count",
    ),
    ("tokio.runtime.workers.count", "tokio.runtime.workers_count"),
    ("process.cpu.utilization.ratio", "process.cpu.utilization"),
    ("process.memory.usage.bytes", "process.memory.usage"),
];

fn canonical_metric_display_name(name: &str) -> String {
    METRIC_DISPLAY_ALIASES
        .iter()
        .find_map(|(legacy, canonical)| (*legacy == name).then_some((*canonical).to_string()))
        .unwrap_or_else(|| name.to_string())
}

fn metric_name_query_names(name: &str) -> Vec<String> {
    let mut names = vec![name.to_string(), canonical_metric_display_name(name)];
    for (legacy, canonical) in METRIC_DISPLAY_ALIASES {
        if *canonical == name {
            names.push((*legacy).to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

fn metric_name_sql_filter(column: &str, name: &str) -> String {
    let names = metric_name_query_names(name);
    if names.len() == 1 {
        format!(r#"{column} = '{}'"#, escape(&names[0]))
    } else {
        let quoted = names
            .iter()
            .map(|name| format!("'{}'", escape(name)))
            .collect::<Vec<_>>()
            .join(",");
        format!("{column} IN ({quoted})")
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
            format!(
                r#"CREATE TABLE IF NOT EXISTS metric_exemplars (
                   "ts" TIMESTAMP(9) NOT NULL,
                   "service" STRING, "name" STRING, "value" DOUBLE,
                   "trace_id" STRING, "span_id" STRING, "run_id" STRING SKIPPING INDEX,
                   "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name", "trace_id", "span_id")
                 ) WITH (append_mode = 'true', ttl = '{metrics_ttl}')"#
            ),
        ];
        for statement in statements {
            self.sql(&statement).await?;
        }
        self.try_traces_deviations().await;
        self.try_logs_deviations().await;
        self.reconcile_ttls(metrics_ttl, error_events_ttl).await;
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
            ("metric_exemplars", metrics_ttl),
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
            // TODO(plan-010): unpopulated; filling it touches the zero-copy
            // ingest path and is deferred to a dedicated ingest change.
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
        let _ = self.sql(&sql).await;
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
            let _ = self.sql(&sql).await;
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

    /// Run one SQL statement; return the first result set's rows.
    pub async fn sql(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        let response: serde_json::Value = self
            .client
            .post(format!("{}/v1/sql?db=public", self.base_url))
            .header("X-Greptime-Timeout", SQL_QUERY_TIMEOUT_HEADER)
            .form(&[("sql", sql)])
            .send()
            .await?
            .json()
            .await?;
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

    /// Like [`Self::sql`], but also returns the result-set column names
    /// (the raw-SQL surface needs a generic grid, not a fixed projection).
    pub async fn sql_with_schema(&self, sql: &str) -> anyhow::Result<crate::adapter::SqlResult> {
        let response: serde_json::Value = self
            .client
            .post(format!("{}/v1/sql?db=public", self.base_url))
            .header("X-Greptime-Timeout", SQL_QUERY_TIMEOUT_HEADER)
            .form(&[("sql", sql)])
            .send()
            .await?
            .json()
            .await?;
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

    async fn metric_table_for_name(
        &self,
        name: &str,
        suffix: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
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
        Ok(candidates
            .into_iter()
            .find(|candidate| existing.contains(candidate)))
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
            .sql_with_schema_lenient(&Self::select_spans_sql(where_clause, order, limit_clause))
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
            .sql_lenient(&Self::select_logs_sql(where_clause, order, limit_clause))
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
    by_name: std::collections::HashMap<&'a str, usize>,
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
    row.get(index)
        .and_then(|v| v.as_u64())
        .map(u128::from)
        .unwrap_or(0)
}

fn absorb_observed_run(
    runs: &mut std::collections::HashMap<String, crate::adapter::ObservedRun>,
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

fn duration_quantile_ms(durations: &mut [u128], q: f64) -> f64 {
    if durations.is_empty() {
        return 0.0;
    }
    durations.sort_unstable();
    if durations.len() == 1 {
        return durations[0] as f64 / 1_000_000.0;
    }
    let pos = q.clamp(0.0, 1.0) * (durations.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return durations[lo] as f64 / 1_000_000.0;
    }
    let weight = pos - lo as f64;
    (durations[lo] as f64 + (durations[hi] as f64 - durations[lo] as f64) * weight) / 1_000_000.0
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
impl TelemetryStore for GreptimeStore {
    async fn ingest_traces(&self, _spans: Vec<SpanRow>, raw: bytes::Bytes) -> anyhow::Result<()> {
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

    async fn ingest_logs(&self, _logs: Vec<LogRow>, raw: bytes::Bytes) -> anyhow::Result<()> {
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
        self.insert(
            "metric_exemplars",
            "\"ts\", \"service\", \"name\", \"value\", \"trace_id\", \"span_id\", \"run_id\", \"attributes\"",
            values,
        )
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
        let mut ids = Vec::new();
        for trace_id in trace_ids.iter().filter(|trace_id| !trace_id.is_empty()) {
            if ids.iter().any(|id| id == trace_id) {
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

    async fn spans_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<SpanRow>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let escaped_run_id = escape(run_id);
        let limit_clause = format!(" LIMIT {limit}");
        let trace_run_column = resource_attr_ident(semconv::PARALLAX_RUN_ID);
        let mut spans = match self
            .select_spans(
                &format!(r#"{trace_run_column} = '{escaped_run_id}'"#),
                r#" ORDER BY "timestamp" DESC"#,
                &limit_clause,
            )
            .await
        {
            Ok(spans) => spans,
            Err(error) if is_missing_column(&error) => Vec::new(),
            Err(error) => return Err(error),
        };

        if spans.len() < limit {
            let via_logs = match self
                .select_spans(
                    &format!(
                        r#""trace_id" IN (
                    SELECT DISTINCT "trace_id" FROM opentelemetry_logs
                    WHERE {} = '{}'
                  )"#,
                        wire_attr_ident(semconv::PARALLAX_RUN_ID),
                        escaped_run_id
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
        let result = match self.sql_with_schema_lenient(&sql).await {
            Ok(result) => result,
            Err(error) if is_missing_column(&error) => {
                for run_id in run_ids {
                    out.insert(
                        run_id.clone(),
                        self.spans_by_run(run_id, limit_per_run).await?,
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
                run_id: cols
                    .opt_string(&semconv::resource_column(semconv::PARALLAX_RUN_ID), row),
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

    async fn metric_names(&self) -> anyhow::Result<Vec<String>> {
        // native: one table per metric name. Discover them from the schema,
        // dropping the otel_/extension/system tables and collapsing histogram
        // `_bucket`/`_count`/`_sum` siblings back to the base metric name.
        Ok(self.discover_metric_names().await?.into_iter().collect())
    }

    async fn metric_labels(&self, name: &str) -> anyhow::Result<Vec<String>> {
        let table = match self.metric_table_for_name(name, None).await? {
            Some(table) => table,
            None => match self.metric_table_for_name(name, Some("_bucket")).await? {
                Some(table) => table,
                None => return Ok(Vec::new()),
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
        Ok(rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|column| {
                !METRIC_BOOKKEEPING_COLUMNS.contains(&column.as_str())
                    && metric_group_label_allowed(column)
            })
            .collect())
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
        let table = match self.metric_table_for_name(name, None).await? {
            Some(table) => table,
            None => match self.metric_table_for_name(name, Some("_bucket")).await? {
                Some(table) => table,
                None => return Ok(Vec::new()),
            },
        };
        let labels = self.metric_labels(name).await?;
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

    async fn service_names(&self) -> anyhow::Result<Vec<String>> {
        // Any signal makes a service real: traces' `service_name`, logs'
        // resource service name, plus the run-metric extension table.
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT "service_name" AS "svc" FROM opentelemetry_traces
                   UNION SELECT DISTINCT
                          {} AS "svc"
                          FROM opentelemetry_logs
                   UNION SELECT DISTINCT "service" AS "svc" FROM run_metric_points
                   ORDER BY "svc""#,
                log_service_name_expr()
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|r| str_at(r, 0))
            .filter(|s| !s.is_empty())
            .collect())
    }

    async fn overview_totals(&self, range: RangeInclusive<u128>) -> anyhow::Result<OverviewTotals> {
        let trace_rows = self
            .sql_lenient(&format!(
                r#"SELECT COUNT(*) AS "spans", COUNT(DISTINCT "trace_id") AS "traces",
                          SUM(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
                          AS "errors",
                          COUNT(DISTINCT "service_name") AS "services"
                   FROM opentelemetry_traces
                   WHERE "timestamp" >= {} AND "timestamp" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?;
        let log_rows = self
            .sql_lenient(&format!(
                r#"SELECT COUNT(*) AS "logs" FROM opentelemetry_logs
                   WHERE "timestamp" >= {} AND "timestamp" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?;
        let service_rows = self
            .sql_lenient(&format!(
                r#"SELECT COUNT(DISTINCT "svc") FROM (
                     SELECT "service_name" AS "svc" FROM opentelemetry_traces
                     WHERE "timestamp" >= {} AND "timestamp" <= {}
                     UNION ALL
                     SELECT {} AS "svc"
                     FROM opentelemetry_logs
                     WHERE "timestamp" >= {} AND "timestamp" <= {}
                   ) WHERE "svc" IS NOT NULL AND "svc" != ''"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
                log_service_name_expr(),
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?;
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
        let log_count = log_rows.first().map(|r| u128_at(r, 0) as u64).unwrap_or(0);
        let active_services = service_rows
            .first()
            .map(|r| u128_at(r, 0) as u64)
            .unwrap_or(0);
        Ok(OverviewTotals {
            span_count,
            trace_count,
            log_count,
            // V1 gap: native metric-engine logical table fan-out has no cheap
            // cross-table count here; trend endpoint returns empty too.
            metric_point_count: 0,
            error_count,
            error_rate: if span_count == 0 {
                0.0
            } else {
                error_count as f64 / span_count as f64
            },
            active_services,
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
                self.sql_lenient(&format!(
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
                self.sql_lenient(&format!(
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
                self.sql_lenient(&format!(
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
            .sql_lenient(&format!(
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
            self.sql_lenient(&format!(
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
            self.sql_lenient(&format!(
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
            series = crate::memory::rate_from_buckets(&series, step_secs * 1_000_000_000);
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
        let Some(bucket_table) = self.metric_table_for_name(name, Some("_bucket")).await? else {
            return Ok(Vec::new());
        };
        // native: explicit-bucket histograms split into `<name>_bucket`
        // (cumulative `greptime_value` per `le` tag), `<name>_count`, `<name>_sum`.
        // Read the bucket rows, merge per time window, then interpolate.
        let service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let rows = self
            .sql_lenient(&Self::histogram_quantile_bucket_sql(
                &bucket_table,
                range.start() / 1_000_000,
                range.end() / 1_000_000,
                &service_clause,
            ))
            .await?;
        let step = step_nanos.max(1);
        // (window) → (bound → summed cumulative count across rows in window).
        let mut windows: std::collections::BTreeMap<
            u128,
            std::collections::BTreeMap<OrderedF64, f64>,
        > = Default::default();
        for row in &rows {
            let ts_nanos = u128_at(row, 0) * 1_000_000;
            let le = row.get(1).and_then(|v| v.as_f64()).unwrap_or(f64::INFINITY);
            let cumulative = row.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);
            *windows
                .entry((ts_nanos / step) * step)
                .or_default()
                .entry(OrderedF64(le))
                .or_default() += cumulative;
        }
        Ok(windows
            .into_iter()
            .map(|(ts_nanos, bounds)| SeriesPoint {
                ts_nanos,
                value: quantile_from_cumulative(&bounds, q),
            })
            .collect())
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
                   FROM metric_exemplars
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
    ) -> anyhow::Result<Vec<crate::adapter::ObservedRun>> {
        let mut runs: std::collections::HashMap<String, crate::adapter::ObservedRun> =
            std::collections::HashMap::new();

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

        // Fallback for older trace schemas where run id only appears on logs.
        let sources = [
            (
                format!(
                    r#"SELECT l.{} AS "run_id",
                          CAST(MIN(s."timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX(s."timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT s."span_id") AS "n",
                          MAX(s."service_name") AS "svc"
                   FROM opentelemetry_logs l
                   JOIN opentelemetry_traces s ON s."trace_id" = l."trace_id"
                   WHERE l.{} IS NOT NULL
                     AND l.{} != ''
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#,
                    wire_attr_ident(semconv::PARALLAX_RUN_ID),
                    wire_attr_ident(semconv::PARALLAX_RUN_ID),
                    wire_attr_ident(semconv::PARALLAX_RUN_ID)
                ),
                true,
            ),
            (
                format!(
                    r#"SELECT {} AS "run_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(*) AS "n",
                          MAX({}) AS "svc"
                   FROM opentelemetry_logs
                   WHERE {} IS NOT NULL AND {} != ''
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#,
                    wire_attr_ident(semconv::PARALLAX_RUN_ID),
                    log_service_name_expr(),
                    wire_attr_ident(semconv::PARALLAX_RUN_ID),
                    wire_attr_ident(semconv::PARALLAX_RUN_ID)
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
        let total_rows = self
            .sql_lenient(&format!(r#"SELECT COUNT(*) AS "total" FROM ({listed})"#))
            .await?;
        let roots = self.sql_lenient(&page_sql).await?;
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
            total: total_rows
                .first()
                .map(|r| u128_at(r, 0) as u64)
                .unwrap_or(0),
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

        let mut rows = Vec::new();
        for key in candidate_keys {
            let ((selected_total, selected_counts), (baseline_total, baseline_counts)) =
                tokio::try_join!(
                    self.span_attribute_counts(&key, &selected, service, error_only),
                    self.span_attribute_counts(&key, &baseline, service, error_only),
                )?;
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
        let sample_sql = format!(
            r#"SELECT {value_expr} AS "value"
               FROM opentelemetry_traces
               WHERE {sample_where}
               ORDER BY "timestamp" DESC
               LIMIT {MAX_ROWS}"#
        );

        let distinct_rows = self
            .sql_lenient(&format!(
                r#"SELECT COUNT(DISTINCT "value") AS "distinct_count"
                   FROM ({sample_sql}) AS "field_sample""#
            ))
            .await?;
        let distinct_count = distinct_rows
            .first()
            .map(|row| u128_at(row, 0) as u64)
            .unwrap_or(0);

        let value_rows = self
            .sql_lenient(&format!(
                r#"SELECT "value", COUNT(*) AS "count"
                   FROM ({sample_sql}) AS "field_sample"
                   GROUP BY "value"
                   ORDER BY "count" DESC, "value" ASC
                   LIMIT {FIELD_TOP_VALUES_CAP}"#
            ))
            .await?;
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
            .sql_lenient(&format!(
                r#"SELECT "parent"."service_name" AS "source",
                          "child"."service_name" AS "target",
                          "child"."span_status_code" AS "status",
                          CAST("child"."duration_nano" AS BIGINT) AS "duration_ns"
                   FROM opentelemetry_traces AS "child"
                   JOIN opentelemetry_traces AS "parent"
                     ON "child"."trace_id" = "parent"."trace_id"
                    AND "child"."parent_span_id" = "parent"."span_id"
                   WHERE "child"."trace_id" IN ({id_list})
                     AND "child"."timestamp" >= {}
                     AND "child"."timestamp" <= {}
                     AND "child"."span_kind" = 'SPAN_KIND_SERVER'
                     AND "child"."service_name" != "parent"."service_name""#,
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ))
            .await?;

        let mut grouped: BTreeMap<(String, String), (u64, u64, Vec<u128>)> = BTreeMap::new();
        for row in &rows {
            let source = str_at(row, 0);
            let target = str_at(row, 1);
            if source.is_empty() || target.is_empty() || source == target {
                continue;
            }
            let entry = grouped.entry((source, target)).or_default();
            entry.0 += 1;
            if str_at(row, 2) == "STATUS_CODE_ERROR" {
                entry.1 += 1;
            }
            entry.2.push(u128_at(row, 3));
        }

        Ok(grouped
            .into_iter()
            .map(
                |((source, target), (call_count, error_count, mut durations))| {
                    let p50_ms = duration_quantile_ms(&mut durations, 0.5);
                    let p95_ms = duration_quantile_ms(&mut durations, 0.95);
                    ServiceEdge {
                        source,
                        target,
                        call_count,
                        error_count,
                        p50_ms,
                        p95_ms,
                    }
                },
            )
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
        let labels = self.metric_labels(name).await?;
        anyhow::ensure!(
            labels.iter().any(|label| label == group_by),
            "unknown metric label"
        );
        let Some(table) = self.metric_table_for_name(name, None).await? else {
            return Ok(Vec::new());
        };
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
            .sql_lenient(&format!(
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
        let mut groups: std::collections::BTreeMap<String, Vec<SeriesPoint>> = Default::default();
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
                    crate::memory::rate_from_buckets(&series, step_secs * 1_000_000_000)
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
        let mut rows = Vec::new();
        for metric in self.metric_names().await? {
            let Some(family) = runtime_metric_family(&metric) else {
                continue;
            };
            let points = self
                .metric_series(
                    &metric,
                    service,
                    run_id,
                    range.clone(),
                    step_nanos,
                    MetricAgg::Avg,
                )
                .await?;
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
            .sql_lenient(&Self::histogram_count_series_sql(
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
            .sql_lenient(&format!(
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
    async fn discover_metric_names(&self) -> anyhow::Result<std::collections::BTreeSet<String>> {
        const RESERVED: &[&str] = &[
            "opentelemetry_traces",
            "opentelemetry_traces_services",
            "opentelemetry_traces_operations",
            "opentelemetry_logs",
            "error_events",
            "run_metric_points",
            "metric_exemplars",
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
        let table_set = tables
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let mut names = std::collections::BTreeSet::new();
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
            .sql_lenient(
                r#"SELECT DISTINCT "name" FROM run_metric_points
                   WHERE "name" IS NOT NULL AND "name" != ''"#,
            )
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
fn quantile_from_cumulative(bounds: &std::collections::BTreeMap<OrderedF64, f64>, q: f64) -> f64 {
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
mod tests {
    use super::*;

    #[test]
    fn escape_ident_doubles_double_quotes_only() {
        assert_eq!(
            escape_ident(r#"http."server".duration"#),
            r#"http.""server"".duration"#
        );
        assert_eq!(escape_ident("metric's/name"), "metric's/name");
    }

    #[test]
    fn escape_handles_quotes_newlines_backslash() {
        assert_eq!(escape("o'brien"), "o''brien");
        assert_eq!(escape("already''doubled"), "already''''doubled");
        assert_eq!(escape("line\nbreak"), "line\nbreak");
        assert_eq!(escape(""), "");
        // Backslash passes through unchanged today; Step 5 of plan 074 verifies
        // GreptimeDB dialect treatment of trailing backslash in string literals.
        assert_eq!(escape(r"ends\"), r"ends\");
        assert_eq!(quoted_ident(r#"a"b"#), r#""a""b""#);
    }

    
    #[test]
    fn body_search_clauses_single_term() {
        let mut clauses = Vec::new();
        push_body_search_clause(&mut clauses, "error");
        assert_eq!(clauses, vec![r#"matches_term("body", 'error')"#]);
    }

    #[test]
    fn body_search_clauses_two_terms_and_combined() {
        let mut clauses = Vec::new();
        push_body_search_clause(&mut clauses, "connection reset");
        assert_eq!(
            clauses,
            vec![
                r#"matches_term("body", 'connection')"#,
                r#"matches_term("body", 'reset')"#,
            ]
        );
    }

    #[test]
    fn body_search_clauses_quoted_phrase() {
        let mut clauses = Vec::new();
        push_body_search_clause(&mut clauses, r#""connection reset""#);
        assert_eq!(clauses, vec![r#"matches_term("body", 'connection reset')"#]);
    }

    #[test]
    fn body_search_clauses_punctuation_falls_back_to_like() {
        let mut clauses = Vec::new();
        push_body_search_clause(&mut clauses, "???");
        assert_eq!(clauses.len(), 1);
        assert!(clauses[0].contains(r#""body" LIKE"#));
        assert!(clauses[0].contains("???"));
    }

    #[test]
    fn body_search_escapes_sql_quotes_in_term() {
        let mut clauses = Vec::new();
        push_body_search_clause(&mut clauses, "o'brien");
        assert_eq!(clauses, vec![r#"matches_term("body", 'o''brien')"#]);
    }

    #[test]
    fn log_filter_clauses_use_matches_term_and_service_coalesce() {
        let clauses = log_filter_clauses(Some("checkout"), &(0..=1000), None, None, Some("error"));
        let joined = clauses.join(" AND ");
        assert!(joined.contains(r#"matches_term("body", 'error')"#));
        assert!(joined.contains(r#"COALESCE("service.name""#));
        assert!(!joined.contains(r#""body" LIKE"#));
    }

#[test]
    fn golden_traces_search_sql_includes_adversarial_service() {
        let participation = format!(
            r#" AND "trace_id" IN (SELECT "trace_id" FROM opentelemetry_traces WHERE "service_name" = '{}')"#,
            escape("svc'quote")
        );
        let (listed, page) = GreptimeStore::traces_search_sql(
            r#""timestamp" >= 1"#,
            &participation,
            r#""rn" = 1"#,
            r#""ts_nanos" DESC"#,
            50,
            0,
        );
        assert!(listed.contains("svc''quote"));
        assert!(page.contains("LIMIT 50 OFFSET 0"));
        assert!(page.contains(&listed));
    }

    #[test]
    fn golden_histogram_count_series_sql() {
        let sql = GreptimeStore::histogram_count_series_sql(
            "http_server_request_duration_count",
            60,
            1_000,
            2_000,
            r#" AND "service_name" = 'api'"#,
        );
        assert!(sql.contains(r#"FROM "http_server_request_duration_count""#));
        assert!(sql.contains("date_bin"));
    }

    #[test]
    fn golden_select_spans_and_logs_sql() {
        let spans = GreptimeStore::select_spans_sql(
            r#""trace_id" = 'abc""def'"#,
            " ORDER BY \"timestamp\"",
            " LIMIT 10",
        );
        assert!(spans.contains("opentelemetry_traces"));
        assert!(spans.contains("LIMIT 10"));
        let logs = GreptimeStore::select_logs_sql("1 = 1", "", " LIMIT 5");
        assert!(logs.contains("opentelemetry_logs"));
        assert!(logs.contains("json_to_string"));
    }

    #[test]
    fn golden_span_attribute_counts_sql() {
        let sql = GreptimeStore::span_attribute_counts_sql(
            "http.route",
            &(0..=1000),
            Some("svc'x"),
            true,
        );
        assert!(sql.contains("http.route"));
        assert!(sql.contains("svc''x"));
        assert!(sql.contains("STATUS_CODE_ERROR"));
    }

    #[test]
    fn raw_sql_read_only_guard_rejects_writes_and_explain_analyze() {
        assert!(raw_sql_read_only("SELECT * FROM opentelemetry_logs"));
        assert!(raw_sql_read_only(
            "EXPLAIN SELECT * FROM opentelemetry_logs"
        ));
        assert!(!raw_sql_read_only(
            "EXPLAIN ANALYZE SELECT * FROM opentelemetry_logs"
        ));
        assert!(!raw_sql_read_only(
            "SELECT 1; DROP TABLE opentelemetry_logs"
        ));
        assert!(!raw_sql_read_only("DELETE FROM opentelemetry_logs"));
    }

    #[test]
    fn metric_table_candidates_normalizes_dotted_count_suffix() {
        let candidates = metric_table_candidates("http.server.request.duration", Some("_count"));
        assert!(
            candidates
                .iter()
                .any(|c| c == "http_server_request_duration_count"),
            "expected underscore-normalized count table among {candidates:?}"
        );
    }

    #[test]
    fn sql_error_prefix_is_char_boundary_safe() {
        // 3-byte codepoints so byte index 200 is mid-character (would panic on
        // `&s[..200]`). "é" is 2 bytes and lands on a boundary at 200.
        let s = "あ".repeat(300);
        assert!(!s.is_char_boundary(200));
        let prefix: String = s.chars().take(200).collect();
        assert_eq!(prefix.chars().count(), 200);
    }
}
