//! Durable GitHub deploy delivery ledger (plan 121).

use super::*;
use sha2::{Digest, Sha256};

/// Normalized deploy row ready for durable storage (HTTP layer supplies this).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeployDeliveryRecord {
    pub delivery_id: String,
    pub provider: String,
    pub event_name: String,
    pub deployment_id: i64,
    pub repo_full_name: Option<String>,
    pub ref_name: Option<String>,
    pub commit_sha: Option<String>,
    pub environment: Option<String>,
    pub state: String,
    pub task: Option<String>,
    pub actor_login: Option<String>,
    pub edge_strength: String,
    pub lossiness: Vec<String>,
    pub payload_hash: String,
    pub received_at_nanos: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployAccept {
    /// First durable write for this delivery id.
    Inserted,
    /// Exact redelivery of the same payload hash.
    Duplicate,
}

#[derive(Debug)]
pub enum DeployStoreError {
    Collision(String),
    Internal(anyhow::Error),
}

impl std::fmt::Display for DeployStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collision(id) => {
                write!(f, "deploy delivery payload collision for delivery_id {id}")
            }
            Self::Internal(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DeployStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal(error) => Some(error.as_ref()),
            Self::Collision(_) => None,
        }
    }
}

impl From<anyhow::Error> for DeployStoreError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl TursoMetadataStore {
    /// Idempotent accept of a verified GitHub deploy delivery.
    ///
    /// Same `delivery_id` + same `payload_hash` → [`DeployAccept::Duplicate`].
    /// Same `delivery_id` + different hash → [`DeployStoreError::Collision`].
    pub async fn accept_deploy_delivery(
        &self,
        record: &DeployDeliveryRecord,
    ) -> Result<DeployAccept, DeployStoreError> {
        if record.delivery_id.trim().is_empty() || record.payload_hash.len() != 64 {
            return Err(DeployStoreError::Internal(anyhow::anyhow!(
                "delivery_id and 64-hex payload_hash are required"
            )));
        }
        let conn = self.conn.lock().await;
        let mut existing = conn
            .query(
                "SELECT payload_hash FROM deploy_deliveries WHERE delivery_id = ?1",
                (record.delivery_id.as_str(),),
            )
            .await
            .map_err(|e| DeployStoreError::Internal(e.into()))?;
        if let Some(row) = existing
            .next()
            .await
            .map_err(|e| DeployStoreError::Internal(e.into()))?
        {
            let hash = text(&row, 0);
            if hash == record.payload_hash {
                return Ok(DeployAccept::Duplicate);
            }
            return Err(DeployStoreError::Collision(record.delivery_id.clone()));
        }
        conn.execute(
            "INSERT INTO deploy_deliveries
               (delivery_id, provider, event_name, deployment_id, repo_full_name,
                ref_name, commit_sha, environment, state, task, actor_login,
                edge_strength, lossiness, payload_hash, received_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            (
                record.delivery_id.as_str(),
                record.provider.as_str(),
                record.event_name.as_str(),
                record.deployment_id,
                record.repo_full_name.clone(),
                record.ref_name.clone(),
                record.commit_sha.clone(),
                record.environment.clone(),
                record.state.as_str(),
                record.task.clone(),
                record.actor_login.clone(),
                record.edge_strength.as_str(),
                serde_json::to_string(&record.lossiness)
                    .map_err(|e| DeployStoreError::Internal(e.into()))?,
                record.payload_hash.as_str(),
                nanos_to_millis(record.received_at_nanos),
            ),
        )
        .await
        .map_err(|e| DeployStoreError::Internal(e.into()))?;
        Ok(DeployAccept::Inserted)
    }

    pub async fn deploy_delivery(
        &self,
        delivery_id: &str,
    ) -> anyhow::Result<Option<DeployDeliveryRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT delivery_id, provider, event_name, deployment_id, repo_full_name,
                        ref_name, commit_sha, environment, state, task, actor_login,
                        edge_strength, lossiness, payload_hash, received_at
                 FROM deploy_deliveries WHERE delivery_id = ?1",
                (delivery_id,),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| decode_deploy_delivery(&row))
            .transpose()
    }

    /// Inventory for `parallax doctor` deploy-context (plan 121 residual).
    pub async fn count_deploy_deliveries(&self) -> anyhow::Result<u64> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query("SELECT COUNT(*) FROM deploy_deliveries", ())
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing count row"))?;
        Ok(u64::try_from(integer(&row, 0)).unwrap_or(0))
    }
}

fn decode_deploy_delivery(row: &turso::Row) -> anyhow::Result<DeployDeliveryRecord> {
    Ok(DeployDeliveryRecord {
        delivery_id: text(row, 0),
        provider: text(row, 1),
        event_name: text(row, 2),
        deployment_id: integer(row, 3),
        repo_full_name: opt_text(row, 4),
        ref_name: opt_text(row, 5),
        commit_sha: opt_text(row, 6),
        environment: opt_text(row, 7),
        state: text(row, 8),
        task: opt_text(row, 9),
        actor_login: opt_text(row, 10),
        edge_strength: text(row, 11),
        lossiness: serde_json::from_str(&text(row, 12))?,
        payload_hash: text(row, 13),
        received_at_nanos: millis_to_nanos(integer(row, 14)),
    })
}

/// SHA-256 hex of raw webhook body bytes (idempotency payload key).
#[must_use]
pub fn payload_sha256_hex(body: &[u8]) -> String {
    let digest = Sha256::digest(body);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
