use super::*;

#[async_trait::async_trait]
impl crate::adapter::RumSessionStore for GreptimeStore {
    async fn rum_sessions(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        error_only: bool,
        limit: usize,
    ) -> StorageResult<Vec<crate::adapter::RumSession>> {
        let session_expr = trace_attr_expr(semconv::SESSION_ID);
        let mut clauses = vec![
            format!(
                r#""timestamp" >= {} AND "timestamp" <= {}"#,
                sql_ts(*range.start()),
                sql_ts(*range.end())
            ),
            format!(r#"{session_expr} IS NOT NULL AND {session_expr} != ''"#),
        ];
        if let Some(service) = service {
            clauses.push(format!(r#""service_name" = '{}'"#, escape(service)));
        }
        let having = if error_only {
            r#" HAVING SUM(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END) > 0"#
        } else {
            ""
        };
        let sql = format!(
            r#"SELECT {session_expr} AS "session_id",
                      MAX("service_name") AS "svc",
                      CAST(MIN("timestamp") AS BIGINT) AS "start_ts",
                      CAST(MAX("timestamp") AS BIGINT) AS "end_ts",
                      COUNT(*) AS "n",
                      COUNT(DISTINCT "trace_id") AS "traces",
                      SUM(CASE WHEN "span_name" = '{}' THEN 1 ELSE 0 END) AS "views",
                      SUM(CASE WHEN "span_name" = '{}' THEN 1 ELSE 0 END) AS "vitals",
                      SUM(CASE WHEN "span_status_code" = 'STATUS_CODE_ERROR' THEN 1 ELSE 0 END) AS "errors"
               FROM opentelemetry_traces
               WHERE {}
               GROUP BY {session_expr}{having} ORDER BY "end_ts" DESC LIMIT {limit}"#,
            semconv::APP_SCREEN_NAME,
            semconv::BROWSER_WEB_VITAL,
            clauses.join(" AND "),
        );
        let rows = match self.sql_lenient(&sql).await {
            Ok(rows) => rows,
            Err(error) if is_missing_column(&error) || is_missing_table(&error) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        Ok(rows
            .iter()
            .map(|row| {
                let error_count = u64::try_from(u128_at(row, 8)).unwrap_or(u64::MAX);
                crate::adapter::RumSession {
                    session_id: str_at(row, 0),
                    service: str_at(row, 1),
                    start_nanos: u128_at(row, 2),
                    end_nanos: u128_at(row, 3),
                    span_count: u64::try_from(u128_at(row, 4)).unwrap_or(u64::MAX),
                    trace_count: u64::try_from(u128_at(row, 5)).unwrap_or(u64::MAX),
                    view_count: u64::try_from(u128_at(row, 6)).unwrap_or(u64::MAX),
                    vital_count: u64::try_from(u128_at(row, 7)).unwrap_or(u64::MAX),
                    error_count,
                    has_error: error_count > 0,
                }
            })
            .filter(|session| !session.session_id.is_empty())
            .collect())
    }

    async fn rum_session_detail(
        &self,
        session_id: &str,
        limit: usize,
    ) -> StorageResult<Option<crate::adapter::RumSessionDetail>> {
        let session_expr = trace_attr_expr(semconv::SESSION_ID);
        let spans = match self
            .select_spans(
                &format!(r#"{session_expr} = '{}'"#, escape(session_id)),
                r#" ORDER BY "timestamp" ASC"#,
                &format!(" LIMIT {MAX_ROWS}"),
            )
            .await
        {
            Ok(spans) => spans,
            Err(error) if is_missing_column(&error) || is_missing_table(&error) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        Ok(parallax_storage::projections::project_rum_session_detail(
            &spans, session_id, limit,
        ))
    }
}
