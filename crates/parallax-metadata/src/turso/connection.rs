use super::*;

impl TursoMetadataStore {
    pub async fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = turso::Builder::new_local(path.as_ref().to_string_lossy().as_ref())
            .build()
            .await?;
        let conn = db.connect()?;
        for statement in SCHEMA.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            conn.execute(statement, ()).await?;
        }
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
        Ok(Self {
            conn: tokio::sync::Mutex::new(conn),
        })
    }
}
