use super::*;

type MetadataStore = TursoMetadataStore;
fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("metadata.db");
    (directory, path)
}

fn occurrence<'a>(
    fingerprint: &'a str,
    service: &'a str,
    ts_nanos: u128,
    attributes: &'a serde_json::Value,
) -> IssueOccurrence<'a> {
    IssueOccurrence {
        occurrence_id: format!("{fingerprint}:{ts_nanos}").into(),
        fingerprint,
        title: format!("Error: {fingerprint}"),
        error_type: "Error",
        culprit: None,
        service,
        ts_nanos,
        trace_id: None,
        attributes,
    }
}

fn occurrence_with_id<'a>(
    occurrence_id: &str,
    fingerprint: &'a str,
    ts_nanos: u128,
    attributes: &'a serde_json::Value,
) -> IssueOccurrence<'a> {
    let mut occurrence = occurrence(fingerprint, "svc", ts_nanos, attributes);
    occurrence.occurrence_id = occurrence_id.to_string().into();
    occurrence
}

#[tokio::test]
async fn occurrence_claim_survives_restart_concurrency_and_prunes() {
    let (_directory, path) = temp_db();
    let attrs = serde_json::json!({"region": "us"});
    let store = MetadataStore::open(&path).await.expect("open");
    store
        .upsert_issue_occurrence(&occurrence_with_id("same", "fp", 1, &attrs))
        .await
        .expect("first claim");
    drop(store);

    let store = std::sync::Arc::new(MetadataStore::open(&path).await.expect("reopen"));
    let mut deliveries = Vec::new();
    for _ in 0..8 {
        let store = std::sync::Arc::clone(&store);
        deliveries.push(tokio::spawn(async move {
            let attrs = serde_json::json!({"region": "us"});
            store
                .upsert_issue_occurrence(&occurrence_with_id("same", "fp", 1, &attrs))
                .await
        }));
    }
    for delivery in deliveries {
        delivery.await.expect("delivery task").expect("delivery");
    }
    let issue = store.issue("fp").await.expect("issue").expect("present");
    assert_eq!(issue.event_count, 1);

    let beyond_retention =
        u128::try_from(OCCURRENCE_RETENTION_MILLIS + 1).expect("positive retention") * 1_000_000;
    store
        .upsert_issue_occurrence(&occurrence_with_id("new", "fp", beyond_retention, &attrs))
        .await
        .expect("new occurrence");
    let conn = store.conn.lock().await;
    let mut rows = conn
        .query(
            "SELECT occurrence_id FROM issue_occurrences ORDER BY occurrence_id",
            (),
        )
        .await
        .expect("ledger query");
    let mut ids = Vec::new();
    while let Some(row) = rows.next().await.expect("ledger row") {
        ids.push(text(&row, 0));
    }
    assert_eq!(ids, vec!["new"]);
}

#[tokio::test]
async fn batch_upsert_merges_shared_fingerprint_tags_once() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    let shared_a = serde_json::json!({"http.route": "/checkout", "region": "us"});
    let shared_b = serde_json::json!({"http.route": "/checkout", "region": "eu"});
    let other = serde_json::json!({"http.route": "/cart"});
    store
        .upsert_issue_occurrences(&[
            occurrence("fp-shared", "checkout", 1_000_000_000, &shared_a),
            occurrence("fp-other", "checkout", 2_000_000_000, &other),
            occurrence("fp-shared", "checkout", 3_000_000_000, &shared_b),
        ])
        .await
        .expect("batch upsert");

    let shared = store
        .issue("fp-shared")
        .await
        .expect("issue")
        .expect("present");
    let other_issue = store
        .issue("fp-other")
        .await
        .expect("issue")
        .expect("present");
    assert_eq!(shared.event_count, 2);
    assert_eq!(other_issue.event_count, 1);
    assert_eq!(shared.first_seen_nanos, 1_000_000_000);
    assert_eq!(shared.last_seen_nanos, 3_000_000_000);

    let tags: serde_json::Value = serde_json::from_str(&shared.tags).expect("tags");
    assert_eq!(tags["http.route"]["/checkout"], 2);
    assert_eq!(tags["region"]["us"], 1);
    assert_eq!(tags["region"]["eu"], 1);

    let other_tags: serde_json::Value =
        serde_json::from_str(&other_issue.tags).expect("other tags");
    assert_eq!(other_tags["http.route"]["/cart"], 1);

    let trend = store.issue_trend("fp-shared", 0, 60).await.expect("trend");
    let total: u64 = trend.iter().map(|p| p.count).sum();
    assert_eq!(total, 2);
    let other_trend = store.issue_trend("fp-other", 0, 60).await.expect("trend");
    let other_total: u64 = other_trend.iter().map(|p| p.count).sum();
    assert_eq!(other_total, 1);
}

#[tokio::test]
async fn tags_accumulate_bounded() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    let attrs = serde_json::json!({
        "http.route": "/checkout",
        "exception.message": "ignored",
        "nested": {"skip": true},
        "attempt": 3,
    });
    for index in 0..2 {
        store
            .upsert_issue_occurrence(&occurrence("fp1", "svc", 1_000_000_000 + index, &attrs))
            .await
            .expect("upsert");
    }
    let issue = store.issue("fp1").await.expect("issue").expect("present");
    let tags: serde_json::Value = serde_json::from_str(&issue.tags).expect("tags json");
    assert_eq!(tags["http.route"]["/checkout"], 2);
    assert_eq!(tags["attempt"]["3"], 2);
    assert!(tags.get("exception.message").is_none());
    assert!(tags.get("nested").is_none());
}

