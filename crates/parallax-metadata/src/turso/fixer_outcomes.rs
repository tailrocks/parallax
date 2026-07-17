//! Append-only fixer outcome rows (plan 123 residual).
//!
//! Never updates terminal truth in place — each transition is a new row keyed
//! by `(request_id, immutable_hash)`.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixerOutcomeStoreRecord {
    pub request_id: String,
    pub phase: String,
    pub terminal: Option<String>,
    pub draft_pr_opened: bool,
    pub human_review_ok: bool,
    pub runtime_recurrence: bool,
    pub immutable_hash: String,
    pub canonical_json: String,
    pub recorded_at_nanos: u128,
}

impl TursoMetadataStore {
    pub async fn append_fixer_outcome(
        &self,
        record: &FixerOutcomeStoreRecord,
    ) -> anyhow::Result<()> {
        if record.request_id.trim().is_empty() || record.immutable_hash.len() != 64 {
            anyhow::bail!("request_id and 64-hex immutable_hash required");
        }
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR IGNORE INTO fixer_outcomes
               (request_id, phase, terminal, draft_pr_opened, human_review_ok,
                runtime_recurrence, immutable_hash, canonical_json, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                record.request_id.as_str(),
                record.phase.as_str(),
                record.terminal.clone(),
                i64::from(record.draft_pr_opened),
                i64::from(record.human_review_ok),
                i64::from(record.runtime_recurrence),
                record.immutable_hash.as_str(),
                record.canonical_json.as_str(),
                nanos_to_millis(record.recorded_at_nanos),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn count_fixer_outcomes(&self) -> anyhow::Result<u64> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query("SELECT COUNT(*) FROM fixer_outcomes", ())
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing count"))?;
        Ok(u64::try_from(integer(&row, 0)).unwrap_or(0))
    }

    pub async fn latest_fixer_outcome(
        &self,
        request_id: &str,
    ) -> anyhow::Result<Option<FixerOutcomeStoreRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT request_id, phase, terminal, draft_pr_opened, human_review_ok,
                        runtime_recurrence, immutable_hash, canonical_json, recorded_at
                 FROM fixer_outcomes
                 WHERE request_id = ?1
                 ORDER BY recorded_at DESC, immutable_hash DESC
                 LIMIT 1",
                (request_id,),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| {
                Ok(FixerOutcomeStoreRecord {
                    request_id: text(&row, 0),
                    phase: text(&row, 1),
                    terminal: opt_text(&row, 2),
                    draft_pr_opened: integer(&row, 3) != 0,
                    human_review_ok: integer(&row, 4) != 0,
                    runtime_recurrence: integer(&row, 5) != 0,
                    immutable_hash: text(&row, 6),
                    canonical_json: text(&row, 7),
                    recorded_at_nanos: millis_to_nanos(integer(&row, 8)),
                })
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn append_only_preserves_history_hashes() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("store");
        let first = FixerOutcomeStoreRecord {
            request_id: "fixreq_x".into(),
            phase: "requested".into(),
            terminal: None,
            draft_pr_opened: false,
            human_review_ok: false,
            runtime_recurrence: false,
            immutable_hash: "a".repeat(64),
            canonical_json: "{}".into(),
            recorded_at_nanos: 1,
        };
        store.append_fixer_outcome(&first).await.expect("first");
        let second = FixerOutcomeStoreRecord {
            phase: "draft_pr_opened".into(),
            draft_pr_opened: true,
            immutable_hash: "b".repeat(64),
            recorded_at_nanos: 2,
            ..first.clone()
        };
        store.append_fixer_outcome(&second).await.expect("second");
        assert_eq!(store.count_fixer_outcomes().await.expect("count"), 2);
        let latest = store
            .latest_fixer_outcome("fixreq_x")
            .await
            .expect("latest")
            .expect("present");
        assert!(latest.draft_pr_opened);
        assert_eq!(latest.immutable_hash, "b".repeat(64));
    }
}
