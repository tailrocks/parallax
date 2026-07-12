//! OTLP → normalized rows, per the implementation-spec §7 mapping.
#![expect(
    clippy::cast_precision_loss,
    clippy::excessive_nesting,
    reason = "mechanically transferred OTLP projection; Plan 098 owns the split"
)]
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]

use parallax_model::{HistogramRow, LogRow, MetricExemplarRow, MetricPointRow, SpanRow};
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::exemplar::Value as ExemplarValue;
use parallax_proto::metrics::metric::Data;
use parallax_proto::metrics::number_data_point::Value as NumberValue;
use parallax_proto::semconv;

mod values;

use values::{any_value_to_json, attr_str, attributes_to_json, hex};

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
            .map_or(&[][..], |r| r.attributes.as_slice());
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
            .map_or(&[][..], |r| r.attributes.as_slice());
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

#[derive(Debug)]
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
            .map_or(&[][..], |r| r.attributes.as_slice());
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
mod tests;
