//! OTLP → normalized rows, per the implementation-spec §7 mapping.

use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::exemplar::Value as ExemplarValue;
use parallax_proto::metrics::metric::Data;
use parallax_proto::metrics::number_data_point::Value as NumberValue;
use parallax_proto::semconv;
use parallax_storage::model::{HistogramRow, LogRow, MetricExemplarRow, MetricPointRow, SpanRow};

pub fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn any_value_to_json(value: &AnyValue) -> serde_json::Value {
    match &value.value {
        Some(AnyValueEnum::StringValue(s)) => serde_json::Value::String(s.clone()),
        Some(AnyValueEnum::BoolValue(b)) => serde_json::Value::Bool(*b),
        Some(AnyValueEnum::IntValue(i)) => serde_json::Value::from(*i),
        Some(AnyValueEnum::DoubleValue(d)) => serde_json::Number::from_f64(*d)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        Some(AnyValueEnum::ArrayValue(items)) => {
            serde_json::Value::Array(items.values.iter().map(any_value_to_json).collect())
        }
        Some(AnyValueEnum::KvlistValue(kvs)) => attributes_to_json(&kvs.values),
        Some(AnyValueEnum::BytesValue(b)) => serde_json::Value::String(hex(b)),
        // String-table indexed values (newer OTLP encodings) need the table
        // context to resolve; standard SDK exports do not use them. Null out.
        Some(_) | None => serde_json::Value::Null,
    }
}

pub fn attributes_to_json(attributes: &[KeyValue]) -> serde_json::Value {
    let map: serde_json::Map<String, serde_json::Value> = attributes
        .iter()
        .map(|kv| {
            (
                kv.key.clone(),
                kv.value
                    .as_ref()
                    .map_or(serde_json::Value::Null, any_value_to_json),
            )
        })
        .collect();
    serde_json::Value::Object(map)
}

pub fn attr_str<'a>(attributes: &'a [KeyValue], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|kv| kv.key == key)
        .and_then(|kv| match &kv.value {
            Some(AnyValue {
                value: Some(AnyValueEnum::StringValue(s)),
            }) => Some(s.as_str()),
            _ => None,
        })
}

fn has_attr(attributes: &[KeyValue], key: &str) -> bool {
    attributes.iter().any(|kv| kv.key == key)
}

fn string_attr(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn int_attr(key: &str, value: u64) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::IntValue(
                i64::try_from(value).unwrap_or(i64::MAX),
            )),
        }),
        key_strindex: 0,
    }
}

/// Mirror top-level OTLP log identity fields into attributes before native
/// GreptimeDB ingest. GreptimeDB v1.1.2/v1.2 nightly do not map
/// `LogRecord.event_name` or `observed_time_unix_nano` to native log columns,
/// but they do persist/extract log attributes into `opentelemetry_logs`.
pub fn promote_log_identity_attributes(request: &mut ExportLogsServiceRequest) -> bool {
    let mut changed = false;
    for resource_logs in &mut request.resource_logs {
        for scope_logs in &mut resource_logs.scope_logs {
            for record in &mut scope_logs.log_records {
                if !record.event_name.is_empty()
                    && !has_attr(&record.attributes, semconv::EVENT_NAME)
                {
                    record
                        .attributes
                        .push(string_attr(semconv::EVENT_NAME, &record.event_name));
                    changed = true;
                }
                if record.observed_time_unix_nano != 0
                    && !has_attr(&record.attributes, semconv::LOG_OBSERVED_TS_NANOS)
                {
                    record.attributes.push(int_attr(
                        semconv::LOG_OBSERVED_TS_NANOS,
                        record.observed_time_unix_nano,
                    ));
                    changed = true;
                }
            }
        }
    }
    changed
}

fn service_name(resource_attrs: &[KeyValue]) -> String {
    attr_str(resource_attrs, semconv::SERVICE_NAME)
        .unwrap_or("unknown")
        .to_string()
}

