//! In-memory error analytics capability.

use super::*;

#[async_trait::async_trait]
impl adapter::ErrorAnalyticsStore for MemoryStore {
    async fn error_count_series(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: BTreeMap<u128, u64> = Default::default();
        for event in self
            .lock()
            .error_events
            .iter()
            .filter(|e| e.service == service && range.contains(&e.ts_nanos))
        {
            *buckets.entry((event.ts_nanos / step) * step).or_default() += 1;
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
