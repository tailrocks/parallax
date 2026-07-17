//! In-memory metric analytics capability.

use super::*;
use parallax_storage::adapter::AttributeFilter;

/// Evaluate plan-168 where filters against one exported point/histogram row.
/// `service`/`service.name` reads the row's service; other keys read the
/// attribute object (absent values satisfy only negative operators).
pub(super) fn metric_row_matches(
    filters: &[AttributeFilter],
    service: &str,
    attributes: &serde_json::Value,
) -> bool {
    filters.iter().all(|filter| {
        let key = filter.key.trim();
        let observed: Option<String> = if key == "service" || key == "service.name" {
            Some(service.to_string())
        } else {
            attributes.get(key).map(|value| match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
        };
        filter.matches(observed.as_deref())
    })
}

#[async_trait::async_trait]
impl MetricAnalyticsStore for MemoryStore {
    async fn metric_series(
        &self,
        name: &str,
        service: Option<&str>,
        invocation_id: Option<&str>,
        attribute_filters: &[AttributeFilter],
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> StorageResult<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: BTreeMap<u128, Vec<(u128, f64)>> = Default::default();
        for point in self.lock().metric_points.iter().filter(|p| {
            p.name == name
                && service.is_none_or(|svc| p.service == svc)
                && invocation_id.is_none_or(|id| p.invocation_id.as_deref() == Some(id))
                && metric_row_matches(attribute_filters, &p.service, &p.attributes)
                && range.contains(&p.ts_nanos)
        }) {
            buckets
                .entry((point.ts_nanos / step) * step)
                .or_default()
                .push((point.ts_nanos, point.value));
        }
        let mut series: Vec<SeriesPoint> = buckets
            .into_iter()
            .map(|(ts_nanos, samples)| {
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
                    // RATE/INCREASE start from the per-bucket sum of the counter.
                    MetricAgg::Sum | MetricAgg::Rate | MetricAgg::Increase => values.sum::<f64>(),
                };
                SeriesPoint { ts_nanos, value }
            })
            .collect();
        if agg == MetricAgg::Rate {
            series = adapter::rate_from_buckets(&series, step);
        } else if agg == MetricAgg::Increase {
            series = adapter::increase_from_buckets(&series);
        }
        Ok(series)
    }

    async fn histogram_quantile(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        range: RangeInclusive<u128>,
        step_nanos: u128,
        q: f64,
    ) -> StorageResult<Vec<SeriesPoint>> {
        // Latest sample per window (plan 085) — align with greptime MAX merge.
        let step = step_nanos.max(1);
        let mut latest: BTreeMap<u128, HistogramRow> = Default::default();
        for row in self.lock().histograms.iter().filter(|h| {
            h.name == name
                && service.is_none_or(|svc| h.service == svc)
                && metric_row_matches(attribute_filters, &h.service, &h.attributes)
                && range.contains(&h.ts_nanos)
        }) {
            let window = (row.ts_nanos / step) * step;
            match latest.get(&window) {
                Some(cur) if cur.ts_nanos >= row.ts_nanos => {}
                _ => {
                    latest.insert(window, row.clone());
                }
            }
        }
        Ok(latest
            .into_iter()
            .map(|(ts_nanos, row)| SeriesPoint {
                ts_nanos,
                value: quantile_from_histograms(&[row], q),
            })
            .collect())
    }

    async fn histogram_avg(
        &self,
        name: &str,
        service: Option<&str>,
        attribute_filters: &[AttributeFilter],
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>> {
        // Latest cumulative export per bucket, then Δsum/Δcount client math —
        // the same shape the Greptime `_sum`/`_count` stat tables produce.
        let step = step_nanos.max(1);
        let mut latest: BTreeMap<u128, HistogramRow> = Default::default();
        for row in self.lock().histograms.iter().filter(|h| {
            h.name == name
                && service.is_none_or(|svc| h.service == svc)
                && metric_row_matches(attribute_filters, &h.service, &h.attributes)
                && range.contains(&h.ts_nanos)
        }) {
            let window = (row.ts_nanos / step) * step;
            match latest.get(&window) {
                Some(cur) if cur.ts_nanos >= row.ts_nanos => {}
                _ => {
                    latest.insert(window, row.clone());
                }
            }
        }
        let sums: Vec<SeriesPoint> = latest
            .iter()
            .map(|(ts_nanos, row)| SeriesPoint {
                ts_nanos: *ts_nanos,
                value: row.sum,
            })
            .collect();
        let counts: Vec<SeriesPoint> = latest
            .iter()
            .map(|(ts_nanos, row)| SeriesPoint {
                ts_nanos: *ts_nanos,
                value: row.count as f64,
            })
            .collect();
        Ok(adapter::histogram_avg_from_cumulative(&sums, &counts))
    }

    async fn invocation_metric_summaries(
        &self,
        invocation_id: &str,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<InvocationMetricSummary>> {
        let mut by_name: BTreeMap<String, InvocationMetricSummary> = BTreeMap::new();
        for point in self.lock().metric_points.iter().filter(|p| {
            p.invocation_id.as_deref() == Some(invocation_id)
                && range.contains(&p.ts_nanos)
                && p.value.is_finite()
        }) {
            let canonical = parallax_semconv::native_metric_table_base(&point.name);
            let entry =
                by_name
                    .entry(canonical.clone())
                    .or_insert_with(|| InvocationMetricSummary {
                        name: canonical,
                        point_count: 0,
                        last_value: point.value,
                        last_ts_nanos: point.ts_nanos,
                    });
            entry.point_count += 1;
            if point.ts_nanos >= entry.last_ts_nanos {
                entry.last_ts_nanos = point.ts_nanos;
                entry.last_value = point.value;
            }
        }
        Ok(by_name.into_values().take(limit).collect())
    }

    async fn metric_exemplars(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        limit: usize,
    ) -> StorageResult<Vec<MetricExemplarRow>> {
        let mut rows: Vec<MetricExemplarRow> = self
            .lock()
            .metric_exemplars
            .iter()
            .filter(|row| {
                row.name == name
                    && service.is_none_or(|svc| row.service == svc)
                    && range.contains(&row.ts_nanos)
            })
            .cloned()
            .collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.ts_nanos));
        rows.truncate(limit.min(MAX_ROWS));
        Ok(rows)
    }
}
