//! GreptimeDB `TelemetryStore` adapter: SQL over the HTTP API, DDL from the
//! implementation spec §5. All engine-specific SQL lives in this module.

use crate::adapter::{
    ATTRIBUTE_COMPARE_KEY_SCAN_LIMIT, ATTRIBUTE_COMPARE_TOP_N_CAP, AttributeCompareRow, MAX_ROWS,
    OverviewTotals, ReleaseWindow, SERVICE_MAP_TRACE_CAP, ServiceEdge, ServiceSummary, SignalKind,
    SpanRed, TelemetryStore, attribute_compare_key_allowed, attribute_compare_score,
    attribute_compare_value_allowed,
};
use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};
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

/// Escape a value for inclusion inside a double-quoted SQL identifier.
fn escape_ident(text: &str) -> String {
    text.replace('"', "\"\"")
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
            // TODO(plan-010): unpopulated; filling it touches the zero-copy
            // ingest path and is deferred to a dedicated ingest change.
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
                    // native: run id flattens to a resource-attribute column.
                    run_id: cols.opt_string("resource_attributes.parallax.run.id", row),
                    scope_name: cols.string("scope_name", row),
                    events,
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
    severity_max: Option<i32>,
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
    if let Some(max) = severity_max {
        clauses.push(format!(r#""severity_number" <= {max}"#));
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
        let rows = self
            .sql_lenient(&format!(
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
                     WHERE "trace_id" IN ({id_list})
                   ) AS "root"
                   JOIN (
                     SELECT "trace_id", COUNT(*) AS "span_count",
                            MAX(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END)
                            AS "has_error"
                     FROM opentelemetry_traces
                     WHERE "trace_id" IN ({id_list})
                     GROUP BY "trace_id"
                   ) AS "agg" ON "root"."trace_id" = "agg"."trace_id"
                   WHERE "root"."rn" = 1"#
            ))
            .await?;
        let mut by_id: std::collections::HashMap<_, _> = rows
            .iter()
            .map(|row| {
                let summary = crate::adapter::TraceSummary {
                    trace_id: str_at(row, 0),
                    root_name: str_at(row, 1),
                    service: str_at(row, 2),
                    start_nanos: u128_at(row, 3),
                    duration_ns: u128_at(row, 4),
                    span_count: u128_at(row, 5) as u64,
                    has_error: u128_at(row, 6) > 0,
                };
                (summary.trace_id.clone(), summary)
            })
            .collect();
        Ok(ids
            .into_iter()
            .filter_map(|trace_id| by_id.remove(&trace_id))
            .collect())
    }

    async fn spans_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<SpanRow>> {
        let mut spans = self
            .select_spans(
                &format!(
                    r#""trace_id" IN (
                    SELECT DISTINCT "trace_id" FROM opentelemetry_logs
                    WHERE "parallax.run.id" = '{}'
                  )"#,
                    escape(run_id)
                ),
                r#" ORDER BY "timestamp" DESC"#,
                &format!(" LIMIT {limit}"),
            )
            .await?;
        spans.reverse();
        Ok(spans)
    }

    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>> {
        let mut logs = self
            .select_logs(
                &format!(r#""parallax.run.id" = '{}'"#, escape(run_id)),
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
                     SELECT json_get_string("resource_attributes", '$."service.name"') AS "svc"
                     FROM opentelemetry_logs
                     WHERE "timestamp" >= {} AND "timestamp" <= {}
                   ) WHERE "svc" IS NOT NULL AND "svc" != ''"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
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
        let sql = format!(
            r#"SELECT "resource_attributes.service.version" AS "version",
                      MIN(CAST("timestamp" AS BIGINT)) AS "first_seen_nanos",
                      MAX(CAST("timestamp" AS BIGINT)) AS "last_seen_nanos",
                      COUNT(*) AS "span_count"
               FROM opentelemetry_traces
               WHERE "service_name" = '{}'
                 AND "timestamp" >= {}
                 AND "timestamp" <= {}
                 AND "resource_attributes.service.version" IS NOT NULL
                 AND "resource_attributes.service.version" != ''
               GROUP BY "resource_attributes.service.version"
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
                escape_ident(name),
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
                escape_ident(name),
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
        // Native logs promote run id to `parallax.run.id`. Some GreptimeDB
        // trace schemas do not flatten the run resource attribute to a column,
        // so span counts derive from traces linked through run-scoped logs.
        let sources = [
            (
                r#"SELECT l."parallax.run.id" AS "run_id",
                          CAST(MIN(s."timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX(s."timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT s."span_id") AS "n",
                          MAX(s."service_name") AS "svc"
                   FROM opentelemetry_logs l
                   JOIN opentelemetry_traces s ON s."trace_id" = l."trace_id"
                   WHERE l."parallax.run.id" IS NOT NULL
                     AND l."parallax.run.id" != ''
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
    ) -> anyhow::Result<crate::adapter::TraceList> {
        // One representative span per trace — its root (no parent), else the
        // earliest span when no root was stored (all-INTERNAL traces).
        //
        // `service` matches any trace the service **participates in** (a span
        // of that service anywhere), not only the root — so a cross-service
        // trace rooted at `checkout` still surfaces under `--service catalog`.
        let participation = match &query.service {
            Some(service) => format!(
                r#" AND "trace_id" IN (SELECT "trace_id" FROM opentelemetry_traces WHERE "service_name" = '{}')"#,
                escape(service)
            ),
            None => String::new(),
        };
        // Scan window — also bounds which span becomes the representative.
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
                 FROM opentelemetry_traces GROUP BY "trace_id"
               ) AS "agg" ON "root"."trace_id" = "agg"."trace_id"
               WHERE {rep_where}"#,
            rep_where = rep.join(" AND "),
        );
        let total_rows = self
            .sql_lenient(&format!(r#"SELECT COUNT(*) AS "total" FROM ({listed})"#))
            .await?;
        let roots = self
            .sql_lenient(&format!(
                r#"SELECT * FROM ({listed}) ORDER BY {order} LIMIT {} OFFSET {}"#,
                query.limit, query.offset,
            ))
            .await?;
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
            let (selected_total, selected_counts) = self
                .span_attribute_counts(&key, &selected, service, error_only)
                .await?;
            let (baseline_total, baseline_counts) = self
                .span_attribute_counts(&key, &baseline, service, error_only)
                .await?;
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
        let group_col = format!(r#""{}""#, escape_ident(group_by));
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT COALESCE(CAST({group_col} AS STRING), '(none)') AS "grp",
                          CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", {sql_agg}("greptime_value") AS "agg_value"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                   GROUP BY "grp", "bucket_ms" ORDER BY "grp", "bucket_ms""#,
                escape_ident(name),
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
                escape_ident(name),
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

        let rows = self
            .sql_lenient(&format!(
                r#"SELECT {value_expr} AS "value", COUNT(*) AS "n"
                   FROM opentelemetry_traces
                   WHERE {}
                   GROUP BY {value_expr}
                   ORDER BY "n" DESC
                   LIMIT 512"#,
                clauses.join(" AND ")
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
}
