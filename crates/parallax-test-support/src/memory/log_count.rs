use super::MemoryStore;
use parallax_model::SeriesPoint;
use parallax_storage::adapter::{LogCountStore, StorageResult};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

#[async_trait::async_trait]
impl LogCountStore for MemoryStore {
    async fn log_count_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        step_nanos: u128,
    ) -> StorageResult<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let mut buckets: BTreeMap<u128, u64> = Default::default();
        for log in self.lock().logs.iter().filter(|log| {
            range.contains(&log.ts_nanos)
                && service.is_none_or(|candidate| log.service == candidate)
                && severity_min.is_none_or(|min| log.severity_num >= min)
                && severity_max.is_none_or(|max| log.severity_num <= max)
                && body_contains.is_none_or(|needle| log.body.contains(needle))
        }) {
            *buckets.entry((log.ts_nanos / step) * step).or_default() += 1;
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
