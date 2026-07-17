//! Durable Claude Code session import ledger (plan 120 residual).
//!
//! Consent-only imports: callers normalize first; this module stores the
//! redacted structural projection, never raw transcripts by default.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSessionImportRecord {
    pub import_id: String,
    pub source_tool: String,
    pub source_version: Option<String>,
    pub capture_surface: String,
    pub session_id: Option<String>,
    pub payload_hash: String,
    pub action_count: u32,
    pub lossiness: Vec<String>,
    pub canonical_json: String,
    pub imported_at_nanos: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSessionImportAccept {
    Inserted,
    Duplicate,
}

#[derive(Debug)]
pub enum AgentSessionImportError {
    Collision(String),
    Internal(anyhow::Error),
}

impl std::fmt::Display for AgentSessionImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collision(id) => write!(f, "agent session import collision for {id}"),
            Self::Internal(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for AgentSessionImportError {}

impl From<anyhow::Error> for AgentSessionImportError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl TursoMetadataStore {
    pub async fn accept_agent_session_import(
        &self,
        record: &AgentSessionImportRecord,
    ) -> Result<AgentSessionImportAccept, AgentSessionImportError> {
        if record.import_id.trim().is_empty() || record.payload_hash.len() != 64 {
            return Err(AgentSessionImportError::Internal(anyhow::anyhow!(
                "import_id and 64-hex payload_hash are required"
            )));
        }
        let conn = self.conn.lock().await;
        let mut existing = conn
            .query(
                "SELECT payload_hash FROM agent_session_imports WHERE import_id = ?1",
                (record.import_id.as_str(),),
            )
            .await
            .map_err(|error| AgentSessionImportError::Internal(error.into()))?;
        if let Some(row) = existing
            .next()
            .await
            .map_err(|error| AgentSessionImportError::Internal(error.into()))?
        {
            if text(&row, 0) == record.payload_hash {
                return Ok(AgentSessionImportAccept::Duplicate);
            }
            return Err(AgentSessionImportError::Collision(record.import_id.clone()));
        }
        let lossiness = serde_json::to_string(&record.lossiness)
            .map_err(|error| AgentSessionImportError::Internal(error.into()))?;
        conn.execute(
            "INSERT INTO agent_session_imports
               (import_id, source_tool, source_version, capture_surface, session_id,
                payload_hash, action_count, lossiness, canonical_json, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                record.import_id.as_str(),
                record.source_tool.as_str(),
                record.source_version.clone(),
                record.capture_surface.as_str(),
                record.session_id.clone(),
                record.payload_hash.as_str(),
                i64::from(record.action_count),
                lossiness,
                record.canonical_json.as_str(),
                nanos_to_millis(record.imported_at_nanos),
            ),
        )
        .await
        .map_err(|error| AgentSessionImportError::Internal(error.into()))?;
        Ok(AgentSessionImportAccept::Inserted)
    }

    pub async fn count_agent_session_imports(&self) -> anyhow::Result<u64> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query("SELECT COUNT(*) FROM agent_session_imports", ())
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing count row"))?;
        Ok(u64::try_from(integer(&row, 0)).unwrap_or(0))
    }

    pub async fn agent_session_import(
        &self,
        import_id: &str,
    ) -> anyhow::Result<Option<AgentSessionImportRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT import_id, source_tool, source_version, capture_surface, session_id,
                        payload_hash, action_count, lossiness, canonical_json, imported_at
                 FROM agent_session_imports WHERE import_id = ?1",
                (import_id,),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| {
                Ok(AgentSessionImportRecord {
                    import_id: text(&row, 0),
                    source_tool: text(&row, 1),
                    source_version: opt_text(&row, 2),
                    capture_surface: text(&row, 3),
                    session_id: opt_text(&row, 4),
                    payload_hash: text(&row, 5),
                    action_count: u32::try_from(integer(&row, 6)).unwrap_or(0),
                    lossiness: serde_json::from_str(&text(&row, 7)).unwrap_or_default(),
                    canonical_json: text(&row, 8),
                    imported_at_nanos: millis_to_nanos(integer(&row, 9)),
                })
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn agent_session_import_is_idempotent_and_collision_safe() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("store");
        let record = AgentSessionImportRecord {
            import_id: "import-1".into(),
            source_tool: "claude_code".into(),
            source_version: Some("2.1.212".into()),
            capture_surface: "stream_json".into(),
            session_id: Some("sess-1".into()),
            payload_hash: "a".repeat(64),
            action_count: 3,
            lossiness: vec!["prompt_body_redacted".into()],
            canonical_json: r#"{"session_id":"sess-1"}"#.into(),
            imported_at_nanos: 1_000_000,
        };
        assert_eq!(
            store
                .accept_agent_session_import(&record)
                .await
                .expect("insert"),
            AgentSessionImportAccept::Inserted
        );
        assert_eq!(
            store
                .accept_agent_session_import(&record)
                .await
                .expect("dup"),
            AgentSessionImportAccept::Duplicate
        );
        let mut conflict = record.clone();
        conflict.payload_hash = "b".repeat(64);
        assert!(matches!(
            store.accept_agent_session_import(&conflict).await,
            Err(AgentSessionImportError::Collision(_))
        ));
        assert_eq!(store.count_agent_session_imports().await.expect("count"), 1);
    }
}
