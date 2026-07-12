pub(crate) const RUNTIME_METRIC_PREFIXES: &[&str] = &[
    "process.",
    "system.",
    "jvm.",
    "tokio.runtime.",
    "container.",
    "db.client.connection.",
];

pub fn runtime_metric_family(name: &str) -> Option<&'static str> {
    RUNTIME_METRIC_PREFIXES
        .iter()
        .find(|prefix| name.starts_with(**prefix))
        .map(|prefix| prefix.trim_end_matches('.'))
}

pub fn runtime_metric_unit(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let unit = if lower.ends_with("_bytes")
        || lower.ends_with(".bytes")
        || lower.contains(".memory.")
        || lower.contains("_memory_")
    {
        "bytes"
    } else if lower.ends_with("_ms") || lower.ends_with(".ms") {
        "ms"
    } else if lower.contains("cpu.utilization") || lower.contains("cpu_usage") {
        "ratio"
    } else {
        return None;
    };
    Some(unit.to_string())
}

pub fn metric_group_label_allowed(label: &str) -> bool {
    let lower = label.trim().to_ascii_lowercase();
    if lower.is_empty() || lower.len() > 128 {
        return false;
    }
    let compact = lower.replace(['.', '-'], "_");
    let leaf = lower.rsplit('.').next().unwrap_or(lower.as_str());
    let leaf_compact = leaf.replace('-', "_");
    !matches!(
        lower.as_str(),
        "trace.id" | "run.id" | "user.id" | "session.id"
    ) && !matches!(
        compact.as_str(),
        "trace_id" | "run_id" | "user_id" | "session_id"
    ) && !matches!(
        leaf_compact.as_str(),
        "trace_id" | "run_id" | "user_id" | "session_id"
    )
}

pub fn field_key_namespace(key: &str) -> String {
    let logical = key.strip_prefix("resource.").unwrap_or(key);
    logical
        .split('.')
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or("custom")
        .to_string()
}

pub fn field_key_identifier_like(key: &str) -> bool {
    let logical = key.strip_prefix("resource.").unwrap_or(key);
    let lower = logical.to_ascii_lowercase();
    let compact = lower.replace(['.', '-'], "_");
    let leaf = lower.rsplit('.').next().unwrap_or(lower.as_str());
    let leaf_compact = leaf.replace('-', "_");

    matches!(
        leaf_compact.as_str(),
        "id" | "trace_id" | "span_id" | "run_id" | "user_id" | "session_id" | "enduser_id"
    ) || lower.ends_with(".id")
        || lower.ends_with("_id")
        || compact.contains("trace_id")
        || compact.contains("span_id")
        || compact.contains("run_id")
        || compact.contains("user_id")
        || compact.contains("session_id")
        || lower.contains("uuid")
        || lower.contains("guid")
        || lower.contains("fingerprint")
        || lower.contains("hash")
}
