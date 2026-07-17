use super::*;

#[async_trait::async_trait]
impl crate::adapter::TraceAnalyticsStore for GreptimeStore {
    async fn traces_search(
        &self,
        query: &crate::adapter::TraceQuery,
    ) -> StorageResult<crate::adapter::TraceList> {
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
        // Materialized client-side: `"trace_id" IN (SELECT …)` semi-joins
        // return zero rows on the live engine (see invocation_trace_ids).
        let participation = match &query.service {
            Some(service) => {
                let ids_sql = format!(
                    r#"SELECT DISTINCT "trace_id" FROM opentelemetry_traces
                       WHERE "service_name" = '{}' AND {scan_where}
                       LIMIT {MAX_ROWS}"#,
                    escape(service)
                );
                let ids: Vec<String> = self
                    .sql_lenient(&ids_sql)
                    .await?
                    .iter()
                    .filter_map(|row| row.first().and_then(|v| v.as_str()))
                    .filter(|id| !id.is_empty())
                    .map(|id| format!("'{}'", escape(id)))
                    .collect();
                if ids.is_empty() {
                    return Ok(crate::adapter::TraceList {
                        items: Vec::new(),
                        total: 0,
                    });
                }
                format!(r#" AND "trace_id" IN ({})"#, ids.join(", "))
            }
            None => String::new(),
        };
        // Representative-span filters, applied after the per-trace pick.
        let mut rep = vec!["\"rn\" = 1".to_string()];
        if let Some(min) = query.min_duration_ns {
            rep.push(format!(
                r#""dur" >= {}"#,
                u64::try_from(min).map_err(anyhow::Error::from)?
            ));
        }
        if let Some(max) = query.max_duration_ns {
            rep.push(format!(
                r#""dur" <= {}"#,
                u64::try_from(max).map_err(anyhow::Error::from)?
            ));
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
    ) -> StorageResult<Vec<AttributeCompareRow>> {
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

    async fn span_field_keys(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<FieldKey>> {
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
    ) -> StorageResult<FieldStats> {
        if !span_field_key_allowed(key) {
            return Err(StorageError::query(anyhow::anyhow!("invalid field key")));
        }
        let Some(column) = self
            .span_field_columns()
            .await?
            .into_iter()
            .find(|column| column.key == key)
        else {
            return Err(StorageError::query(anyhow::anyhow!(
                "unknown span field key"
            )));
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
    ) -> StorageResult<Vec<ServiceEdge>> {
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
    ) -> StorageResult<Vec<ErrorEventRow>> {
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
