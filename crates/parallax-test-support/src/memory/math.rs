use std::ops::RangeInclusive;

use parallax_model::{HistogramRow, SpanRow};
use parallax_storage::adapter::{attribute_compare_value_allowed, field_value_display};

pub(super) fn group_value(attributes: &serde_json::Value, key: &str) -> String {
    match attributes.get(key) {
        Some(serde_json::Value::String(value)) => value.clone(),
        Some(serde_json::Value::Bool(value)) => value.to_string(),
        Some(serde_json::Value::Number(value)) => value.to_string(),
        _ => "(none)".to_string(),
    }
}

pub(super) fn scalar_attribute_value(attributes: &serde_json::Value, key: &str) -> Option<String> {
    let value = match attributes.get(key)? {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        _ => return None,
    };
    attribute_compare_value_allowed(&value).then_some(value)
}

pub(super) fn field_scalar_value(attributes: &serde_json::Value, key: &str) -> Option<String> {
    let value = match attributes.get(key)? {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        _ => return None,
    };
    field_value_display(&value)
}

pub(super) fn resource_string(resource: &serde_json::Value, key: &str) -> Option<String> {
    resource
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(super) fn span_matches_compare(
    span: &SpanRow,
    range: &RangeInclusive<u128>,
    service: Option<&str>,
    error_only: bool,
) -> bool {
    range.contains(&span.ts_nanos)
        && service.is_none_or(|candidate| span.service == candidate)
        && (!error_only || span.status_code == "STATUS_CODE_ERROR")
}

pub(super) fn duration_quantile_ms(durations: &mut [u128], q: f64) -> f64 {
    durations.sort_unstable();
    quantile_from_sorted(durations, q) / 1_000_000.0
}

pub(super) fn quantile_from_histograms(rows: &[HistogramRow], q: f64) -> f64 {
    let Some(first) = rows.first() else {
        return 0.0;
    };
    let bounds = &first.bounds;
    let mut counts = vec![0u64; bounds.len() + 1];
    for row in rows {
        for (index, count) in row.bucket_counts.iter().enumerate() {
            if let Some(slot) = counts.get_mut(index) {
                *slot += count;
            }
        }
    }
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }
    let target = q.clamp(0.0, 1.0) * total as f64;
    let mut cumulative = 0u64;
    for (index, count) in counts.iter().enumerate() {
        let next = cumulative + count;
        if next as f64 >= target {
            let lower = if index == 0 { 0.0 } else { bounds[index - 1] };
            let upper = bounds.get(index).copied().unwrap_or(lower);
            let within = if *count == 0 {
                0.0
            } else {
                (target - cumulative as f64) / *count as f64
            };
            return lower + (upper - lower) * within;
        }
        cumulative = next;
    }
    bounds.last().copied().unwrap_or(0.0)
}

pub(super) fn quantile_from_sorted(values: &[u128], q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    if values.len() == 1 {
        return values[0] as f64;
    }
    let position = q.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let low = position.floor() as usize;
    let high = position.ceil() as usize;
    if low == high {
        return values[low] as f64;
    }
    let weight = position - low as f64;
    values[low] as f64 + (values[high] as f64 - values[low] as f64) * weight
}
