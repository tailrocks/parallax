#![expect(clippy::too_many_lines, reason = "measured real-engine scenario")]

//! Real-engine migration coverage for Plan 092.
//!
//! Run with: `cargo nextest run -p parallax-server --test m7_metric_exemplar_migration_greptime --run-ignored only`

#![allow(clippy::panic, reason = "test helpers fail fast with engine context")]

use parallax_greptime::GreptimeStore;
use parallax_server::greptime_supervisor::{GreptimeSupervisor, ensure_binary};
use parallax_storage::adapter::{IngestStore, MetricAnalyticsStore};
use parallax_storage::model::MetricExemplarRow;

const LEGACY_DDL: &str = r#"CREATE TABLE {table} (
    "ts" TIMESTAMP(9) NOT NULL,
    "service" STRING, "name" STRING, "value" DOUBLE,
    "trace_id" STRING, "span_id" STRING,
    "run_id" STRING SKIPPING INDEX, "attributes" JSON,
    TIME INDEX ("ts"),
    PRIMARY KEY ("service", "name", "trace_id", "span_id")
) WITH (append_mode = 'true', ttl = '7d')"#;

const CURRENT_DDL: &str = r#"CREATE TABLE {table} (
    "ts" TIMESTAMP(9) NOT NULL,
    "service" STRING, "name" STRING, "value" DOUBLE,
    "trace_id" STRING SKIPPING INDEX, "span_id" STRING,
    "run_id" STRING SKIPPING INDEX, "attributes" JSON,
    TIME INDEX ("ts"), PRIMARY KEY ("service", "name")
) WITH (append_mode = 'true', ttl = '7d')"#;

const ROWS: &str = r#"(1741437296123456789, 'checkout', 'http.duration', 42.5,
    'trace-a', 'span-a', 'run-a', parse_json('{"nested":{"ok":true},"n":42}')),
    (1741437296123456790, 'checkout', 'http.duration', 9.25,
    'trace-b', 'span-b', NULL, parse_json('{"route":"/pay"}'))"#;

async fn create_table(store: &GreptimeStore, ddl: &str, table: &str) {
    store
        .sql(&ddl.replace("{table}", table))
        .await
        .unwrap_or_else(|error| panic!("create {table}: {error:#}"));
}

async fn insert_rows(store: &GreptimeStore, table: &str, rows: &str) {
    store
        .sql(&format!(
            r#"INSERT INTO {table}
               ("ts", "service", "name", "value", "trace_id", "span_id", "run_id", "attributes")
               VALUES {rows}"#
        ))
        .await
        .unwrap_or_else(|error| panic!("insert {table}: {error:#}"));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn migrates_legacy_metric_exemplars_without_mutation() {
    let _subscriber_already_installed = tracing_subscriber::fmt()
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

    create_table(&store, LEGACY_DDL, "metric_exemplars").await;
    insert_rows(&store, "metric_exemplars", ROWS).await;

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
    let show_create = store
        .sql("SHOW CREATE TABLE metric_exemplars")
        .await
        .expect("show create")
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str().map(str::to_owned))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(show_create.contains("trace_id"));
    assert!(show_create.contains("run_id"));
    assert!(show_create.matches("SKIPPING INDEX").count() >= 2);
    assert!(show_create.contains("append_mode = 'true'"));
    assert!(show_create.contains("ttl = '7days'"), "{show_create}");

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
            r"SELECT table_name FROM information_schema.tables
               WHERE table_schema = 'public' AND table_name LIKE 'metric_exemplars%'",
        )
        .await
        .expect("migration tables");
    assert_eq!(tables, vec![vec![serde_json::json!("metric_exemplars")]]);

    // Restart after replacement creation/partial copy/failed verification:
    // the legacy canonical remains authoritative, so bootstrap discards and
    // rebuilds the partial replacement.
    store
        .sql("DROP TABLE metric_exemplars")
        .await
        .expect("reset canonical");
    create_table(&store, LEGACY_DDL, "metric_exemplars").await;
    insert_rows(&store, "metric_exemplars", ROWS).await;
    create_table(&store, CURRENT_DDL, "metric_exemplars_v2").await;
    insert_rows(
        &store,
        "metric_exemplars_v2",
        &ROWS[..=ROWS.find("),\n").expect("row split")],
    )
    .await;
    store
        .bootstrap("7d", "7d")
        .await
        .expect("resume partial copy");
    assert_eq!(
        store
            .sql("SELECT COUNT(*) FROM metric_exemplars")
            .await
            .expect("count migrated exemplars")[0][0],
        2
    );

    // Restart after the first rename: canonical is absent, the verified old
    // source is retained as legacy, and a replacement may exist.
    create_table(&store, LEGACY_DDL, "metric_exemplars_v1_legacy").await;
    insert_rows(&store, "metric_exemplars_v1_legacy", ROWS).await;
    store
        .sql("DROP TABLE metric_exemplars")
        .await
        .expect("simulate first rename");
    create_table(&store, CURRENT_DDL, "metric_exemplars_v2").await;
    insert_rows(&store, "metric_exemplars_v2", ROWS).await;
    store
        .bootstrap("7d", "7d")
        .await
        .expect("resume first rename");

    // Restart after the second rename/before cleanup: corrected canonical and
    // retained legacy must be compared again before legacy deletion.
    create_table(&store, LEGACY_DDL, "metric_exemplars_v1_legacy").await;
    insert_rows(&store, "metric_exemplars_v1_legacy", ROWS).await;
    store.bootstrap("7d", "7d").await.expect("resume cleanup");
    let final_tables = store
        .sql(
            r"SELECT table_name FROM information_schema.tables
               WHERE table_schema = 'public' AND table_name LIKE 'metric_exemplars%'",
        )
        .await
        .expect("final migration tables");
    assert_eq!(
        final_tables,
        vec![vec![serde_json::json!("metric_exemplars")]]
    );

    // The production ingest and query surfaces continue to use the canonical
    // name after migration. An empty protobuf message is a valid empty OTLP
    // metrics request; the exemplar is the derived in-process tee row.
    store
        .ingest_metrics(
            Vec::new(),
            Vec::new(),
            vec![MetricExemplarRow {
                ts_nanos: 1_741_437_296_123_456_791,
                service: "payments".to_string(),
                name: "http.duration".to_string(),
                value: 88.0,
                trace_id: "trace-new".to_string(),
                span_id: "span-new".to_string(),
                run_id: Some("run-new".to_string()),
                attributes: serde_json::json!({"route": "/new"}),
            }],
            bytes::Bytes::new(),
        )
        .await
        .expect("post-migration exemplar ingest");
    let queried = store
        .metric_exemplars(
            "http.duration",
            Some("payments"),
            1_741_437_296_123_456_790..=1_741_437_296_123_456_792,
            10,
        )
        .await
        .expect("post-migration exemplar query");
    assert_eq!(queried.len(), 1);
    assert_eq!(queried[0].trace_id, "trace-new");
    assert_eq!(queried[0].span_id, "span-new");
    assert_eq!(queried[0].run_id.as_deref(), Some("run-new"));
    assert_eq!(queried[0].attributes, serde_json::json!({"route": "/new"}));
    engine.stop();
}
