use super::*;

#[async_trait::async_trait]
impl crate::adapter::IngestStore for GreptimeStore {
    async fn ingest_traces(
        &self,
        _request: &parallax_proto::collector_trace::ExportTraceServiceRequest,
        raw: bytes::Bytes,
    ) -> StorageResult<()> {
        // Forward the raw OTLP verbatim to the native traces endpoint; the
        // `greptime_trace_v1` pipeline auto-creates `opentelemetry_traces`. The
        // decoded spans are the worker's tee (errors/live/invocations), not stored here.
        let hints = format!("ttl={},append_mode=true", self.traces_ttl);
        self.forward_otlp(
            "v1/traces",
            &[
                ("x-greptime-pipeline-name", "greptime_trace_v1"),
                ("x-greptime-hints", &hints),
            ],
            raw,
        )
        .await
        .map_err(StorageError::transport)?;
        self.ensure_traces_deviations().await;
        Ok(())
    }

    async fn ingest_logs(
        &self,
        _request: &parallax_proto::collector_logs::ExportLogsServiceRequest,
        raw: bytes::Bytes,
    ) -> StorageResult<()> {
        // The extract-keys header promotes the invocation/session correlation
        // ids and typed-log identity attributes to native columns in
        // opentelemetry_logs.
        let hints = format!("ttl={},append_mode=true", self.logs_ttl);
        let extract_keys = format!(
            "{},{},{},{},{}",
            semconv::SERVICE_NAME,
            semconv::CLI_INVOCATION_ID,
            semconv::SESSION_ID,
            semconv::EVENT_NAME,
            semconv::LOG_OBSERVED_TS_NANOS
        );
        self.forward_otlp(
            "v1/logs",
            &[
                ("x-greptime-log-extract-keys", &extract_keys),
                ("x-greptime-hints", &hints),
            ],
            raw,
        )
        .await
        .map_err(StorageError::transport)?;
        self.ensure_logs_deviations().await;
        Ok(())
    }

    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        _histograms: Vec<HistogramRow>,
        exp_histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
        raw: bytes::Bytes,
    ) -> StorageResult<()> {
        // Forward all metrics to the native metric engine (one table per metric
        // name; histograms split into `_bucket`/`_count`/`_sum`). The worker
        // strips exponential histograms from `raw` first — the engine has no
        // exp type, so converted rows persist to the extension table below.
        let hints = format!("ttl={}", self.metrics_ttl);
        self.forward_otlp("v1/metrics", &[("x-greptime-hints", &hints)], raw)
            .await
            .map_err(StorageError::transport)?;
        // Converted exp rows: explicit-histogram tables are native-managed, so
        // these live in their own table (never merged with native `_bucket`
        // rows, which would double-count mixed-encoding metrics).
        let values = exp_histograms
            .iter()
            .map(|r| {
                // A bare NaN/inf literal breaks the whole INSERT batch; the
                // count survives and the sum degrades to 0 rather than
                // dropping every sibling row (same poison-batch precedent as
                // scalar points, which filter instead — a histogram row carries
                // buckets worth keeping even when its sum is junk).
                let sum = if r.sum.is_finite() { r.sum } else { 0.0 };
                let counts =
                    serde_json::Value::Array(r.bucket_counts.iter().map(|c| (*c).into()).collect());
                let bounds = serde_json::Value::Array(
                    r.bounds
                        .iter()
                        .map(|b| {
                            serde_json::Number::from_f64(*b)
                                .map_or(serde_json::Value::Null, serde_json::Value::Number)
                        })
                        .collect(),
                );
                format!(
                    "({},'{}','{}',{},{},{},{},{})",
                    r.ts_nanos,
                    escape(&r.service),
                    escape(&r.name),
                    r.count.min(i64::MAX as u64),
                    sum,
                    json_literal(&counts),
                    json_literal(&bounds),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(EXP_HISTOGRAMS_TABLE, EXP_HISTOGRAM_COLUMNS, values)
            .await?;
        // Run-scoped points (Q6, Approach 2): the metric engine cannot hold a
        // high-card `invocation_id` tag, so persist those points to `invocation_metric_points`
        // where `invocation_id` is an indexed column.
        let values = points
            .iter()
            // Non-finite samples never count (metric-summary contract) and a
            // bare NaN/inf literal breaks the whole INSERT batch — filter here
            // so one poisoned sample cannot drop every sibling point.
            .filter(|p| {
                p.value.is_finite()
                    && p.invocation_id.as_deref().is_some_and(|id| !id.is_empty())
            })
            .map(|p| {
                format!(
                    "({},'{}','{}','{}','{}',{},{})",
                    p.ts_nanos, // TIMESTAMP(9): nanos
                    escape(p.invocation_id.as_deref().unwrap_or_default()),
                    escape(&p.service),
                    escape(&p.name),
                    escape(&parallax_semconv::native_metric_table_base(&p.name)),
                    p.value,
                    json_literal(&p.attributes),
                )
            })
            .collect();
        self.insert(
            "invocation_metric_points",
            "\"ts\", \"invocation_id\", \"service\", \"name\", \"canonical_name\", \"value\", \"attributes\"",
            values,
        )
        .await?;

        let values = exemplars
            .iter()
            .map(|r| {
                format!(
                    "({},'{}','{}',{},'{}','{}',{},{})",
                    r.ts_nanos,
                    escape(&r.service),
                    escape(&r.name),
                    r.value,
                    escape(&r.trace_id),
                    escape(&r.span_id),
                    opt_literal(&r.invocation_id),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(METRIC_EXEMPLARS_TABLE, METRIC_EXEMPLAR_COLUMNS, values)
            .await
            .map_err(Into::into)
    }

    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> StorageResult<()> {
        let values = rows
            .iter()
            .map(|r| {
                let source = serde_json::to_string(&r.source).unwrap_or_default();
                format!(
                    "({},'{}','{}','{}','{}',{},'{}','{}','{}',{},{},{},{},{})",
                    r.ts_nanos,
                    escape(&r.service),
                    escape(&r.fingerprint),
                    escape(&r.error_type),
                    escape(&r.message),
                    opt_literal(&r.stacktrace),
                    source.trim_matches('"'),
                    escape(&r.trace_id),
                    escape(&r.span_id),
                    opt_literal(&r.invocation_id),
                    opt_literal(&r.session_id),
                    opt_literal(&r.service_version),
                    opt_literal(&r.environment),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(
            "error_events",
            "\"ts\", \"service\", \"fingerprint\", \"error_type\", \"message\", \"stacktrace\", \"source\", \"trace_id\", \"span_id\", \"invocation_id\", \"session_id\", \"service_version\", \"environment\", \"attributes\"",
            values,
        )
        .await
        .map_err(Into::into)
    }
}
