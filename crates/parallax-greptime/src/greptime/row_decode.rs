use super::*;

/// A row → `LogRow` projection for the fixed native log column order used by
/// [`GreptimeStore::select_logs`] and `logs_search`.
pub(super) fn log_row_from_row(row: &[serde_json::Value]) -> LogRow {
    LogRow {
        ts_nanos: u128_at(row, 0),
        service: str_at(row, 1),
        severity_num: row.get(2).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        severity_text: str_at(row, 3),
        body: str_at(row, 4),
        trace_id: str_at(row, 5),
        span_id: str_at(row, 6),
        invocation_id: opt_str_at(row, 7),
        session_id: opt_str_at(row, 8),
        scope_name: str_at(row, 9),
        attributes: json_at(row, 10),
        resource: json_at(row, 11),
        event_name: str_at(row, 12),
        observed_ts_nanos: u128_at(row, 13),
    }
}

/// Maps native result-column names to their position in a row, so a `SELECT *`
/// (whose schema auto-widens with new attribute keys) can be read by name and
/// the `span_attributes.*` / `resource_attributes.*` columns folded back into
/// the `attributes` / `resource` JSON objects the model carries.
pub(super) struct ColumnIndex<'a> {
    columns: &'a [String],
    by_name: HashMap<&'a str, usize>,
}

impl<'a> ColumnIndex<'a> {
    pub(super) fn new(columns: &'a [String]) -> Self {
        let by_name = columns
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();
        Self { columns, by_name }
    }

    pub(super) fn idx(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }

    pub(super) fn string(&self, name: &str, row: &[serde_json::Value]) -> String {
        self.idx(name).map(|i| str_at(row, i)).unwrap_or_default()
    }

    pub(super) fn opt_string(&self, name: &str, row: &[serde_json::Value]) -> Option<String> {
        self.idx(name)
            .and_then(|i| opt_str_at(row, i))
            .filter(|s| !s.is_empty())
    }

    pub(super) fn u128(&self, name: &str, row: &[serde_json::Value]) -> u128 {
        self.idx(name).map(|i| u128_at(row, i)).unwrap_or(0)
    }

    pub(super) fn json(&self, name: &str, row: &[serde_json::Value]) -> serde_json::Value {
        self.idx(name)
            .map(|i| json_at(row, i))
            .unwrap_or(serde_json::Value::Null)
    }

    /// Fold the flattened native attribute columns back into two JSON maps:
    /// `span_attributes.<k>` → attributes, `resource_attributes.<k>` → resource
    /// (the dotted prefix stripped). Non-null scalar values only.
    pub(super) fn reassemble_attrs(
        &self,
        row: &[serde_json::Value],
    ) -> (serde_json::Value, serde_json::Value) {
        let mut attributes = serde_json::Map::new();
        let mut resource = serde_json::Map::new();
        for (i, name) in self.columns.iter().enumerate() {
            let Some(value) = row.get(i) else { continue };
            if value.is_null() {
                continue;
            }
            if let Some(key) = name.strip_prefix("span_attributes.") {
                attributes.insert(key.to_string(), value.clone());
            } else if let Some(key) = name.strip_prefix("resource_attributes.") {
                resource.insert(key.to_string(), value.clone());
            }
        }
        (
            serde_json::Value::Object(attributes),
            serde_json::Value::Object(resource),
        )
    }
}

#[derive(Debug, Clone)]
pub(super) struct SpanFieldColumn {
    pub(super) key: String,
    pub(super) column: String,
    pub(super) source: FieldSource,
}

pub(super) fn span_field_column_from_name(column: &str) -> Option<SpanFieldColumn> {
    if let Some(key) = column.strip_prefix("span_attributes.") {
        let key = key.to_string();
        return span_field_key_allowed(&key).then_some(SpanFieldColumn {
            key,
            column: column.to_string(),
            source: FieldSource::Span,
        });
    }
    if let Some(key) = column.strip_prefix("resource_attributes.") {
        let key = format!("resource.{key}");
        return span_field_key_allowed(&key).then_some(SpanFieldColumn {
            key,
            column: column.to_string(),
            source: FieldSource::Resource,
        });
    }
    None
}

