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
        histograms: Vec<HistogramRow>,
        exp_histograms: Vec<HistogramRow>,
        exemplars: Vec<MetricExemplarRow>,
        raw: bytes::Bytes,
    ) -> StorageResult<()> {
        // Split explicit histograms out of the native forward. GreptimeDB
        // metric-engine batch-creates `_bucket`/`_count`/`_sum` siblings; an
        // empty `_sum` table (no field column) fails the whole DDL and drops
        // gauge/sum tables in the same request. Scalars go first; histograms
        // are forwarded alone and fall back to the extension table.
        let hints = format!("ttl={}", self.metrics_ttl);
        let header = ("x-greptime-hints", hints.as_str());
        let (scalar_raw, histogram_raw) = split_explicit_histogram_forward(raw);
        if let Some(scalar_raw) = scalar_raw {
            self.forward_otlp("v1/metrics", &[header], scalar_raw)
                .await
                .map_err(StorageError::transport)?;
        }
        let persist_explicit = match histogram_raw {
            None => Vec::new(),
            Some(histogram_raw) => {
                match self
                    .forward_otlp("v1/metrics", &[header], histogram_raw)
                    .await
                {
                    Ok(()) => Vec::new(),
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            "greptime native histogram forward failed; persisting explicit histograms to extension table"
                        );
                        histograms
                    }
                }
            }
        };
        // Converted exp rows: explicit-histogram tables are native-managed, so
        // these live in their own table (never merged with native `_bucket`
        // rows, which would double-count mixed-encoding metrics). Explicit
        // histograms that failed native create use the same table.
        let mut extension_histograms = exp_histograms;
        extension_histograms.extend(persist_explicit);
        self.insert_histogram_extension_rows(extension_histograms)
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

impl GreptimeStore {
    async fn insert_histogram_extension_rows(&self, rows: Vec<HistogramRow>) -> StorageResult<()> {
        let values = rows
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
            .await
            .map_err(Into::into)
    }
}

/// Split explicit histograms from the native OTLP body so gauge/sum tables
/// are never created in the same Greptime metric-engine DDL batch as
/// `_bucket`/`_count`/`_sum` siblings. Undecodable bodies pass through
/// unchanged (same as a pre-split forward).
fn split_explicit_histogram_forward(
    raw: bytes::Bytes,
) -> (Option<bytes::Bytes>, Option<bytes::Bytes>) {
    use parallax_ingest::strip_explicit_histograms;
    use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
    use prost::Message;

    let Ok(mut request) = ExportMetricsServiceRequest::decode(raw.clone()) else {
        return (Some(raw), None);
    };
    let mut histograms = request.clone();
    if !strip_explicit_histograms(&mut request) {
        return (Some(raw), None);
    }
    let _ = strip_metrics_except_histograms(&mut histograms);
    let scalar =
        (!request.resource_metrics.is_empty()).then(|| bytes::Bytes::from(request.encode_to_vec()));
    let histogram = (!histograms.resource_metrics.is_empty())
        .then(|| bytes::Bytes::from(histograms.encode_to_vec()));
    (scalar, histogram)
}

fn strip_metrics_except_histograms(
    request: &mut parallax_proto::collector_metrics::ExportMetricsServiceRequest,
) -> bool {
    use parallax_proto::metrics::metric::Data;
    let mut changed = false;
    for rm in &mut request.resource_metrics {
        for sm in &mut rm.scope_metrics {
            let before = sm.metrics.len();
            sm.metrics
                .retain(|metric| matches!(metric.data, Some(Data::Histogram(_))));
            changed |= sm.metrics.len() != before;
        }
        rm.scope_metrics.retain(|sm| !sm.metrics.is_empty());
    }
    request
        .resource_metrics
        .retain(|rm| !rm.scope_metrics.is_empty());
    changed
}

#[cfg(test)]
mod tests {
    use super::split_explicit_histogram_forward;
    use parallax_ingest::strip_explicit_histograms;
    use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
    use parallax_proto::metrics::metric::Data;
    use parallax_proto::metrics::{Gauge, Histogram, Metric, ResourceMetrics, ScopeMetrics};
    use prost::Message;

    fn request(metrics: Vec<Metric>) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    #[test]
    fn split_leaves_gauge_only_body_unchanged() {
        let raw = bytes::Bytes::from(
            request(vec![Metric {
                name: "load".into(),
                data: Some(Data::Gauge(Gauge::default())),
                ..Default::default()
            }])
            .encode_to_vec(),
        );
        let (scalar, histogram) = split_explicit_histogram_forward(raw.clone());
        assert_eq!(scalar.as_deref(), Some(raw.as_ref()));
        assert!(histogram.is_none());
    }

    #[test]
    fn split_separates_histogram_from_gauge() {
        let raw = bytes::Bytes::from(
            request(vec![
                Metric {
                    name: "load".into(),
                    data: Some(Data::Gauge(Gauge::default())),
                    ..Default::default()
                },
                Metric {
                    name: "latency".into(),
                    data: Some(Data::Histogram(Histogram::default())),
                    ..Default::default()
                },
            ])
            .encode_to_vec(),
        );
        let (scalar, histogram) = split_explicit_histogram_forward(raw);
        let mut scalar = ExportMetricsServiceRequest::decode(scalar.expect("scalar")).unwrap();
        let histogram = ExportMetricsServiceRequest::decode(histogram.expect("histogram")).unwrap();
        assert!(!strip_explicit_histograms(&mut scalar));
        assert_eq!(scalar.resource_metrics[0].scope_metrics[0].metrics.len(), 1);
        assert_eq!(
            scalar.resource_metrics[0].scope_metrics[0].metrics[0].name,
            "load"
        );
        assert_eq!(
            histogram.resource_metrics[0].scope_metrics[0].metrics.len(),
            1
        );
        assert_eq!(
            histogram.resource_metrics[0].scope_metrics[0].metrics[0].name,
            "latency"
        );
    }
}
