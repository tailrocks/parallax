//! Real-engine migration coverage for Plan 092.
//!
//! Run with: `cargo nextest run -p parallax-server --test m7_metric_exemplar_migration_greptime --run-ignored only`

use parallax_server::greptime_supervisor::{GreptimeSupervisor, ensure_binary};
use parallax_storage::greptime::GreptimeStore;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn migrates_legacy_metric_exemplars_without_mutation() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("parallax_server=info")
        .try_init();
    let tmp = tempfile::tempdir().expect("tempdir");
    let bin_dir = tmp.path().join("bin");
    if let Some(home) = std::env::home_dir() {
        let cached = home.join(".parallax/bin/greptime");
        if cached.exists() {
            std::fs::create_dir_all(&bin_dir).expect("bin dir");
            std::fs::copy(cached, bin_dir.join("greptime")).expect("seed cached GreptimeDB");
        }
    }
    let binary = ensure_binary(&bin_dir, "1.1.2", true)
        .await
        .expect("resolve GreptimeDB");
    let engine = GreptimeSupervisor::start(binary, tmp.path())
        .await
        .expect("start GreptimeDB");
    let store = GreptimeStore::connect(&engine.http_url, "7d", "7d", "7d")
        .await
        .expect("connect");

    store
        .sql(
            r#"CREATE TABLE metric_exemplars (
                 "ts" TIMESTAMP(9) NOT NULL,
                 "service" STRING, "name" STRING, "value" DOUBLE,
                 "trace_id" STRING, "span_id" STRING,
                 "run_id" STRING SKIPPING INDEX, "attributes" JSON,
                 TIME INDEX ("ts"),
                 PRIMARY KEY ("service", "name", "trace_id", "span_id")
               ) WITH (append_mode = 'true', ttl = '7d')"#,
        )
        .await
        .expect("legacy table");
    store
        .sql(
            r#"INSERT INTO metric_exemplars
               ("ts", "service", "name", "value", "trace_id", "span_id", "run_id", "attributes")
               VALUES
               (1741437296123456789, 'checkout', 'http.duration', 42.5,
                'trace-a', 'span-a', 'run-a', parse_json('{"nested":{"ok":true},"n":42}')),
               (1741437296123456790, 'checkout', 'http.duration', 9.25,
                'trace-b', 'span-b', NULL, parse_json('{"route":"/pay"}'))"#,
        )
        .await
        .expect("legacy rows");

    store.bootstrap("7d", "7d").await.expect("first migration");
    store
        .bootstrap("7d", "7d")
        .await
        .expect("idempotent restart");

    let describe = store
        .sql("DESCRIBE metric_exemplars")
        .await
        .expect("describe");
    let primary_tags = describe
        .iter()
        .filter(|row| row.get(2).and_then(serde_json::Value::as_str) == Some("PRI"))
        .filter(|row| row.get(5).and_then(serde_json::Value::as_str) == Some("TAG"))
        .filter_map(|row| row.first().and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(primary_tags, ["service", "name"]);

    let rows = store
        .sql(
            r#"SELECT CAST("ts" AS BIGINT) AS "ts_nanos", "trace_id", "span_id", "run_id",
                      json_to_string("attributes")
               FROM metric_exemplars ORDER BY "ts""#,
        )
        .await
        .expect("migrated rows");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], serde_json::json!(1741437296123456789_i64));
    assert_eq!(rows[0][1], "trace-a");
    assert_eq!(rows[0][2], "span-a");
    assert_eq!(rows[0][3], "run-a");
    let first_json: serde_json::Value =
        serde_json::from_str(rows[0][4].as_str().expect("JSON string")).expect("valid JSON");
    assert_eq!(
        first_json,
        serde_json::json!({"nested": {"ok": true}, "n": 42})
    );
    assert!(rows[1][3].is_null());

    let tables = store
        .sql(
            r#"SELECT table_name FROM information_schema.tables
               WHERE table_schema = 'public' AND table_name LIKE 'metric_exemplars%'"#,
        )
        .await
        .expect("migration tables");
    assert_eq!(tables, vec![vec![serde_json::json!("metric_exemplars")]]);
    engine.stop();
}
