#![expect(
    clippy::too_many_arguments,
    reason = "stable GraphQL metric filter contract"
)]

//! GraphQL metrics domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;

use crate::{
    ApiContext, clamp_limit, field_err, nanos_string, parse_range, retained_recent_range,
    step_nanos, validate_metric_group_label, validate_metric_name,
};

use crate::resolvers::common::Point;
use crate::resolvers::traces::AttributeFilterInput;
use parallax_storage::adapter::RuntimeMetricSeries as StorageRuntimeMetricSeries;
use parallax_storage::model::{MetricAgg, SeriesPoint};

pub(crate) struct Series {
    group_value: Option<String>,
    points: Vec<SeriesPoint>,
}

#[graphql_object(context = ApiContext)]
impl Series {
    fn group_value(&self) -> Option<&str> {
        self.group_value.as_deref()
    }
    fn points(&self) -> Vec<Point> {
        self.points.iter().copied().map(Point).collect()
    }
}

pub(crate) struct RuntimeMetric(pub(crate) StorageRuntimeMetricSeries);

#[graphql_object(context = ApiContext)]
impl RuntimeMetric {
    fn family(&self) -> &str {
        &self.0.family
    }
    fn metric(&self) -> &str {
        &self.0.metric
    }
    fn unit(&self) -> Option<&str> {
        self.0.unit.as_deref()
    }
    fn points(&self) -> Vec<Point> {
        self.0.points.iter().copied().map(Point).collect()
    }
}

pub(crate) struct MetricExemplar(pub(crate) model::MetricExemplarRow);

#[graphql_object(context = ApiContext)]
impl MetricExemplar {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn value(&self) -> f64 {
        self.0.value
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn invocation_id(&self) -> Option<&str> {
        self.0.invocation_id.as_deref()
    }
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
    }
}

/// Effective bucket step per the metric-summary contract: requested step is
/// rounded up, never down, to cover the window with at most 120 buckets;
/// minimum one second; default `max(1s, ceil(window / 60))`.
pub(crate) fn effective_step_seconds(from: u128, to: u128, requested: Option<i32>) -> u128 {
    const MAX_BUCKETS: u128 = 120;
    let window_seconds = (to.saturating_sub(from)).div_ceil(1_000_000_000).max(1);
    let step = match requested {
        Some(step) if step >= 1 => u128::try_from(step).unwrap_or(1),
        _ => window_seconds.div_ceil(60).max(1),
    };
    step.max(window_seconds.div_ceil(MAX_BUCKETS))
}

/// One `metricQuery` response: the shared explorer/dashboard/alert read path
/// (plan 168 — no client forks its own metric SQL).
pub(crate) struct MetricQueryOut {
    kind: model::MetricKind,
    effective_step_seconds: u128,
    series: Vec<Series>,
}

#[graphql_object(context = ApiContext)]
impl MetricQueryOut {
    fn kind(&self) -> &str {
        self.kind.as_str()
    }
    /// The step actually used after contract rounding (≤120 buckets).
    fn effective_step_seconds(&self) -> i32 {
        i32::try_from(self.effective_step_seconds).unwrap_or(i32::MAX)
    }
    fn series(&self) -> &[Series] {
        &self.series
    }
}

/// Aggregation legality per metric kind (contract decision 2 — illegal
/// combinations are rejected with the legal set named).
fn legal_aggregations(kind: model::MetricKind) -> &'static [&'static str] {
    match kind {
        model::MetricKind::Gauge => &["avg", "min", "max", "last"],
        model::MetricKind::Sum => &["sum", "rate", "increase"],
        model::MetricKind::Histogram => &["p50", "p95", "p99", "avg"],
    }
}

