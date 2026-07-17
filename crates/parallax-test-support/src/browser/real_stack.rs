//! Real-stack (plan 145) public-boundary seed payloads and stable IDs.
//!
//! Builds deterministic OTLP export requests for managed GreptimeDB + Turso
//! browser suites. Callers post the encoded protobuf through public OTLP/HTTP
//! only — never direct native-table inserts.

use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::{AnyValue, KeyValue, any_value};
use parallax_proto::logs::{LogRecord, ResourceLogs, ScopeLogs};
use parallax_proto::metrics::{
    Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric, number_data_point,
};
use parallax_proto::resource::Resource;
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span, Status, status};

/// Stable product IDs for one full-stack seed run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealStackIds {
    pub dataset_id: String,
    pub service: String,
    pub invocation_id: String,
    pub session_id: String,
    pub trace_id_hex: String,
    pub span_id_hex: String,
    pub error_type: String,
    pub error_message: String,
    pub log_body: String,
    pub metric_name: String,
    pub start_nanos: u64,
}

impl RealStackIds {
    /// Build IDs from a unique dataset suffix (nanos, CI job id, etc.).
    pub fn new(dataset_suffix: &str, start_nanos: u64) -> Self {
        let dataset_id = format!("pw-storage-{dataset_suffix}");
        // Fixed-width hex identity material derived from the dataset suffix hash.
        let digest = simple_digest(dataset_suffix.as_bytes());
        let trace_id_hex = hex(&digest[..16]);
        let span_id_hex = hex(&digest[16..24]);
        Self {
            service: format!("pw-storage-{dataset_suffix}"),
            invocation_id: format!("inv-{dataset_suffix}"),
            session_id: format!("sess-{dataset_suffix}"),
            error_type: format!("test::PwStorage::{dataset_suffix}"),
            error_message: format!("pw-storage boom {dataset_suffix}"),
            log_body: format!("pw-storage log {dataset_suffix}"),
            metric_name: "pw.storage.seed.count".into(),
            dataset_id,
            trace_id_hex,
            span_id_hex,
            start_nanos,
        }
    }

    pub fn trace_id_bytes(&self) -> Vec<u8> {
        hex_decode(&self.trace_id_hex)
    }

    pub fn span_id_bytes(&self) -> Vec<u8> {
        hex_decode(&self.span_id_hex)
    }
}

/// Trace export: one error root span with exception event (drives Turso issue).
pub fn traces_request(ids: &RealStackIds) -> ExportTraceServiceRequest {
    let start = ids.start_nanos;
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(resource(ids)),
            scope_spans: vec![ScopeSpans {
                spans: vec![Span {
                    trace_id: ids.trace_id_bytes(),
                    span_id: ids.span_id_bytes(),
                    name: "pw.storage.root".into(),
                    kind: 2,
                    start_time_unix_nano: start,
                    end_time_unix_nano: start + 5_000_000,
                    status: Some(Status {
                        code: status::StatusCode::Error as i32,
                        message: ids.error_message.clone(),
                    }),
                    attributes: vec![
                        string_kv("cli.invocation.id", &ids.invocation_id),
                        string_kv("session.id", &ids.session_id),
                        string_kv("parallax.dataset.id", &ids.dataset_id),
                    ],
                    events: vec![parallax_proto::trace::span::Event {
                        time_unix_nano: start + 1_000_000,
                        name: "exception".into(),
                        attributes: vec![
                            string_kv("exception.type", &ids.error_type),
                            string_kv("exception.message", &ids.error_message),
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..ScopeSpans::default()
            }],
            ..ResourceSpans::default()
        }],
    }
}

/// Correlated ERROR log on the same trace/service.
pub fn logs_request(ids: &RealStackIds) -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(resource(ids)),
            scope_logs: vec![ScopeLogs {
                log_records: vec![LogRecord {
                    time_unix_nano: ids.start_nanos + 2_000_000,
                    observed_time_unix_nano: ids.start_nanos + 2_000_001,
                    severity_number: 17,
                    severity_text: "ERROR".into(),
                    event_name: "pw.storage.log".into(),
                    body: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(ids.log_body.clone())),
                    }),
                    trace_id: ids.trace_id_bytes(),
                    span_id: ids.span_id_bytes(),
                    attributes: vec![
                        string_kv("cli.invocation.id", &ids.invocation_id),
                        string_kv("parallax.dataset.id", &ids.dataset_id),
                    ],
                    ..LogRecord::default()
                }],
                ..ScopeLogs::default()
            }],
            ..ResourceLogs::default()
        }],
    }
}

/// One gauge sample for metric discovery.
pub fn metrics_request(ids: &RealStackIds) -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(resource(ids)),
            scope_metrics: vec![ScopeMetrics {
                metrics: vec![Metric {
                    name: ids.metric_name.clone(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            time_unix_nano: ids.start_nanos + 3_000_000,
                            value: Some(number_data_point::Value::AsDouble(1.0)),
                            attributes: vec![string_kv("parallax.dataset.id", &ids.dataset_id)],
                            ..NumberDataPoint::default()
                        }],
                    })),
                    ..Metric::default()
                }],
                ..ScopeMetrics::default()
            }],
            ..ResourceMetrics::default()
        }],
    }
}

/// One-event live-transport follow-up log (distinct body, same service).
pub fn live_followup_log(
    ids: &RealStackIds,
    body: &str,
    ts_nanos: u64,
) -> ExportLogsServiceRequest {
    live_followup_logs(ids, &[(body, ts_nanos)])
}

