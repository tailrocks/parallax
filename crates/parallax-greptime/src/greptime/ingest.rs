use super::*;

#[async_trait::async_trait]
impl crate::adapter::IngestStore for GreptimeStore {
    async fn ingest_traces(
        &self,
        _request: &parallax_proto::collector_trace::ExportTraceServiceRequest,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        // Forward the raw OTLP verbatim to the native traces endpoint; the
        // `greptime_trace_v1` pipeline auto-creates `opentelemetry_traces`. The
        // decoded spans are the worker's tee (errors/live/runs), not stored here.
        let hints = format!("ttl={},append_mode=true", self.traces_ttl);
        self.forward_otlp(
            "v1/traces",
            &[
                ("x-greptime-pipeline-name", "greptime_trace_v1"),
                ("x-greptime-hints", &hints),
            ],
            raw,
        )
        .await?;
        self.ensure_traces_deviations().await;
        Ok(())
    }

    async fn ingest_logs(
        &self,
        _request: &parallax_proto::collector_logs::ExportLogsServiceRequest,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        // The extract-keys header promotes run id and typed-log identity
        // attributes to native columns in opentelemetry_logs.
        let hints = format!("ttl={},append_mode=true", self.logs_ttl);
        let extract_keys = format!(
            "{},{},{},{}",
            semconv::SERVICE_NAME,
            semconv::PARALLAX_RUN_ID,
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
        .await?;
        self.ensure_logs_deviations().await;
        Ok(())
    }

    async fn ingest_metrics(
        &self,
        points: Vec<MetricPointRow>,
        _histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
        raw: bytes::Bytes,
    ) -> anyhow::Result<()> {
        // Forward all metrics to the native metric engine (one table per metric
        // name; histograms split into `_bucket`/`_count`/`_sum`).
        let hints = format!("ttl={}", self.metrics_ttl);
        self.forward_otlp("v1/metrics", &[("x-greptime-hints", &hints)], raw)
            .await?;
        // Run-scoped points (Q6, Approach 2): the metric engine cannot hold a
        // high-card `run_id` tag, so persist those points to `run_metric_points`
        // where `run_id` is an indexed column.
        let values = points
            .iter()
            .filter(|p| p.run_id.as_deref().is_some_and(|id| !id.is_empty()))
            .map(|p| {
                format!(
                    "({},'{}','{}','{}',{},{})",
                    p.ts_nanos, // TIMESTAMP(9): nanos
                    escape(p.run_id.as_deref().unwrap_or_default()),
                    escape(&p.service),
                    escape(&p.name),
                    p.value,
                    json_literal(&p.attributes),
                )
            })
            .collect();
        self.insert(
            "run_metric_points",
            "\"ts\", \"run_id\", \"service\", \"name\", \"value\", \"attributes\"",
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
                    opt_literal(&r.run_id),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(METRIC_EXEMPLARS_TABLE, METRIC_EXEMPLAR_COLUMNS, values)
            .await
    }

    async fn write_error_events(&self, rows: Vec<ErrorEventRow>) -> anyhow::Result<()> {
        let values = rows
            .iter()
            .map(|r| {
                let source = serde_json::to_string(&r.source).unwrap_or_default();
                format!(
                    "({},'{}','{}','{}','{}',{},'{}','{}','{}',{})",
                    r.ts_nanos,
                    escape(&r.service),
                    escape(&r.fingerprint),
                    escape(&r.error_type),
                    escape(&r.message),
                    opt_literal(&r.stacktrace),
                    source.trim_matches('"'),
                    escape(&r.trace_id),
                    escape(&r.span_id),
                    json_literal(&r.attributes),
                )
            })
            .collect();
        self.insert(
            "error_events",
            "\"ts\", \"service\", \"fingerprint\", \"error_type\", \"message\", \"stacktrace\", \"source\", \"trace_id\", \"span_id\", \"attributes\"",
            values,
        )
        .await
    }
}
