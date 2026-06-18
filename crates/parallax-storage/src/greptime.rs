//! GreptimeDB `TelemetryStore` adapter: SQL over the HTTP API, DDL from the
//! implementation spec §5. All engine-specific SQL lives in this module.

use crate::adapter::TelemetryStore;
use crate::model::*;
use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicBool, Ordering};

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
        let store = Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
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

    /// Create the *extension* tables (idempotent), interpolating TTLs. The
    /// native OTLP tables (`opentelemetry_traces`/`_logs` + per-metric tables)
    /// are NOT created here — they auto-create on the first forward; their
    /// post-create deviations run via [`Self::ensure_native_deviations`].
    pub async fn bootstrap(&self, metrics_ttl: &str, error_events_ttl: &str) -> anyhow::Result<()> {
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
                r#"CREATE TABLE IF NOT EXISTS rollups_fingerprint_minute (
                   "bucket_ts" TIMESTAMP(0) NOT NULL, "service" STRING, "fingerprint" STRING,
                   "count" BIGINT,
                   TIME INDEX ("bucket_ts"), PRIMARY KEY ("service", "fingerprint")
                 ) WITH (ttl = '{error_events_ttl}')"#
            ),
            // Run-scoped metric points (Q6, Approach 2): high-card `run_id` is a
            // SKIPPING-indexed column, not a metric-engine tag, so per-run series
            // cost nothing on the metric engine.
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
        // The native tables may not exist yet (they auto-create on first
        // forward), so try the deviations now and swallow not-found — the lazy
        // per-signal guards re-run them after each signal's first forward (e.g.
        // when a prior run already created the tables in a persistent data dir).
        self.try_traces_deviations().await;
        self.try_logs_deviations().await;
        Ok(())
    }

    /// Run a batch of idempotent post-create ALTERs, swallowing the benign
    /// "already exists" / "not found" outcomes (the table may not exist yet, or
    /// the deviation may already be applied from a prior run).
    async fn try_deviations(&self, statements: &[&str]) {
        for statement in statements {
            if let Err(error) = self.sql(statement).await {
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
        self.try_deviations(&[
            r#"ALTER TABLE opentelemetry_traces ADD COLUMN "fingerprint" STRING"#,
        ])
        .await;
    }

    /// Logs deviations: an INVERTED index on `trace_id` and a FULLTEXT index on
    /// `body` (the one native shortfall), plus an explicit `parallax.run.id`
    /// column. The run-id column is normally promoted by the
    /// `x-greptime-log-extract-keys` header, but only when an ingested log
    /// actually carries that resource attribute — adding it here guarantees the
    /// column exists so run-scoped log reads never reference a missing field.
    async fn try_logs_deviations(&self) {
        self.try_deviations(&[
            r#"ALTER TABLE opentelemetry_logs MODIFY COLUMN "trace_id" SET INVERTED INDEX"#,
            r#"ALTER TABLE opentelemetry_logs MODIFY COLUMN "body" SET FULLTEXT INDEX"#,
            r#"ALTER TABLE opentelemetry_logs ADD COLUMN "parallax.run.id" STRING"#,
        ])
        .await;
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
            .form(&[("sql", sql)])
            .send()
            .await?
            .json()
            .await?;
        // Success responses carry `output` (no `code`); failures carry
        // `error` (+ a non-zero `code`).
        if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
            let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            anyhow::bail!(
                "greptime sql failed (code {code}): {error} — sql: {}",
                &sql[..sql.len().min(200)]
            );
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
            .sql_with_schema_lenient(&format!(
                r#"SELECT * FROM opentelemetry_traces WHERE {where_clause}{order}{limit_clause}"#
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
                    // native: run id flattens to a resource-attribute column.
                    run_id: cols.opt_string("resource_attributes.parallax.run.id", row),
                    scope_name: cols.string("scope_name", row),
                    links: cols.json("span_links", row),
                    attributes,
                    resource,
                }
            })
            .collect())
    }

    /// Select logs from the native `opentelemetry_logs` table. Logs keep their
    /// attributes as JSON columns (`log_attributes`/`resource_attributes`), and
    /// have no `service_name` column — service is derived from the resource
    /// JSON. The promoted `parallax.run.id` column carries the run id.
    async fn select_logs(
        &self,
        where_clause: &str,
        order: &str,
        limit_clause: &str,
    ) -> anyhow::Result<Vec<LogRow>> {
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT CAST("timestamp" AS BIGINT) AS "ts_nanos",
                          json_get_string("resource_attributes", '$."service.name"') AS "service",
                          "severity_number", "severity_text", "body", "trace_id", "span_id",
                          "parallax.run.id", "scope_name",
                          json_to_string("log_attributes"),
                          json_to_string("resource_attributes")
                   FROM opentelemetry_logs WHERE {where_clause}{order}{limit_clause}"#
            ))
            .await?;
        Ok(rows.iter().map(|row| log_row_from_row(row)).collect())
    }
}