fn span_kind_name(kind: i32) -> &'static str {
    match kind {
        1 => "SPAN_KIND_INTERNAL",
        2 => "SPAN_KIND_SERVER",
        3 => "SPAN_KIND_CLIENT",
        4 => "SPAN_KIND_PRODUCER",
        5 => "SPAN_KIND_CONSUMER",
        _ => "SPAN_KIND_UNSPECIFIED",
    }
}

fn status_code_name(code: i32) -> &'static str {
    match code {
        1 => "STATUS_CODE_OK",
        2 => "STATUS_CODE_ERROR",
        _ => "STATUS_CODE_UNSET",
    }
}

/// Resolve the run id from resource attributes. Parallax intentionally keeps
/// this to one key so one wrapped command has one lookup id.
fn run_id(resource_attrs: &[KeyValue]) -> Option<String> {
    attr_str(resource_attrs, semconv::PARALLAX_RUN_ID).map(str::to_string)
}

pub fn resource_run_ids(
    request: &ExportTraceServiceRequest,
) -> impl Iterator<Item = (String, u128)> + '_ {
    request.resource_spans.iter().filter_map(|rs| {
        let resource_attrs = rs
            .resource
            .as_ref()
            .map(|r| r.attributes.as_slice())
            .unwrap_or(&[]);
        let run_id = run_id(resource_attrs)?;
        let ts = rs
            .scope_spans
            .iter()
            .flat_map(|ss| ss.spans.iter())
            .map(|span| u128::from(span.start_time_unix_nano))
            .min()
            .unwrap_or(0);
        Some((run_id, ts))
    })
}

/// OTel span links → `[{traceId, spanId, attributes}]` JSON. Links are the
/// standard cross-trace correlation: a span references spans in other
/// traces (batch/async sub-operations) without a parent/child edge.
fn links_to_json(links: &[parallax_proto::trace::span::Link]) -> serde_json::Value {
    serde_json::Value::Array(
        links
            .iter()
            .map(|link| {
                serde_json::json!({
                    "traceId": hex(&link.trace_id),
                    "spanId": hex(&link.span_id),
                    "attributes": attributes_to_json(&link.attributes),
                })
            })
            .collect(),
    )
}

pub fn normalize_traces(request: &ExportTraceServiceRequest) -> Vec<SpanRow> {
    let mut rows = Vec::new();
    for rs in &request.resource_spans {
        let resource_attrs = rs
            .resource
            .as_ref()
            .map(|r| r.attributes.as_slice())
            .unwrap_or(&[]);
        let service = service_name(resource_attrs);
        let run_id = run_id(resource_attrs);
        let resource_json = attributes_to_json(resource_attrs);
        for ss in &rs.scope_spans {
            let scope_name = ss
                .scope
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();
            for span in &ss.spans {
                let (status_code, status_message) = span
                    .status
                    .as_ref()
                    .map(|s| (status_code_name(s.code), s.message.clone()))
                    .unwrap_or(("STATUS_CODE_UNSET", String::new()));
                rows.push(SpanRow {
                    ts_nanos: u128::from(span.start_time_unix_nano),
                    service: service.clone(),
                    trace_id: hex(&span.trace_id),
                    span_id: hex(&span.span_id),
                    parent_span_id: (!span.parent_span_id.is_empty())
                        .then(|| hex(&span.parent_span_id)),
                    name: span.name.clone(),
                    kind: span_kind_name(span.kind).to_string(),
                    status_code: status_code.to_string(),
                    status_message,
                    duration_ns: u128::from(
                        span.end_time_unix_nano
                            .saturating_sub(span.start_time_unix_nano),
                    ),
                    run_id: run_id.clone(),
                    scope_name: scope_name.clone(),
                    events: None,
                    links: links_to_json(&span.links),
                    attributes: attributes_to_json(&span.attributes),
                    resource: resource_json.clone(),
                });
            }
        }
    }
    rows
}

