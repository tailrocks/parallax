use crate::{EvidencePinRecord, TursoMetadataStore};

fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("metadata.db");
    (directory, path)
}

#[tokio::test]
async fn upgrade_v0_issue_row_survives_current_schema() {
    let (_directory, path) = temp_db();
    {
        let db = turso::Builder::new_local(path.to_string_lossy().as_ref())
            .build()
            .await
            .expect("raw db");
        let conn = db.connect().expect("connect");
        conn.execute(
            "CREATE TABLE issues (
                fingerprint TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                error_type TEXT NOT NULL,
                culprit TEXT,
                service TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                first_seen INTEGER NOT NULL,
                last_seen INTEGER NOT NULL,
                event_count INTEGER NOT NULL DEFAULT 0,
                last_trace_id TEXT,
                tags TEXT NOT NULL DEFAULT '{}'
            )",
            (),
        )
        .await
        .expect("issues");
        conn.execute(
            "INSERT INTO issues (fingerprint, title, error_type, service, first_seen, last_seen, event_count, tags)
             VALUES ('old', 'kept-title', 'E', 'svc', 2, 2, 1, '{}')",
            (),
        )
        .await
        .expect("row");
    }
    let store = TursoMetadataStore::open(&path).await.expect("adopt");
    let issue = store.issue("old").await.expect("read").expect("present");
    assert_eq!(issue.title, "kept-title");
    assert_eq!(issue.service, "svc");
    let pin = EvidencePinRecord {
        pin_id: "pin-old".into(),
        anchor_kind: "issue".into(),
        anchor_id: "old".into(),
        schema_version: "bundle-v1".into(),
        canonical_hash: "sha256-jcs:upgrade-canary".into(),
        bundle_json: r#"{"schema_version":"bundle-v1"}"#.into(),
        byte_len: 32,
        pinned_at_nanos: 1,
        expires_at_nanos: None,
        pinned_by: "upgrade".into(),
        source_state: "present".into(),
    };
    store.evidence_pin_upsert(&pin).await.expect("pin");
    drop(store);
    let reopened = TursoMetadataStore::open(&path).await.expect("reopen");
    let kept = reopened
        .evidence_pin("pin-old")
        .await
        .expect("read pin")
        .expect("present");
    assert_eq!(kept.canonical_hash, "sha256-jcs:upgrade-canary");
}
