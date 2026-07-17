//! Durable GitHub Actions attempt delivery ledger (plan 124).

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiAttemptDeliveryRecord {
    pub delivery_id: String,
    pub attempt_id: String,
    pub provider: String,
    pub repo_full_name: String,
    pub workflow_run_id: i64,
    pub job_id: i64,
    pub attempt: u32,
    pub conclusion: Option<String>,
    pub name: Option<String>,
    pub lossiness: Vec<String>,
    pub payload_hash: String,
    pub received_at_nanos: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiAttemptAccept {
    Inserted,
    Duplicate,
}

#[derive(Debug)]
pub enum CiAttemptStoreError {
    Collision(String),
    Internal(anyhow::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiBackfillState {
    pub repo_full_name: String,
    pub completed_at_nanos: u128,
    pub workflow_run_id: i64,
    pub etag: Option<String>,
    pub last_success_at_nanos: Option<u128>,
    pub last_error: Option<String>,
    pub rate_limit_reset_at_nanos: Option<u128>,
}

impl std::fmt::Display for CiAttemptStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collision(id) => {
                write!(
                    f,
                    "CI attempt delivery payload collision for delivery_id {id}"
                )
            }
            Self::Internal(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CiAttemptStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal(error) => Some(error.as_ref()),
            Self::Collision(_) => None,
        }
    }
}

impl From<anyhow::Error> for CiAttemptStoreError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl TursoMetadataStore {
    pub async fn accept_ci_attempt_delivery(
        &self,
        record: &CiAttemptDeliveryRecord,
    ) -> Result<CiAttemptAccept, CiAttemptStoreError> {
        if record.delivery_id.trim().is_empty()
            || record.attempt_id.trim().is_empty()
            || record.payload_hash.len() != 64
        {
            return Err(CiAttemptStoreError::Internal(anyhow::anyhow!(
                "delivery_id, attempt_id, and 64-hex payload_hash are required"
            )));
        }
        let conn = self.conn.lock().await;
        let mut existing = conn
            .query(
                "SELECT payload_hash FROM ci_attempt_deliveries WHERE delivery_id = ?1",
                (record.delivery_id.as_str(),),
            )
            .await
            .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
        let existing_row = existing
            .next()
            .await
            .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
        drop(existing);
        if let Some(row) = existing_row {
            if text(&row, 0) == record.payload_hash {
                return Ok(CiAttemptAccept::Duplicate);
            }
            return Err(CiAttemptStoreError::Collision(record.delivery_id.clone()));
        }
        upsert_ci_attempt(&conn, record).await?;
        conn.execute(
            "INSERT INTO ci_attempt_deliveries
               (delivery_id, attempt_id, payload_hash, received_at)
             VALUES (?1, ?2, ?3, ?4)",
            (
                record.delivery_id.as_str(),
                record.attempt_id.as_str(),
                record.payload_hash.as_str(),
                nanos_to_millis(record.received_at_nanos),
            ),
        )
        .await
        .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
        Ok(CiAttemptAccept::Inserted)
    }

    pub async fn count_ci_attempt_deliveries(&self) -> anyhow::Result<u64> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query("SELECT COUNT(*) FROM ci_attempt_deliveries", ())
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing count row"))?;
        Ok(u64::try_from(integer(&row, 0)).unwrap_or(0))
    }

    pub async fn count_ci_attempts(&self) -> anyhow::Result<u64> {
        let conn = self.conn.lock().await;
        let mut rows = conn.query("SELECT COUNT(*) FROM ci_attempts", ()).await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing count row"))?;
        Ok(u64::try_from(integer(&row, 0)).unwrap_or(0))
    }

    pub async fn ci_backfill_state(
        &self,
        repo_full_name: &str,
    ) -> anyhow::Result<Option<CiBackfillState>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT repo_full_name, completed_at, workflow_run_id, etag,
                        last_success_at, last_error, rate_limit_reset_at
                 FROM ci_backfill_state WHERE repo_full_name = ?1",
                (repo_full_name,),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| decode_backfill_state(&row))
            .transpose()
    }

    /// Advance only after a complete REST page has been durably persisted.
    pub async fn advance_ci_backfill(
        &self,
        repo_full_name: &str,
        completed_at_nanos: u128,
        workflow_run_id: i64,
        etag: Option<&str>,
        succeeded_at_nanos: u128,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO ci_backfill_state
               (repo_full_name, completed_at, workflow_run_id, etag, last_success_at,
                last_error, rate_limit_reset_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL)
             ON CONFLICT(repo_full_name) DO UPDATE SET
               completed_at = CASE
                 WHEN excluded.completed_at > completed_at
                   OR (excluded.completed_at = completed_at
                       AND excluded.workflow_run_id > workflow_run_id)
                 THEN excluded.completed_at ELSE completed_at END,
               workflow_run_id = CASE
                 WHEN excluded.completed_at > completed_at
                   OR (excluded.completed_at = completed_at
                       AND excluded.workflow_run_id > workflow_run_id)
                 THEN excluded.workflow_run_id ELSE workflow_run_id END,
               etag = excluded.etag,
               last_success_at = excluded.last_success_at,
               last_error = NULL,
               rate_limit_reset_at = NULL",
            (
                repo_full_name,
                nanos_to_millis(completed_at_nanos),
                workflow_run_id,
                etag,
                nanos_to_millis(succeeded_at_nanos),
            ),
        )
        .await?;
        Ok(())
    }

    /// Record a failed/rate-limited tick without moving its durable cursor.
    pub async fn fail_ci_backfill(
        &self,
        repo_full_name: &str,
        error: &str,
        rate_limit_reset_at_nanos: Option<u128>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO ci_backfill_state
               (repo_full_name, last_error, rate_limit_reset_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(repo_full_name) DO UPDATE SET
               last_error = excluded.last_error,
               rate_limit_reset_at = excluded.rate_limit_reset_at",
            (
                repo_full_name,
                error,
                rate_limit_reset_at_nanos.map(nanos_to_millis),
            ),
        )
        .await?;
        Ok(())
    }

    /// Newest-first attempt inventory for bundle correlation (bounded).
    pub async fn list_ci_attempts_for_repo(
        &self,
        repo_full_name: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<CiAttemptDeliveryRecord>> {
        let limit = i64::try_from(limit.clamp(1, 100)).unwrap_or(100);
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT attempt_id, provider, repo_full_name, workflow_run_id, job_id,
                        attempt, conclusion, name, lossiness, updated_at
                 FROM ci_attempts
                 WHERE repo_full_name = ?1
                 ORDER BY updated_at DESC, attempt_id DESC
                 LIMIT ?2",
                (repo_full_name, limit),
            )
            .await?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            let lossiness: Vec<String> = serde_json::from_str(&text(&row, 8)).unwrap_or_default();
            out.push(CiAttemptDeliveryRecord {
                delivery_id: String::new(),
                attempt_id: text(&row, 0),
                provider: text(&row, 1),
                repo_full_name: text(&row, 2),
                workflow_run_id: integer(&row, 3),
                job_id: integer(&row, 4),
                attempt: u32::try_from(integer(&row, 5)).unwrap_or(u32::MAX),
                conclusion: opt_text(&row, 6),
                name: opt_text(&row, 7),
                lossiness,
                payload_hash: String::new(),
                received_at_nanos: millis_to_nanos(integer(&row, 9)),
            });
        }
        Ok(out)
    }
}

