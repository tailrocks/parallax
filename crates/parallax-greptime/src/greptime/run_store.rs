use super::*;

#[async_trait::async_trait]
impl crate::adapter::RunStore for GreptimeStore {
    async fn error_events_by_fingerprint(
        &self,
        fingerprint: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<ErrorEventRow>> {
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
        Ok(rows.iter().map(|row| error_event_row(row)).collect())
    }

    async fn error_events_by_fingerprints(
        &self,
        fingerprints: &[String],
        range: RangeInclusive<u128>,
        limit_per_fingerprint: usize,
    ) -> StorageResult<HashMap<String, Vec<ErrorEventRow>>> {
        let mut events: HashMap<String, Vec<ErrorEventRow>> = fingerprints
            .iter()
            .map(|fingerprint| (fingerprint.clone(), Vec::new()))
            .collect();
        if fingerprints.is_empty() || limit_per_fingerprint == 0 {
            return Ok(events);
        }
        let fingerprints_sql = fingerprints
            .iter()
            .map(|fingerprint| format!("'{}'", escape(fingerprint)))
            .collect::<Vec<_>>()
            .join(", ");
        let rows = self
            .sql(&format!(
                r#"SELECT "ts_nanos", "service", "fingerprint", "error_type", "message",
                          "stacktrace", "source", "trace_id", "span_id", "attributes"
                   FROM (
                     SELECT CAST("ts" AS BIGINT) AS "ts_nanos", "service", "fingerprint",
                            "error_type", "message", "stacktrace", "source", "trace_id",
                            "span_id", json_to_string("attributes") AS "attributes",
                            ROW_NUMBER() OVER (
                              PARTITION BY "fingerprint" ORDER BY "ts" DESC
                            ) AS "event_rank"
                     FROM error_events
                     WHERE "fingerprint" IN ({fingerprints_sql})
                       AND "ts" >= {} AND "ts" <= {}
                   ) WHERE "event_rank" <= {}
                   ORDER BY "fingerprint", "ts_nanos" DESC"#,
                sql_ts(*range.start()),
                sql_ts(*range.end()),
                limit_per_fingerprint.min(MAX_ROWS),
            ))
            .await?;
        for row in &rows {
            let event = error_event_row(row);
            events
                .entry(event.fingerprint.clone())
                .or_default()
                .push(event);
        }
        Ok(events)
    }

    async fn observed_runs(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<crate::adapter::ObservedRun>> {
        let mut runs: HashMap<String, crate::adapter::ObservedRun> = HashMap::new();
        let start = sql_ts(*range.start());
        let end = sql_ts(*range.end());
        let trace_run_column = resource_attr_ident(semconv::PARALLAX_RUN_ID);
        let native_span_rows = match self
            .sql_lenient(&format!(
                r#"SELECT {trace_run_column} AS "run_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT "span_id") AS "n",
                          MAX("service_name") AS "svc"
                   FROM opentelemetry_traces
                   WHERE {trace_run_column} IS NOT NULL AND {trace_run_column} != ''
                     AND "timestamp" >= {start} AND "timestamp" <= {end}
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT {limit}"#
            ))
            .await
        {
            Ok(rows) => rows,
            Err(error) if is_missing_column(&error) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        let mut native_span_run_ids = BTreeSet::new();
        for row in &native_span_rows {
            if let Some(run_id) = absorb_observed_run(&mut runs, row, true) {
                native_span_run_ids.insert(run_id);
            }
        }
        if native_span_rows.len() >= limit {
            let mut runs: Vec<_> = runs.into_values().collect();
            runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
            runs.truncate(limit);
            return Ok(runs);
        }
        let run_col = wire_attr_ident(semconv::PARALLAX_RUN_ID);
        let log_svc = log_service_name_expr();
        let sources = [
            (
                format!(
                    r#"SELECT l.{run_col} AS "run_id",
                          CAST(MIN(s."timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX(s."timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT s."span_id") AS "n",
                          MAX(s."service_name") AS "svc"
                   FROM opentelemetry_logs l
                   JOIN opentelemetry_traces s ON s."trace_id" = l."trace_id"
                   WHERE l.{run_col} IS NOT NULL
                     AND l.{run_col} != ''
                     AND l."timestamp" >= {start} AND l."timestamp" <= {end}
                     AND s."timestamp" >= {start} AND s."timestamp" <= {end}
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#
                ),
                true,
            ),
            (
                format!(
                    r#"SELECT {run_col} AS "run_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(*) AS "n",
                          MAX({log_svc}) AS "svc"
                   FROM opentelemetry_logs
                   WHERE {run_col} IS NOT NULL AND {run_col} != ''
                     AND "timestamp" >= {start} AND "timestamp" <= {end}
                   GROUP BY "run_id" ORDER BY "last_ts" DESC LIMIT "#
                ),
                false,
            ),
        ];
        for (query, is_span) in sources {
            let rows = match self.sql_lenient(&format!("{query}{limit}")).await {
                Ok(rows) => rows,
                Err(error) if is_missing_column(&error) => Vec::new(),
                Err(error) => return Err(error.into()),
            };
            for row in &rows {
                if is_span && native_span_run_ids.contains(&str_at(row, 0)) {
                    continue;
                }
                absorb_observed_run(&mut runs, row, is_span);
            }
        }
        let mut runs: Vec<_> = runs.into_values().collect();
        runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
        runs.truncate(limit);
        Ok(runs)
    }
}

fn error_event_row(row: &[serde_json::Value]) -> ErrorEventRow {
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