#[tokio::test]
async fn first_seen_lowers_on_out_of_order_occurrence() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    let attrs = serde_json::json!({});
    // Later-timestamped occurrence first.
    store
        .upsert_issue_occurrence(&occurrence(
            "fp-order",
            "svc",
            2_000_000_000, // 2000 ms
            &attrs,
        ))
        .await
        .expect("upsert later");
    // Earlier-timestamped occurrence second must pull first_seen back.
    store
        .upsert_issue_occurrence(&occurrence(
            "fp-order",
            "svc",
            1_000_000_000, // 1000 ms
            &attrs,
        ))
        .await
        .expect("upsert earlier");
    let issue = store
        .issue("fp-order")
        .await
        .expect("issue")
        .expect("present");
    assert_eq!(issue.first_seen_nanos, 1_000_000_000);
    assert_eq!(issue.last_seen_nanos, 2_000_000_000);
    assert_eq!(issue.event_count, 2);
}

#[tokio::test]
async fn filtered_issues_page_and_total() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    let attrs = serde_json::json!({"env": "dev"});
    for i in 0..5u128 {
        let fingerprint = format!("fp{i}");
        let service = if i % 2 == 0 { "alpha" } else { "beta" };
        store
            .upsert_issue_occurrence(&occurrence(
                &fingerprint,
                service,
                (i + 1) * 60_000_000_000,
                &attrs,
            ))
            .await
            .expect("upsert");
    }
    let (page, total) = store
        .issues_filtered(
            &IssueQuery {
                service: Some("alpha".into()),
                ..Default::default()
            },
            IssueSortKey::LastSeen,
            2,
            0,
        )
        .await
        .expect("filtered");
    assert_eq!(total, 3);
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].fingerprint, "fp4"); // newest last_seen first

    let (tagged, tagged_total) = store
        .issues_filtered(
            &IssueQuery {
                tag_key: Some("env".into()),
                tag_value: Some("dev".into()),
                ..Default::default()
            },
            IssueSortKey::Events,
            10,
            0,
        )
        .await
        .expect("tag filtered");
    assert_eq!(tagged_total, 5);
    assert_eq!(tagged.len(), 5);

    let (none, none_total) = store
        .issues_filtered(
            &IssueQuery {
                query: Some("missing-needle".into()),
                ..Default::default()
            },
            IssueSortKey::LastSeen,
            10,
            0,
        )
        .await
        .expect("query filtered");
    assert_eq!(none_total, 0);
    assert!(none.is_empty());
}

#[tokio::test]
async fn saved_views_round_trip_update_filter_and_delete() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    store
        .saved_view_save("logs-errors", "Errors", "/logs", "?sev=17", 1_000_000)
        .await
        .expect("save logs");
    store
        .saved_view_save("traces-api", "API", "/traces", "?service=api", 2_000_000)
        .await
        .expect("save traces");
    store
        .saved_view_save("logs-errors", "Errors v2", "/logs", "?sev=13", 3_000_000)
        .await
        .expect("update logs");

    let logs = store.saved_views(Some("/logs")).await.expect("list logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].id, "logs-errors");
    assert_eq!(logs[0].name, "Errors v2");
    assert_eq!(logs[0].state, "?sev=13");

    let all = store.saved_views(None).await.expect("list all");
    assert_eq!(
        all.iter().map(|view| view.id.as_str()).collect::<Vec<_>>(),
        vec!["logs-errors", "traces-api"]
    );
    assert!(
        store
            .saved_view_delete("logs-errors")
            .await
            .expect("delete")
    );
    assert!(
        store
            .saved_view("logs-errors")
            .await
            .expect("fetch")
            .is_none()
    );
}

#[tokio::test]
async fn investigations_round_trip_update_list_and_delete() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    store
        .investigation_save(
            "case-a",
            "Checkout errors",
            r#"{"version":1,"window":{"range":"24h"},"pins":[],"notes":""}"#,
            1_000_000,
        )
        .await
        .expect("save case a");
    store
        .investigation_save(
            "case-b",
            "Slow checkout",
            r#"{"version":1,"window":{"range":"1h"},"pins":[],"notes":""}"#,
            2_000_000,
        )
        .await
        .expect("save case b");
    store
        .investigation_save(
            "case-a",
            "Checkout errors v2",
            r#"{"version":1,"window":{"range":"custom","from":"1","to":"2"},"pins":[{"kind":"trace","ref":"/traces/t1","label":"trace"}],"notes":"triage"}"#,
            3_000_000,
        )
        .await
        .expect("update case a");

    let all = store.investigations().await.expect("list");
    assert_eq!(
        all.iter().map(|case| case.id.as_str()).collect::<Vec<_>>(),
        vec!["case-a", "case-b"]
    );
    let case = store
        .investigation("case-a")
        .await
        .expect("get")
        .expect("present");
    assert_eq!(case.name, "Checkout errors v2");
    assert!(case.state.contains("\"notes\":\"triage\""));
    assert_eq!(case.updated_at_nanos, 3_000_000);

    assert!(store.investigation_delete("case-a").await.expect("delete"));
    assert!(
        store
            .investigation("case-a")
            .await
            .expect("fetch")
            .is_none()
    );
}