pub fn normalize_logs(request: &ExportLogsServiceRequest) -> Vec<LogRow> {
    let mut rows = Vec::new();
    for rl in &request.resource_logs {
        let resource_attrs = rl
            .resource
            .as_ref()
            .map(|r| r.attributes.as_slice())
            .unwrap_or(&[]);
        let service = service_name(resource_attrs);
        let run_id = run_id(resource_attrs);
        let resource_json = attributes_to_json(resource_attrs);
        for sl in &rl.scope_logs {
            let scope_name = sl
                .scope
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();
            for record in &sl.log_records {
                let body = record
                    .body
                    .as_ref()
                    .map(|b| match any_value_to_json(b) {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                let ts = if record.time_unix_nano != 0 {
                    record.time_unix_nano
                } else {
                    record.observed_time_unix_nano
                };
                rows.push(LogRow {
                    ts_nanos: u128::from(ts),
                    event_name: record.event_name.clone(),
                    observed_ts_nanos: u128::from(record.observed_time_unix_nano),
                    service: service.clone(),
                    severity_num: record.severity_number,
                    severity_text: record.severity_text.clone(),
                    body,
                    trace_id: hex(&record.trace_id),
                    span_id: hex(&record.span_id),
                    run_id: run_id.clone(),
                    scope_name: scope_name.clone(),
                    attributes: attributes_to_json(&record.attributes),
                    resource: resource_json.clone(),
                });
            }
        }
    }
    rows
}

pub struct NormalizedMetrics {
    pub points: Vec<MetricPointRow>,
    pub histograms: Vec<HistogramRow>,
    pub exemplars: Vec<MetricExemplarRow>,
}

pub fn normalize_metrics(request: &ExportMetricsServiceRequest) -> NormalizedMetrics {
    let mut points = Vec::new();
    let mut histograms = Vec::new();
    let mut exemplars = Vec::new();
    for rm in &request.resource_metrics {
        let resource_attrs = rm
            .resource
            .as_ref()
            .map(|r| r.attributes.as_slice())
            .unwrap_or(&[]);
        let service = service_name(resource_attrs);
        let run_id = run_id(resource_attrs);
        for sm in &rm.scope_metrics {
            for metric in &sm.metrics {
                match &metric.data {
                    Some(Data::Gauge(g)) => {
                        for dp in &g.data_points {
                            push_exemplars(
                                &mut exemplars,
                                &service,
                                run_id.as_deref(),
                                &metric.name,
                                dp.time_unix_nano,
                                &dp.exemplars,
                            );
                            points.push(number_point(
                                &service,
                                run_id.as_deref(),
                                &metric.name,
                                dp,
                                false,
                            ));
                        }
                    }
                    Some(Data::Sum(s)) => {
                        for dp in &s.data_points {
                            push_exemplars(
                                &mut exemplars,
                                &service,
                                run_id.as_deref(),
                                &metric.name,
                                dp.time_unix_nano,
                                &dp.exemplars,
                            );
                            points.push(number_point(
                                &service,
                                run_id.as_deref(),
                                &metric.name,
                                dp,
                                s.is_monotonic,
                            ));
                        }
                    }
                    Some(Data::Histogram(h)) => {
                        for dp in &h.data_points {
                            push_exemplars(
                                &mut exemplars,
                                &service,
                                run_id.as_deref(),
                                &metric.name,
                                dp.time_unix_nano,
                                &dp.exemplars,
                            );
                            histograms.push(HistogramRow {
                                ts_nanos: u128::from(dp.time_unix_nano),
                                service: service.clone(),
                                name: metric.name.clone(),
                                count: dp.count,
                                sum: dp.sum.unwrap_or(0.0),
                                bucket_counts: dp.bucket_counts.clone(),
                                bounds: dp.explicit_bounds.clone(),
                                attributes: attributes_to_json(&dp.attributes),
                            });
                        }
                    }
                    // Exponential histograms / summaries: V1 stores nothing
                    // yet; arrival is surfaced through doctor counters later.
                    _ => {}
                }
            }
        }
    }
    NormalizedMetrics {
        points,
        histograms,
        exemplars,
    }
}

fn push_exemplars(
    rows: &mut Vec<MetricExemplarRow>,
    service: &str,
    run_id: Option<&str>,
    name: &str,
    point_ts_nanos: u64,
    exemplars: &[parallax_proto::metrics::Exemplar],
) {
    for exemplar in exemplars {
        let Some(value) = exemplar_value(exemplar) else {
            continue;
        };
        if exemplar.trace_id.is_empty() || exemplar.span_id.is_empty() {
            continue;
        }
        let ts_nanos = if exemplar.time_unix_nano == 0 {
            point_ts_nanos
        } else {
            exemplar.time_unix_nano
        };
        rows.push(MetricExemplarRow {
            ts_nanos: u128::from(ts_nanos),
            service: service.to_string(),
            name: name.to_string(),
            value,
            trace_id: hex(&exemplar.trace_id),
            span_id: hex(&exemplar.span_id),
            run_id: run_id.map(str::to_string),
            attributes: attributes_to_json(&exemplar.filtered_attributes),
        });
    }
}

fn exemplar_value(exemplar: &parallax_proto::metrics::Exemplar) -> Option<f64> {
    match exemplar.value {
        Some(ExemplarValue::AsDouble(value)) => Some(value),
        Some(ExemplarValue::AsInt(value)) => Some(value as f64),
        None => None,
    }
}

fn number_point(
    service: &str,
    run_id: Option<&str>,
    name: &str,
    dp: &parallax_proto::metrics::NumberDataPoint,
    is_monotonic: bool,
) -> MetricPointRow {
    let value = match dp.value {
        Some(NumberValue::AsDouble(d)) => d,
        Some(NumberValue::AsInt(i)) => i as f64,
        None => 0.0,
    };
    MetricPointRow {
        ts_nanos: u128::from(dp.time_unix_nano),
        service: service.to_string(),
        name: name.to_string(),
        value,
        is_monotonic,
        run_id: run_id.map(str::to_string),
        attributes: attributes_to_json(&dp.attributes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_proto::metrics::{
        Exemplar, Gauge, Histogram, HistogramDataPoint, Metric, NumberDataPoint,
        exemplar::Value as ExemplarValue, metric::Data, number_data_point::Value as NumberValue,
    };

    fn string_kv(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(AnyValueEnum::StringValue(value.to_string())),
            }),
            key_strindex: 0,
        }
    }

    fn exemplar(value: ExemplarValue, ts: u64) -> Exemplar {
        Exemplar {
            time_unix_nano: ts,
            trace_id: vec![1; 16],
            span_id: vec![2; 8],
            value: Some(value),
            filtered_attributes: vec![string_kv("route", "/checkout")],
        }
    }

    fn logs_request() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![parallax_proto::logs::ResourceLogs {
                resource: Some(parallax_proto::resource::Resource {
                    attributes: vec![string_kv("service.name", "checkout")],
                    ..Default::default()
                }),
                scope_logs: vec![parallax_proto::logs::ScopeLogs {
                    log_records: vec![parallax_proto::logs::LogRecord {
                        time_unix_nano: 1_000_000_000,
                        observed_time_unix_nano: 5_000_000_000,
                        event_name: "checkout.completed".to_string(),
                        body: Some(AnyValue {
                            value: Some(AnyValueEnum::StringValue("done".to_string())),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    #[test]
    fn hex_encodes_known_bytes() {
        assert_eq!(hex(&[0x00, 0xff, 0x1a]), "00ff1a");
        assert_eq!(hex(&[]), "");
        assert_eq!(hex(&[0xab; 16]), "abababababababababababababababab");
    }

    #[test]
    fn normalize_logs_carries_event_name_and_observed_timestamp() {
        let rows = normalize_logs(&logs_request());

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].ts_nanos, 1_000_000_000);
        assert_eq!(rows[0].event_name, "checkout.completed");
        assert_eq!(rows[0].observed_ts_nanos, 5_000_000_000);
    }

    #[test]
    fn normalize_logs_defaults_unset_event_name_and_observed_timestamp() {
        let mut request = logs_request();
        let record = &mut request.resource_logs[0].scope_logs[0].log_records[0];
        record.event_name.clear();
        record.time_unix_nano = 0;
        record.observed_time_unix_nano = 0;

        let rows = normalize_logs(&request);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].ts_nanos, 0);
        assert_eq!(rows[0].event_name, "");
        assert_eq!(rows[0].observed_ts_nanos, 0);
    }

    #[test]
    fn promote_log_identity_attributes_adds_native_greptime_keys() {
        let mut request = logs_request();

        assert!(promote_log_identity_attributes(&mut request));
        let record = &request.resource_logs[0].scope_logs[0].log_records[0];
        assert_eq!(
            attr_str(&record.attributes, semconv::EVENT_NAME),
            Some("checkout.completed")
        );
        let observed = record
            .attributes
            .iter()
            .find(|kv| kv.key == semconv::LOG_OBSERVED_TS_NANOS)
            .and_then(|kv| kv.value.as_ref())
            .and_then(|value| match &value.value {
                Some(AnyValueEnum::IntValue(value)) => Some(*value),
                _ => None,
            });
        assert_eq!(observed, Some(5_000_000_000));
        assert!(!promote_log_identity_attributes(&mut request));
    }

    #[test]
    fn normalize_metrics_collects_number_and_histogram_exemplars() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
                resource: Some(parallax_proto::resource::Resource {
                    attributes: vec![
                        string_kv("service.name", "checkout"),
                        string_kv("parallax.run.id", "run-a"),
                    ],
                    ..Default::default()
                }),
                scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                    metrics: vec![
                        Metric {
                            name: "process.cpu.utilization".into(),
                            data: Some(Data::Gauge(Gauge {
                                data_points: vec![NumberDataPoint {
                                    time_unix_nano: 10,
                                    value: Some(NumberValue::AsDouble(0.8)),
                                    exemplars: vec![exemplar(ExemplarValue::AsDouble(0.9), 11)],
                                    ..Default::default()
                                }],
                            })),
                            ..Default::default()
                        },
                        Metric {
                            name: "http.server.request.duration".into(),
                            data: Some(Data::Histogram(Histogram {
                                data_points: vec![HistogramDataPoint {
                                    time_unix_nano: 20,
                                    count: 1,
                                    sum: Some(120.0),
                                    bucket_counts: vec![0, 1],
                                    explicit_bounds: vec![100.0],
                                    exemplars: vec![exemplar(ExemplarValue::AsInt(120), 21)],
                                    ..Default::default()
                                }],
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let normalized = normalize_metrics(&request);

        assert_eq!(normalized.points.len(), 1);
        assert_eq!(normalized.histograms.len(), 1);
        assert_eq!(normalized.exemplars.len(), 2);
        assert_eq!(normalized.exemplars[0].service, "checkout");
        assert_eq!(normalized.exemplars[0].run_id.as_deref(), Some("run-a"));
        assert_eq!(
            normalized.exemplars[0].trace_id,
            "01010101010101010101010101010101"
        );
        assert_eq!(normalized.exemplars[0].span_id, "0202020202020202");
        assert_eq!(normalized.exemplars[0].attributes["route"], "/checkout");
        assert_eq!(normalized.exemplars[1].name, "http.server.request.duration");
        assert_eq!(normalized.exemplars[1].value, 120.0);
    }
}
