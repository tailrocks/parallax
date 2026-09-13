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
fn exemplar_filter_includes_otel_name_for_prom_catalog_name() {
    let filter = metric_name_sql_filter(r#""name""#, "catalog_product_queries_total");
    assert!(
        filter.contains("'catalog.product.queries'"),
        "expected OTel exemplar name in {filter}"
    );
    assert!(
        filter.contains("'catalog_product_queries_total'"),
        "expected listed name in {filter}"
    );
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
    let clauses = log_filter_clauses(
        Some("checkout"),
        &(0..=1000),
        None,
        None,
        Some("error"),
        &[],
    );
    let joined = clauses.join(" AND ");
    assert!(joined.contains(r#"matches_term("body", 'error')"#));
    assert!(joined.contains(r#"COALESCE("service.name""#));
    assert!(!joined.contains(r#""body" LIKE"#));
}

#[test]
fn golden_traces_search_sql_includes_adversarial_service() {
    // Participation is a materialized literal id list (semi-joins and
    // subquery-to-subquery joins both mis-execute on the live engine).
    let participation = format!(r#" AND "trace_id" IN ('{}')"#, escape("id'quote"));
    let (listed, page) = GreptimeStore::traces_search_sql(
        r#""timestamp" >= 1"#,
        &participation,
        r#""rn" = 1"#,
        r#""ts_nanos" DESC"#,
        50,
        0,
    );
    assert!(listed.contains("id''quote"));
    assert!(listed.contains(r#""timestamp" >= 1"#));
    // Single scan: per-trace stats are window aggregates, never a join —
    // the live engine collapses subquery-to-subquery joins on "trace_id".
    assert!(!listed.contains("JOIN"));
    assert!(listed.contains(r#"COUNT(*) OVER (PARTITION BY "trace_id")"#));
    assert!(listed.contains(r#"OVER (PARTITION BY "trace_id") AS "has_error""#));
    // windowed single-pass total (plan 075)
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
    // Span links/events are Json columns: `*`-selected they cross the arrow
    // wire as raw JSONB (null / integer garbage) — D-005, corpus id t-links.
    assert!(spans.contains(r#"json_to_string("span_links") AS "span_links_json""#));
    assert!(spans.contains(r#"json_to_string("span_events") AS "span_events_json""#));
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
    let sql = GreptimeStore::service_map_edges_sql(&(0..=999), 500);
    assert!(sql.contains("approx_percentile_cont"));
    assert!(sql.contains("GROUP BY"));
    // Whole-window self-join: no most-recent-trace sampling (it silently
    // dropped quiet services' edges behind chatty health-check traces).
    assert!(!sql.contains("IN ("));
    assert!(sql.contains("LIMIT 500"));
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

#[test]
fn external_dependency_sql_requires_a_trigger_column() {
    let range = 0u128..=1_000u128;
    let none: BTreeSet<String> = BTreeSet::new();
    assert!(GreptimeStore::external_dependency_edges_sql(&range, &none, 100).is_none());
    // Non-trigger attribute columns alone still produce no derivation.
    let unrelated: BTreeSet<String> = ["http.route".to_string()].into_iter().collect();
    assert!(GreptimeStore::external_dependency_edges_sql(&range, &unrelated, 100).is_none());
}

#[test]
fn external_dependency_sql_builds_ladder_from_existing_columns_only() {
    let range = 5u128..=99u128;
    let keys: BTreeSet<String> = [
        "db.system.name",
        "db.namespace",
        "messaging.system",
        "messaging.destination.name",
        "server.address",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let sql = GreptimeStore::external_dependency_edges_sql(&range, &keys, 77).unwrap();
    // Anti-join shape: CLIENT/PRODUCER spans with no cross-service
    // SERVER/CONSUMER child in the same trace.
    assert!(sql.contains(r#""client"."span_kind" IN ('SPAN_KIND_CLIENT', 'SPAN_KIND_PRODUCER')"#));
    assert!(sql.contains(r#""child"."span_kind" IN ('SPAN_KIND_SERVER', 'SPAN_KIND_CONSUMER')"#));
    assert!(sql.contains(r#""child"."span_id" IS NULL"#));
    assert!(sql.contains(r#""child"."service_name" != "client"."service_name""#));
    // Ladder columns qualified on the client side; db.system (legacy) and
    // db.name are absent from the schema and must not be referenced.
    assert!(sql.contains(r#""client"."span_attributes.db.system.name""#));
    // Closing-quote match: the absent legacy `db.system` column must not be
    // referenced (`db.system.name` above does not satisfy this substring).
    assert!(!sql.contains(r#""span_attributes.db.system""#));
    assert!(!sql.contains(r#""span_attributes.db.name""#));
    assert!(sql.contains(r#""client"."span_attributes.messaging.destination.name""#));
    assert!(sql.contains(r#""client"."span_attributes.server.address""#));
    assert!(sql.contains("'database'"));
    assert!(sql.contains("'queue'"));
    assert!(sql.contains("'external'"));
    assert!(sql.contains("LIMIT 77"));
}

#[test]
fn external_dependency_sql_supports_partial_schemas() {
    let range = 0u128..=10u128;
    // Only the legacy db.system column exists: database rung still works and
    // the name ladder falls back to the system value.
    let keys: BTreeSet<String> = ["db.system".to_string()].into_iter().collect();
    let sql = GreptimeStore::external_dependency_edges_sql(&range, &keys, 10).unwrap();
    assert!(sql.contains(r#""client"."span_attributes.db.system""#));
    assert!(sql.contains("'database'"));
    // Absent rungs compile to NULL literals, not missing-column references.
    assert!(!sql.contains("messaging"));
    assert!(!sql.contains("server.address"));
    assert!(sql.contains("CAST(NULL AS STRING)"));
}

fn exp_row(ts_nanos: u128, count: u64, sum: f64, attrs: serde_json::Value) -> HistogramRow {
    HistogramRow {
        ts_nanos,
        service: "checkout".to_string(),
        name: "http.duration".to_string(),
        count,
        sum,
        bucket_counts: vec![3, 5],
        bounds: vec![2.0, 4.0],
        attributes: attrs,
    }
}

#[test]
fn exp_histograms_ddl_has_append_ttl_contract() {
    let ddl = GreptimeStore::exp_histograms_ddl(EXP_HISTOGRAMS_TABLE, "30d");
    assert!(ddl.contains(r#""bucket_counts" JSON"#));
    assert!(ddl.contains(r#""bounds" JSON"#));
    assert!(ddl.contains(r#"PRIMARY KEY ("service", "name")"#));
    assert!(ddl.contains("append_mode = 'true'"));
    assert!(ddl.contains("ttl = '30d'"));
}

#[test]
fn exp_histogram_from_row_decodes_buckets() {
    let row = vec![
        serde_json::json!(2_000_000_000u64),
        serde_json::json!("checkout"),
        serde_json::json!("http.duration"),
        serde_json::json!(10u64),
        serde_json::json!(42.0),
        serde_json::json!("[3,5]"),
        serde_json::json!("[2.0,4.0]"),
        serde_json::json!(r#"{"route":"/pay"}"#),
    ];
    let decoded = exp_histogram_from_row(&row);
    assert_eq!(decoded.ts_nanos, 2_000_000_000);
    assert_eq!(decoded.count, 10);
    assert_eq!(decoded.sum, 42.0);
    assert_eq!(decoded.bucket_counts, vec![3, 5]);
    assert_eq!(decoded.bounds, vec![2.0, 4.0]);
    assert_eq!(decoded.attributes, serde_json::json!({"route": "/pay"}));
}

#[test]
fn exp_quantiles_use_latest_export_per_window() {
    let rows = vec![
        exp_row(1_000, 8, 20.0, serde_json::json!({})),
        exp_row(1_500, 8, 20.0, serde_json::json!({})),
        exp_row(61_000_000_000, 16, 40.0, serde_json::json!({})),
    ];
    let series = exp_quantiles_from_rows(&rows, 60_000_000_000, &[0.5]);
    assert_eq!(series.len(), 1);
    // bounds [2,4] counts [3,5]: total 8, p50 target 4 → 2 + 2*(1/5) = 2.4.
    assert_eq!(series[0].len(), 2);
    assert!((series[0][0].value - 2.4).abs() < 1e-9);
    assert!((series[0][1].value - 2.4).abs() < 1e-9);
}

#[test]
fn exp_avg_is_delta_sum_over_delta_count() {
    let rows = vec![
        exp_row(1_000, 8, 20.0, serde_json::json!({})),
        exp_row(61_000_000_000, 16, 44.0, serde_json::json!({})),
    ];
    let series = exp_avg_from_rows(&rows, 60_000_000_000);
    assert_eq!(series.len(), 1);
    assert!((series[0].value - 3.0).abs() < 1e-9);
}

#[test]
fn exp_counts_sum_per_series_deltas() {
    let rows = vec![
        exp_row(1_000, 10, 0.0, serde_json::json!({"route": "/a"})),
        exp_row(2_000, 20, 0.0, serde_json::json!({"route": "/b"})),
        exp_row(61_000_000_000, 15, 0.0, serde_json::json!({"route": "/a"})),
        exp_row(62_000_000_000, 30, 0.0, serde_json::json!({"route": "/b"})),
    ];
    let series = exp_counts_from_rows(&rows, 60_000_000_000);
    // Window stocks 30 → 45; first window omitted (no baseline), delta 15.
    assert_eq!(series.len(), 1);
    assert!((series[0].value - 15.0).abs() < 1e-9);
}

#[test]
fn exp_row_matches_reads_service_and_attributes() {
    use crate::adapter::{AttributeFilter, AttributeFilterOp};
    let attrs = serde_json::json!({"route": "/pay"});
    let eq = AttributeFilter {
        key: "route".to_string(),
        op: AttributeFilterOp::Eq,
        value: "/pay".to_string(),
    };
    assert!(exp_row_matches(&[eq], "checkout", &attrs));
    let svc = AttributeFilter {
        key: "service.name".to_string(),
        op: AttributeFilterOp::Eq,
        value: "other".to_string(),
    };
    assert!(!exp_row_matches(&[svc], "checkout", &attrs));
    let missing = AttributeFilter {
        key: "absent".to_string(),
        op: AttributeFilterOp::Eq,
        value: "x".to_string(),
    };
    assert!(!exp_row_matches(&[missing], "checkout", &attrs));
}

#[test]
fn exp_catalog_and_bucket_arms_match_output_contract() {
    let range = 0u128..=10u128;
    let arm = exp_catalog_arm("http.duration", "http.duration", &range);
    assert!(arm.contains("FROM \"exp_histograms\""));
    assert!(arm.contains("AS \"last_ms\""));
    assert!(arm.contains("COUNT(*) AS \"cnt\""));
    assert!(arm.contains("GROUP BY \"service\""));
    let buckets = exp_point_buckets_arm("http.duration", 60, &range, Some("checkout"));
    assert!(buckets.contains("FROM \"exp_histograms\""));
    assert!(buckets.contains(r#""service" = 'checkout'"#));
    assert!(buckets.contains("AS \"bucket_ms\""));
}
