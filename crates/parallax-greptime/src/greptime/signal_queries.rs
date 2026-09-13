use super::*;

#[async_trait::async_trait]
impl crate::adapter::LogStore for GreptimeStore {
    async fn logs_by_invocation(
        &self,
        invocation_id: &str,
        limit: usize,
    ) -> StorageResult<Vec<LogRow>> {
        let mut logs = self
            .select_logs(
                &format!(
                    r#"{} = '{}'"#,
                    wire_attr_ident(semconv::CLI_INVOCATION_ID),
                    escape(invocation_id)
                ),
                r#" ORDER BY "timestamp" DESC, "body" ASC"#,
                &format!(" LIMIT {limit}"),
            )
            .await?;
        logs.reverse();
        Ok(logs)
    }

    async fn logs_by_trace(&self, trace_id: &str) -> StorageResult<Vec<LogRow>> {
        self.select_logs(
            &format!(r#""trace_id" = '{}'"#, escape(trace_id)),
            r#" ORDER BY "timestamp" ASC, "body" ASC"#,
            "",
        )
        .await
        .map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl MetricStore for GreptimeStore {
    async fn metric_names(&self, range: RangeInclusive<u128>) -> StorageResult<Vec<String>> {
        Ok(self
            .discover_metric_names(&range)
            .await?
            .into_iter()
            .collect())
    }

    async fn metric_labels(&self, name: &str) -> StorageResult<Vec<String>> {
        if let Some((_, labels)) = self.resolved_metric_table(name).await? {
            return Ok(labels);
        }
        Ok(self.exp_metric_labels(name).await?)
    }

    async fn metric_label_values(
        &self,
        name: &str,
        label: &str,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<String>> {
        if !metric_group_label_allowed(label) {
            return Err(StorageError::query(anyhow::anyhow!(
                "high-cardinality identifier - filter, don't group"
            )));
        }
        let Some((table, labels)) = self.resolved_metric_table(name).await? else {
            return self.exp_metric_label_values(name, label, range).await;
        };
        if !labels.iter().any(|known| known == label) {
            return Err(StorageError::query(anyhow::anyhow!("unknown metric label")));
        }
        let label_ident = format!(r#""{}""#, escape_ident(label));
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT DISTINCT CAST({label_ident} AS STRING) AS "value"
                   FROM "{}"
                   WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}
                     AND {label_ident} IS NOT NULL
                   ORDER BY "value" LIMIT {METRIC_LABEL_VALUES_CAP}"#,
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

    async fn metric_catalog(
        &self,
        range: RangeInclusive<u128>,
        q: Option<&str>,
        kind: Option<MetricKind>,
        limit: usize,
    ) -> StorageResult<Vec<MetricCatalogEntry>> {
        // One schema scan classifies families, then one batched UNION ALL
        // collects per-service window stats — never per-metric round trips
        // (metric-summary contract: bounded and batched).
        let families = self.discover_metric_families().await?;
        let needle = q.map(str::to_ascii_lowercase);
        let selected: Vec<_> = families
            .into_iter()
            .filter(|family| {
                needle
                    .as_deref()
                    .is_none_or(|n| family.display.to_ascii_lowercase().contains(n))
                    && kind.is_none_or(|k| k == family.kind)
            })
            .take(limit)
            .collect();
        if selected.is_empty() {
            return Ok(Vec::new());
        }
        let from_ms = range.start() / 1_000_000;
        let to_ms = range.end() / 1_000_000;
        let arms: Vec<String> = selected
            .iter()
            .map(|family| {
                if let Some(exp_name) = family.exp_name.as_deref() {
                    return exp_catalog_arm(exp_name, &family.display, &range);
                }
                format!(
                    r#"SELECT '{}' AS "name", CAST("service_name" AS STRING) AS "service",
                              CAST(MAX("greptime_timestamp") AS BIGINT) AS "last_ms",
                              COUNT("greptime_value") AS "cnt"
                       FROM "{}"
                       WHERE "greptime_timestamp" >= {} AND "greptime_timestamp" <= {}
                       GROUP BY "service_name""#,
                    escape(&family.display),
                    escape_ident(&family.stats_table),
                    sql_ts(from_ms),
                    sql_ts(to_ms),
                )
            })
            .collect();
        // One giant UNION ALL over hundreds of per-metric tables can exhaust
        // the engine (observed: standalone GreptimeDB killed mid-query on a
        // 7-day catalog scan). Batch the arms: still one bounded query per
        // chunk, never per-metric round trips.
        const CATALOG_UNION_CHUNK: usize = 24;
        let mut rows = Vec::new();
        for chunk in arms.chunks(CATALOG_UNION_CHUNK) {
            rows.extend(self.sql_arrow_lenient(&chunk.join("\nUNION ALL\n")).await?);
        }
        let mut by_name: BTreeMap<String, MetricCatalogEntry> = BTreeMap::new();
        for family in &selected {
            by_name.insert(
                family.display.clone(),
                MetricCatalogEntry {
                    name: family.display.clone(),
                    kind: family.kind,
                    unit: runtime_metric_unit(&family.display),
                    services: Vec::new(),
                    last_datapoint_nanos: 0,
                    point_count: 0,
                },
            );
        }
        for row in &rows {
            let name = str_at(row, 0);
            let Some(entry) = by_name.get_mut(&name) else {
                continue;
            };
            let service = str_at(row, 1);
            if !service.is_empty() && !entry.services.contains(&service) {
                entry.services.push(service);
            }
            entry.last_datapoint_nanos =
                entry.last_datapoint_nanos.max(u128_at(row, 2) * 1_000_000);
            entry.point_count += u128_at(row, 3) as u64;
        }
        let mut out: Vec<MetricCatalogEntry> = by_name
            .into_values()
            .filter(|entry| entry.point_count > 0)
            .collect();
        for entry in &mut out {
            entry.services.sort();
        }
        Ok(out)
    }
}

impl GreptimeStore {
    /// Labels for converted-exp metrics: the extension table has no
    /// per-attribute tag columns, so keys come from a bounded sample of
    /// stored attribute objects (same scalar-only rule as native tables).
    async fn exp_metric_labels(&self, name: &str) -> StorageResult<Vec<String>> {
        let name_filter = metric_name_sql_filter(r#""name""#, name);
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT json_to_string("attributes") FROM "{EXP_HISTOGRAMS_TABLE}"
                   WHERE {name_filter} ORDER BY "ts" DESC LIMIT 500"#,
            ))
            .await?;
        let mut labels = BTreeSet::new();
        for row in &rows {
            if let Some(object) = json_at(row, 0).as_object() {
                for (key, value) in object {
                    if metric_group_label_allowed(key)
                        && matches!(
                            value,
                            serde_json::Value::String(_)
                                | serde_json::Value::Bool(_)
                                | serde_json::Value::Number(_)
                        )
                    {
                        labels.insert(key.clone());
                    }
                }
            }
        }
        Ok(labels.into_iter().collect())
    }

    /// Label values for converted-exp metrics: bounded newest-first attribute
    /// sample, filtered client-side (same unknown-label error and
    /// [`METRIC_LABEL_VALUES_CAP`] as the native path).
    async fn exp_metric_label_values(
        &self,
        name: &str,
        label: &str,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<String>> {
        let labels = self.exp_metric_labels(name).await?;
        if !labels.iter().any(|known| known == label) {
            return Err(StorageError::query(anyhow::anyhow!("unknown metric label")));
        }
        let name_filter = metric_name_sql_filter(r#""name""#, name);
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT json_to_string("attributes") FROM "{EXP_HISTOGRAMS_TABLE}"
                   WHERE {name_filter} AND "ts" >= {} AND "ts" <= {}
                   ORDER BY "ts" DESC LIMIT 2000"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
            ))
            .await?;
        let mut values = BTreeSet::new();
        for row in &rows {
            let value = match json_at(row, 0).get(label) {
                Some(serde_json::Value::String(value)) => value.clone(),
                Some(serde_json::Value::Bool(value)) => value.to_string(),
                Some(serde_json::Value::Number(value)) => value.to_string(),
                _ => continue,
            };
            if attribute_compare_value_allowed(&value) {
                values.insert(value);
                if values.len() >= METRIC_LABEL_VALUES_CAP {
                    break;
                }
            }
        }
        Ok(values.into_iter().collect())
    }
}
