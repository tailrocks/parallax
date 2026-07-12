use super::*;

pub(super) const BUCKET_MILLIS: i64 = 60_000;

/// Window cap for filtered issue scans; `issues_filtered`'s `total` is exact
/// up to this many matching rows.
pub(super) const ISSUE_SCAN_CAP: usize = 1000;

/// Nanosecond timestamps are stored as INTEGER milliseconds in the metadata
/// store (SQLite-class integers are i64; nanos since 1970 overflow in 2262 as
/// i64 but UI/sorting only needs millis precision here).
pub(super) fn nanos_to_millis(nanos: u128) -> i64 {
    i64::try_from(nanos / 1_000_000).unwrap_or(i64::MAX)
}

pub(super) fn millis_to_nanos(millis: i64) -> u128 {
    u128::try_from(millis.max(0)).unwrap_or(0) * 1_000_000
}

/// Bounds for the per-issue tag-values cache (`issues.tags`).
pub(super) const TAGS_MAX_KEYS: usize = 16;
pub(super) const TAGS_MAX_VALUES_PER_KEY: usize = 8;
pub(super) const TAGS_MAX_VALUE_LEN: usize = 64;

/// Merge an event's scalar attributes into the `{key: {value: count}}` cache.
/// Exception keys are the event body, not tags; nested values are skipped.
pub(super) fn merge_tags(existing: &str, attributes: &serde_json::Value) -> String {
    let mut tags: BTreeMap<String, BTreeMap<String, u64>> =
        serde_json::from_str(existing).unwrap_or_default();
    if let Some(map) = attributes.as_object() {
        for (key, value) in map {
            if key.starts_with(semconv::EXCEPTION_EVENT_NAME)
                && key
                    .as_bytes()
                    .get(semconv::EXCEPTION_EVENT_NAME.len())
                    .is_some_and(|byte| *byte == b'.')
            {
                continue;
            }
            let rendered = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => continue,
            };
            if rendered.is_empty() || rendered.len() > TAGS_MAX_VALUE_LEN {
                continue;
            }
            if !tags.contains_key(key) && tags.len() >= TAGS_MAX_KEYS {
                continue;
            }
            let values = tags.entry(key.clone()).or_default();
            if !values.contains_key(&rendered) && values.len() >= TAGS_MAX_VALUES_PER_KEY {
                continue;
            }
            *values.entry(rendered).or_insert(0) += 1;
        }
    }
    serde_json::to_string(&tags).unwrap_or_else(|_| "{}".to_string())
}
