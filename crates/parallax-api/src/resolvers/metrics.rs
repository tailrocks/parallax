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
