//! Turso-backed mutable metadata adapter and migrations.

mod port;
#[expect(clippy::excessive_nesting, reason = "transaction flow")]
mod turso;

pub use turso::pins::{
    EVIDENCE_PIN_MAX_BYTES, EVIDENCE_PIN_PROTECTION_CAP, EvidencePinProtection, EvidencePinRecord,
};
pub use turso::{
    ALERT_CHECKS_KEEP_PER_RULE, AgentSessionImportAccept, AgentSessionImportError,
    AgentSessionImportRecord, AlertCheckRecord, AlertDeliveryEventRecord, AlertDestinationRecord,
    AlertIncidentRecord, AlertRuleRecord, AlertRuleStateRecord, CiAttemptAccept,
    CiAttemptDeliveryRecord, CiAttemptStoreError, CiBackfillState, DeployAccept,
    DeployDeliveryRecord, DeployStoreError, EvidenceClaimRow, FixerOutcomeStoreRecord, SentryAck,
    SentryAckError, TursoMetadataStore, payload_sha256_hex,
};
// re-export keeps pin types public while the module stays crate-private
