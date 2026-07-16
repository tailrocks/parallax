//! Real-engine coverage for the forward-only exemplar bootstrap (plan 156;
//! operator 2026-07-17: no backward compatibility — legacy shapes are
//! dropped, never read or migrated).
//!
//! Run with: `cargo nextest run -p parallax-server --test m7_metric_exemplar_migration_greptime --run-ignored only`

#![allow(clippy::panic, reason = "test helpers fail fast with engine context")]

use parallax_greptime::GreptimeStore;
use parallax_server::{GreptimeSupervisor, ensure_greptime_binary};
use parallax_storage::adapter::{IngestStore, MetricAnalyticsStore};
use parallax_storage::model::MetricExemplarRow;

/// The retired pre-cutover shape: a `run_id` column and the legacy wide
/// primary key. Bootstrap must drop this without reading it.
const LEGACY_RUN_ID_DDL: &str = r#"CREATE TABLE {table} (
    "ts" TIMESTAMP(9) NOT NULL,
    "service" STRING, "name" STRING, "value" DOUBLE,
    "trace_id" STRING, "span_id" STRING,
    "run_id" STRING SKIPPING INDEX, "attributes" JSON,
    TIME INDEX ("ts"),
    PRIMARY KEY ("service", "name", "trace_id", "span_id")
) WITH (append_mode = 'true', ttl = '7d')"#;

async fn create_table(store: &GreptimeStore, ddl: &str, table: &str) {
    store
        .sql(&ddl.replace("{table}", table))
        .await
        .unwrap_or_else(|error| panic!("create {table}: {error:#}"));
}

async fn insert_legacy_rows(store: &GreptimeStore, table: &str) {
    store
        .sql(&format!(
            r#"INSERT INTO {table}
               ("ts", "service", "name", "value", "trace_id", "span_id", "run_id", "attributes")
               VALUES (1741437296123456789, 'checkout', 'http.duration', 42.5,
                       'trace-a', 'span-a', 'run-a', parse_json('{{"n":42}}'))"#
        ))
        .await
        .unwrap_or_else(|error| panic!("insert {table}: {error:#}"));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]
async fn forward_only_bootstrap_drops_legacy_exemplar_shapes() {
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
    let binary = ensure_greptime_binary(&bin_dir, "1.1.2", true)
        .await
        .expect("resolve GreptimeDB");
    let engine = GreptimeSupervisor::start(binary, tmp.path())
        .await
        .expect("start GreptimeDB");
    let store = GreptimeStore::connect(&engine.http_url, "7d", "7d", "7d")
        .await
        .expect("connect");

    // A pre-cutover canonical plus stray migration leftovers all disappear.
    create_table(&store, LEGACY_RUN_ID_DDL, "metric_exemplars").await;
    insert_legacy_rows(&store, "metric_exemplars").await;
    create_table(&store, LEGACY_RUN_ID_DDL, "metric_exemplars_v2").await;
    create_table(&store, LEGACY_RUN_ID_DDL, "metric_exemplars_v1_legacy").await;

    store.bootstrap("7d", "7d").await.expect("first bootstrap");
    store
        .bootstrap("7d", "7d")
        .await
        .expect("idempotent restart");

    let tables = store
        .sql(
            r"SELECT table_name FROM information_schema.tables
               WHERE table_schema = 'public' AND table_name LIKE 'metric_exemplars%'",
        )
        .await
        .expect("exemplar tables");
    assert_eq!(tables, vec![vec![serde_json::json!("metric_exemplars")]]);

    let describe = store
        .sql("DESCRIBE metric_exemplars")
        .await
        .expect("describe");
    let columns = describe
        .iter()
        .filter_map(|row| row.first().and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();
    assert!(columns.contains(&"invocation_id"), "{columns:?}");
    assert!(!columns.contains(&"run_id"), "{columns:?}");
    let primary_tags = describe
        .iter()
        .filter(|row| row.get(2).and_then(serde_json::Value::as_str) == Some("PRI"))
        .filter(|row| row.get(5).and_then(serde_json::Value::as_str) == Some("TAG"))
        .filter_map(|row| row.first().and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(primary_tags, ["service", "name"]);

    // No legacy row survives — the table is fresh by contract.
    assert_eq!(
        store
            .sql("SELECT COUNT(*) FROM metric_exemplars")
            .await
            .expect("count exemplars")[0][0],
        0
    );

    // The production ingest and query surfaces work on the fresh table. An
    // empty protobuf message is a valid empty OTLP metrics request; the
    // exemplar is the derived in-process tee row.
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
                invocation_id: Some("inv-new".to_string()),
                attributes: serde_json::json!({"route": "/new"}),
            }],
            bytes::Bytes::new(),
        )
        .await
        .expect("post-bootstrap exemplar ingest");
    let queried = store
        .metric_exemplars(
            "http.duration",
            Some("payments"),
            1_741_437_296_123_456_790..=1_741_437_296_123_456_792,
            10,
        )
        .await
        .expect("post-bootstrap exemplar query");
    assert_eq!(queried.len(), 1);
    assert_eq!(queried[0].trace_id, "trace-new");
    assert_eq!(queried[0].span_id, "span-new");
    assert_eq!(queried[0].invocation_id.as_deref(), Some("inv-new"));
    assert_eq!(queried[0].attributes, serde_json::json!({"route": "/new"}));
    engine.stop();
}
