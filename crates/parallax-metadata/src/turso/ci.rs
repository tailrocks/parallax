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
}
