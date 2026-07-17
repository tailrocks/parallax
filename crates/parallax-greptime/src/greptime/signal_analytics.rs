use super::*;
use crate::adapter::AttributeFilter;

#[async_trait::async_trait]
impl crate::adapter::LogAnalyticsStore for GreptimeStore {
    async fn logs_search(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        attribute_filters: &[AttributeFilter],
        limit: usize,
    ) -> StorageResult<Vec<LogRow>> {
        let clauses = log_filter_clauses(
            service,
            &range,
            severity_min,
            severity_max,
            body_contains,
            attribute_filters,
        );
        // Body tiebreak keeps equal-timestamp rows in a stable order across
        // refreshes (corpus id l-bodies: five rows share one nanosecond).
        self.select_logs(
            &clauses.join(" AND "),
            r#" ORDER BY "timestamp" DESC, "body" ASC"#,
            &format!(" LIMIT {limit}"),
        )
        .await
        .map_err(Into::into)
    }

    async fn log_facets(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        attribute_filters: &[AttributeFilter],
    ) -> StorageResult<Vec<crate::adapter::Facet>> {
        let clauses = log_filter_clauses(
            service,
            &range,
            severity_min,
            severity_max,
            body_contains,
            attribute_filters,
        );
        let base = clauses.join(" AND ");
        let mut facets = Vec::new();
        for dimension in crate::adapter::LOG_FACET_DIMENSIONS {
            let Some(expr) = log_string_expr(dimension) else {
                continue;
            };
            let sql = format!(
                r#"SELECT {expr} AS "value", COUNT(*) AS "n"
                   FROM opentelemetry_logs
                   WHERE {base} AND {expr} IS NOT NULL AND {expr} != ''
                   GROUP BY "value"
                   ORDER BY "n" DESC, "value" ASC
                   LIMIT {}"#,
                crate::adapter::FACET_VALUES_CAP
            );
            let rows = self.sql_lenient(&sql).await?;
            let values = rows
                .iter()
                .filter_map(|row| {
                    let value = row.first()?.as_str()?.to_string();
                    let count = row.get(1).and_then(|n| {
                        n.as_u64()
                            .or_else(|| n.as_str().and_then(|s| s.parse().ok()))
                    })?;
                    Some(FieldValueCount { value, count })
                })
                .collect();
            facets.push(crate::adapter::Facet {
                dimension: (*dimension).to_string(),
                values,
            });
        }
        Ok(facets)
    }
}

#[async_trait::async_trait]
impl crate::adapter::RuntimeMetricStore for GreptimeStore {
    async fn metric_series_grouped(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        group_by: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> StorageResult<Vec<(String, Vec<SeriesPoint>)>> {
        if !metric_group_label_allowed(group_by) {
            return Err(StorageError::query(anyhow::anyhow!(
                "high-cardinality identifier - filter, don't group"
            )));
        }
        let Some((table, labels)) = self.resolved_metric_table(name).await? else {
            return Ok(Vec::new());
        };
        if !labels.iter().any(|label| label == group_by) {
            return Err(StorageError::query(anyhow::anyhow!("unknown metric label")));
        }
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let agg_expr = metric_agg_expr(agg, "greptime_value", "greptime_timestamp");
        let mut service_clause = service
            .map(|svc| format!(r#" AND "service_name" = '{}'"#, escape(svc)))
            .unwrap_or_default();
        if let Some(filters) = metric_attribute_filters_sql(attribute_filters) {
            service_clause.push_str(&format!(" AND {filters}"));
        }
        // native: metric-engine tags are real columns (resource attrs promoted
        // to tags); group on the quoted tag column, missing → "(none)".
        let group_col = format!(r#""{}""#, escape_ident(group_by));
        let rows = self
            .sql_arrow_lenient(&format!(
                r#"SELECT COALESCE(CAST({group_col} AS STRING), '(none)') AS "grp",
                          CAST(date_bin(INTERVAL '{step_secs} seconds', "greptime_timestamp") AS BIGINT)
                          AS "bucket_ms", {agg_expr} AS "agg_value"
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
                let series = match agg {
                    MetricAgg::Rate => {
                        crate::adapter::rate_from_buckets(&series, step_secs * 1_000_000_000)
                    }
                    MetricAgg::Increase => crate::adapter::increase_from_buckets(&series),
                    _ => series,
                };
                (group, series)
            })
            .collect())
    }

    async fn runtime_snapshot(
        &self,
        service: Option<&str>,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<RuntimeMetricSeries>> {
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
                        .metric_series(
                            &metric,
                            service,
                            invocation_id,
                            &[],
                            range,
                            step_nanos,
                            MetricAgg::Avg,
                        )
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
    ) -> StorageResult<Vec<SeriesPoint>> {
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
    ) -> StorageResult<Vec<SeriesPoint>> {
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
        attribute_filters: &[AttributeFilter],
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>> {
        let step_secs = (step_nanos / 1_000_000_000).max(1);
        let clauses = log_filter_clauses(
            service,
            &range,
            severity_min,
            severity_max,
            body_contains,
            attribute_filters,
        );
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
    async fn raw_sql(&self, query: &str) -> StorageResult<crate::adapter::SqlResult> {
        if !raw_sql_read_only(query) {
            return Err(StorageError::query(anyhow::anyhow!(
                "raw_sql: read-only statements only"
            )));
        }
        self.sql_with_schema(query).await.map_err(Into::into)
    }
}
