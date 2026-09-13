use super::*;

#[derive(Debug)]
pub struct NormalizedMetrics {
    pub points: Vec<MetricPointRow>,
    pub histograms: Vec<HistogramRow>,
    /// Exponential histograms converted to explicit buckets at ingest (the
    /// native engine has no exp type; converted rows are persisted separately
    /// so explicit-histogram queries never double-count them).
    pub exp_histograms: Vec<HistogramRow>,
    pub exemplars: Vec<MetricExemplarRow>,
    /// Summary metrics received but not stored (no lossless representation),
    /// plus exp points refused conversion (degenerate scale).
    pub dropped_unsupported: u64,
}

/// Remove exponential-histogram metrics from a request before the native OTLP
/// forward. Converted rows are persisted from [`NormalizedMetrics`] instead,
/// so forwarding the originals would double-store on engines that accept exp
/// histograms and poison the whole batch on engines that reject them.
/// Prunes emptied scopes/resources. Returns whether anything was removed.
pub fn strip_exp_histograms(request: &mut ExportMetricsServiceRequest) -> bool {
    use parallax_proto::metrics::metric::Data as MetricData;
    let mut changed = false;
    for rm in &mut request.resource_metrics {
        for sm in &mut rm.scope_metrics {
            let before = sm.metrics.len();
            sm.metrics
                .retain(|metric| !matches!(metric.data, Some(MetricData::ExponentialHistogram(_))));
            changed |= sm.metrics.len() != before;
        }
        rm.scope_metrics.retain(|sm| !sm.metrics.is_empty());
    }
    request
        .resource_metrics
        .retain(|rm| !rm.scope_metrics.is_empty());
    changed
}

pub fn normalize_metrics(request: &ExportMetricsServiceRequest) -> NormalizedMetrics {
    let mut points = Vec::new();
    let mut histograms = Vec::new();
    let mut exp_histograms = Vec::new();
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
                        push_histograms(
                            h,
                            &service,
                            invocation_id.as_deref(),
                            &metric.name,
                            &mut histograms,
                            &mut exemplars,
                        );
                    }
                    Some(Data::ExponentialHistogram(h)) => {
                        push_exp_histograms(
                            h,
                            &service,
                            invocation_id.as_deref(),
                            &metric.name,
                            &mut exp_histograms,
                            &mut exemplars,
                            &mut dropped_unsupported,
                        );
                    }
                    Some(Data::Summary(_)) => {
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
        exp_histograms,
        exemplars,
        dropped_unsupported,
    }
}

/// Project one explicit histogram's points to rows.
fn push_histograms(
    histogram: &parallax_proto::metrics::Histogram,
    service: &str,
    invocation_id: Option<&str>,
    name: &str,
    histograms: &mut Vec<HistogramRow>,
    exemplars: &mut Vec<MetricExemplarRow>,
) {
    for dp in &histogram.data_points {
        push_exemplars(
            exemplars,
            service,
            invocation_id,
            name,
            dp.time_unix_nano,
            &dp.exemplars,
        );
        histograms.push(HistogramRow {
            ts_nanos: u128::from(dp.time_unix_nano),
            service: service.to_string(),
            name: name.to_string(),
            count: dp.count,
            sum: dp.sum.unwrap_or(0.0),
            bucket_counts: dp.bucket_counts.clone(),
            bounds: dp.explicit_bounds.clone(),
            attributes: attributes_to_json(&dp.attributes),
        });
    }
}

/// Convert one exponential histogram's points to explicit-bucket rows.
/// Exemplars survive (same trace/span linkage as other encodings); points
/// refused conversion count as unsupported instead of vanishing silently.
#[expect(clippy::too_many_arguments, reason = "one sink per output column")]
fn push_exp_histograms(
    histogram: &parallax_proto::metrics::ExponentialHistogram,
    service: &str,
    invocation_id: Option<&str>,
    name: &str,
    exp_histograms: &mut Vec<HistogramRow>,
    exemplars: &mut Vec<MetricExemplarRow>,
    dropped_unsupported: &mut u64,
) {
    for dp in &histogram.data_points {
        push_exemplars(
            exemplars,
            service,
            invocation_id,
            name,
            dp.time_unix_nano,
            &dp.exemplars,
        );
        match exp_histogram::convert(dp) {
            Some(converted) => {
                exp_histograms.push(HistogramRow {
                    ts_nanos: u128::from(dp.time_unix_nano),
                    service: service.to_string(),
                    name: name.to_string(),
                    count: converted.count,
                    sum: converted.sum,
                    bucket_counts: converted.bucket_counts,
                    bounds: converted.bounds,
                    attributes: attributes_to_json(&dp.attributes),
                });
            }
            None => {
                *dropped_unsupported += 1;
            }
        }
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
