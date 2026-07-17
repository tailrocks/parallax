use super::*;

#[async_trait::async_trait]
impl MetricAnalyticsStore for GreptimeStore {
    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> StorageResult<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        // Run-scoped reads hit the `invocation_metric_points` extension table (ns time
        // index, `value` column); aggregate reads hit the per-metric native
        // table (ms `greptime_timestamp`, `greptime_value`, `service_name` tag).
        let rows = if let Some(invocation_id) = invocation_id {
            let service_clause = service
                .map(|svc| format!(r#" AND "service" = '{}'"#, escape(svc)))
                .unwrap_or_default();
            let name_filter = metric_name_sql_filter(r#""name""#, name);
            let agg_expr = metric_agg_expr(agg, "value", "ts");
            self.sql_arrow_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "ts") AS BIGINT)
                          AS "bucket_ns", {agg_expr} AS "agg_value"
                   FROM invocation_metric_points
                   WHERE {name_filter} AND "invocation_id" = '{}'{service_clause}
                     AND "ts" >= {} AND "ts" <= {}
                   GROUP BY "bucket_ns" ORDER BY "bucket_ns""#,
                escape(invocation_id),
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
            let agg_expr = metric_agg_expr(agg, "greptime_value", "greptime_timestamp");
            self.sql_arrow_lenient(&format!(
                r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", {agg_expr} AS "agg_value"
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
        let scale = if invocation_id.is_some() {
            1
        } else {
            1_000_000
        };
        let mut series: Vec<SeriesPoint> = rows
            .iter()
            .map(|row| SeriesPoint {
                ts_nanos: u128_at(row, 0) * scale,
                value: row.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            })
            .collect();
        if agg == MetricAgg::Rate {
            series = crate::adapter::rate_from_buckets(&series, step_secs * 1_000_000_000);
        } else if agg == MetricAgg::Increase {
            series = crate::adapter::increase_from_buckets(&series);
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
    ) -> StorageResult<Vec<SeriesPoint>> {
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
    ) -> StorageResult<Vec<Vec<SeriesPoint>>> {
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

    async fn histogram_avg(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>> {
        // Latest cumulative `_sum`/`_count` per bucket (MAX merge, plan 085
        // shape), Δsum/Δcount computed client-side with reset clamping.
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let mut stat_series: Vec<Vec<SeriesPoint>> = Vec::with_capacity(2);
        for suffix in ["_sum", "_count"] {
            let Some(table) = self.metric_table_for_name(name, Some(suffix)).await? else {
                return Ok(Vec::new());
            };
            let rows = self
                .sql_arrow_lenient(&format!(
                    r#"SELECT CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                              AS "bucket_ms", max("greptime_value") AS "cumulative"
                       FROM "{}"
                       WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}{service_clause}
                       GROUP BY "bucket_ms" ORDER BY "bucket_ms""#,
                    escape_ident(&table),
                    sql_ts(range.start() / 1_000_000),
                    sql_ts(range.end() / 1_000_000),
                ))
                .await?;
            stat_series.push(
                rows.iter()
                    .map(|row| SeriesPoint {
                        ts_nanos: u128_at(row, 0) * 1_000_000,
                        value: row.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
                    })
                    .collect(),
            );
        }
        let counts = stat_series.pop().unwrap_or_default();
        let sums = stat_series.pop().unwrap_or_default();
        Ok(crate::adapter::histogram_avg_from_cumulative(
            &sums, &counts,
        ))
    }

    async fn metric_exemplars(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<MetricExemplarRow>> {
        let service_clause = service
            .map(|svc| format!(r#" AND "service" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT CAST("ts" AS BIGINT) AS "ts_nanos",
                          "service", "name", "value", "trace_id", "span_id", "invocation_id",
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
                invocation_id: opt_str_at(row, 6),
                attributes: json_at(row, 7),
            })
            .collect())
    }
}
