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
        Ok(self
            .resolved_metric_table(name)
            .await?
            .map(|(_, labels)| labels)
            .unwrap_or_default())
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
            return Ok(Vec::new());
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
        let rows = self.sql_arrow_lenient(&arms.join("\nUNION ALL\n")).await?;
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