pub(crate) async fn metric_query(
    context: &ApiContext,
    name: String,
    kind: String,
    agg: String,
    from_nanos: String,
    to_nanos: String,
    service: Option<String>,
    attribute_filters: Option<Vec<AttributeFilterInput>>,
    group_by: Option<String>,
    step_seconds: Option<i32>,
) -> FieldResult<MetricQueryOut> {
    validate_metric_name(&name)?;
    let filters = attribute_filters
        .unwrap_or_default()
        .into_iter()
        .map(|filter| filter.into_adapter().map_err(field_err))
        .collect::<Result<Vec<_>, _>>()?;
    let kind = model::MetricKind::parse(&kind)
        .ok_or_else(|| field_err("kind must be gauge|sum|histogram"))?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let agg = agg.to_ascii_lowercase();
    if !legal_aggregations(kind).contains(&agg.as_str()) {
        return Err(field_err(format!(
            "agg '{agg}' is illegal for kind '{}' — legal: {}",
            kind.as_str(),
            legal_aggregations(kind).join("|"),
        )));
    }
    let step = effective_step_seconds(from, to, step_seconds);
    let step_ns = step * 1_000_000_000;
    let series = match kind {
        model::MetricKind::Histogram => {
            if group_by.is_some() {
                return Err(field_err(
                    "groupBy is not supported for histogram quantiles yet",
                ));
            }
            let points = if agg == "avg" {
                context
                    .store
                    .histogram_avg(&name, service.as_deref(), &filters, from..=to, step_ns)
                    .await
                    .map_err(crate::internal_field_err)?
            } else {
                let q = match agg.as_str() {
                    "p50" => 0.50,
                    "p95" => 0.95,
                    _ => 0.99,
                };
                context
                    .store
                    .histogram_quantile(&name, service.as_deref(), &filters, from..=to, step_ns, q)
                    .await
                    .map_err(crate::internal_field_err)?
            };
            vec![Series {
                group_value: None,
                points,
            }]
        }
        model::MetricKind::Gauge | model::MetricKind::Sum => {
            let agg = MetricAgg::parse(&agg).ok_or_else(|| field_err("unsupported agg"))?;
            if let Some(group_by) = group_by {
                validate_metric_group_label(&group_by)?;
                context
                    .store
                    .metric_series_grouped(
                        &name,
                        service.as_deref(),
                        &filters,
                        &group_by,
                        from..=to,
                        step_ns,
                        agg,
                    )
                    .await
                    .map_err(crate::internal_field_err)?
                    .into_iter()
                    .map(|(group_value, points)| Series {
                        group_value: Some(group_value),
                        points,
                    })
                    .collect()
            } else {
                vec![Series {
                    group_value: None,
                    points: context
                        .store
                        .metric_series(
                            &name,
                            service.as_deref(),
                            None,
                            &filters,
                            from..=to,
                            step_ns,
                            agg,
                        )
                        .await
                        .map_err(crate::internal_field_err)?,
                }]
            }
        }
    };
    // Contract: sum-family buckets zero-fill so aligned windows stay
    // comparable; gauges and histograms keep honest gaps (no fabricated
    // samples).
    let series = if kind == model::MetricKind::Sum {
        series
            .into_iter()
            .map(|s| Series {
                group_value: s.group_value,
                points: zero_fill_buckets(s.points, from, to, step_ns),
            })
            .collect()
    } else {
        series
    };
    Ok(MetricQueryOut {
        kind,
        effective_step_seconds: step,
        series,
    })
}

/// Fill missing epoch-aligned buckets in `[from, to]` with zero values.
/// Empty input stays empty — an absent series is not fabricated.
fn zero_fill_buckets(
    points: Vec<SeriesPoint>,
    from: u128,
    to: u128,
    step_ns: u128,
) -> Vec<SeriesPoint> {
    if points.is_empty() || step_ns == 0 {
        return points;
    }
    let by_ts: std::collections::BTreeMap<u128, f64> =
        points.into_iter().map(|p| (p.ts_nanos, p.value)).collect();
    let mut out = Vec::new();
    let mut ts = (from / step_ns) * step_ns;
    while ts <= to {
        out.push(SeriesPoint {
            ts_nanos: ts,
            value: by_ts.get(&ts).copied().unwrap_or(0.0),
        });
        ts += step_ns;
    }
    out
}

pub(crate) struct MetricCatalogRow(pub(crate) model::MetricCatalogEntry);

#[graphql_object(context = ApiContext)]
impl MetricCatalogRow {
    /// Canonical native-table display name (metric-summary contract).
    fn name(&self) -> &str {
        &self.0.name
    }
    /// gauge | sum | histogram — bounds legal aggregations client-side.
    fn kind(&self) -> &str {
        self.0.kind.as_str()
    }
    fn unit(&self) -> Option<&str> {
        self.0.unit.as_deref()
    }
    /// Emitting services inside the window, deduplicated and sorted.
    fn services(&self) -> &[String] {
        &self.0.services
    }
    fn last_datapoint_nanos(&self) -> String {
        nanos_string(self.0.last_datapoint_nanos)
    }
    /// Finite exported samples in the window; one count per histogram export.
    fn point_count(&self) -> String {
        self.0.point_count.to_string()
    }
}

pub(crate) async fn metric_catalog(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
    q: Option<String>,
    kind: Option<String>,
    limit: Option<i32>,
) -> FieldResult<Vec<MetricCatalogRow>> {
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let kind = match kind.as_deref() {
        None => None,
        Some(raw) => Some(
            model::MetricKind::parse(raw)
                .ok_or_else(|| field_err("kind must be gauge|sum|histogram"))?,
        ),
    };
    let rows = context
        .store
        .metric_catalog(from..=to, q.as_deref(), kind, clamp_limit(limit, 100))
        .await
        .map_err(crate::internal_field_err)?;
    Ok(rows.into_iter().map(MetricCatalogRow).collect())
}