/// Regression guard for the turso pitfall the tag cache hit: an UPDATE
/// executed while a SELECT statement is still open on the same connection
/// reports success but does not persist. Read-merge-write paths must drop
/// the reading statement first.
#[tokio::test]
async fn update_with_open_statement_is_lost() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    let conn = store.conn.lock().await;
    conn.execute(
        "INSERT INTO dashboards (id, name, layout, created_at, updated_at)
         VALUES ('k', 'v1', '[]', 1, 1)",
        (),
    )
    .await
    .expect("insert");

    // Open statement held across the UPDATE: the write is lost.
    let mut open_rows = conn
        .query("SELECT name FROM dashboards WHERE id = 'k'", ())
        .await
        .expect("open select");
    let _row = open_rows.next().await.expect("next").expect("row");
    conn.execute("UPDATE dashboards SET name = 'lost' WHERE id = 'k'", ())
        .await
        .expect("update during open statement");
    drop(open_rows);

    // Statement dropped first: the write persists.
    conn.execute("UPDATE dashboards SET name = 'v2' WHERE id = 'k'", ())
        .await
        .expect("update");
    let mut rows = conn
        .query("SELECT name FROM dashboards WHERE id = 'k'", ())
        .await
        .expect("select");
    let row = rows.next().await.expect("next").expect("row");
    assert_eq!(text(&row, 0), "v2");
}
#[tokio::test]
async fn external_runs_register_once() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(path).await.expect("open");
    store
        .ensure_invocation("jk-run-1", 5_000_000_000)
        .await
        .expect("ensure");
    store
        .ensure_invocation("jk-run-1", 9_000_000_000)
        .await
        .expect("ensure again");
    let run = store
        .invocation("jk-run-1")
        .await
        .expect("run")
        .expect("registered");
    assert_eq!(run.status, "external");
    assert_eq!(run.started_at_nanos, 5_000_000_000);

    // A wrapper-started run keeps its own record.
    store
        .start_invocation(
            "run_cli",
            Some("cargo test"),
            Some("one_shot"),
            1_000_000_000,
        )
        .await
        .expect("start");
    store
        .ensure_invocation("run_cli", 2_000_000_000)
        .await
        .expect("ensure existing");
    let cli_run = store
        .invocation("run_cli")
        .await
        .expect("run")
        .expect("present");
    assert_eq!(cli_run.status, "running");
    assert_eq!(cli_run.command.as_deref(), Some("cargo test"));
}

#[tokio::test]
async fn legacy_runs_table_is_dropped_forward_only() {
    // Operator 2026-07-17: no backward compatibility — a pre-cutover `runs`
    // table is dropped at bootstrap, never read or migrated.
    let (_directory, path) = temp_db();
    {
        let database = turso::Builder::new_local(path.to_str().expect("utf8 path"))
            .build()
            .await
            .expect("open raw db");
        let connection = database.connect().expect("connect raw db");
        connection
            .execute(
                "CREATE TABLE runs (
                   run_id TEXT PRIMARY KEY, command TEXT, started_at INTEGER NOT NULL,
                   ended_at INTEGER, exit_code INTEGER, status TEXT NOT NULL DEFAULT 'running'
                 )",
                (),
            )
            .await
            .expect("create legacy table");
        connection
            .execute(
                "INSERT INTO runs (run_id, started_at) VALUES ('legacy-run', 1)",
                (),
            )
            .await
            .expect("seed legacy row");
    }
    let store = MetadataStore::open(&path).await.expect("open");
    assert!(
        store.invocations(10).await.expect("invocations").is_empty(),
        "legacy runs rows must not surface as invocations"
    );
    let conn = store.conn.lock().await;
    let mut rows = conn
        .query(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'runs'",
            (),
        )
        .await
        .expect("query sqlite_master");
    let row = rows.next().await.expect("next").expect("row");
    assert_eq!(integer(&row, 0), 0, "legacy runs table must be dropped");
}

#[tokio::test]
async fn test_reporting_schema_has_reference_and_mutable_state_tables() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    let conn = store.conn.lock().await;
    for table in [
        "test_cases",
        "test_variants",
        "test_results",
        "test_flaky_states",
    ] {
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                (table,),
            )
            .await
            .expect("query schema");
        let row = rows.next().await.expect("read count").expect("count row");
        assert_eq!(integer(&row, 0), 1, "missing {table}");
    }
    let mut rows = conn
        .query(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
            ("test_results_flaky_scan",),
        )
        .await
        .expect("query flaky index");
    let row = rows.next().await.expect("read count").expect("count row");
    assert_eq!(integer(&row, 0), 1, "missing flaky scan index");
}

