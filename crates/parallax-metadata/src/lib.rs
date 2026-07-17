//! Turso-backed mutable metadata adapter and migrations.

mod port;
#[expect(clippy::excessive_nesting, reason = "transaction flow")]
mod turso;

pub use turso::{
    ALERT_CHECKS_KEEP_PER_RULE, AlertCheckRecord, AlertDeliveryEventRecord, AlertDestinationRecord,
    AlertIncidentRecord, AlertRuleRecord, AlertRuleStateRecord, TursoMetadataStore,
};
