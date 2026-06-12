//! OTLP → normalized rows, per the implementation-spec §7 mapping.

use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::metric::Data;
use parallax_proto::metrics::number_data_point::Value as NumberValue;
use parallax_storage::model::{HistogramRow, LogRow, MetricPointRow, SpanRow};

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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

fn service_name(resource_attrs: &[KeyValue]) -> String {
    attr_str(resource_attrs, "service.name")
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

/// Resolve the run id from resource attributes. `parallax.run.id` is
/// canonical; the standard aliases are accepted so OTel-conventional
/// emitters correlate without Parallax-specific wiring (no OTel standard
/// exists for a CLI run id — `session.id` is the closest concept,
/// `cicd.pipeline.run.id` the literal "run id" semconv; see
/// docs/research/capture/run-id-standardization.md).
fn run_id(resource_attrs: &[KeyValue]) -> Option<String> {
    ["parallax.run.id", "session.id", "cicd.pipeline.run.id"]
        .iter()
        .find_map(|key| attr_str(resource_attrs, key))
        .map(str::to_string)
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
}

pub fn normalize_metrics(request: &ExportMetricsServiceRequest) -> NormalizedMetrics {
    let mut points = Vec::new();
    let mut histograms = Vec::new();
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
    NormalizedMetrics { points, histograms }
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