fn decode_backfill_state(row: &turso::Row) -> anyhow::Result<CiBackfillState> {
    Ok(CiBackfillState {
        repo_full_name: text(row, 0),
        completed_at_nanos: millis_to_nanos(integer(row, 1)),
        workflow_run_id: integer(row, 2),
        etag: opt_text(row, 3),
        last_success_at_nanos: opt_integer(row, 4).map(millis_to_nanos),
        last_error: opt_text(row, 5),
        rate_limit_reset_at_nanos: opt_integer(row, 6).map(millis_to_nanos),
    })
}

async fn upsert_ci_attempt(
    conn: &turso::Connection,
    record: &CiAttemptDeliveryRecord,
) -> Result<(), CiAttemptStoreError> {
    let mut rows = conn
        .query(
            "SELECT conclusion FROM ci_attempts WHERE attempt_id = ?1",
            (record.attempt_id.as_str(),),
        )
        .await
        .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
    let row = rows
        .next()
        .await
        .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
    drop(rows);
    let lossiness = serde_json::to_string(&record.lossiness)
        .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
    if let Some(row) = row {
        let existing = opt_text(&row, 0);
        if existing.is_some() && record.conclusion.is_some() && existing != record.conclusion {
            return Err(CiAttemptStoreError::Collision(record.attempt_id.clone()));
        }
        conn.execute(
            "UPDATE ci_attempts
             SET conclusion = COALESCE(?1, conclusion), name = COALESCE(?2, name),
                 lossiness = ?3, updated_at = ?4 WHERE attempt_id = ?5",
            (
                record.conclusion.clone(),
                record.name.clone(),
                lossiness,
                nanos_to_millis(record.received_at_nanos),
                record.attempt_id.as_str(),
            ),
        )
        .await
        .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
    } else {
        conn.execute(
            "INSERT INTO ci_attempts
               (attempt_id, provider, repo_full_name, workflow_run_id, job_id,
                attempt, conclusion, name, lossiness, first_seen_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            (
                record.attempt_id.as_str(),
                record.provider.as_str(),
                record.repo_full_name.as_str(),
                record.workflow_run_id,
                record.job_id,
                i64::from(record.attempt),
                record.conclusion.clone(),
                record.name.clone(),
                lossiness,
                nanos_to_millis(record.received_at_nanos),
            ),
        )
        .await
        .map_err(|error| CiAttemptStoreError::Internal(error.into()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(
        delivery_id: &str,
        hash_byte: char,
        conclusion: Option<&str>,
    ) -> CiAttemptDeliveryRecord {
        CiAttemptDeliveryRecord {
            delivery_id: delivery_id.into(),
            attempt_id: "github_actions:tailrocks/parallax:10:20:1".into(),
            provider: "github_actions".into(),
            repo_full_name: "tailrocks/parallax".into(),
            workflow_run_id: 10,
            job_id: 20,
            attempt: 1,
            conclusion: conclusion.map(str::to_owned),
            name: Some("test".into()),
            lossiness: Vec::new(),
            payload_hash: std::iter::repeat_n(hash_byte, 64).collect(),
            received_at_nanos: 1_000_000,
        }
    }

    #[tokio::test]
    async fn attempt_identity_unifies_lifecycle_and_delivery_redelivery() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("store");
        let queued = record("delivery-1", 'a', None);
        assert_eq!(
            store
                .accept_ci_attempt_delivery(&queued)
                .await
                .expect("queued"),
            CiAttemptAccept::Inserted
        );
        assert_eq!(
            store
                .accept_ci_attempt_delivery(&queued)
                .await
                .expect("redelivery"),
            CiAttemptAccept::Duplicate
        );
        let completed = record("delivery-2", 'b', Some("success"));
        assert_eq!(
            store
                .accept_ci_attempt_delivery(&completed)
                .await
                .expect("completed"),
            CiAttemptAccept::Inserted
        );
        assert_eq!(store.count_ci_attempts().await.expect("attempts"), 1);
        assert_eq!(
            store
                .count_ci_attempt_deliveries()
                .await
                .expect("deliveries"),
            2
        );

        let conflicting = record("delivery-3", 'c', Some("failure"));
        assert!(matches!(
            store.accept_ci_attempt_delivery(&conflicting).await,
            Err(CiAttemptStoreError::Collision(_))
        ));
        let mut collided_delivery = queued;
        collided_delivery.payload_hash = "d".repeat(64);
        assert!(matches!(
            store.accept_ci_attempt_delivery(&collided_delivery).await,
            Err(CiAttemptStoreError::Collision(_))
        ));
    }

    #[tokio::test]
    async fn backfill_cursor_is_monotonic_restart_safe_and_failure_stable() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("meta.db");
        let store = TursoMetadataStore::open(&path).await.expect("store");
        store
            .advance_ci_backfill(
                "tailrocks/parallax",
                2_000_000,
                20,
                Some("etag-1"),
                3_000_000,
            )
            .await
            .expect("advance");
        store
            .advance_ci_backfill(
                "tailrocks/parallax",
                2_000_000,
                19,
                Some("etag-2"),
                4_000_000,
            )
            .await
            .expect("ignore regression");
        store
            .fail_ci_backfill("tailrocks/parallax", "rate limited", Some(9_000_000))
            .await
            .expect("failure");
        drop(store);

        let store = TursoMetadataStore::open(path).await.expect("restart");
        let state = store
            .ci_backfill_state("tailrocks/parallax")
            .await
            .expect("state")
            .expect("present");
        assert_eq!(
            (state.completed_at_nanos, state.workflow_run_id),
            (2_000_000, 20)
        );
        assert_eq!(state.etag.as_deref(), Some("etag-2"));
        assert_eq!(state.last_success_at_nanos, Some(4_000_000));
        assert_eq!(state.last_error.as_deref(), Some("rate limited"));
        assert_eq!(state.rate_limit_reset_at_nanos, Some(9_000_000));
    }
}