pub(super) fn quoted_field_column(column: &SpanFieldColumn) -> String {
    format!(r#""{}""#, escape_ident(&column.column))
}

pub(super) fn str_at(row: &[serde_json::Value], index: usize) -> String {
    row.get(index)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

pub(super) fn opt_str_at(row: &[serde_json::Value], index: usize) -> Option<String> {
    row.get(index).and_then(|v| v.as_str()).map(str::to_string)
}

pub(super) fn opt_nonempty_str_at(row: &[serde_json::Value], index: usize) -> Option<String> {
    opt_str_at(row, index).filter(|value| !value.trim().is_empty())
}

/// Clamp a u128 time bound to what the engine's TIMESTAMP cast accepts
/// (i64); open-ended `..=u128::MAX` ranges otherwise fail query planning
/// ("Casting value to Timestamp is invalid").
pub(super) fn sql_ts(bound: u128) -> i64 {
    i64::try_from(bound).unwrap_or(i64::MAX)
}

/// Shared WHERE clauses for `logs_search` and `log_count_series`.
///
/// Body search uses `matches_term` (FULLTEXT bloom): term match, not substring;
/// whitespace tokens AND-combined; double-quoted phrase; punctuation → LIKE.
/// Memory adapter stays substring (Plan 084 intentional divergence).
pub(super) fn log_filter_clauses(
    service: Option<&str>,
    range: &RangeInclusive<u128>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    body_contains: Option<&str>,
) -> Vec<String> {
    let mut clauses = vec![format!(
        r#""timestamp" >= {} AND "timestamp" <= {}"#,
        sql_ts(*range.start()),
        sql_ts(*range.end())
    )];
    if let Some(service) = service {
        clauses.push(format!(
            r#"{} = '{}'"#,
            log_service_name_expr(),
            escape(service)
        ));
    }
    if let Some(min) = severity_min {
        clauses.push(format!(r#""severity_number" >= {min}"#));
    }
    if let Some(max) = severity_max {
        clauses.push(format!(r#""severity_number" <= {max}"#));
    }
    if let Some(needle) = body_contains {
        push_body_search_clause(&mut clauses, needle);
    }
    clauses
}

pub(super) fn push_body_search_clause(clauses: &mut Vec<String>, needle: &str) {
    let needle = needle.trim();
    if needle.is_empty() {
        return;
    }
    if needle.len() >= 2 && needle.starts_with('"') && needle.ends_with('"') {
        let phrase = &needle[1..needle.len() - 1];
        clauses.push(format!(r#"matches_term("body", '{}')"#, escape(phrase)));
        return;
    }
    if !needle.chars().any(|c| c.is_alphanumeric()) {
        let escaped = escape(
            &needle
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_"),
        );
        clauses.push(format!(r#""body" LIKE '%{escaped}%' ESCAPE '\'"#));
        return;
    }
    for token in needle.split_whitespace() {
        if !token.is_empty() {
            clauses.push(format!(r#"matches_term("body", '{}')"#, escape(token)));
        }
    }
}

pub(super) fn u128_at(row: &[serde_json::Value], index: usize) -> u128 {
    let Some(value) = row.get(index) else {
        return 0;
    };
    if let Some(n) = value.as_u64() {
        return u128::from(n);
    }
    if let Some(n) = value.as_i64()
        && n >= 0
    {
        return u128::try_from(n).unwrap_or(0);
    }
    if let Some(s) = value.as_str()
        && let Ok(n) = s.parse::<u128>()
    {
        tracing::warn!(
            target: "parallax_greptime",
            index,
            "u128_at decoded JSON string timestamp; prefer integer wire encoding"
        );
        return n;
    }
    if let Some(f) = value.as_f64()
        && f.is_finite()
        && f >= 0.0
    {
        tracing::warn!(
            target: "parallax_greptime",
            index,
            "u128_at decoded JSON float timestamp; prefer integer wire encoding"
        );
        return f.max(0.0) as u128;
    }
    0
}

pub(super) fn absorb_observed_run(
    runs: &mut HashMap<String, crate::adapter::ObservedInvocation>,
    row: &[serde_json::Value],
    is_span: bool,
) -> Option<String> {
    let invocation_id = str_at(row, 0);
    if invocation_id.is_empty() {
        return None;
    }
    let first = u128_at(row, 1);
    let last = u128_at(row, 2);
    let count = u128_at(row, 3) as u64;
    let entry = runs
        .entry(invocation_id.clone())
        .or_insert_with(|| crate::adapter::ObservedInvocation {
            invocation_id: invocation_id.clone(),
            first_nanos: first,
            last_nanos: last,
            span_count: 0,
            log_count: 0,
            service: str_at(row, 4),
            last_command: None,
            app_mode: None,
        });
    entry.first_nanos = entry.first_nanos.min(first);
    entry.last_nanos = entry.last_nanos.max(last);
    if entry.last_command.is_none() {
        entry.last_command = opt_nonempty_str_at(row, 5);
    }
    if entry.app_mode.is_none() {
        entry.app_mode = opt_nonempty_str_at(row, 6);
    }
    if is_span {
        entry.span_count += count;
    } else {
        entry.log_count += count;
    }
    Some(invocation_id)
}

pub(super) fn f64_at(row: &[serde_json::Value], index: usize) -> f64 {
    row.get(index).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

pub(super) fn trace_filter_clauses(
    service: Option<&str>,
    range: &RangeInclusive<u128>,
) -> Vec<String> {
    let mut clauses = vec![format!(
        r#""timestamp" >= {} AND "timestamp" <= {}"#,
        sql_ts(*range.start()),
        sql_ts(*range.end())
    )];
    if let Some(service) = service {
        clauses.push(format!(r#""service_name" = '{}'"#, escape(service)));
    }
    clauses
}

pub(super) fn json_at(row: &[serde_json::Value], index: usize) -> serde_json::Value {
    match row.get(index) {
        Some(serde_json::Value::String(s)) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
        }
        Some(other) => other.clone(),
        None => serde_json::Value::Null,
    }
}
