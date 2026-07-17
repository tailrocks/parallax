//! In-memory runtime metrics capability.

use super::metric_analytics::metric_row_matches;
use super::*;
use parallax_storage::adapter::AttributeFilter;

#[async_trait::async_trait]
impl adapter::RuntimeMetricStore for MemoryStore {
    async fn metric_series_grouped(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        group_by: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> StorageResult<Vec<(String, Vec<SeriesPoint>)>> {
        if !metric_group_label_allowed(group_by) {
            return Err(adapter::StorageError::query(anyhow::anyhow!(
                "high-cardinality identifier - filter, don't group"
            )));
        }
        let labels = self.metric_labels(name).await?;
        if !labels.iter().any(|label| label == group_by) {
            return Err(adapter::StorageError::query(anyhow::anyhow!(
                "unknown metric label"
            )));
        }
        let step = step_nanos.max(1);
        let mut buckets: BTreeMap<(String, u128), Vec<(u128, f64)>> = Default::default();
        for point in self.lock().metric_points.iter().filter(|p| {
            p.name == name
                && service.is_none_or(|svc| p.service == svc)
                && metric_row_matches(attribute_filters, &p.service, &p.attributes)
                && range.contains(&p.ts_nanos)
        }) {
            buckets
                .entry((
                    group_value(&point.attributes, group_by),
                    (point.ts_nanos / step) * step,
                ))
                .or_default()
                .push((point.ts_nanos, point.value));
        }
        let mut groups: BTreeMap<String, Vec<SeriesPoint>> = Default::default();
        for ((group, ts_nanos), samples) in buckets {
            let values = samples.iter().map(|(_, v)| *v);
            let value = match agg {
                MetricAgg::Avg => values.sum::<f64>() / samples.len() as f64,
                MetricAgg::Min => values.fold(f64::INFINITY, f64::min),
                MetricAgg::Max => values.fold(f64::NEG_INFINITY, f64::max),
                MetricAgg::Last => samples
                    .iter()
                    .max_by_key(|(ts, _)| *ts)
                    .map(|(_, v)| *v)
                    .unwrap_or(0.0),
                MetricAgg::Sum | MetricAgg::Rate | MetricAgg::Increase => values.sum::<f64>(),
            };
            groups
                .entry(group)
                .or_default()
                .push(SeriesPoint { ts_nanos, value });
        }
        Ok(groups
            .into_iter()
            .map(|(group, series)| {
                let series = match agg {
                    MetricAgg::Rate => adapter::rate_from_buckets(&series, step),
                    MetricAgg::Increase => adapter::increase_from_buckets(&series),
                    _ => series,
                };
                (group, series)
            })
            .collect())
    }

    async fn runtime_snapshot(
        &self,
        service: Option<&str>,
        invocation_id: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<RuntimeMetricSeries>> {
        let mut rows = Vec::new();
        for metric in self.metric_names(range.clone()).await? {
            let Some(family) = runtime_metric_family(&metric) else {
                continue;
            };
            let points = self
                .metric_series(
                    &metric,
                    service,
                    invocation_id,
                    &[],
                    range.clone(),
                    step_nanos,
                    MetricAgg::Avg,
                )
                .await?;
            if points.is_empty() {
                continue;
            }
            rows.push(RuntimeMetricSeries {
                family: family.to_string(),
                metric: metric.clone(),
                unit: runtime_metric_unit(&metric),
                points,
            });
        }
        rows.sort_by(|a, b| a.family.cmp(&b.family).then(a.metric.cmp(&b.metric)));
        Ok(rows)
    }

    async fn histogram_count_series(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: BTreeMap<u128, u64> = Default::default();
        for row in self.lock().histograms.iter().filter(|h| {
            h.name == name
                && service.is_none_or(|svc| h.service == svc)
                && range.contains(&h.ts_nanos)
        }) {
            *buckets.entry((row.ts_nanos / step) * step).or_default() += row.count;
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, count)| SeriesPoint {
                ts_nanos,
                value: count as f64,
            })
            .collect())
    }
}
