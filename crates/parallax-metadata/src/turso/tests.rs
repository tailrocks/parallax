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
    for _ in 0..2 {
        store
            .upsert_issue_occurrence(&occurrence("fp1", "svc", 1_000_000_000, &attrs))
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
        .ensure_run("jk-run-1", 5_000_000_000)
        .await
        .expect("ensure");
    store
        .ensure_run("jk-run-1", 9_000_000_000)
        .await
        .expect("ensure again");
    let run = store
        .run("jk-run-1")
        .await
        .expect("run")
        .expect("registered");
    assert_eq!(run.status, "external");
    assert_eq!(run.started_at_nanos, 5_000_000_000);

    // A wrapper-started run keeps its own record.
    store
        .start_run("run_cli", Some("cargo test"), 1_000_000_000)
        .await
        .expect("start");
    store
        .ensure_run("run_cli", 2_000_000_000)
        .await
        .expect("ensure existing");
    let cli_run = store.run("run_cli").await.expect("run").expect("present");
    assert_eq!(cli_run.status, "running");
    assert_eq!(cli_run.command.as_deref(), Some("cargo test"));
}
