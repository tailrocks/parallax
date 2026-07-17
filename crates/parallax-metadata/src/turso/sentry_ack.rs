//! Sentry event-id acknowledgement ledger (plan 118 residual).

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentryAck {
    /// First durable accept for `(project_id, event_id)`.
    Inserted,
    /// Exact redelivery of the same payload hash.
    Duplicate,
}

#[derive(Debug)]
pub enum SentryAckError {
    Collision {
        project_id: String,
        event_id: String,
    },
    Internal(anyhow::Error),
}

impl std::fmt::Display for SentryAckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collision {
                project_id,
                event_id,
            } => write!(
                f,
                "sentry event_id collision for project {project_id} event {event_id}"
            ),
            Self::Internal(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for SentryAckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal(error) => Some(error.as_ref()),
            Self::Collision { .. } => None,
        }
    }
}

impl From<anyhow::Error> for SentryAckError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl TursoMetadataStore {
    /// Record or verify a Sentry event_id after durable spool accept.
    pub async fn accept_sentry_event_ack(
        &self,
        project_id: &str,
        event_id: &str,
        payload_hash: &str,
        received_at_nanos: u128,
    ) -> Result<SentryAck, SentryAckError> {
        if project_id.is_empty() || event_id.len() != 32 || payload_hash.len() != 64 {
            return Err(SentryAckError::Internal(anyhow::anyhow!(
                "project_id, 32-hex event_id, and 64-hex payload_hash are required"
            )));
        }
        let conn = self.conn.lock().await;
        let mut existing = conn
            .query(
                "SELECT payload_hash FROM sentry_event_acks
                 WHERE project_id = ?1 AND event_id = ?2",
                (project_id, event_id),
            )
            .await
            .map_err(|e| SentryAckError::Internal(e.into()))?;
        if let Some(row) = existing
            .next()
            .await
            .map_err(|e| SentryAckError::Internal(e.into()))?
        {
            let hash = text(&row, 0);
            if hash == payload_hash {
                return Ok(SentryAck::Duplicate);
            }
            return Err(SentryAckError::Collision {
                project_id: project_id.to_string(),
                event_id: event_id.to_string(),
            });
        }
        conn.execute(
            "INSERT INTO sentry_event_acks (project_id, event_id, payload_hash, received_at)
             VALUES (?1, ?2, ?3, ?4)",
            (
                project_id,
                event_id,
                payload_hash,
                nanos_to_millis(received_at_nanos),
            ),
        )
        .await
        .map_err(|e| SentryAckError::Internal(e.into()))?;
        Ok(SentryAck::Inserted)
    }
}
