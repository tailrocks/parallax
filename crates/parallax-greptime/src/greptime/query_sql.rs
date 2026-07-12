use super::*;

impl GreptimeStore {
    /// Pure SQL builder for `traces_search` (golden-tested).
    pub(super) fn traces_search_sql(
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

    pub(super) fn histogram_count_series_sql(
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
    pub(super) fn histogram_quantile_bucket_sql(
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

    pub(super) fn service_names_sql(range: &RangeInclusive<u128>) -> String {
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

    pub(super) fn span_field_stats_sample_sql(value_expr: &str, sample_where: &str) -> String {
        format!(
            r#"SELECT {value_expr} AS "value"
               FROM opentelemetry_traces
               WHERE {sample_where}
               ORDER BY "timestamp" DESC
               LIMIT {MAX_ROWS}"#
        )
    }

    pub(super) fn span_field_stats_top_values_sql(sample_sql: &str) -> String {
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

    pub(super) fn service_map_edges_sql(id_list: &str, range: &RangeInclusive<u128>) -> String {
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

    pub(super) fn select_spans_sql(where_clause: &str, order: &str, limit_clause: &str) -> String {
        format!(r#"SELECT * FROM opentelemetry_traces WHERE {where_clause}{order}{limit_clause}"#)
    }

    pub(super) fn select_logs_sql(where_clause: &str, order: &str, limit_clause: &str) -> String {
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

    pub(super) fn span_attribute_counts_sql(
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

pub(super) fn raw_sql_read_only(query: &str) -> bool {
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
pub(super) fn is_missing_table(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .to_ascii_lowercase()
        .contains("table not found")
}

pub(super) fn is_missing_column(error: &anyhow::Error) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    (message.contains("column") || message.contains("field"))
        && (message.contains("not found") || message.contains("not exist"))
}

pub(super) fn json_literal(value: &serde_json::Value) -> String {
    format!("parse_json('{}')", escape(&value.to_string()))
}

pub(super) fn opt_literal(value: &Option<String>) -> String {
    match value {
        Some(s) => format!("'{}'", escape(s)),
        None => "NULL".to_string(),
    }
}
