use super::*;

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
