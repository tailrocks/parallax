use super::*;

#[async_trait::async_trait]
impl crate::adapter::ServiceAnalyticsStore for GreptimeStore {
    async fn service_names(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<String>> {
        let rows = self.sql_lenient(&Self::service_names_sql(&range)).await?;
        Ok(rows
            .iter()
            .map(|r| str_at(r, 0))
            .filter(|s| !s.is_empty())
            .collect())
    }

    async fn overview_totals(&self, range: RangeInclusive<u128>) -> StorageResult<OverviewTotals> {
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
    ) -> StorageResult<Vec<SeriesPoint>> {
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
    ) -> StorageResult<Vec<ServiceSummary>> {
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
    ) -> StorageResult<Vec<ReleaseWindow>> {
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
    ) -> StorageResult<Vec<ServiceCatalogRow>> {
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
    ) -> StorageResult<SpanRed> {
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