async fn seed_test_reporting(store: &dyn parallax_storage::metadata::MetadataStore) {
    use parallax_model::{
        FlakyEvidence, FlakyState, TestAttempt, TestCaseIdentitySource, TestCaseKey,
        TestCaseRecord, TestConfiguration, TestFlakyStateRecord, TestResultKey, TestResultRecord,
        TestStatus, TestVariantKey, TestVariantRecord, TraceId,
    };
    use std::str::FromStr;

    let case_key = TestCaseKey::from_str(&format!("tc1:{}", "a".repeat(64))).expect("case");
    let variant_key =
        TestVariantKey::from_str(&format!("tv1:{}", "b".repeat(64))).expect("variant");
    store
        .upsert_test_case(&TestCaseRecord {
            key: case_key.clone(),
            identity_source: TestCaseIdentitySource::CodeReference,
            explicit_id: None,
            code_reference: Some("crate::suite::test".into()),
            suite_path: vec!["suite".into()],
            name: "test".into(),
            first_seen_nanos: 2_000_000,
            last_seen_nanos: 3_000_000,
        })
        .await
        .expect("case upsert");
    store
        .upsert_test_variant(&TestVariantRecord {
            key: variant_key.clone(),
            case_key,
            parameters: Vec::new(),
            first_seen_nanos: 2_000_000,
            last_seen_nanos: 3_000_000,
        })
        .await
        .expect("variant upsert");
    store
        .upsert_test_result(&TestResultRecord {
            key: TestResultKey {
                variant_key: variant_key.clone(),
                invocation_id: "inv-test".into(),
                attempt: TestAttempt::new(1).expect("attempt"),
            },
            status: TestStatus::Failed,
            trace_id: TraceId::from_str("abababababababababababababababab").expect("trace"),
            span_id: "cdcdcdcdcdcdcdcd".into(),
            started_at_nanos: 2_000_000,
            ended_at_nanos: 3_000_000,
            service: "checkout".into(),
            service_version: Some("1.2.3".into()),
            vcs_head_revision: Some("deadbeef".into()),
            configuration: TestConfiguration::default(),
            failure_fingerprint: Some("fp-test".into()),
        })
        .await
        .expect("result upsert");
    store
        .upsert_test_flaky_state(&TestFlakyStateRecord {
            variant_key,
            state: FlakyState::Flaky,
            evidence: FlakyEvidence {
                intra_invocation_mix: true,
                ..FlakyEvidence::default()
            },
            updated_at_nanos: 3_000_000,
        })
        .await
        .expect("flaky upsert");
}

#[tokio::test]
async fn test_reporting_upserts_are_idempotent_and_reference_native_spans() {
    use parallax_model::{FlakyState, TestStatus};

    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    let port: &dyn parallax_storage::metadata::MetadataStore = &store;
    seed_test_reporting(port).await;

    assert_eq!(
        port.test_case(&format!("tc1:{}", "a".repeat(64)))
            .await
            .expect("read case")
            .expect("case")
            .name,
        "test"
    );
    assert!(
        port.test_variant(&format!("tv1:{}", "b".repeat(64)))
            .await
            .expect("read variant")
            .is_some()
    );
    let results = port
        .test_results_for_invocation("inv-test", 10)
        .await
        .expect("read results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, TestStatus::Failed);
    assert_eq!(results[0].key.attempt.get(), 1);
    assert_eq!(
        port.test_flaky_state(&format!("tv1:{}", "b".repeat(64)))
            .await
            .expect("read flaky")
            .expect("flaky")
            .state,
        FlakyState::Flaky
    );

    let conn = store.conn.lock().await;
    for table in [
        "test_cases",
        "test_variants",
        "test_results",
        "test_flaky_states",
    ] {
        let mut rows = conn
            .query(&format!("SELECT COUNT(*) FROM {table}"), ())
            .await
            .expect("count query");
        let row = rows.next().await.expect("read count").expect("count row");
        assert_eq!(integer(&row, 0), 1, "unexpected {table} count");
    }
    let mut rows = conn
        .query("SELECT trace_id, span_id FROM test_results", ())
        .await
        .expect("result refs");
    let row = rows.next().await.expect("read refs").expect("refs row");
    assert_eq!(text(&row, 0), "abababababababababababababababab");
    assert_eq!(text(&row, 1), "cdcdcdcdcdcdcdcd");
}

#[tokio::test]
async fn test_case_variants_and_variant_history_are_bounded_and_isolated() {
    use parallax_model::{
        TestAttempt, TestCaseIdentitySource, TestCaseKey, TestCaseRecord, TestConfiguration,
        TestResultKey, TestResultRecord, TestStatus, TestVariantKey, TestVariantRecord, TraceId,
    };
    use parallax_storage::metadata::{MetadataErrorKind, MetadataStore as MetadataStorePort};
    use std::str::FromStr;

    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    let port: &dyn MetadataStorePort = &store;
    seed_test_reporting(port).await;

    let case_key = TestCaseKey::from_str(&format!("tc1:{}", "a".repeat(64))).expect("case");
    let other_case = TestCaseKey::from_str(&format!("tc1:{}", "c".repeat(64))).expect("case");
    let newer_variant =
        TestVariantKey::from_str(&format!("tv1:{}", "d".repeat(64))).expect("variant");
    let other_variant =
        TestVariantKey::from_str(&format!("tv1:{}", "e".repeat(64))).expect("variant");

    port.upsert_test_case(&TestCaseRecord {
        key: other_case.clone(),
        identity_source: TestCaseIdentitySource::NamePath,
        explicit_id: None,
        code_reference: None,
        suite_path: vec!["other".into()],
        name: "other".into(),
        first_seen_nanos: 1_000_000,
        last_seen_nanos: 8_000_000,
    })
    .await
    .expect("other case");
    for (key, parent, last_seen) in [
        (newer_variant.clone(), case_key.clone(), 9_000_000),
        (other_variant.clone(), other_case, 8_000_000),
    ] {
        port.upsert_test_variant(&TestVariantRecord {
            key,
            case_key: parent,
            parameters: Vec::new(),
            first_seen_nanos: 1_000_000,
            last_seen_nanos: last_seen,
        })
        .await
        .expect("variant");
    }

    for (invocation, attempt, status, started) in [
        ("inv-old", 1, TestStatus::Failed, 4_000_000),
        ("inv-new", 1, TestStatus::Failed, 6_000_000),
        ("inv-new", 2, TestStatus::Passed, 7_000_000),
    ] {
        port.upsert_test_result(&TestResultRecord {
            key: TestResultKey {
                variant_key: newer_variant.clone(),
                invocation_id: invocation.into(),
                attempt: TestAttempt::new(attempt).expect("attempt"),
            },
            status,
            trace_id: TraceId::from_str("abababababababababababababababab").expect("trace"),
            span_id: "cdcdcdcdcdcdcdcd".into(),
            started_at_nanos: started,
            ended_at_nanos: started + 1_000_000,
            service: "checkout".into(),
            service_version: None,
            vcs_head_revision: None,
            configuration: TestConfiguration::default(),
            failure_fingerprint: None,
        })
        .await
        .expect("result");
    }

    let variants = port
        .test_variants_for_case(case_key.as_str(), 1)
        .await
        .expect("case variants");
    assert_eq!(variants.len(), 1);
    assert_eq!(variants[0].key, newer_variant);

    let history = port
        .test_results_for_variant(newer_variant.as_str(), 10)
        .await
        .expect("history");
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].key.attempt.get(), 2);
    assert_eq!(history[1].key.invocation_id, "inv-new");
    assert_eq!(history[2].key.invocation_id, "inv-old");

    assert_eq!(
        port.test_results_for_variant(newer_variant.as_str(), 1)
            .await
            .expect("bounded history")
            .len(),
        1
    );

    let invalid = port
        .test_variants_for_case("not-a-versioned-key", 10)
        .await
        .expect_err("invalid case key");
    assert_eq!(invalid.kind(), MetadataErrorKind::InvalidInput);
}

