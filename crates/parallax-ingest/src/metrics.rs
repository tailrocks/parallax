use super::*;

#[derive(Debug)]
pub struct NormalizedMetrics {
    pub points: Vec<MetricPointRow>,
    pub histograms: Vec<HistogramRow>,
    pub exemplars: Vec<MetricExemplarRow>,
    /// Exponential-histogram / summary metrics received but not stored (V1).
    pub dropped_unsupported: u64,
}

pub fn normalize_metrics(request: &ExportMetricsServiceRequest) -> NormalizedMetrics {
    let mut points = Vec::new();
    let mut histograms = Vec::new();
    let mut exemplars = Vec::new();
    let mut dropped_unsupported = 0_u64;
    for rm in &request.resource_metrics {
        let resource_attrs = rm
            .resource
            .as_ref()
            .map_or(&[][..], |r| r.attributes.as_slice());
        let service = service_name(resource_attrs);
        let invocation_id = invocation_id(&[], resource_attrs);
        for sm in &rm.scope_metrics {
            for metric in &sm.metrics {
                match &metric.data {
                    Some(Data::Gauge(g)) => {
                        for dp in &g.data_points {
                            push_exemplars(
                                &mut exemplars,
                                &service,
                                invocation_id.as_deref(),
                                &metric.name,
                                dp.time_unix_nano,
                                &dp.exemplars,
                            );
                            points.push(number_point(
                                &service,
                                invocation_id.as_deref(),
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
                                invocation_id.as_deref(),
                                &metric.name,
                                dp.time_unix_nano,
                                &dp.exemplars,
                            );
                            points.push(number_point(
                                &service,
                                invocation_id.as_deref(),
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
                                invocation_id.as_deref(),
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
                    Some(Data::ExponentialHistogram(_) | Data::Summary(_)) => {
                        dropped_unsupported += 1;
                    }
                    None => {}
                }
            }
        }
    }
    NormalizedMetrics {
        points,
        histograms,
        exemplars,
        dropped_unsupported,
    }
}

fn push_exemplars(
    rows: &mut Vec<MetricExemplarRow>,
    service: &str,
    invocation_id: Option<&str>,
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
            invocation_id: invocation_id.map(str::to_string),
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
    invocation_id: Option<&str>,
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
        invocation_id: invocation_id.map(str::to_string),
        attributes: attributes_to_json(&dp.attributes),
    }
}
