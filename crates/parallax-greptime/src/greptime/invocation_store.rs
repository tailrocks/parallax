use super::*;

#[async_trait::async_trait]
impl crate::adapter::InvocationStore for GreptimeStore {
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

    async fn observed_invocations(
        &self,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<crate::adapter::ObservedInvocation>> {
        let mut runs: HashMap<String, crate::adapter::ObservedInvocation> = HashMap::new();
        let start = sql_ts(*range.start());
        let end = sql_ts(*range.end());
        let trace_invocation_column = trace_attr_expr(semconv::CLI_INVOCATION_ID);
        let command_column = span_attr_ident(semconv::CLI_COMMAND_NAME);
        let app_mode_column = span_attr_ident(semconv::APP_MODE);
        let native_span_rows = match self
            .sql_lenient(&format!(
                r#"SELECT {trace_invocation_column} AS "invocation_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT "span_id") AS "n",
                          MAX("service_name") AS "svc",
                          MAX({command_column}) AS "last_command",
                          MAX({app_mode_column}) AS "app_mode"
                   FROM opentelemetry_traces
                   WHERE {trace_invocation_column} IS NOT NULL AND {trace_invocation_column} != ''
                     AND "timestamp" >= {start} AND "timestamp" <= {end}
                   GROUP BY {trace_invocation_column} ORDER BY "last_ts" DESC LIMIT {limit}"#
            ))
            .await
        {
            Ok(rows) => rows,
            Err(error) if is_missing_column(&error) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        let mut native_span_invocation_ids = BTreeSet::new();
        for row in &native_span_rows {
            if let Some(invocation_id) = absorb_observed_run(&mut runs, row, true) {
                native_span_invocation_ids.insert(invocation_id);
            }
        }
        if native_span_rows.len() >= limit {
            let mut runs: Vec<_> = runs.into_values().collect();
            runs.sort_by_key(|r| std::cmp::Reverse(r.last_nanos));
            runs.truncate(limit);
            return Ok(runs);
        }
        let invocation_col = wire_attr_ident(semconv::CLI_INVOCATION_ID);
        let log_svc = log_service_name_expr();
        let sources = [
            (
                format!(
                    r#"SELECT l.{invocation_col} AS "invocation_id",
                          CAST(MIN(s."timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX(s."timestamp") AS BIGINT) AS "last_ts",
                          COUNT(DISTINCT s."span_id") AS "n",
                          MAX(s."service_name") AS "svc"
                   FROM opentelemetry_logs l
                   JOIN opentelemetry_traces s ON s."trace_id" = l."trace_id"
                   WHERE l.{invocation_col} IS NOT NULL
                     AND l.{invocation_col} != ''
                     AND l."timestamp" >= {start} AND l."timestamp" <= {end}
                     AND s."timestamp" >= {start} AND s."timestamp" <= {end}
                   GROUP BY "invocation_id" ORDER BY "last_ts" DESC LIMIT "#
                ),
                true,
            ),
            (
                format!(
                    r#"SELECT {invocation_col} AS "invocation_id",
                          CAST(MIN("timestamp") AS BIGINT) AS "first_ts",
                          CAST(MAX("timestamp") AS BIGINT) AS "last_ts",
                          COUNT(*) AS "n",
                          MAX({log_svc}) AS "svc"
                   FROM opentelemetry_logs
                   WHERE {invocation_col} IS NOT NULL AND {invocation_col} != ''
                     AND "timestamp" >= {start} AND "timestamp" <= {end}
                   GROUP BY "invocation_id" ORDER BY "last_ts" DESC LIMIT "#
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
                if is_span && native_span_invocation_ids.contains(&str_at(row, 0)) {
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

    async fn sessions_by_invocation(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<crate::adapter::InvocationSession>> {
        let rows = self
            .invocation_event_logs(
                invocation_id,
                &[
                    semconv::SESSION_START_EVENT_NAME,
                    semconv::SESSION_END_EVENT_NAME,
                ],
                &range,
            )
            .await?;
        Ok(parallax_storage::projections::pair_sessions(&rows, limit))
    }

    async fn screen_visits(
        &self,
        invocation_id: Option<&str>,
        session_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<crate::adapter::ScreenVisit>> {
        let mut clauses = vec![
            format!(
                r#""timestamp" >= {} AND "timestamp" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ),
            format!(
                r#"{} IN ('{}','{}')"#,
                wire_attr_ident(semconv::EVENT_NAME),
                semconv::UI_SCREEN_ENTERED_EVENT_NAME,
                semconv::UI_SCREEN_EXITED_EVENT_NAME,
            ),
        ];
        if let Some(invocation_id) = invocation_id {
            clauses.push(format!(
                r#"{} = '{}'"#,
                wire_attr_ident(semconv::CLI_INVOCATION_ID),
                escape(invocation_id)
            ));
        }
        if let Some(session_id) = session_id {
            clauses.push(format!(
                r#"{} = '{}'"#,
                wire_attr_ident(semconv::SESSION_ID),
                escape(session_id)
            ));
        }
        let rows = match self
            .select_logs(
                &clauses.join(" AND "),
                r#" ORDER BY "timestamp" ASC"#,
                &format!(" LIMIT {MAX_ROWS}"),
            )
            .await
        {
            Ok(rows) => rows,
            Err(error) if is_missing_column(&error) || is_missing_table(&error) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        Ok(parallax_storage::projections::pair_screen_visits(
            &rows, session_id, limit,
        ))
    }

    async fn ui_actions(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<crate::adapter::UiAction>> {
        let spans = self
            .invocation_named_spans(
                Some(invocation_id),
                &format!(r#""span_name" = '{}'"#, semconv::UI_ACTION_SPAN_NAME),
                &range,
            )
            .await?;
        Ok(parallax_storage::projections::project_ui_actions(
            &spans, limit,
        ))
    }

    async fn background_cycles(
        &self,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<crate::adapter::BackgroundCycleSummary>> {
        let spans = self
            .invocation_named_spans(
                invocation_id,
                &format!(r#""span_name" = '{}'"#, semconv::BACKGROUND_CYCLE_SPAN_NAME),
                &range,
            )
            .await?;
        Ok(parallax_storage::projections::summarize_background_cycles(
            &spans, limit,
        ))
    }

    async fn jobs(
        &self,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<crate::adapter::JobSummary>> {
        let job_column = span_attr_ident(semconv::JOB_ID);
        let spans = self
            .invocation_named_spans(
                invocation_id,
                &format!(r#"{job_column} IS NOT NULL AND {job_column} != ''"#),
                &range,
            )
            .await?;
        Ok(parallax_storage::projections::summarize_jobs(&spans, limit))
    }

    async fn conversations(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<crate::adapter::ConversationSummary>> {
        let conversation_column = span_attr_ident(semconv::GEN_AI_CONVERSATION_ID);
        let spans = self
            .invocation_named_spans(
                Some(invocation_id),
                &format!(r#"{conversation_column} IS NOT NULL AND {conversation_column} != ''"#),
                &range,
            )
            .await?;
        Ok(parallax_storage::projections::summarize_conversations(
            &spans, limit,
        ))
    }
}

impl GreptimeStore {
    /// Bounded typed-event log window for one invocation (projection input).
    async fn invocation_event_logs(
        &self,
        invocation_id: &str,
        event_names: &[&str],
        range: &RangeInclusive<u128>,
    ) -> StorageResult<Vec<LogRow>> {
        let events = event_names
            .iter()
            .map(|name| format!("'{}'", escape(name)))
            .collect::<Vec<_>>()
            .join(",");
        let where_clause = format!(
            r#"{} = '{}' AND {} IN ({events})
               AND "timestamp" >= {} AND "timestamp" <= {}"#,
            wire_attr_ident(semconv::CLI_INVOCATION_ID),
            escape(invocation_id),
            wire_attr_ident(semconv::EVENT_NAME),
            sql_ts(*range.start()),
            sql_ts(*range.end()),
        );
        match self
            .select_logs(
                &where_clause,
                r#" ORDER BY "timestamp" ASC"#,
                &format!(" LIMIT {MAX_ROWS}"),
            )
            .await
        {
            Ok(rows) => Ok(rows),
            Err(error) if is_missing_column(&error) || is_missing_table(&error) => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
    }

    /// Bounded span window for the invocation projections. `extra_where`
    /// selects the span family (name or attribute presence); the invocation
    /// scope matches whole traces whose root/resource carries the id.
    async fn invocation_named_spans(
        &self,
        invocation_id: Option<&str>,
        extra_where: &str,
        range: &RangeInclusive<u128>,
    ) -> StorageResult<Vec<SpanRow>> {
        let mut clauses = vec![
            format!(
                r#""timestamp" >= {} AND "timestamp" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ),
            extra_where.to_string(),
        ];
        if let Some(invocation_id) = invocation_id {
            clauses.push(format!(
                r#""trace_id" IN (
                     SELECT DISTINCT "trace_id" FROM opentelemetry_traces
                     WHERE {} = '{}'
                   )"#,
                trace_attr_expr(semconv::CLI_INVOCATION_ID),
                escape(invocation_id)
            ));
        }
        match self
            .select_spans(
                &clauses.join(" AND "),
                r#" ORDER BY "timestamp" DESC"#,
                &format!(" LIMIT {MAX_ROWS}"),
            )
            .await
        {
            Ok(spans) => Ok(spans),
            Err(error) if is_missing_column(&error) || is_missing_table(&error) => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
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