#[tokio::test]
async fn test_case_detail_batches_history_and_clamps_per_variant() {
    use parallax_model::{
        TestAttempt, TestCaseIdentitySource, TestCaseKey, TestCaseRecord, TestConfiguration,
        TestResultKey, TestResultRecord, TestStatus, TestVariantKey, TestVariantRecord, TraceId,
    };
    use parallax_storage::metadata::MetadataStore as MetadataStorePort;
    use std::str::FromStr;

    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    let port: &dyn MetadataStorePort = &store;
    let case_key = TestCaseKey::from_str(&format!("tc1:{}", "a".repeat(64))).expect("case");
    let variant_key =
        TestVariantKey::from_str(&format!("tv1:{}", "d".repeat(64))).expect("variant");
    port.upsert_test_case(&TestCaseRecord {
        key: case_key.clone(),
        identity_source: TestCaseIdentitySource::NamePath,
        explicit_id: None,
        code_reference: None,
        suite_path: vec!["suite".into()],
        name: "case".into(),
        first_seen_nanos: 1_000_000,
        last_seen_nanos: 9_000_000,
    })
    .await
    .expect("case");
    port.upsert_test_variant(&TestVariantRecord {
        key: variant_key.clone(),
        case_key: case_key.clone(),
        parameters: Vec::new(),
        first_seen_nanos: 1_000_000,
        last_seen_nanos: 9_000_000,
    })
    .await
    .expect("variant");
    for (invocation, attempt, started) in [
        ("inv-old", 1_u32, 4_000_000_u128),
        ("inv-new", 1, 6_000_000),
        ("inv-new", 2, 7_000_000),
    ] {
        port.upsert_test_result(&TestResultRecord {
            key: TestResultKey {
                variant_key: variant_key.clone(),
                invocation_id: invocation.into(),
                attempt: TestAttempt::new(attempt).expect("attempt"),
            },
            status: TestStatus::Failed,
            trace_id: TraceId::from_str("abababababababababababababababab").expect("trace"),
            span_id: "cdcdcdcdcdcdcdcd".into(),
            started_at_nanos: started,
            ended_at_nanos: started + 1_000_000,
            service: "checkout".into(),
            service_version: None,
            vcs_head_revision: None,
            configuration: TestConfiguration::default(),
            failure_fingerprint: None,
        })
        .await
        .expect("result");
    }

    let detail = port
        .test_case_detail(case_key.as_str(), 10, 2)
        .await
        .expect("detail")
        .expect("case present");
    assert_eq!(detail.case.key, case_key);
    assert_eq!(detail.variants.len(), 1);
    assert_eq!(detail.variants[0].variant.key, variant_key);
    assert_eq!(detail.variants[0].history.len(), 2);
    assert_eq!(detail.variants[0].history[0].key.attempt.get(), 2);
    assert_eq!(detail.variants[0].history[1].key.invocation_id, "inv-new");
}