pub(crate) async fn metric_names(
    context: &ApiContext,
    prefix: Option<String>,
) -> FieldResult<Vec<String>> {
    let mut names = context
        .store
        .metric_names(retained_recent_range())
        .await
        .map_err(crate::internal_field_err)?;
    if let Some(prefix) = prefix {
        names.retain(|n| n.starts_with(&prefix));
    }
    Ok(names)
}

pub(crate) async fn metric_labels(context: &ApiContext, name: String) -> FieldResult<Vec<String>> {
    validate_metric_name(&name)?;
    context
        .store
        .metric_labels(&name)
        .await
        .map_err(crate::internal_field_err)
}

pub(crate) async fn metric_label_values(
    context: &ApiContext,
    name: String,
    label: String,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<String>> {
    validate_metric_name(&name)?;
    validate_metric_group_label(&label)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    context
        .store
        .metric_label_values(&name, &label, from..=to)
        .await
        .map_err(crate::internal_field_err)
}

pub(crate) async fn services(context: &ApiContext) -> FieldResult<Vec<String>> {
    context
        .store
        .service_names(retained_recent_range())
        .await
        .map_err(crate::internal_field_err)
}

pub(crate) async fn runtime_snapshot(
    context: &ApiContext,
    service: Option<String>,
    invocation_id: Option<String>,
    from_nanos: String,
    to_nanos: String,
    step_seconds: i32,
) -> FieldResult<Vec<RuntimeMetric>> {
    match (service.as_deref(), invocation_id.as_deref()) {
        (Some(_), Some(_)) | (None, None) => {
            return Err(field_err("runtimeSnapshot takes exactly one scope"));
        }
        _ => {}
    }
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let rows = context
        .store
        .runtime_snapshot(
            service.as_deref(),
            invocation_id.as_deref(),
            from..=to,
            step_nanos(Some(step_seconds)),
        )
        .await
        .map_err(crate::internal_field_err)?;
    Ok(rows.into_iter().map(RuntimeMetric).collect())
}

#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn metric_series(
    context: &ApiContext,
    name: String,
    from_nanos: String,
    to_nanos: String,
    service: Option<String>,
    invocation_id: Option<String>,
    group_by: Option<String>,
    step_seconds: Option<i32>,
    agg: Option<String>,
) -> FieldResult<Vec<Series>> {
    validate_metric_name(&name)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let agg = MetricAgg::parse(agg.as_deref().unwrap_or("avg"))
        .ok_or_else(|| field_err("agg must be avg|min|max|sum|rate"))?;
    if let Some(group_by) = group_by {
        validate_metric_group_label(&group_by)?;
        if invocation_id.is_some() {
            return Err(field_err("invocationId with groupBy is not supported yet"));
        }
        let groups = context
            .store
            .metric_series_grouped(
                &name,
                service.as_deref(),
                &[],
                &group_by,
                from..=to,
                step_nanos(step_seconds),
                agg,
            )
            .await
            .map_err(crate::internal_field_err)?;
        Ok(groups
            .into_iter()
            .map(|(group_value, points)| Series {
                group_value: Some(group_value),
                points,
            })
            .collect())
    } else {
        let points = context
            .store
            .metric_series(
                &name,
                service.as_deref(),
                invocation_id.as_deref(),
                &[],
                from..=to,
                step_nanos(step_seconds),
                agg,
            )
            .await
            .map_err(crate::internal_field_err)?;
        Ok(vec![Series {
            group_value: None,
            points,
        }])
    }
}

pub(crate) async fn histogram_quantile(
    context: &ApiContext,
    name: String,
    from_nanos: String,
    to_nanos: String,
    q: f64,
    service: Option<String>,
    step_seconds: Option<i32>,
) -> FieldResult<Vec<Point>> {
    validate_metric_name(&name)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let series = context
        .store
        .histogram_quantile(
            &name,
            service.as_deref(),
            &[],
            from..=to,
            step_nanos(step_seconds),
            q,
        )
        .await
        .map_err(crate::internal_field_err)?;
    Ok(series.into_iter().map(Point).collect())
}

pub(crate) async fn metric_exemplars(
    context: &ApiContext,
    name: String,
    from_nanos: String,
    to_nanos: String,
    service: Option<String>,
    limit: Option<i32>,
) -> FieldResult<Vec<MetricExemplar>> {
    validate_metric_name(&name)?;
    let (from, to) = parse_range(&from_nanos, &to_nanos)?;
    let rows = context
        .store
        .metric_exemplars(&name, service.as_deref(), from..=to, clamp_limit(limit, 50))
        .await
        .map_err(crate::internal_field_err)?;
    Ok(rows.into_iter().map(MetricExemplar).collect())
}

#[cfg(test)]
mod tests;
