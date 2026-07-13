use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_storage::model::TraceId;

pub(super) fn trace_ids(request: &ExportTraceServiceRequest) -> Result<(), &'static str> {
    for resource in &request.resource_spans {
        for scope in &resource.scope_spans {
            for span in &scope.spans {
                TraceId::from_otlp_bytes(&span.trace_id)
                    .map_err(|_| "OTLP span has an invalid trace_id")?;
            }
        }
    }
    Ok(())
}

fn optional_trace_id(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.is_empty() {
        return Ok(());
    }
    TraceId::from_otlp_bytes(bytes)
        .map(|_| ())
        .map_err(|_| "OTLP record has an invalid trace_id")
}

pub(super) fn log_trace_ids(request: &ExportLogsServiceRequest) -> Result<(), &'static str> {
    for resource in &request.resource_logs {
        for scope in &resource.scope_logs {
            for record in &scope.log_records {
                optional_trace_id(&record.trace_id)?;
            }
        }
    }
    Ok(())
}

fn exemplars(values: &[parallax_proto::metrics::Exemplar]) -> Result<(), &'static str> {
    for exemplar in values {
        optional_trace_id(&exemplar.trace_id)?;
    }
    Ok(())
}

fn metric_trace_ids_one(metric: &parallax_proto::metrics::Metric) -> Result<(), &'static str> {
    use parallax_proto::metrics::metric::Data;
    match metric.data.as_ref() {
        Some(Data::Gauge(points)) => {
            for point in &points.data_points {
                exemplars(&point.exemplars)?;
            }
        }
        Some(Data::Sum(points)) => {
            for point in &points.data_points {
                exemplars(&point.exemplars)?;
            }
        }
        Some(Data::Histogram(points)) => {
            for point in &points.data_points {
                exemplars(&point.exemplars)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn metric_trace_ids(request: &ExportMetricsServiceRequest) -> Result<(), &'static str> {
    let metrics = request
        .resource_metrics
        .iter()
        .flat_map(|resource| &resource.scope_metrics)
        .flat_map(|scope| &scope.metrics);
    for metric in metrics {
        metric_trace_ids_one(metric)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
