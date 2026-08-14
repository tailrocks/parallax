use super::{log_trace_ids, metric_trace_ids, trace_ids};
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::logs::{LogRecord, ResourceLogs, ScopeLogs};
use parallax_proto::metrics::{
    Exemplar, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric::Data,
};
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span};

fn request(trace_id: Vec<u8>) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            scope_spans: vec![ScopeSpans {
                spans: vec![Span {
                    trace_id,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn otlp_trace_boundary_accepts_only_valid_trace_ids() {
    trace_ids(&request(vec![1; 16])).expect("valid trace id");
    trace_ids(&request(vec![0; 16])).expect_err("zero trace id rejected");
    trace_ids(&request(vec![1; 15])).expect_err("short trace id rejected");
}

fn log_request(trace_id: Vec<u8>) -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records: vec![LogRecord {
                    trace_id,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn otlp_log_boundary_accepts_empty_or_valid_trace_ids() {
    log_trace_ids(&log_request(Vec::new())).expect("empty optional id");
    log_trace_ids(&log_request(vec![1; 16])).expect("valid");
    log_trace_ids(&log_request(vec![0; 16])).expect_err("zero rejected");
    log_trace_ids(&log_request(vec![1; 8])).expect_err("short rejected");
}

fn metric_request(trace_id: Vec<u8>) -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            scope_metrics: vec![ScopeMetrics {
                metrics: vec![Metric {
                    data: Some(Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            exemplars: vec![Exemplar {
                                trace_id,
                                ..Default::default()
                            }],
                            ..Default::default()
                        }],
                    })),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn otlp_metric_exemplar_trace_ids_are_validated() {
    metric_trace_ids(&metric_request(Vec::new())).expect("empty optional id");
    metric_trace_ids(&metric_request(vec![1; 16])).expect("valid");
    metric_trace_ids(&metric_request(vec![0; 16])).expect_err("zero rejected");
}
