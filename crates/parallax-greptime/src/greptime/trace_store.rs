use super::*;

#[async_trait::async_trait]
impl crate::adapter::TraceStore for GreptimeStore {
    async fn spans_by_trace(&self, trace_id: &str) -> StorageResult<Vec<SpanRow>> {
        self.select_spans(
            &format!(r#""trace_id" = '{}'"#, escape(trace_id)),
            r#" ORDER BY "timestamp" ASC"#,
            "",
        )
        .await
        .map_err(Into::into)
    }

    async fn traces_by_ids(
        &self,
        trace_ids: &[String],
    ) -> StorageResult<Vec<crate::adapter::TraceSummary>> {
        // O(n) dedup preserving request order (MAX_ROWS still caps fan-out).
        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        for trace_id in trace_ids.iter().filter(|trace_id| !trace_id.is_empty()) {
            if !seen.insert(trace_id.as_str()) {
                continue;
            }
            ids.push(trace_id.clone());
            if ids.len() >= MAX_ROWS {
                break;
            }
        }
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_list = ids
            .iter()
            .map(|trace_id| format!("'{}'", escape(trace_id)))
            .collect::<Vec<_>>()
            .join(",");
        let spans = self
            .select_spans(
                &format!(r#""trace_id" IN ({id_list})"#),
                r#" ORDER BY "timestamp" ASC"#,
                "",
            )
            .await?;
        let mut grouped: BTreeMap<String, Vec<SpanRow>> = BTreeMap::new();
        for span in spans {
            grouped.entry(span.trace_id.clone()).or_default().push(span);
        }
        let mut by_id: BTreeMap<_, _> = grouped
            .into_iter()
            .filter_map(|(trace_id, spans)| {
                let root = spans.iter().min_by_key(|span| {
                    (
                        !span.parent_span_id.as_deref().is_none_or(str::is_empty),
                        span.ts_nanos,
                    )
                })?;
                Some((
                    trace_id.clone(),
                    crate::adapter::TraceSummary {
                        trace_id,
                        root_name: root.name.clone(),
                        service: root.service.clone(),
                        start_nanos: root.ts_nanos,
                        duration_ns: root.duration_ns,
                        span_count: spans.len() as u64,
                        has_error: spans
                            .iter()
                            .any(|span| span.status_code == "STATUS_CODE_ERROR"),
                    },
                ))
            })
            .collect();
        Ok(ids
            .into_iter()
            .filter_map(|trace_id| by_id.remove(&trace_id))
            .collect())
    }

    async fn spans_by_invocation(
        &self,
        invocation_id: &str,
        limit: usize,
        range: RangeInclusive<u128>,
    ) -> StorageResult<Vec<SpanRow>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let limit_clause = format!(" LIMIT {limit}");
        // Two-step read: resolve the invocation's trace ids first, then read
        // whole traces by literal id list. A `trace_id IN (SELECT …)`
        // semi-join returns zero rows on the live engine, so the id list is
        // materialized client-side (bounded by MAX_ROWS).
        let mut trace_ids = self.invocation_trace_ids(invocation_id, &range).await?;
        if trace_ids.is_empty() {
            trace_ids = self
                .invocation_trace_ids_via_logs(invocation_id, &range)
                .await?;
        }
        if trace_ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_list = trace_ids
            .iter()
            .map(|trace_id| format!("'{}'", escape(trace_id)))
            .collect::<Vec<_>>()
            .join(",");
        let mut spans = self
            .select_spans(
                &format!(r#""trace_id" IN ({id_list})"#),
                r#" ORDER BY "timestamp" DESC"#,
                &limit_clause,
            )
            .await
            .map_err(StorageError::from)?;
        spans.sort_by_key(|span| span.ts_nanos);
        if spans.len() > limit {
            spans.drain(0..spans.len() - limit);
        }
        Ok(spans)
    }

    async fn spans_by_invocations(
        &self,
        invocation_ids: &[String],
        limit_per_invocation: usize,
    ) -> StorageResult<HashMap<String, Vec<SpanRow>>> {
        let mut out: HashMap<String, Vec<SpanRow>> = HashMap::with_capacity(invocation_ids.len());
        for invocation_id in invocation_ids {
            out.entry(invocation_id.clone()).or_default();
        }
        if invocation_ids.is_empty() || limit_per_invocation == 0 {
            return Ok(out);
        }
        let escaped = invocation_ids
            .iter()
            .filter(|id| !id.is_empty())
            .map(|id| format!("'{}'", escape(id)))
            .collect::<Vec<_>>();
        if escaped.is_empty() {
            return Ok(out);
        }
        let id_list = escaped.join(",");
        let trace_invocation_column = trace_attr_expr(semconv::CLI_INVOCATION_ID);
        // The correlation id is stamped on the root span (jackin shape) or the
        // resource; children inherit their trace's id via the trace-window MAX.
        let sql = format!(
            r#"SELECT * FROM (
                 SELECT *, ROW_NUMBER() OVER (
                   PARTITION BY "invocation_group"
                   ORDER BY "timestamp" DESC
                 ) AS "rn"
                 FROM (
                   SELECT *, MAX({trace_invocation_column}) OVER (
                     PARTITION BY "trace_id"
                   ) AS "invocation_group"
                   FROM opentelemetry_traces
                 )
                 WHERE "invocation_group" IN ({id_list})
               ) WHERE "rn" <= {limit_per_invocation}
               ORDER BY "timestamp" ASC"#
        );
        let result = match self.sql_with_schema_arrow_lenient(&sql).await {
            Ok(result) => result,
            Err(error) if is_missing_column(&error) => {
                for invocation_id in invocation_ids {
                    out.insert(
                        invocation_id.clone(),
                        self.spans_by_invocation(
                            invocation_id,
                            limit_per_invocation,
                            0..=u128::MAX,
                        )
                        .await?,
                    );
                }
                return Ok(out);
            }
            Err(error) => return Err(error.into()),
        };
        let cols = ColumnIndex::new(&result.columns);
        for row in &result.rows {
            let (attributes, resource) = cols.reassemble_attrs(row);
            let events = match cols.json("span_events", row) {
                serde_json::Value::Null => None,
                value => Some(value.to_string()),
            };
            let span = SpanRow {
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
                invocation_id: cols.opt_string("invocation_group", row).or_else(|| {
                    cols.opt_string(&semconv::span_column(semconv::CLI_INVOCATION_ID), row)
                        .or_else(|| {
                            cols.opt_string(
                                &semconv::resource_column(semconv::CLI_INVOCATION_ID),
                                row,
                            )
                        })
                }),
                session_id: cols
                    .opt_string(&semconv::span_column(semconv::SESSION_ID), row)
                    .or_else(|| {
                        cols.opt_string(&semconv::resource_column(semconv::SESSION_ID), row)
                    }),
                scope_name: cols.string("scope_name", row),
                events,
                links: cols.json("span_links", row),
                attributes,
                resource,
            };
            if let Some(invocation_id) = span.invocation_id.clone() {
                out.entry(invocation_id).or_default().push(span);
            }
        }
        Ok(out)
    }
}
