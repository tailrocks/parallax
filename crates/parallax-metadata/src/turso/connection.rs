use super::*;

#[expect(
    clippy::too_many_lines,
    reason = "ordered migration statements document the complete schema lifecycle"
)]
async fn apply_schema_migrations(conn: &turso::Connection) -> anyhow::Result<()> {
    let version = {
        let mut rows = conn.query("PRAGMA user_version", ()).await?;
        let Some(row) = rows.next().await? else {
            anyhow::bail!("PRAGMA user_version returned no row");
        };
        i32::try_from(integer(&row, 0))?
    };
    if version > SCHEMA_USER_VERSION {
        anyhow::bail!(
            "metadata schema user_version {version} is newer than supported {SCHEMA_USER_VERSION}"
        );
    }
    if version < 1 {
        conn.execute("DROP TABLE IF EXISTS runs", ()).await?;
    }
    if version < 2 {
        let mut columns = conn.query("PRAGMA table_info(issues)", ()).await?;
        let mut has_resolved_at = false;
        while let Some(row) = columns.next().await? {
            has_resolved_at |= text(&row, 1) == "resolved_at";
        }
        drop(columns);
        if !has_resolved_at {
            conn.execute("ALTER TABLE issues ADD COLUMN resolved_at INTEGER", ())
                .await?;
        }
    }
    if version < 3 {
        for column in [
            "ALTER TABLE alert_incidents ADD COLUMN bundle_hash TEXT",
            "ALTER TABLE alert_incidents ADD COLUMN bundle_assembled_at INTEGER",
            "ALTER TABLE alert_incidents ADD COLUMN bundle_top_hypothesis TEXT",
            "ALTER TABLE alert_incidents ADD COLUMN bundle_deploy_adjacency TEXT",
            "ALTER TABLE alert_incidents ADD COLUMN bundle_error TEXT",
        ] {
            drop(conn.execute(column, ()).await);
        }
    }
    if version < 4 {
        // Issue identity becomes (service, fingerprint): rebuild the issue
        // tables from their fingerprint-keyed shapes. Pre-v4 rows carry one
        // service per fingerprint (first-writer freeze), so the copy is 1:1
        // for issues; buckets and occurrences inherit the parent's service.
        // Fresh databases already bootstrap the new shape — rebuild only when
        // `service` is not yet part of the issues primary key.
        let mut columns = conn.query("PRAGMA table_info(issues)", ()).await?;
        let mut service_in_pk = false;
        while let Some(row) = columns.next().await? {
            service_in_pk |= text(&row, 1) == "service" && integer(&row, 5) > 0;
        }
        drop(columns);
        if !service_in_pk {
            conn.execute(
                "CREATE TABLE issues_identity (
              service       TEXT NOT NULL,
              fingerprint   TEXT NOT NULL,
              title         TEXT NOT NULL,
              error_type    TEXT NOT NULL,
              culprit       TEXT,
              status        TEXT NOT NULL DEFAULT 'open',
              resolved_at   INTEGER,
              first_seen    INTEGER NOT NULL,
              last_seen     INTEGER NOT NULL,
              event_count   INTEGER NOT NULL DEFAULT 0,
              last_trace_id TEXT,
              tags          TEXT NOT NULL DEFAULT '{}',
              PRIMARY KEY (service, fingerprint)
            )",
                (),
            )
            .await?;
            conn.execute(
                "INSERT OR REPLACE INTO issues_identity
               (service, fingerprint, title, error_type, culprit, status,
                resolved_at, first_seen, last_seen, event_count, last_trace_id, tags)
             SELECT service, fingerprint, title, error_type, culprit, status,
                    resolved_at, first_seen, last_seen, event_count, last_trace_id, tags
             FROM issues",
                (),
            )
            .await?;
            conn.execute("DROP TABLE issues", ()).await?;
            conn.execute("ALTER TABLE issues_identity RENAME TO issues", ())
                .await?;
            conn.execute(
                "CREATE TABLE issue_buckets_identity (
              service     TEXT NOT NULL,
              fingerprint TEXT NOT NULL,
              bucket_ts   INTEGER NOT NULL,
              count       INTEGER NOT NULL DEFAULT 0,
              PRIMARY KEY (service, fingerprint, bucket_ts)
            )",
                (),
            )
            .await?;
            conn.execute(
                "INSERT OR REPLACE INTO issue_buckets_identity
               (service, fingerprint, bucket_ts, count)
             SELECT i.service, b.fingerprint, b.bucket_ts, b.count
             FROM issue_buckets b JOIN issues i USING (fingerprint)",
                (),
            )
            .await?;
            conn.execute("DROP TABLE issue_buckets", ()).await?;
            conn.execute(
                "ALTER TABLE issue_buckets_identity RENAME TO issue_buckets",
                (),
            )
            .await?;
            drop(
                conn.execute(
                    "ALTER TABLE issue_occurrences ADD COLUMN service TEXT NOT NULL DEFAULT ''",
                    (),
                )
                .await,
            );
            conn.execute(
                "UPDATE issue_occurrences
             SET service = (SELECT i.service FROM issues i
                            WHERE i.fingerprint = issue_occurrences.fingerprint)
             WHERE service = ''",
                (),
            )
            .await?;
        }
    }
    if version < 5 {
        // CLI child-output capture: nullable text heads + truncation counts.
        // Fresh databases already bootstrap the shape via SCHEMA.
        let mut columns = conn.query("PRAGMA table_info(invocations)", ()).await?;
        let mut present = std::collections::HashSet::new();
        while let Some(row) = columns.next().await? {
            present.insert(text(&row, 1));
        }
        drop(columns);
        for (column, statement) in [
            (
                "stdout_text",
                "ALTER TABLE invocations ADD COLUMN stdout_text TEXT",
            ),
            (
                "stdout_truncated_bytes",
                "ALTER TABLE invocations ADD COLUMN stdout_truncated_bytes INTEGER NOT NULL DEFAULT 0",
            ),
            (
                "stderr_text",
                "ALTER TABLE invocations ADD COLUMN stderr_text TEXT",
            ),
            (
                "stderr_truncated_bytes",
                "ALTER TABLE invocations ADD COLUMN stderr_truncated_bytes INTEGER NOT NULL DEFAULT 0",
            ),
        ] {
            if !present.contains(column) {
                conn.execute(statement, ()).await?;
            }
        }
    }
    if version < 6 {
        // R1 artifact store: fresh databases already bootstrap the shape via
        // SCHEMA; this adopts pre-v6 databases.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS source_maps (
               service     TEXT NOT NULL,
               version     TEXT NOT NULL,
               file        TEXT NOT NULL,
               debug_id    TEXT,
               uploaded_at INTEGER NOT NULL,
               map_bytes   INTEGER NOT NULL DEFAULT 0,
               map_sha256  TEXT NOT NULL DEFAULT '',
               map_json    TEXT NOT NULL,
               PRIMARY KEY (service, version, file)
             )",
            (),
        )
        .await?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS source_maps_service_version
             ON source_maps(service, version)",
            (),
        )
        .await?;
    }
    conn.execute(&format!("PRAGMA user_version = {SCHEMA_USER_VERSION}"), ())
        .await?;
    Ok(())
}

impl TursoMetadataStore {
    pub async fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = turso::Builder::new_local(path.as_ref().to_string_lossy().as_ref())
            .build()
            .await?;
        let conn = db.connect()?;
        for statement in SCHEMA.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            conn.execute(statement, ()).await?;
        }
        apply_schema_migrations(&conn).await?;
        Ok(Self {
            conn: tokio::sync::Mutex::new(conn),
        })
    }

    /// Wipe alert tables so browser dataset reset is deterministic.
    pub async fn alert_reset(&self) {
        let conn = self.conn.lock().await;
        for table in [
            "alert_delivery_events",
            "alert_checks",
            "alert_incidents",
            "alert_rule_states",
            "alert_rules",
            "alert_destinations",
        ] {
            let _deleted = conn.execute(&format!("DELETE FROM {table}"), ()).await;
        }
    }
}
