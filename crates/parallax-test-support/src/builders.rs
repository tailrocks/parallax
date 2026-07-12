//! Typed reusable OTLP requests for integration and adapter tests.

pub use crate::memory::MemoryStore;

use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::{AnyValue, KeyValue, any_value};
use parallax_proto::logs::{LogRecord, ResourceLogs, ScopeLogs};
use parallax_proto::metrics::{
    Exemplar, Histogram, HistogramDataPoint, Metric, ResourceMetrics, ScopeMetrics, exemplar,
    metric,
};
use parallax_proto::resource::Resource;
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span, Status, status};

pub fn span(
    service: &str,
    trace_id: &str,
    span_id: &str,
    ts_nanos: u128,
    duration_ns: u128,
) -> parallax_model::SpanRow {
    parallax_model::SpanRow {
        ts_nanos,
        service: service.into(),
        trace_id: trace_id.into(),
        span_id: span_id.into(),
        parent_span_id: None,
        name: "handler".into(),
        kind: "SPAN_KIND_SERVER".into(),
        status_code: "STATUS_CODE_UNSET".into(),
        status_message: String::new(),
        duration_ns,
        run_id: None,
        scope_name: String::new(),
        events: None,
        links: serde_json::Value::Null,
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

pub fn log_row(
    service: &str,
    trace_id: &str,
    ts_nanos: u128,
    body: &str,
) -> parallax_model::LogRow {
    parallax_model::LogRow {
        ts_nanos,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: service.into(),
        severity_num: 9,
        severity_text: "INFO".into(),
        body: body.into(),
        trace_id: trace_id.into(),
        span_id: format!("span-{ts_nanos}"),
        run_id: None,
        scope_name: String::new(),
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

pub fn span_with_release(
    service: &str,
    trace_id: &str,
    span_id: &str,
    ts_nanos: u128,
    version: &str,
) -> parallax_model::SpanRow {
    let mut row = span(service, trace_id, span_id, ts_nanos, 1_000);
    row.resource = serde_json::json!({ "service.version": version });
    row
}

fn string_kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.into())),
        }),
        key_strindex: 0,
    }
}

fn resource(service: &str) -> Resource {
    Resource {
        attributes: vec![
            string_kv("service.name", service),
            string_kv("parallax.run.id", "run-conformance"),
        ],
        ..Resource::default()
    }
}

pub fn conformance_traces(service: &str, start: u64) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(resource(service)),
            scope_spans: vec![ScopeSpans {
                spans: vec![Span {
                    trace_id: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xa1],
                    span_id: vec![0, 0, 0, 0, 0, 0, 0, 0xb1],
                    name: "conformance.root".into(),
                    kind: 2,
                    start_time_unix_nano: start,
                    end_time_unix_nano: start + 2_000_000,
                    status: Some(Status {
                        code: status::StatusCode::Error as i32,
                        message: "boom".into(),
                    }),
                    attributes: vec![string_kv("http.route", "/quoted")],
                    ..Span::default()
                }],
                ..ScopeSpans::default()
            }],
            ..ResourceSpans::default()
        }],
    }
}

pub fn conformance_logs(service: &str, start: u64) -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(resource(service)),
            scope_logs: vec![ScopeLogs {
                log_records: vec![LogRecord {
                    time_unix_nano: start + 2_000,
                    observed_time_unix_nano: start + 2_001,
                    severity_number: 17,
                    severity_text: "ERROR".into(),
                    event_name: "conformance.log".into(),
                    body: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(
                            "quoted backslash unicode failure".into(),
                        )),
                    }),
                    trace_id: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xa1],
                    span_id: vec![0, 0, 0, 0, 0, 0, 0, 0xb1],
                    ..LogRecord::default()
                }],
                ..ScopeLogs::default()
            }],
            ..ResourceLogs::default()
        }],
    }
}

pub fn conformance_metrics(service: &str, start: u64) -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(resource(service)),
            scope_metrics: vec![ScopeMetrics {
                metrics: vec![Metric {
                    name: "conformance.duration".into(),
                    data: Some(metric::Data::Histogram(Histogram {
                        data_points: vec![HistogramDataPoint {
                            time_unix_nano: start + 4_000,
                            count: 2,
                            sum: Some(0.6),
                            bucket_counts: vec![1, 1, 0],
                            explicit_bounds: vec![0.25, 0.5],
                            exemplars: vec![Exemplar {
                                time_unix_nano: start + 4_000,
                                trace_id: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xa1],
                                span_id: vec![0, 0, 0, 0, 0, 0, 0, 0xb1],
                                value: Some(exemplar::Value::AsDouble(0.48)),
                                filtered_attributes: vec![string_kv("route", "quoted")],
                            }],
                            ..HistogramDataPoint::default()
                        }],
                        ..Histogram::default()
                    })),
                    ..Metric::default()
                }],
                ..ScopeMetrics::default()
            }],
            ..ResourceMetrics::default()
        }],
    }
}
