//! Structured where-clause compilation (plan 164).
//!
//! The UI parses `service = "checkout" AND attr.http.route != "/health"`
//! into typed `AttributeFilter`s; this module is the single compiler from
//! those filters to SQL conditions over the native span table. Values are
//! never concatenated raw: string literals go through `escape` (and LIKE
//! wildcard escaping), numeric literals are re-serialized from the parsed
//! number, and identifiers are vetted by `span_field_key_allowed` before
//! being quoted — a disallowed key compiles to `1 = 0` (narrow, never
//! silently vanish).

use super::*;
use crate::adapter::{AttributeFilter, AttributeFilterOp};

/// Intrinsic key → raw span-table column; everything else is a
/// `span_attributes.<key>` auto-widened column.
fn intrinsic_column(key: &str) -> Option<&'static str> {
    match key {
        "service.name" | "service" => Some(r#""service_name""#),
        "name" | "span.name" => Some(r#""span_name""#),
        "kind" | "span.kind" => Some(r#""span_kind""#),
        "status" | "span.status" => Some(r#""span_status_code""#),
        "duration_ns" | "duration" => Some(r#""duration_nano""#),
        "trace_id" => Some(r#""trace_id""#),
        "span_id" => Some(r#""span_id""#),
        _ => None,
    }
}

fn is_numeric_intrinsic(key: &str) -> bool {
    matches!(key, "duration_ns" | "duration")
}

pub(super) fn string_expr(key: &str) -> Option<String> {
    if let Some(column) = intrinsic_column(key) {
        return Some(column.to_string());
    }
    span_field_key_allowed(key).then(|| format!("CAST({} AS STRING)", span_attr_ident(key)))
}

fn numeric_expr(key: &str) -> Option<String> {
    if let Some(column) = intrinsic_column(key) {
        return is_numeric_intrinsic(key).then(|| format!("CAST({column} AS DOUBLE)"));
    }
    span_field_key_allowed(key).then(|| format!("CAST({} AS DOUBLE)", span_attr_ident(key)))
}

fn like_pattern(value: &str) -> String {
    escape(value).replace('%', r"\%").replace('_', r"\_")
}

const NO_MATCH: &str = "1 = 0";

/// Table-specific expressions for one where-clause key.
struct FilterExprs {
    string: Option<String>,
    numeric: Option<String>,
    /// A numeric-only intrinsic (e.g. duration) rejects non-numeric literals.
    numeric_only: bool,
}

fn span_filter_exprs(key: &str) -> FilterExprs {
    FilterExprs {
        string: string_expr(key),
        numeric: numeric_expr(key),
        numeric_only: is_numeric_intrinsic(key),
    }
}

/// Log-table expressions (plan 164): intrinsics map to native log columns;
/// other keys read the `log_attributes` JSON.
fn log_filter_exprs(key: &str) -> FilterExprs {
    let string = match key {
        "service.name" | "service" => Some(log_service_name_expr()),
        "severity" | "severity_text" => Some(r#""severity_text""#.to_string()),
        "severity_number" => Some(r#"CAST("severity_number" AS STRING)"#.to_string()),
        "body" => Some(r#""body""#.to_string()),
        "trace_id" => Some(r#""trace_id""#.to_string()),
        "span_id" => Some(r#""span_id""#.to_string()),
        key => span_field_key_allowed(key).then(|| {
            format!(
                r#"json_get_string("log_attributes", '{}')"#,
                semconv::resource_json_path(key)
            )
        }),
    };
    let numeric = match key {
        "severity_number" => Some(r#""severity_number""#.to_string()),
        "severity" | "severity_text" | "body" | "service" | "service.name" | "trace_id"
        | "span_id" => None,
        key => span_field_key_allowed(key).then(|| {
            format!(
                r#"CAST(json_get_string("log_attributes", '{}') AS DOUBLE)"#,
                semconv::resource_json_path(key)
            )
        }),
    };
    FilterExprs {
        string,
        numeric,
        numeric_only: false,
    }
}

/// One filter → one SQL condition against the raw span scan. Absent values
/// satisfy only the negative operators, matching `AttributeFilter::matches`.
pub(super) fn span_attribute_filter_sql(filter: &AttributeFilter) -> String {
    filter_condition(&span_filter_exprs(filter.key.trim()), filter)
}

/// One filter → one SQL condition against the raw log scan.
pub(super) fn log_attribute_filter_sql(filter: &AttributeFilter) -> String {
    filter_condition(&log_filter_exprs(filter.key.trim()), filter)
}

fn filter_condition(exprs: &FilterExprs, filter: &AttributeFilter) -> String {
    let value = filter.value.as_str();
    let string_expr = || exprs.string.clone();
    let numeric_expr = || exprs.numeric.clone();
    match filter.op {
        AttributeFilterOp::Eq => match string_expr() {
            Some(expr) => format!("{expr} = '{}'", escape(value)),
            None => NO_MATCH.to_string(),
        },
        AttributeFilterOp::Ne => match string_expr() {
            Some(expr) => {
                format!("({expr} IS NULL OR {expr} != '{}')", escape(value))
            }
            None => NO_MATCH.to_string(),
        },
        AttributeFilterOp::Contains => match string_expr() {
            Some(expr) => {
                format!(r#"{expr} LIKE '%{}%' ESCAPE '\'"#, like_pattern(value))
            }
            None => NO_MATCH.to_string(),
        },
        AttributeFilterOp::NotContains => match string_expr() {
            Some(expr) => format!(
                r#"({expr} IS NULL OR {expr} NOT LIKE '%{}%' ESCAPE '\')"#,
                like_pattern(value)
            ),
            None => NO_MATCH.to_string(),
        },
        AttributeFilterOp::Gt
        | AttributeFilterOp::Lt
        | AttributeFilterOp::Gte
        | AttributeFilterOp::Lte => {
            let op = match filter.op {
                AttributeFilterOp::Gt => ">",
                AttributeFilterOp::Lt => "<",
                AttributeFilterOp::Gte => ">=",
                AttributeFilterOp::Lte => "<=",
                _ => unreachable!(),
            };
            // Numeric when the literal parses as a number (re-serialized
            // from the parsed value — the raw string never reaches SQL);
            // otherwise lexicographic, mirroring AttributeFilter::matches.
            if let Ok(number) = value.trim().parse::<f64>() {
                if !number.is_finite() {
                    return NO_MATCH.to_string();
                }
                match numeric_expr() {
                    Some(expr) => format!("{expr} {op} {number}"),
                    None => NO_MATCH.to_string(),
                }
            } else if exprs.numeric_only {
                NO_MATCH.to_string()
            } else {
                match string_expr() {
                    Some(expr) => format!("{expr} {op} '{}'", escape(value)),
                    None => NO_MATCH.to_string(),
                }
            }
        }
    }
}

/// All filters ANDed against the same span row ("one span satisfies ALL
/// filters"), or None when the list is empty.
pub(super) fn span_attribute_filters_sql(filters: &[AttributeFilter]) -> Option<String> {
    if filters.is_empty() {
        return None;
    }
    Some(
        filters
            .iter()
            .map(|filter| format!("({})", span_attribute_filter_sql(filter)))
            .collect::<Vec<_>>()
            .join(" AND "),
    )
}

/// Native per-metric-table expressions (plan 168): metric labels are
/// promoted tag columns; `service`/`service.name` maps to `service_name`.
/// Keys are vetted by `metric_group_label_allowed` before quoting.
fn metric_filter_exprs(key: &str) -> FilterExprs {
    let column = match key {
        "service.name" | "service" => Some(r#""service_name""#.to_string()),
        key => metric_group_label_allowed(key).then(|| format!(r#""{}""#, escape_ident(key))),
    };
    FilterExprs {
        string: column.as_ref().map(|c| format!("CAST({c} AS STRING)")),
        numeric: column.map(|c| format!("CAST({c} AS DOUBLE)")),
        numeric_only: false,
    }
}

/// One filter -> one SQL condition against a native per-metric table.
pub(super) fn metric_attribute_filter_sql(filter: &AttributeFilter) -> String {
    filter_condition(&metric_filter_exprs(filter.key.trim()), filter)
}

/// All filters ANDed against the same metric row, or None when empty.
pub(super) fn metric_attribute_filters_sql(filters: &[AttributeFilter]) -> Option<String> {
    if filters.is_empty() {
        return None;
    }
    Some(
        filters
            .iter()
            .map(|filter| format!("({})", metric_attribute_filter_sql(filter)))
            .collect::<Vec<_>>()
            .join(" AND "),
    )
}

/// Log-table string expression for a facet dimension / where-clause key.
pub(super) fn log_string_expr(key: &str) -> Option<String> {
    log_filter_exprs(key).string
}

/// All filters ANDed against the same log row, or None when empty.
pub(super) fn log_attribute_filters_sql(filters: &[AttributeFilter]) -> Option<String> {
    if filters.is_empty() {
        return None;
    }
    Some(
        filters
            .iter()
            .map(|filter| format!("({})", log_attribute_filter_sql(filter)))
            .collect::<Vec<_>>()
            .join(" AND "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(key: &str, op: AttributeFilterOp, value: &str) -> AttributeFilter {
        AttributeFilter {
            key: key.to_string(),
            op,
            value: value.to_string(),
        }
    }

    #[test]
    fn intrinsics_compile_to_raw_columns() {
        assert_eq!(
            span_attribute_filter_sql(&filter("service.name", AttributeFilterOp::Eq, "checkout")),
            r#""service_name" = 'checkout'"#
        );
        assert_eq!(
            span_attribute_filter_sql(&filter(
                "status",
                AttributeFilterOp::Eq,
                "STATUS_CODE_ERROR"
            )),
            r#""span_status_code" = 'STATUS_CODE_ERROR'"#
        );
    }

    #[test]
    fn attribute_keys_compile_to_span_attribute_columns() {
        assert_eq!(
            span_attribute_filter_sql(&filter(
                "http.request.method",
                AttributeFilterOp::Eq,
                "POST"
            )),
            r#"CAST("span_attributes.http.request.method" AS STRING) = 'POST'"#
        );
    }

    #[test]
    fn values_are_escaped_never_concatenated() {
        let sql =
            span_attribute_filter_sql(&filter("http.route", AttributeFilterOp::Eq, "x' OR 1=1--"));
        // The quote is doubled: the whole payload stays one string literal.
        assert_eq!(
            sql,
            r#"CAST("span_attributes.http.route" AS STRING) = 'x'' OR 1=1--'"#
        );
    }

    #[test]
    fn contains_escapes_like_wildcards() {
        let sql = span_attribute_filter_sql(&filter(
            "http.route",
            AttributeFilterOp::Contains,
            "100%_done",
        ));
        assert_eq!(
            sql,
            r#"CAST("span_attributes.http.route" AS STRING) LIKE '%100\%\_done%' ESCAPE '\'"#
        );
    }

    #[test]
    fn negative_operators_match_absent_values() {
        let sql =
            span_attribute_filter_sql(&filter("http.route", AttributeFilterOp::Ne, "/health"));
        assert_eq!(
            sql,
            r#"(CAST("span_attributes.http.route" AS STRING) IS NULL OR CAST("span_attributes.http.route" AS STRING) != '/health')"#
        );
    }

    #[test]
    fn numeric_ordering_reserializes_the_parsed_number() {
        assert_eq!(
            span_attribute_filter_sql(&filter("duration_ns", AttributeFilterOp::Gte, " 1500 ")),
            r#"CAST("duration_nano" AS DOUBLE) >= 1500"#
        );
        assert_eq!(
            span_attribute_filter_sql(&filter("http.status_code", AttributeFilterOp::Gt, "499")),
            r#"CAST("span_attributes.http.status_code" AS DOUBLE) > 499"#
        );
    }

    #[test]
    fn string_ordering_when_value_is_not_numeric() {
        assert_eq!(
            span_attribute_filter_sql(&filter("http.route", AttributeFilterOp::Gt, "/a")),
            r#"CAST("span_attributes.http.route" AS STRING) > '/a'"#
        );
        // Numeric-only intrinsics reject non-numeric literals.
        assert_eq!(
            span_attribute_filter_sql(&filter("duration_ns", AttributeFilterOp::Gt, "fast")),
            "1 = 0"
        );
    }

    #[test]
    fn disallowed_keys_narrow_to_no_rows() {
        for key in ["", "api_token", "weird\"key", "x".repeat(200).as_str()] {
            assert_eq!(
                span_attribute_filter_sql(&filter(key, AttributeFilterOp::Eq, "v")),
                "1 = 0",
                "key {key:?} must not compile"
            );
        }
    }

    #[test]
    fn filters_join_with_and() {
        let joined = span_attribute_filters_sql(&[
            filter("service.name", AttributeFilterOp::Eq, "checkout"),
            filter("http.request.method", AttributeFilterOp::Eq, "POST"),
        ])
        .unwrap();
        assert_eq!(
            joined,
            r#"("service_name" = 'checkout') AND (CAST("span_attributes.http.request.method" AS STRING) = 'POST')"#
        );
        assert!(span_attribute_filters_sql(&[]).is_none());
    }

    #[test]
    fn log_intrinsics_compile_to_log_columns() {
        assert!(
            log_attribute_filter_sql(&filter("service", AttributeFilterOp::Eq, "checkout"))
                .contains(r#"COALESCE("service.name""#)
        );
        assert_eq!(
            log_attribute_filter_sql(&filter("severity_number", AttributeFilterOp::Gte, "13")),
            r#""severity_number" >= 13"#
        );
        assert_eq!(
            log_attribute_filter_sql(&filter("body", AttributeFilterOp::Contains, "timeout")),
            r#""body" LIKE '%timeout%' ESCAPE '\'"#
        );
    }

    #[test]
    fn log_attribute_keys_read_the_json_column_escaped() {
        assert_eq!(
            log_attribute_filter_sql(&filter("http.route", AttributeFilterOp::Eq, "x' OR 1=1--")),
            r#"json_get_string("log_attributes", '$."http.route"') = 'x'' OR 1=1--'"#
        );
        assert_eq!(
            log_attribute_filter_sql(&filter("api_token", AttributeFilterOp::Eq, "v")),
            "1 = 0"
        );
    }

    #[test]
    fn log_filters_join_with_and() {
        let joined = log_attribute_filters_sql(&[
            filter("severity_number", AttributeFilterOp::Gte, "13"),
            filter("http.route", AttributeFilterOp::Ne, "/health"),
        ])
        .unwrap();
        assert!(joined.starts_with(r#"("severity_number" >= 13) AND ("#));
        assert!(log_attribute_filters_sql(&[]).is_none());
    }
}