fn flaky_result(
    variant_key: parallax_model::TestVariantKey,
    invocation_id: &str,
    attempt: u32,
    status: parallax_model::TestStatus,
    ended_at_nanos: u128,
) -> parallax_model::TestResultRecord {
    use parallax_model::{TestAttempt, TestConfiguration, TestResultKey, TraceId};
    use std::str::FromStr;

    parallax_model::TestResultRecord {
        key: TestResultKey {
            variant_key,
            invocation_id: invocation_id.into(),
            attempt: TestAttempt::new(attempt).expect("attempt"),
        },
        status,
        trace_id: TraceId::from_str("efefefefefefefefefefefefefefefef").expect("trace"),
        span_id: "abababababababab".into(),
        started_at_nanos: ended_at_nanos.saturating_sub(1_000_000),
        ended_at_nanos,
        service: "checkout".into(),
        service_version: None,
        vcs_head_revision: Some("deadbeef".into()),
        configuration: TestConfiguration::default(),
        failure_fingerprint: None,
    }
}

#[tokio::test]
async fn flaky_candidate_cursor_and_result_window_are_lossless_and_bounded() {
    use parallax_model::{TestFlakyCursor, TestStatus, TestVariantKey};
    use parallax_storage::metadata::{MetadataErrorKind, MetadataStore as MetadataStorePort};
    use std::str::FromStr;

    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    let port: &dyn MetadataStorePort = &store;
    seed_test_reporting(port).await;
    let newer = TestVariantKey::from_str(&format!("tv1:{}", "d".repeat(64))).expect("newer");
    let tied = TestVariantKey::from_str(&format!("tv1:{}", "e".repeat(64))).expect("tied");
    for row in [
        flaky_result(newer.clone(), "old", 1, TestStatus::Failed, 5_000_000),
        flaky_result(newer.clone(), "new", 1, TestStatus::Failed, 7_000_000),
        flaky_result(newer.clone(), "new", 2, TestStatus::Passed, 8_000_000),
        flaky_result(tied.clone(), "other", 1, TestStatus::Passed, 8_000_000),
    ] {
        port.upsert_test_result(&row).await.expect("result");
    }

    let first = port
        .test_flaky_candidates(0, 10_000_000, None, 1)
        .await
        .expect("first");
    assert!(first.has_more);
    assert_eq!(first.items[0].last_ended_nanos, 3_000_000);
    let cursor = TestFlakyCursor {
        last_ended_nanos: first.items[0].last_ended_nanos,
        variant_key: first.items[0].variant_key.clone(),
    };
    let second = port
        .test_flaky_candidates(0, 10_000_000, Some(&cursor), 1)
        .await
        .expect("second");
    assert!(second.has_more);
    assert_eq!(second.items[0].variant_key, newer);
    let tie_cursor = TestFlakyCursor {
        last_ended_nanos: second.items[0].last_ended_nanos,
        variant_key: second.items[0].variant_key.clone(),
    };
    let third = port
        .test_flaky_candidates(0, 10_000_000, Some(&tie_cursor), 1)
        .await
        .expect("third");
    assert!(!third.has_more);
    assert_eq!(third.items[0].variant_key, tied);

    let exact = port
        .test_flaky_candidates(3_000_000, 3_000_000, None, 10)
        .await
        .expect("exact");
    assert_eq!(exact.items, first.items);
    let window = port
        .test_results_for_variant_window(newer.as_str(), 5_000_000, 8_000_000, 2)
        .await
        .expect("window");
    assert!(window.truncated);
    assert_eq!(window.items.len(), 2);
    assert_eq!(window.items[0].ended_at_nanos, 5_000_000);
    assert_eq!(window.items[1].ended_at_nanos, 7_000_000);
    let reversed = port
        .test_results_for_variant_window(newer.as_str(), 9, 8, 10)
        .await
        .expect_err("reversed");
    assert_eq!(reversed.kind(), MetadataErrorKind::InvalidInput);
    let invalid = port
        .test_results_for_variant_window("not-versioned", 0, 1, 10)
        .await
        .expect_err("invalid variant");
    assert_eq!(invalid.kind(), MetadataErrorKind::InvalidInput);
    let outside = TestFlakyCursor {
        last_ended_nanos: 11_000_000,
        variant_key: newer,
    };
    let outside = port
        .test_flaky_candidates(0, 10_000_000, Some(&outside), 10)
        .await
        .expect_err("outside cursor");
    assert_eq!(outside.kind(), MetadataErrorKind::InvalidInput);
}

