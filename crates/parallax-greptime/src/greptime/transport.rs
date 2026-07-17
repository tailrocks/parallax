use super::*;

impl GreptimeStore {
    /// Forward a raw OTLP/HTTP protobuf body to one of GreptimeDB's native
    /// `/v1/otlp/v1/...` endpoints. `headers` carries the per-signal pipeline /
    /// extract-keys / hints; the body is sent verbatim.
    pub(super) async fn forward_otlp(
        &self,
        path: &str,
        headers: &[(&str, &str)],
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        let mut request = self
            .client
            .post(format!("{}/v1/otlp/{path}", self.base_url))
            .header("content-type", "application/x-protobuf");
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        request.body(raw).send().await?.error_for_status()?;
        Ok(())
    }

    /// Like [`Self::sql`], but tolerant of a not-yet-created native table: the
    /// native OTLP tables (`opentelemetry_traces`/`_logs`, the per-metric engine
    /// tables) only exist after the first forward, so a read issued before any
    /// matching signal has arrived must read as **empty**, not error. Used by the
    /// typed read paths; the raw-SQL surface keeps strict [`Self::sql`].
    pub(super) async fn sql_lenient(
        &self,
        sql: &str,
    ) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        match self.sql(sql).await {
            Err(error) if is_missing_table(&error) => Ok(Vec::new()),
            other => other,
        }
    }

    /// Arrow+zstd sibling of [`Self::sql_lenient`] for heavy typed reads (plan 091).
    pub(super) async fn sql_arrow_lenient(
        &self,
        sql: &str,
    ) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        match self.sql_arrow(sql).await {
            Err(error) if is_missing_table(&error) => Ok(Vec::new()),
            other => other,
        }
    }

    /// Run one SQL statement; return the first result set's rows.
    ///
    /// Uses `greptimedb_v1` JSON — keep for DDL/admin, `information_schema`,
    /// single-row counts, `LIMIT 0` schema probes, and other tiny results.
    /// Heavy page/series reads use [`Self::sql_arrow`].
    pub async fn sql(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        let response = self.sql_json_response(sql).await?;
        // Success responses carry `output` (no `code`); failures carry
        // `error` (+ a non-zero `code`).
        if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
            let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            // Char-boundary-safe: byte slice at 200 can panic on multi-byte UTF-8.
            let sql_prefix: String = sql.chars().take(200).collect();
            anyhow::bail!("greptime sql failed (code {code}): {error} — sql: {sql_prefix}");
        }
        anyhow::ensure!(
            response.get("output").is_some(),
            "greptime sql returned neither output nor error: {response}"
        );
        let rows = response
            .pointer("/output/0/records/rows")
            .and_then(|r| r.as_array())
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.as_array().cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(rows)
    }

    /// Heavy read path: HTTP `format=arrow&compression=zstd` → same row shape
    /// as [`Self::sql`] (plan 091; measured GO in plan 090).
    pub async fn sql_arrow(&self, sql: &str) -> anyhow::Result<Vec<Vec<serde_json::Value>>> {
        let result = self.sql_with_schema_arrow(sql).await?;
        Ok(result.rows)
    }

    /// [`Self::sql_with_schema`] with the not-yet-created-table tolerance of
    /// [`Self::sql_lenient`]: returns an empty result set instead of erroring
    /// when the native table has not auto-created yet.
    pub(super) async fn sql_with_schema_lenient(
        &self,
        sql: &str,
    ) -> anyhow::Result<crate::adapter::SqlResult> {
        match self.sql_with_schema(sql).await {
            Err(error) if is_missing_table(&error) => Ok(crate::adapter::SqlResult {
                columns: Vec::new(),
                rows: Vec::new(),
            }),
            other => other,
        }
    }

    /// Arrow+zstd sibling of [`Self::sql_with_schema_lenient`].
    pub(super) async fn sql_with_schema_arrow_lenient(
        &self,
        sql: &str,
    ) -> anyhow::Result<crate::adapter::SqlResult> {
        match self.sql_with_schema_arrow(sql).await {
            Err(error) if is_missing_table(&error) => Ok(crate::adapter::SqlResult {
                columns: Vec::new(),
                rows: Vec::new(),
            }),
            other => other,
        }
    }

    /// Like [`Self::sql`], but also returns the result-set column names
    /// (the raw-SQL surface needs a generic grid, not a fixed projection).
    pub async fn sql_with_schema(&self, sql: &str) -> anyhow::Result<crate::adapter::SqlResult> {
        let response = self.sql_json_response(sql).await?;
        if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
            let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            anyhow::bail!("greptime sql failed (code {code}): {error}");
        }
        let columns = response
            .pointer("/output/0/records/schema/column_schemas")
            .and_then(|c| c.as_array())
            .map(|cols| {
                cols.iter()
                    .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let rows = response
            .pointer("/output/0/records/rows")
            .and_then(|r| r.as_array())
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.as_array().cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(crate::adapter::SqlResult { columns, rows })
    }

    /// Arrow+zstd variant of [`Self::sql_with_schema`] for wide/tall result sets.
    pub async fn sql_with_schema_arrow(
        &self,
        sql: &str,
    ) -> anyhow::Result<crate::adapter::SqlResult> {
        let bytes = self
            .client
            .post(format!(
                "{}/v1/sql?db=public&format=arrow&compression=zstd",
                self.base_url
            ))
            .header("X-Greptime-Timeout", SQL_QUERY_TIMEOUT_HEADER)
            .form(&[("sql", sql)])
            .send()
            .await?
            .bytes()
            .await?;
        let (columns, rows) = crate::arrow_sql::decode_arrow_ipc(&bytes).map_err(|error| {
            let sql_prefix: String = sql.chars().take(200).collect();
            anyhow::anyhow!("{error} — sql: {sql_prefix}")
        })?;
        Ok(crate::adapter::SqlResult { columns, rows })
    }

    pub(super) async fn sql_json_response(&self, sql: &str) -> anyhow::Result<serde_json::Value> {
        Ok(self
            .client
            .post(format!("{}/v1/sql?db=public", self.base_url))
            .header("X-Greptime-Timeout", SQL_QUERY_TIMEOUT_HEADER)
            .form(&[("sql", sql)])
            .send()
            .await?
            .json()
            .await?)
    }

    pub(super) async fn metric_table_for_name(
        &self,
        name: &str,
        suffix: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
        let cache_key = (name.to_string(), suffix.map(str::to_string));
        {
            let cache = self.metric_table_cache.read().await;
            if let Some(table) = cache.get(&cache_key) {
                return Ok(Some(table.clone()));
            }
        }
        let candidates = metric_table_candidates(name, suffix);
        if candidates.is_empty() {
            return Ok(None);
        }
        let quoted = candidates
            .iter()
            .map(|candidate| format!("'{}'", escape(candidate)))
            .collect::<Vec<_>>()
            .join(",");
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT "table_name" FROM information_schema.tables
                   WHERE "table_schema" = 'public' AND "table_name" IN ({quoted})"#
            ))
            .await?;
        let existing = rows
            .iter()
            .map(|row| str_at(row, 0))
            .collect::<BTreeSet<_>>();
        let found = candidates
            .into_iter()
            .find(|candidate| existing.contains(candidate));
        if let Some(ref table) = found {
            self.metric_table_cache
                .write()
                .await
                .insert(cache_key, table.clone());
        }
        Ok(found)
    }

    pub(super) async fn resolved_metric_table(
        &self,
        name: &str,
    ) -> anyhow::Result<Option<(String, Vec<String>)>> {
        let table = match self.metric_table_for_name(name, None).await? {
            Some(table) => table,
            None => match self.metric_table_for_name(name, Some("_bucket")).await? {
                Some(table) => table,
                None => return Ok(None),
            },
        };
        let rows = self
            .sql_lenient(&format!(
                r#"SELECT "column_name" FROM information_schema.columns
                   WHERE "table_schema" = 'public' AND "table_name" = '{}'
                   ORDER BY "column_name""#,
                escape(&table),
            ))
            .await?;
        let labels = rows
            .iter()
            .map(|row| str_at(row, 0))
            .filter(|column| {
                !METRIC_BOOKKEEPING_COLUMNS.contains(&column.as_str())
                    && metric_group_label_allowed(column)
            })
            .collect();
        Ok(Some((table, labels)))
    }

    pub(super) async fn insert(
        &self,
        table: &str,
        columns: &str,
        values: Vec<String>,
    ) -> anyhow::Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        let sql = format!(
            "INSERT INTO {table} ({columns}) VALUES {}",
            values.join(",")
        );
        self.sql(&sql).await?;
        Ok(())
    }

    /// Select spans from the native `opentelemetry_traces` table. `SELECT *` is
    /// used so the per-attribute columns (`span_attributes.*` /
    /// `resource_attributes.*`) — which auto-widen over time — are all present
    /// and can be folded back into the `attributes`/`resource` JSON maps.
    pub(super) async fn select_spans(
        &self,
        where_clause: &str,
        order: &str,
        limit_clause: &str,
    ) -> anyhow::Result<Vec<SpanRow>> {
        let result = self
            .sql_with_schema_arrow_lenient(&Self::select_spans_sql(
                where_clause,
                order,
                limit_clause,
            ))
            .await?;
        let cols = ColumnIndex::new(&result.columns);
        Ok(result
            .rows
            .iter()
            .map(|row| {
                // native: `timestamp` is the span start TIME INDEX (ns);
                // `duration_nano` is the generated duration in ns.
                let (attributes, resource) = cols.reassemble_attrs(row);
                let events = match cols.json("span_events_json", row) {
                    serde_json::Value::Null => None,
                    value => Some(value.to_string()),
                };
                SpanRow {
                    ts_nanos: cols.u128("timestamp", row),
                    service: cols.string("service_name", row),
                    trace_id: cols.string("trace_id", row),
                    span_id: cols.string("span_id", row),
                    parent_span_id: cols.opt_string("parent_span_id", row),
                    name: cols.string("span_name", row),
                    kind: cols.string("span_kind", row),
                    status_code: cols.string("span_status_code", row),
                    status_message: cols.string("span_status_message", row),
                    duration_ns: cols.u128("duration_nano", row),
                    // Correlation ids flatten to attribute columns; the
                    // explicit span attribute (jackin shape) wins.
                    invocation_id: cols
                        .opt_string(&semconv::span_column(semconv::CLI_INVOCATION_ID), row)
                        .or_else(|| {
                            cols.opt_string(
                                &semconv::resource_column(semconv::CLI_INVOCATION_ID),
                                row,
                            )
                        }),
                    session_id: cols
                        .opt_string(&semconv::span_column(semconv::SESSION_ID), row)
                        .or_else(|| {
                            cols.opt_string(&semconv::resource_column(semconv::SESSION_ID), row)
                        }),
                    scope_name: cols.string("scope_name", row),
                    events,
                    links: cols.json("span_links_json", row),
                    attributes,
                    resource,
                }
            })
            .collect())
    }

    /// Select logs from the native `opentelemetry_logs` table. Top-level OTLP
    /// log identity fields are mirrored into attributes before native forward
    /// because GreptimeDB does not map them to columns yet.
    pub(super) async fn select_logs(
        &self,
        where_clause: &str,
        order: &str,
        limit_clause: &str,
    ) -> anyhow::Result<Vec<LogRow>> {
        let rows = self
            .sql_arrow_lenient(&Self::select_logs_sql(where_clause, order, limit_clause))
            .await?;
        Ok(rows.iter().map(|row| log_row_from_row(row)).collect())
    }
}
