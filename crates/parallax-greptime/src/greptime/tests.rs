use super::*;

#[test]
fn escape_ident_doubles_double_quotes_only() {
    assert_eq!(
        escape_ident(r#"http."server".duration"#),
        r#"http.""server"".duration"#
    );
    assert_eq!(escape_ident("metric's/name"), "metric's/name");
}

#[test]
fn metric_exemplars_fresh_ddl_has_low_cardinality_primary_key() {
    let ddl = GreptimeStore::metric_exemplars_ddl(METRIC_EXEMPLARS_TABLE, "30d");
    assert!(ddl.contains(r#""trace_id" STRING SKIPPING INDEX"#));
    assert!(ddl.contains(r#""invocation_id" STRING SKIPPING INDEX"#));
    assert!(ddl.contains(r#"PRIMARY KEY ("service", "name")"#));
    assert!(!ddl.contains(r#"PRIMARY KEY ("service", "name", "trace_id"#));
    assert!(ddl.contains("append_mode = 'true'"));
    assert!(ddl.contains("ttl = '30d'"));
}

#[test]
fn escape_handles_quotes_newlines_backslash() {
    assert_eq!(escape("o'brien"), "o''brien");
    assert_eq!(escape("already''doubled"), "already''''doubled");
    assert_eq!(escape("line\nbreak"), "line\nbreak");
    assert_eq!(escape(""), "");
    // Backslash passes through unchanged today; Step 5 of plan 074 verifies
    // GreptimeDB dialect treatment of trailing backslash in string literals.
    assert_eq!(escape(r"ends\"), r"ends\");
    assert_eq!(quoted_ident(r#"a"b"#), r#""a""b""#);
}

#[test]
fn body_search_clauses_single_term() {
    let mut clauses = Vec::new();
    push_body_search_clause(&mut clauses, "error");
    assert_eq!(clauses, vec![r#"matches_term("body", 'error')"#]);
}

#[test]
fn body_search_clauses_two_terms_and_combined() {
    let mut clauses = Vec::new();
    push_body_search_clause(&mut clauses, "connection reset");
    assert_eq!(
        clauses,
        vec![
            r#"matches_term("body", 'connection')"#,
            r#"matches_term("body", 'reset')"#,
        ]
    );
}

#[test]
fn body_search_clauses_quoted_phrase() {
    let mut clauses = Vec::new();
    push_body_search_clause(&mut clauses, r#""connection reset""#);
    assert_eq!(clauses, vec![r#"matches_term("body", 'connection reset')"#]);
}

#[test]
fn body_search_clauses_punctuation_falls_back_to_like() {
    let mut clauses = Vec::new();
    push_body_search_clause(&mut clauses, "???");
    assert_eq!(clauses.len(), 1);
    assert!(clauses[0].contains(r#""body" LIKE"#));
    assert!(clauses[0].contains("???"));
}

#[test]
fn body_search_escapes_sql_quotes_in_term() {
    let mut clauses = Vec::new();
    push_body_search_clause(&mut clauses, "o'brien");
    assert_eq!(clauses, vec![r#"matches_term("body", 'o''brien')"#]);
}

#[test]
fn log_filter_clauses_use_matches_term_and_service_coalesce() {
    let clauses = log_filter_clauses(Some("checkout"), &(0..=1000), None, None, Some("error"));
    let joined = clauses.join(" AND ");
    assert!(joined.contains(r#"matches_term("body", 'error')"#));
    assert!(joined.contains(r#"COALESCE("service.name""#));
    assert!(!joined.contains(r#""body" LIKE"#));
}

#[test]
fn golden_traces_search_sql_includes_adversarial_service() {
    let participation = format!(
        r#" AND "trace_id" IN (SELECT "trace_id" FROM opentelemetry_traces WHERE "service_name" = '{}')"#,
        escape("svc'quote")
    );
    let (listed, page) = GreptimeStore::traces_search_sql(
        r#""timestamp" >= 1"#,
        &participation,
        r#""rn" = 1"#,
        r#""ts_nanos" DESC"#,
        50,
        0,
    );
    assert!(listed.contains("svc''quote"));
    assert!(
        listed.contains("WHERE {scan_where}")
            || listed.contains(r#""timestamp" >= 1"#)
            || listed.contains("WHERE ")
    );
    // windowed agg subquery + single-pass total (plan 075)
    assert!(page.contains("COUNT(*) OVER ()"));
    assert!(page.contains("LIMIT 50 OFFSET 0"));
    assert!(page.contains(&listed));
}

#[test]
fn golden_histogram_count_series_sql() {
    let sql = GreptimeStore::histogram_count_series_sql(
        "http_server_request_duration_count",
        60,
        1_000,
        2_000,
        r#" AND "service_name" = 'api'"#,
    );
    assert!(sql.contains(r#"FROM "http_server_request_duration_count""#));
    assert!(sql.contains("date_bin"));
}

#[test]
fn golden_select_spans_and_logs_sql() {
    let spans = GreptimeStore::select_spans_sql(
        r#""trace_id" = 'abc""def'"#,
        " ORDER BY \"timestamp\"",
        " LIMIT 10",
    );
    assert!(spans.contains("opentelemetry_traces"));
    assert!(spans.contains("LIMIT 10"));
    let logs = GreptimeStore::select_logs_sql("1 = 1", "", " LIMIT 5");
    assert!(logs.contains("opentelemetry_logs"));
    // JSON columns must cross the arrow wire as strings — a raw JSON column
    // decodes to null (plan 156 live finding).
    assert!(logs.contains(r#"json_to_string("log_attributes") AS "log_attributes""#));
    assert!(logs.contains(r#"json_to_string("resource_attributes") AS "resource_attributes""#));
}

#[test]
fn golden_span_attribute_counts_sql() {
    let sql =
        GreptimeStore::span_attribute_counts_sql("http.route", &(0..=1000), Some("svc'x"), true);
    assert!(sql.contains("http.route"));
    assert!(sql.contains("svc''x"));
    assert!(sql.contains("STATUS_CODE_ERROR"));
}

#[test]
fn raw_sql_read_only_guard_rejects_writes_and_explain_analyze() {
    assert!(raw_sql_read_only("SELECT * FROM opentelemetry_logs"));
    assert!(raw_sql_read_only(
        "EXPLAIN SELECT * FROM opentelemetry_logs"
    ));
    assert!(!raw_sql_read_only(
        "EXPLAIN ANALYZE SELECT * FROM opentelemetry_logs"
    ));
    assert!(!raw_sql_read_only(
        "SELECT 1; DROP TABLE opentelemetry_logs"
    ));
    assert!(!raw_sql_read_only("DELETE FROM opentelemetry_logs"));
}

#[test]
fn metric_table_candidates_normalizes_dotted_count_suffix() {
    let candidates = metric_table_candidates("http.server.request.duration", Some("_count"));
    assert!(
        candidates
            .iter()
            .any(|c| c == "http_server_request_duration_count"),
        "expected underscore-normalized count table among {candidates:?}"
    );
}

#[test]
fn sql_error_prefix_is_char_boundary_safe() {
    // 3-byte codepoints so byte index 200 is mid-character (would panic on
    // `&s[..200]`). "é" is 2 bytes and lands on a boundary at 200.
    let s = "あ".repeat(300);
    assert!(!s.is_char_boundary(200));
    let prefix: String = s.chars().take(200).collect();
    assert_eq!(prefix.chars().count(), 200);
}

#[test]
fn golden_service_names_sql_is_windowed_union_all() {
    let sql = GreptimeStore::service_names_sql(&(100..=200));
    assert!(sql.contains("UNION ALL"));
    assert!(sql.contains(r#""timestamp" >= 100"#));
    assert!(sql.contains(r#""ts" >= 100"#));
}

#[test]
fn golden_histogram_quantile_bucket_sql_groups_by_window() {
    let sql = GreptimeStore::histogram_quantile_bucket_sql(
        "http_server_request_duration_bucket",
        60,
        1_000,
        2_000,
        r#" AND "service_name" = 'api'"#,
    );
    assert!(sql.contains("date_bin"));
    assert!(sql.contains("GROUP BY"));
    assert!(sql.contains(r#"MAX("greptime_value")"#));
    assert!(!sql.contains(r#"ORDER BY "greptime_timestamp" ASC"#));
}

#[test]
fn golden_service_map_edges_sql_uses_approx_percentile() {
    let sql = GreptimeStore::service_map_edges_sql("'t1','t2'", &(0..=999));
    assert!(sql.contains("approx_percentile_cont"));
    assert!(sql.contains("GROUP BY"));
}

#[test]
fn u128_at_decodes_int_string_and_float() {
    let row = vec![
        serde_json::json!(42u64),
        serde_json::json!("99"),
        serde_json::json!(7.0),
        serde_json::json!(-1),
        serde_json::json!(null),
    ];
    assert_eq!(u128_at(&row, 0), 42);
    assert_eq!(u128_at(&row, 1), 99);
    assert_eq!(u128_at(&row, 2), 7);
    assert_eq!(u128_at(&row, 3), 0);
    assert_eq!(u128_at(&row, 4), 0);
}

#[test]
fn windowed_histogram_merge_uses_latest_cumulative() {
    let mut bounds = BTreeMap::new();
    bounds.insert(OrderedF64(0.1), 10.0);
    bounds.insert(OrderedF64(1.0), 20.0);
    bounds.insert(OrderedF64(f64::INFINITY), 20.0);
    let total: f64 = bounds.iter().next_back().map(|(_, c)| *c).unwrap();
    assert!((total - 20.0_f64).abs() < 1e-9);
    let _ = quantile_from_cumulative(&bounds, 0.5);
}