#[tokio::test]
async fn test_explorer_rolls_up_attempts_and_filters_through_port() {
    use parallax_model::{
        AttemptRollup, TestAttempt, TestConfiguration, TestConfigurationFilter, TestExplorerQuery,
        TestExplorerSort, TestResultKey, TestResultRecord, TestStatus, TestVariantKey, TraceId,
    };
    use parallax_storage::metadata::{MetadataErrorKind, MetadataStore as MetadataStorePort};
    use std::collections::BTreeMap;
    use std::str::FromStr;

    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    seed_test_reporting(&store).await;
    let variant_key =
        TestVariantKey::from_str(&format!("tv1:{}", "b".repeat(64))).expect("variant");
    store
        .upsert_test_result(&TestResultRecord {
            key: TestResultKey {
                variant_key,
                invocation_id: "inv-test".into(),
                attempt: TestAttempt::new(2).expect("attempt"),
            },
            status: TestStatus::Passed,
            trace_id: TraceId::from_str("efefefefefefefefefefefefefefefef").expect("trace"),
            span_id: "abababababababab".into(),
            started_at_nanos: 4_000_000,
            ended_at_nanos: 5_000_000,
            service: "checkout".into(),
            service_version: Some("1.2.3".into()),
            vcs_head_revision: Some("deadbeef".into()),
            configuration: TestConfiguration {
                dimensions: BTreeMap::from([("test.configuration.os".into(), "linux".into())]),
            },
            failure_fingerprint: None,
        })
        .await
        .expect("second attempt");

    let port: &dyn MetadataStorePort = &store;
    let page = port
        .test_explorer(
            &TestExplorerQuery {
                service: Some("checkout".into()),
                service_version: Some("1.2.3".into()),
                status: Some(AttemptRollup::FlakyPass),
                suite: Some("suite".into()),
                configuration: Some(TestConfigurationFilter {
                    key: "test.configuration.os".into(),
                    value: "linux".into(),
                }),
                ..Default::default()
            },
            TestExplorerSort::LastSeen,
            10,
            0,
        )
        .await
        .expect("explorer");
    assert_eq!(page.items.len(), 1);
    assert!(!page.has_more);
    assert_eq!(page.items[0].rollup, AttemptRollup::FlakyPass);
    assert_eq!(page.items[0].attempt_count, 2);
    assert_eq!(page.items[0].last_result.key.attempt.get(), 2);

    let invalid = port
        .test_explorer(
            &TestExplorerQuery {
                configuration: Some(TestConfigurationFilter {
                    key: "x') OR 1=1 --".into(),
                    value: "linux".into(),
                }),
                ..Default::default()
            },
            TestExplorerSort::Name,
            10,
            0,
        )
        .await
        .expect_err("invalid config key");
    assert_eq!(invalid.kind(), MetadataErrorKind::InvalidInput);
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "multi-case explorer fixture proves filters, latest invocation, and rollups together"
)]
async fn test_explorer_filters_latest_variant_invocation_and_rollups() {
    use parallax_model::{
        AttemptRollup, FlakyState, TestAttempt, TestCaseIdentitySource, TestCaseKey,
        TestCaseRecord, TestConfiguration, TestExplorerQuery, TestExplorerSort, TestResultKey,
        TestResultRecord, TestStatus, TestVariantKey, TestVariantRecord, TraceId,
    };
    use std::str::FromStr;

    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    let port: &dyn parallax_storage::metadata::MetadataStore = &store;

    let case_a = TestCaseKey::from_str(&format!("tc1:{}", "a".repeat(64))).expect("case a");
    let case_b = TestCaseKey::from_str(&format!("tc1:{}", "c".repeat(64))).expect("case b");
    let variant_a =
        TestVariantKey::from_str(&format!("tv1:{}", "b".repeat(64))).expect("variant a");
    let variant_b =
        TestVariantKey::from_str(&format!("tv1:{}", "d".repeat(64))).expect("variant b");

    port.upsert_test_case(&TestCaseRecord {
        key: case_a.clone(),
        identity_source: TestCaseIdentitySource::CodeReference,
        explicit_id: None,
        code_reference: Some("crate::suite::alpha".into()),
        suite_path: vec!["suite".into(), "alpha".into()],
        name: "alpha_test".into(),
        first_seen_nanos: 1_000_000,
        last_seen_nanos: 5_000_000,
    })
    .await
    .expect("case a");
    port.upsert_test_case(&TestCaseRecord {
        key: case_b.clone(),
        identity_source: TestCaseIdentitySource::NamePath,
        explicit_id: None,
        code_reference: None,
        suite_path: vec!["suite".into(), "beta".into()],
        name: "beta_test".into(),
        first_seen_nanos: 1_000_000,
        last_seen_nanos: 4_000_000,
    })
    .await
    .expect("case b");
    port.upsert_test_variant(&TestVariantRecord {
        key: variant_a.clone(),
        case_key: case_a,
        parameters: Vec::new(),
        first_seen_nanos: 1_000_000,
        last_seen_nanos: 5_000_000,
    })
    .await
    .expect("variant a");
    port.upsert_test_variant(&TestVariantRecord {
        key: variant_b.clone(),
        case_key: case_b,
        parameters: Vec::new(),
        first_seen_nanos: 1_000_000,
        last_seen_nanos: 4_000_000,
    })
    .await
    .expect("variant b");

    // Older invocation for alpha fails; newer flaky_pass (fail then pass).
    for (invocation, attempt, status, start, end) in [
        ("inv-old", 1, TestStatus::Failed, 1_000_000, 2_000_000),
        ("inv-new", 1, TestStatus::Failed, 3_000_000, 4_000_000),
        ("inv-new", 2, TestStatus::Passed, 4_000_000, 5_000_000),
        ("inv-beta", 1, TestStatus::Broken, 2_500_000, 3_500_000),
    ] {
        let variant = if invocation == "inv-beta" {
            variant_b.clone()
        } else {
            variant_a.clone()
        };
        let service = if invocation == "inv-beta" {
            "billing"
        } else {
            "checkout"
        };
        port.upsert_test_result(&TestResultRecord {
            key: TestResultKey {
                variant_key: variant,
                invocation_id: invocation.into(),
                attempt: TestAttempt::new(attempt).expect("attempt"),
            },
            status,
            trace_id: TraceId::from_str("abababababababababababababababab").expect("trace"),
            span_id: "cdcdcdcdcdcdcdcd".into(),
            started_at_nanos: start,
            ended_at_nanos: end,
            service: service.into(),
            service_version: Some("1.0.0".into()),
            vcs_head_revision: Some("deadbeef".into()),
            configuration: TestConfiguration::default(),
            failure_fingerprint: None,
        })
        .await
        .expect("result");
    }

    let page = port
        .test_explorer(
            &TestExplorerQuery::default(),
            TestExplorerSort::LastSeen,
            10,
            0,
        )
        .await
        .expect("explorer");
    assert_eq!(page.items.len(), 2);
    assert!(!page.has_more);
    assert_eq!(page.items[0].case.name, "alpha_test");
    assert_eq!(page.items[0].invocation_id, "inv-new");
    assert_eq!(page.items[0].rollup, AttemptRollup::FlakyPass);
    assert_eq!(page.items[0].attempt_count, 2);
    assert_eq!(page.items[1].case.name, "beta_test");
    assert_eq!(page.items[1].rollup, AttemptRollup::Broken);

    let filtered = port
        .test_explorer(
            &TestExplorerQuery {
                service: Some("checkout".into()),
                status: Some(AttemptRollup::FlakyPass),
                query: Some("alpha".into()),
                suite: Some("suite".into()),
                ..TestExplorerQuery::default()
            },
            TestExplorerSort::Name,
            10,
            0,
        )
        .await
        .expect("filtered");
    assert_eq!(filtered.items.len(), 1);
    assert_eq!(filtered.items[0].case.name, "alpha_test");
    assert_eq!(filtered.items[0].flaky.as_ref().map(|row| row.state), None);

    let empty = port
        .test_explorer(
            &TestExplorerQuery {
                service: Some("missing".into()),
                ..TestExplorerQuery::default()
            },
            TestExplorerSort::LastSeen,
            10,
            0,
        )
        .await
        .expect("empty");
    assert!(empty.items.is_empty());

    let bad_range = port
        .test_explorer(
            &TestExplorerQuery {
                from_nanos: Some(10),
                to_nanos: Some(1),
                ..TestExplorerQuery::default()
            },
            TestExplorerSort::LastSeen,
            10,
            0,
        )
        .await;
    bad_range.expect_err("reversed explorer range must fail");

    // Seed flaky state and filter on it.
    seed_test_reporting(port).await;
    let flaky = port
        .test_explorer(
            &TestExplorerQuery {
                flaky_state: Some(FlakyState::Flaky),
                ..TestExplorerQuery::default()
            },
            TestExplorerSort::LastSeen,
            10,
            0,
        )
        .await
        .expect("flaky filter");
    assert!(flaky.items.iter().any(|row| {
        row.flaky
            .as_ref()
            .is_some_and(|f| f.state == FlakyState::Flaky)
    }));
}

