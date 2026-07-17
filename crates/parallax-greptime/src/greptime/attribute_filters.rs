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

fn string_expr(key: &str) -> Option<String> {
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

/// One filter → one SQL condition against the raw span scan. Absent values
/// satisfy only the negative operators, matching `AttributeFilter::matches`.
pub(super) fn span_attribute_filter_sql(filter: &AttributeFilter) -> String {
    let key = filter.key.trim();
    let value = filter.value.as_str();
    match filter.op {
        AttributeFilterOp::Eq => match string_expr(key) {
            Some(expr) => format!("{expr} = '{}'", escape(value)),
            None => NO_MATCH.to_string(),
        },
        AttributeFilterOp::Ne => match string_expr(key) {
            Some(expr) => {
                format!("({expr} IS NULL OR {expr} != '{}')", escape(value))
            }
            None => NO_MATCH.to_string(),
        },
        AttributeFilterOp::Contains => match string_expr(key) {
            Some(expr) => {
                format!(r#"{expr} LIKE '%{}%' ESCAPE '\'"#, like_pattern(value))
            }
            None => NO_MATCH.to_string(),
        },
        AttributeFilterOp::NotContains => match string_expr(key) {
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
                match numeric_expr(key) {
                    Some(expr) => format!("{expr} {op} {number}"),
                    None => NO_MATCH.to_string(),
                }
            } else if is_numeric_intrinsic(key) {
                NO_MATCH.to_string()
            } else {
                match string_expr(key) {
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
}
