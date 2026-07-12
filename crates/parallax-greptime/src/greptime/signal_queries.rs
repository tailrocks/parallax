use super::*;

#[async_trait::async_trait]
impl crate::adapter::LogStore for GreptimeStore {
    async fn logs_by_run(&self, run_id: &str, limit: usize) -> anyhow::Result<Vec<LogRow>> {
        let mut logs = self
            .select_logs(
                &format!(
                    r#"{} = '{}'"#,
                    wire_attr_ident(semconv::PARALLAX_RUN_ID),
                    escape(run_id)
                ),
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
}

#[async_trait::async_trait]
impl MetricStore for GreptimeStore {
    async fn metric_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>> {
        Ok(self
            .discover_metric_names(&range)
            .await?
            .into_iter()
            .collect())
    }

    async fn metric_labels(&self, name: &str) -> anyhow::Result<Vec<String>> {
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
    ) -> anyhow::Result<Vec<String>> {
        anyhow::ensure!(
            metric_group_label_allowed(label),
            "high-cardinality identifier - filter, don't group"
        );
        let Some((table, labels)) = self.resolved_metric_table(name).await? else {
            return Ok(Vec::new());
        };
        anyhow::ensure!(
            labels.iter().any(|known| known == label),
            "unknown metric label"
        );
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
}