#[tokio::test]
async fn issue_title_and_culprit_are_sanitized_at_rest() {
    let (_directory, path) = temp_db();
    let store = MetadataStore::open(&path).await.expect("open");
    let attrs = serde_json::json!({});
    let mut occurrence = occurrence("fp-secret", "svc", 1_000_000_000, &attrs);
    occurrence.title = "boom postgres://admin:s3cr3t@db/app".into();
    occurrence.culprit = Some(concat!("token=ghp_", "0123456789ABCDEFGHIJKLMNOPQRST").into());
    store
        .upsert_issue_occurrence(&occurrence)
        .await
        .expect("upsert");

    let issue = store
        .issue("fp-secret")
        .await
        .expect("read")
        .expect("present");
    assert!(
        !issue.title.contains("s3cr3t"),
        "raw dsn secret must not persist: {}",
        issue.title
    );
    assert!(
        issue.title.contains("[REDACTED:dsn_userinfo]"),
        "title should keep redaction marker: {}",
        issue.title
    );
    let culprit = issue.culprit.expect("culprit");
    assert!(
        !culprit.contains("ghp_0123456789"),
        "raw token must not persist: {culprit}"
    );

    // Migration is idempotent on already-sanitized rows.
    let rewritten = store
        .sanitize_existing_issue_text()
        .await
        .expect("sanitize pass");
    assert_eq!(rewritten, 0, "already-clean rows must not thrash");
}

#[tokio::test]
async fn legacy_issues_table_gains_nullable_resolution_time() {
    let (_directory, path) = temp_db();
    {
        let database = turso::Builder::new_local(path.to_str().expect("utf8 path"))
            .build()
            .await
            .expect("open raw db");
        let connection = database.connect().expect("connect raw db");
        connection
            .execute(
                "CREATE TABLE issues (
                   fingerprint TEXT PRIMARY KEY, title TEXT NOT NULL,
                   error_type TEXT NOT NULL, culprit TEXT, service TEXT NOT NULL,
                   status TEXT NOT NULL DEFAULT 'open', first_seen INTEGER NOT NULL,
                   last_seen INTEGER NOT NULL, event_count INTEGER NOT NULL DEFAULT 0,
                   last_trace_id TEXT, tags TEXT NOT NULL DEFAULT '{}'
                 )",
                (),
            )
            .await
            .expect("create legacy issues table");
    }

    let store = MetadataStore::open(&path).await.expect("migrate metadata");
    let conn = store.conn.lock().await;
    let mut columns = conn
        .query("PRAGMA table_info(issues)", ())
        .await
        .expect("read columns");
    let mut resolved_at = false;
    while let Some(row) = columns.next().await.expect("column row") {
        resolved_at |= text(&row, 1) == "resolved_at";
    }
    assert!(resolved_at, "bootstrap must add issue resolution time");
}
