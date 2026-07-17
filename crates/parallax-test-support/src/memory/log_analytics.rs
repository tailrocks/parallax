//! In-memory log analytics capability.

use super::*;

#[async_trait::async_trait]
impl adapter::LogAnalyticsStore for MemoryStore {
    async fn logs_search(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        severity_min: Option<i32>,
        severity_max: Option<i32>,
        body_contains: Option<&str>,
        attribute_filters: &[adapter::AttributeFilter],
        limit: usize,
    ) -> StorageResult<Vec<LogRow>> {
        let mut logs: Vec<LogRow> = self
            .lock()
            .logs
            .iter()
            .filter(|l| {
                range.contains(&l.ts_nanos)
                    && service.is_none_or(|svc| l.service == svc)
                    && severity_min.is_none_or(|min| l.severity_num >= min)
                    && severity_max.is_none_or(|max| l.severity_num <= max)
                    && body_contains.is_none_or(|needle| l.body.contains(needle))
                    && attribute_filters
                        .iter()
                        .all(|f| f.matches(log_filter_observed_value(l, &f.key).as_deref()))
            })
            .cloned()
            .collect();
        logs.sort_by_key(|l| std::cmp::Reverse(l.ts_nanos));
        logs.truncate(limit);
        Ok(logs)
    }
}

/// Observed value for a where-clause key on one log row: intrinsics mirror
/// the GreptimeDB log compiler's column mapping; everything else reads the
/// log attribute object.
pub(super) fn log_filter_observed_value(log: &LogRow, key: &str) -> Option<String> {
    match key.trim() {
        "service.name" | "service" => Some(log.service.clone()),
        "severity" | "severity_text" => Some(log.severity_text.clone()),
        "severity_number" => Some(log.severity_num.to_string()),
        "body" => Some(log.body.clone()),
        "trace_id" => Some(log.trace_id.clone()),
        "span_id" => Some(log.span_id.clone()),
        key => match log.attributes.get(key)? {
            serde_json::Value::String(value) => Some(value.clone()),
            serde_json::Value::Bool(value) => Some(value.to_string()),
            serde_json::Value::Number(value) => Some(value.to_string()),
            _ => None,
        },
    }
}
