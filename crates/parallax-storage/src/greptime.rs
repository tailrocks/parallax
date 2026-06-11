//! GreptimeDB `TelemetryStore` adapter: SQL over the HTTP API, DDL from the
//! implementation spec §5. All engine-specific SQL lives in this module.

use crate::adapter::TelemetryStore;
use crate::model::*;
use std::ops::RangeInclusive;

pub struct GreptimeStore {
    base_url: String,
    client: reqwest::Client,
}

fn escape(text: &str) -> String {
    text.replace('\'', "''")
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
    pub async fn connect(base_url: &str) -> anyhow::Result<Self> {
        let store = Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
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

    /// Create the telemetry tables (idempotent), interpolating TTLs.
    pub async fn bootstrap(
        &self,
        traces_ttl: &str,
        logs_ttl: &str,
        metrics_ttl: &str,
        error_events_ttl: &str,
    ) -> anyhow::Result<()> {
        let statements = [
            format!(
                r#"CREATE TABLE IF NOT EXISTS otel_spans (
                   "ts" TIMESTAMP(9) NOT NULL, "service" STRING, "trace_id" STRING,
                   "span_id" STRING, "parent_span_id" STRING, "name" STRING, "kind" STRING,
                   "status_code" STRING, "status_message" STRING, "duration_ns" BIGINT,
                   "run_id" STRING, "scope_name" STRING, "attributes" JSON, "resource" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service")
                 ) WITH (ttl = '{traces_ttl}')"#
            ),
            format!(
                r#"CREATE TABLE IF NOT EXISTS otel_logs (
                   "ts" TIMESTAMP(9) NOT NULL, "service" STRING, "severity_num" INT,
                   "severity_text" STRING, "body" STRING, "trace_id" STRING, "span_id" STRING, "run_id" STRING,
                   "scope_name" STRING, "attributes" JSON, "resource" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service")
                 ) WITH (ttl = '{logs_ttl}')"#
            ),
            format!(
                r#"CREATE TABLE IF NOT EXISTS otel_metrics_points (
                   "ts" TIMESTAMP(3) NOT NULL, "service" STRING, "name" STRING,
                   "value" DOUBLE, "is_monotonic" BOOLEAN, "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
                 ) WITH (ttl = '{metrics_ttl}')"#
            ),
            format!(
                r#"CREATE TABLE IF NOT EXISTS otel_metrics_histograms (
                   "ts" TIMESTAMP(3) NOT NULL, "service" STRING, "name" STRING,
                   "count" BIGINT, "sum" DOUBLE, "bucket_counts" JSON, "bounds" JSON,
                   "attributes" JSON,
                   TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
                 ) WITH (ttl = '{metrics_ttl}')"#
            ),
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
        ];
        for statement in statements {
            self.sql(&statement).await?;
        }
        Ok(())
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

    async fn select_spans(
        &self,
        where_clause: &str,
        limit_clause: &str,
    ) -> anyhow::Result<Vec<SpanRow>> {
        let rows = self
            .sql(&format!(
                r#"SELECT CAST("ts" AS BIGINT) AS "ts_nanos", "service", "trace_id", "span_id",
                          "parent_span_id", "name", "kind", "status_code",
                          "status_message", "duration_ns", "run_id", "scope_name",
                          json_to_string("attributes"), json_to_string("resource")
                   FROM otel_spans WHERE {where_clause} ORDER BY "ts" ASC{limit_clause}"#
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| SpanRow {
                ts_nanos: u128_at(row, 0),
                service: str_at(row, 1),
                trace_id: str_at(row, 2),
                span_id: str_at(row, 3),
                parent_span_id: opt_str_at(row, 4),
                name: str_at(row, 5),
                kind: str_at(row, 6),
                status_code: str_at(row, 7),
                status_message: str_at(row, 8),
                duration_ns: u128_at(row, 9),
                run_id: opt_str_at(row, 10),
                scope_name: str_at(row, 11),
                attributes: json_at(row, 12),
                resource: json_at(row, 13),
            })
            .collect())
    }

    async fn select_logs(
        &self,
        where_clause: &str,
        limit_clause: &str,
    ) -> anyhow::Result<Vec<LogRow>> {
        let rows = self
            .sql(&format!(
                r#"SELECT CAST("ts" AS BIGINT) AS "ts_nanos", "service", "severity_num",
                          "severity_text", "body", "trace_id", "span_id", "run_id",
                          "scope_name", json_to_string("attributes"),
                          json_to_string("resource")
                   FROM otel_logs WHERE {where_clause} ORDER BY "ts" ASC{limit_clause}"#
            ))
            .await?;
        Ok(rows
            .iter()
            .map(|row| LogRow {
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
            })
            .collect())
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
    async fn write_spans(&self, rows: Vec<SpanRow>) -> anyhow::Result<()> {
        let values = rows
            .iter()
            .map(|r| {
                format!(
                    "({},'{}','{}','{}',{},{},'{}','{}','{}','{}',{},'{}',{},{})",
                    r.ts_nanos,
                    escape(&r.service),
                    escape(&r.trace_id),
                    escape(&r.span_id),
                    opt_literal(&r.parent_span_id),
                    opt_literal(&r.run_id),
                    escape(&r.name),
                    escape(&r.kind),
                    escape(&r.status_code),
                    escape(&r.status_message),
                    r.duration_ns,
                    escape(&r.scope_name),
                    json_literal(&r.attributes),
                    json_literal(&r.resource),
                )
            })
            .collect();
        self.insert(
            "otel_spans",
            "\"ts\", \"service\", \"trace_id\", \"span_id\", \"parent_span_id\", \"run_id\", \"name\", \"kind\", \"status_code\", \"status_message\", \"duration_ns\", \"scope_name\", \"attributes\", \"resource\"",
            values,
        )
        .await
    }

    async fn write_logs(&self, rows: Vec<LogRow>) -> anyhow::Result<()> {
        let values = rows
            .iter()
            .map(|r| {
                format!(
                    "({},'{}',{},'{}','{}','{}','{}',{},'{}',{},{})",
                    r.ts_nanos,
                    escape(&r.service),
                    r.severity_num,
                    escape(&r.severity_text),
                    escape(&r.body),
                    escape(&r.trace_id),
                    escape(&r.span_id),
                    opt_literal(&r.run_id),
                    escape(&r.scope_name),
                    json_literal(&r.attributes),
                    json_literal(&r.resource),
                )
            })
            .collect();
        self.insert(
            "otel_logs",
            "\"ts\", \"service\", \"severity_num\", \"severity_text\", \"body\", \"trace_id\", \"span_id\", \"run_id\", \"scope_name\", \"attributes\", \"resource\"",
            values,
        )
        .await
    }

    async fn write_metric_points(&self, rows: Vec<MetricPointRow>) -> anyhow::Result<()> {
        let values = rows
            .iter()
            .map(|r| {
                format!(
                    "({},'{}','{}',{},{},{})",
                    r.ts_nanos / 1_000_000, // TIMESTAMP(3): millis
                    escape(&r.service),
                    escape(&r.name),
                    r.value,
                    r.is_monotonic,
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(
            "otel_metrics_points",
            "\"ts\", \"service\", \"name\", \"value\", \"is_monotonic\", \"attributes\"",
            values,
        )
        .await
    }

    async fn write_histograms(&self, rows: Vec<HistogramRow>) -> anyhow::Result<()> {
        let values = rows
            .iter()
            .map(|r| {
                format!(
                    "({},'{}','{}',{},{},{},{},{})",
                    r.ts_nanos / 1_000_000,
                    escape(&r.service),
                    escape(&r.name),
                    r.count,
                    r.sum,
                    json_literal(&serde_json::json!(r.bucket_counts)),
                    json_literal(&serde_json::json!(r.bounds)),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(
            "otel_metrics_histograms",
            "\"ts\", \"service\", \"name\", \"count\", \"sum\", \"bucket_counts\", \"bounds\", \"attributes\"",
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
        self.select_spans(&format!(r#""trace_id" = '{}'"#, escape(trace_id)), "")
            .await
    }

    async fn spans_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<SpanRow>> {
        self.select_spans(
            &format!(r#""run_id" = '{}'"#, escape(run_id)),
            &format!(" LIMIT {limit}"),
        )
        .await
    }

    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>> {
        self.select_logs(
            &format!(r#""run_id" = '{}'"#, escape(run_id)),
            &format!(" LIMIT {limit}"),
        )
        .await
    }

    async fn logs_by_trace(&self, trace_id: &str) -> anyhow::Result<Vec<LogRow>> {
        self.select_logs(&format!(r#""trace_id" = '{}'"#, escape(trace_id)), "")
            .await
    }

    async fn metric_names(&self) -> anyhow::Result<Vec<String>> {
        let rows = self
            .sql(
                r#"SELECT DISTINCT "name" FROM otel_metrics_points
                   UNION SELECT DISTINCT "name" FROM otel_metrics_histograms
                   ORDER BY "name""#,
            )
            .await?;
        Ok(rows.iter().map(|r| str_at(r, 0)).collect())
    }

    async fn service_names(&self) -> anyhow::Result<Vec<String>> {
        let rows = self
            .sql(
                r#"SELECT DISTINCT "service" FROM otel_metrics_points
                   UNION SELECT DISTINCT "service" FROM otel_spans
                   ORDER BY "service""#,
            )
            .await?;
        Ok(rows.iter().map(|r| str_at(r, 0)).collect())
    }

    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
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
        let service_clause = service
            .map(|svc| format!(r#" AND "service" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let rows = self
            .sql(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "ts") AS BIGINT)
                          AS "bucket_ms", {sql_agg}("value") AS "agg_value"
                   FROM otel_metrics_points
                   WHERE "name" = '{}'{service_clause}
                     AND "ts" >= {} AND "ts" <= {}
                   GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
                escape(name),
                sql_ts(range.start() / 1_000_000),
                sql_ts(range.end() / 1_000_000),
            ))
            .await?;
        let mut series: Vec<SeriesPoint> = rows
            .iter()
            .map(|row| SeriesPoint {
                ts_nanos: u128_at(row, 0) * 1_000_000,
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
        let service_clause = service
            .map(|svc| format!(r#" AND "service" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let rows = self
            .sql(&format!(
                r#"SELECT CAST("ts" AS BIGINT) AS "ts_ms", "count", "sum",
                          json_to_string("bucket_counts"), json_to_string("bounds")
                   FROM otel_metrics_histograms
                   WHERE "name" = '{}'{service_clause}
                     AND "ts" >= {} AND "ts" <= {}
                   ORDER BY "ts" ASC"#,
                escape(name),
                sql_ts(range.start() / 1_000_000),
                sql_ts(range.end() / 1_000_000),
            ))
            .await?;
        let step = step_nanos.max(1);
        let mut buckets: std::collections::BTreeMap<u128, Vec<HistogramRow>> = Default::default();
        for row in &rows {
            let ts_nanos = u128_at(row, 0) * 1_000_000;
            let histogram = HistogramRow {
                ts_nanos,
                service: String::new(),
                name: name.to_string(),
                count: row.get(1).and_then(|v| v.as_u64()).unwrap_or(0),
                sum: row.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0),
                bucket_counts: serde_json::from_value(json_at(row, 3)).unwrap_or_default(),
                bounds: serde_json::from_value(json_at(row, 4)).unwrap_or_default(),
                attributes: serde_json::Value::Null,
            };
            buckets
                .entry((ts_nanos / step) * step)
                .or_default()
                .push(histogram);
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, rows)| SeriesPoint {
                ts_nanos,
                value: crate::memory::quantile_from_histograms(&rows, q),
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
}