/// Multi-event live log burst (plan 147 @live capacity/identity cases).
pub fn live_followup_logs(ids: &RealStackIds, rows: &[(&str, u64)]) -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(resource(ids)),
            scope_logs: vec![ScopeLogs {
                log_records: rows
                    .iter()
                    .map(|(body, ts_nanos)| LogRecord {
                        time_unix_nano: *ts_nanos,
                        observed_time_unix_nano: *ts_nanos,
                        severity_number: 9,
                        severity_text: "INFO".into(),
                        event_name: "pw.storage.live".into(),
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue((*body).into())),
                        }),
                        attributes: vec![
                            string_kv("cli.invocation.id", &ids.invocation_id),
                            string_kv("parallax.dataset.id", &ids.dataset_id),
                            string_kv("pw.live", "1"),
                        ],
                        ..LogRecord::default()
                    })
                    .collect(),
                ..ScopeLogs::default()
            }],
            ..ResourceLogs::default()
        }],
    }
}

/// One live follow-up span with caller-owned span id/name (plan 147 @live traces).
pub fn live_followup_span(
    ids: &RealStackIds,
    span_id_hex: &str,
    name: &str,
    ts_nanos: u64,
) -> ExportTraceServiceRequest {
    live_followup_spans(ids, &[(span_id_hex, name, ts_nanos)])
}

/// Multi-span live export (plan 147 identity cases — pass the same triple twice).
pub fn live_followup_spans(
    ids: &RealStackIds,
    rows: &[(&str, &str, u64)],
) -> ExportTraceServiceRequest {
    let spans = rows
        .iter()
        .map(|(span_id_hex, name, ts_nanos)| {
            let span_id = hex_decode(span_id_hex);
            let span_id = if span_id.len() == 8 {
                span_id
            } else {
                simple_digest(format!("{span_id_hex}:{name}").as_bytes())[16..24].to_vec()
            };
            // Unique trace id so hub mergeLiveTraces does not skip as known seed root.
            let mut trace_id =
                simple_digest(format!("live-trace:{span_id_hex}:{name}:{ts_nanos}").as_bytes())
                    [..16]
                    .to_vec();
            if trace_id.iter().all(|b| *b == 0) {
                trace_id[0] = 0xC1;
            }
            Span {
                trace_id,
                span_id,
                name: (*name).into(),
                kind: 2,
                start_time_unix_nano: *ts_nanos,
                end_time_unix_nano: ts_nanos.saturating_add(2_000_000),
                status: Some(Status {
                    code: status::StatusCode::Ok as i32,
                    message: String::new(),
                }),
                attributes: vec![
                    string_kv("cli.invocation.id", &ids.invocation_id),
                    string_kv("session.id", &ids.session_id),
                    string_kv("parallax.dataset.id", &ids.dataset_id),
                    string_kv("pw.live", "1"),
                ],
                ..Default::default()
            }
        })
        .collect();
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(resource(ids)),
            scope_spans: vec![ScopeSpans {
                spans,
                ..ScopeSpans::default()
            }],
            ..ResourceSpans::default()
        }],
    }
}

fn resource(ids: &RealStackIds) -> Resource {
    Resource {
        attributes: vec![
            string_kv("service.name", &ids.service),
            string_kv("parallax.dataset.id", &ids.dataset_id),
        ],
        ..Resource::default()
    }
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0))
        .collect()
}

/// Tiny non-crypto digest for stable 24-byte identity material.
fn simple_digest(input: &[u8]) -> [u8; 24] {
    let mut out = [0u8; 24];
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    for (i, byte) in input.iter().enumerate() {
        state = state
            .wrapping_mul(0x100_0000_01b3)
            .wrapping_add(u64::from(*byte))
            .wrapping_add(i as u64);
        out[i % 24] ^= state.to_le_bytes()[i % 8];
        out[(i + 7) % 24] ^= (state >> 8).to_le_bytes()[0];
    }
    // Ensure non-zero trace/span ids.
    if out.iter().all(|b| *b == 0) {
        out[0] = 0xA1;
        out[16] = 0xB1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_for_same_suffix() {
        let a = RealStackIds::new("abc", 1_000);
        let b = RealStackIds::new("abc", 1_000);
        assert_eq!(a, b);
        assert_eq!(a.trace_id_hex.len(), 32);
        assert_eq!(a.span_id_hex.len(), 16);
    }

    #[test]
    fn different_suffix_different_trace() {
        let a = RealStackIds::new("a", 1);
        let b = RealStackIds::new("b", 1);
        assert_ne!(a.trace_id_hex, b.trace_id_hex);
        assert_ne!(a.error_type, b.error_type);
    }

    #[test]
    fn payloads_carry_dataset_and_error() {
        let ids = RealStackIds::new("payload", 42);
        let traces = traces_request(&ids);
        let span = &traces.resource_spans[0].scope_spans[0].spans[0];
        assert_eq!(span.name, "pw.storage.root");
        assert!(span.events.iter().any(|e| e.name == "exception"
            && e.attributes.iter().any(|a| {
                a.key == "exception.type"
                    && a.value
                        .as_ref()
                        .and_then(|v| v.value.as_ref())
                        .is_some_and(|v| {
                            matches!(
                                v,
                                any_value::Value::StringValue(s) if s == &ids.error_type
                            )
                        })
            })));
        let logs = logs_request(&ids);
        assert_eq!(
            logs.resource_logs[0].scope_logs[0].log_records[0]
                .body
                .as_ref()
                .and_then(|b| b.value.as_ref())
                .map(|v| match v {
                    any_value::Value::StringValue(s) => s.as_str(),
                    _ => "",
                }),
            Some(ids.log_body.as_str())
        );
        let metrics = metrics_request(&ids);
        assert_eq!(
            metrics.resource_metrics[0].scope_metrics[0].metrics[0].name,
            ids.metric_name
        );
    }
}