/// A row → `LogRow` projection for the fixed native-logs column order used by
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

fn str_at(row: &[serde_json::Value], index: usize) -> String {
    row.get(index)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn opt_str_at(row: &[serde_json::Value], index: usize) -> Option<String> {
    row.get(index).and_then(|v| v.as_str()).map(str::to_string)
}

/// Clamp a u128 time bound to what the engine's TIMESTAMP cast accepts
/// (i64); open-ended `..=u128::MAX` ranges otherwise fail query planning
/// ("Casting value to Timestamp is invalid").
fn sql_ts(bound: u128) -> i64 {
    i64::try_from(bound).unwrap_or(i64::MAX)
}

/// The shared WHERE clauses for `logs_search` and `log_count_series` — the
/// histogram must count exactly what the table shows. Body search is `LIKE`
/// today; a GreptimeDB FULLTEXT index + `matches_term` is the planned
/// upgrade for large logs (spec §5 note).
fn log_filter_clauses(
    service: Option<&str>,
    range: &RangeInclusive<u128>,
    severity_min: Option<i32>,
    body_contains: Option<&str>,
) -> Vec<String> {
    let mut clauses = vec![format!(
        r#""timestamp" >= {} AND "timestamp" <= {}"#,
        sql_ts(*range.start()),
        sql_ts(*range.end())
    )];
    if let Some(service) = service {
        // native: logs carry no `service_name` column — match on the resource
        // JSON's `service.name`.
        clauses.push(format!(
            r#"json_get_string("resource_attributes", '$."service.name"') = '{}'"#,
            escape(service)
        ));
    }
    if let Some(min) = severity_min {
        clauses.push(format!(r#""severity_number" >= {min}"#));
    }
    if let Some(needle) = body_contains {
        // LIKE wildcards in the needle are literal for a substring search;
        // backslash first (it is the escape char), then %, _, then quotes.
        let escaped = escape(
            &needle
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_"),
        );
        // ESCAPE takes exactly one character — a single backslash in SQL.
        clauses.push(format!(r#""body" LIKE '%{escaped}%' ESCAPE '\'"#));
    }
    clauses
}

fn u128_at(row: &[serde_json::Value], index: usize) -> u128 {
    row.get(index)
        .and_then(|v| v.as_u64())
        .map(u128::from)
        .unwrap_or(0)
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
        // The extract-keys header promotes `parallax.run.id` to a real column.
        let hints = format!("ttl={},append_mode=true", self.logs_ttl);
        self.forward_otlp(
            "v1/logs",
            &[
                ("x-greptime-log-extract-keys", "parallax.run.id"),
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

    async fn spans_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<SpanRow>> {
        self.select_spans(
            &format!(
                r#""resource_attributes.parallax.run.id" = '{}'"#,
                escape(run_id)
            ),
            r#" ORDER BY "timestamp" ASC"#,
            &format!(" LIMIT {limit}"),
        )
        .await
    }

    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>> {
        self.select_logs(
            &format!(r#""parallax.run.id" = '{}'"#, escape(run_id)),
            r#" ORDER BY "timestamp" ASC"#,
            &format!(" LIMIT {limit}"),
        )
        .await
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

    async fn service_names(&self) -> anyhow::Result<Vec<String>> {
        // Any signal makes a service real: traces' `service_name`, logs'
        // resource `service.name`, plus the run-metric extension table.
        let rows = self
            .sql_lenient(
                r#"SELECT DISTINCT "service_name" AS "svc" FROM opentelemetry_traces
                   UNION SELECT DISTINCT
                          json_get_string("resource_attributes", '$."service.name"') AS "svc"
                          FROM opentelemetry_logs
                   UNION SELECT DISTINCT "service" AS "svc" FROM run_metric_points
                   ORDER BY "svc""#,
            )
            .await?;
        Ok(rows
            .iter()
            .map(|r| str_at(r, 0))
            .filter(|s| !s.is_empty())
            .collect())
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
            self.sql_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "ts") AS BIGINT)
                          AS "bucket_ns", {sql_agg}("value") AS "agg_value"
                   FROM run_metric_points
                   WHERE "name" = '{}' AND "run_id" = '{}'{service_clause}
                     AND "ts" >= {} AND "ts" <= {}
                   GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                escape(name),
                escape(run_id),
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?
        } else {
            let service_clause = service
                .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
                .unwrap_or_default();
            self.sql_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", {sql_agg}("greptime_value") AS "agg_value"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
                escape(name),
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
        // native: explicit-bucket histograms split into `<name>_bucket`
        // (cumulative `greptime_value` per `le` tag), `<name>_count`, `<name>_sum`.
        // Read the bucket rows, merge per time window, then interpolate.
        let service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT CAST("greptime_timestamp" AS BIGINT) AS "ts_ms",
                          CAST("le" AS DOUBLE) AS "le", "greptime_value" AS "cumulative"
                   FROM "{}_bucket"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   ORDER BY "greptime_timestamp" ASC"#,
                escape(name),
                sql_ts(range.start() / 1_000_000),
                sql_ts(range.end() / 1_000_000),
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
        // native: traces flatten run id to `resource_attributes.parallax.run.id`
        // with a `service_name` column; logs promote it to `parallax.run.id`
        // with service in the resource JSON.
        let sources = [
            (
                r#"SELECT "resource_attributes.parallax.run.id" AS "run_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(*) AS "n", MAX("service_name") AS "svc"
                   FROM opentelemetry_traces
                   WHERE "resource_attributes.parallax.run.id" IS NOT NULL
                     AND "resource_attributes.parallax.run.id" != ''
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#,
                true,
            ),
            (
                r#"SELECT "parallax.run.id" AS "run_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(*) AS "n",
                          MAX(json_get_string("resource_attributes", '$."service.name"')) AS "svc"
                   FROM opentelemetry_logs
                   WHERE "parallax.run.id" IS NOT NULL AND "parallax.run.id" != ''
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#,
                false,
            ),
        ];
        for (query, is_span) in sources {
            let rows = self.sql_lenient(&format!("{query}{limit}")).await?;
            for row in &rows {
                let run_id = str_at(row, 0);
                if run_id.is_empty() {
                    continue;
                }
                let first = u128_at(row, 1);
                let last = u128_at(row, 2);
                let count = u128_at(row, 3) as u64;
                let entry =
                    runs.entry(run_id.clone())
                        .or_insert_with(|| crate::adapter::ObservedRun {
                            run_id,
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
    ) -> anyhow::Result<Vec<crate::adapter::TraceSummary>> {
        // Root spans (no parent), newest first; aggregates joined per trace.
        // `error_only` filters on the aggregate, so over-fetch roots first.
        let mut clauses = vec![r#"("parent_span_id" IS NULL OR "parent_span_id" = '')"#.into()];
        if let Some(service) = &query.service {
            clauses.push(format!(r#""service_name" = '{}'"#, escape(service)));
        }
        if let Some(from) = query.from_nanos {
            clauses.push(format!(r#""timestamp" >= {}"#, sql_ts(from)));
        }
        if let Some(to) = query.to_nanos {
            clauses.push(format!(r#""timestamp" <= {}"#, sql_ts(to)));
        }
        if let Some(min) = query.min_duration_ns {
            clauses.push(format!(r#""duration_nano" >= {}"#, u64::try_from(min)?));
        }
        if let Some(needle) = &query.name_contains {
            let escaped = escape(needle).replace('%', r"\%").replace('_', r"\_");
            clauses.push(format!(r#""span_name" LIKE '%{escaped}%' ESCAPE '\'"#));
        }
        let fetch = if query.error_only {
            query.limit.saturating_mul(5).max(50)
        } else {
            query.limit
        };
        let roots = self
            .sql_lenient(&format!(
                r#"SELECT "trace_id", "span_name", "service_name",
                          CAST("timestamp" AS BIGINT) AS "ts_nanos",
                          CAST("duration_nano" AS BIGINT) AS "dur"
                   FROM opentelemetry_traces
                   WHERE {}
                   ORDER BY "timestamp" DESC LIMIT {fetch}"#,
                clauses.join(" AND ")
            ))
            .await?;
        if roots.is_empty() {
            return Ok(Vec::new());
        }
        let id_list = roots
            .iter()
            .map(|row| format!("'{}'", escape(&str_at(row, 0))))
            .collect::<Vec<_>>()
            .join(",");
        let aggregates = self
            .sql(&format!(
                r#"SELECT "trace_id", COUNT(*) AS "n",
                          MAX(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END) AS "err"
                   FROM opentelemetry_traces WHERE "trace_id" IN ({id_list})
                   GROUP BY "trace_id""#
            ))
            .await?;
        let mut by_trace: std::collections::HashMap<String, (u64, bool)> =
            std::collections::HashMap::new();
        for row in &aggregates {
            by_trace.insert(
                str_at(row, 0),
                (u128_at(row, 1) as u64, u128_at(row, 2) > 0),
            );
        }
        let mut traces: Vec<_> = roots
            .iter()
            .map(|row| {
                let trace_id = str_at(row, 0);
                let (span_count, has_error) =
                    by_trace.get(&trace_id).copied().unwrap_or((1, false));
                crate::adapter::TraceSummary {
                    trace_id,
                    root_name: str_at(row, 1),
                    service: str_at(row, 2),
                    start_nanos: u128_at(row, 3),
                    duration_ns: u128_at(row, 4),
                    span_count,
                    has_error,
                }
            })
            .collect();
        if query.error_only {
            traces.retain(|t| t.has_error);
        }
        traces.truncate(query.limit);
        Ok(traces)
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
        body_contains: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<LogRow>> {
        let clauses = log_filter_clauses(service, &range, severity_min, body_contains);
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT CAST("timestamp" AS BIGINT) AS "ts_nanos",
                          json_get_string("resource_attributes", '$."service.name"') AS "service",
                          "severity_number", "severity_text", "body", "trace_id", "span_id",
                          "parallax.run.id", "scope_name",
                          json_to_string("log_attributes"),
                          json_to_string("resource_attributes")
                   FROM opentelemetry_logs WHERE {} ORDER BY "timestamp" DESC LIMIT {limit}"#,
                clauses.join(" AND ")
            ))
            .await?;
        Ok(rows.iter().map(|row| log_row_from_row(row)).collect())
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
        let group_col = format!(r#""{}""#, group_by.replace('"', ""));
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT COALESCE(CAST({group_col} AS STRING), '(none)') AS "grp",
                          CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", {sql_agg}("greptime_value") AS "agg_value"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "grp", "bucket_ms" ORDER BY "grp", "bucket_ms""#,
                escape(name),
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
        // native: the `<name>_count` sibling table holds the per-sample count
        // as `greptime_value`; sum it per window for the request-rate numerator.
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", SUM("greptime_value") AS "samples"
                   FROM "{}_count"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
                escape(name),
                sql_ts(range.start() / 1_000_000),
                sql_ts(range.end() / 1_000_000),
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
        body_contains: Option<&str>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let clauses = log_filter_clauses(service, &range, severity_min, body_contains);
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
        self.sql_with_schema(query).await
    }
}

impl GreptimeStore {
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
            "rollups_fingerprint_minute",
            "run_metric_points",
            "greptime_physical_table",
        ];
        let rows = self
            .sql(
                r#"SELECT "table_name" FROM information_schema.tables
                   WHERE "table_schema" = 'public'"#,
            )
            .await?;
        let mut names = std::collections::BTreeSet::new();
        for row in &rows {
            let table = str_at(row, 0);
            if table.is_empty()
                || RESERVED.contains(&table.as_str())
                || table.starts_with("opentelemetry_")
            {
                continue;
            }
            // Collapse explicit-histogram siblings back to the base metric name.
            let base = table
                .strip_suffix("_bucket")
                .or_else(|| table.strip_suffix("_count"))
                .or_else(|| table.strip_suffix("_sum"))
                .unwrap_or(&table);
            names.insert(base.to_string());
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
