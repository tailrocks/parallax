//! Alerting v1 pure evaluation + delivery surface (plan 167).
//!
//! Preliminary: consecutive-breach state machine + delivery payload/backoff/
//! claim helpers. Peer must wire Turso schema/CRUD, evaluator/delivery I/O
//! loops, GraphQL, UI, and playground breach scenarios. Do not treat as Done.

mod delivery;
mod state_machine;

pub use delivery::{
    CLAIM_LEASE_SECS, DeliveryEventType, MAX_DELIVERY_ATTEMPTS, NotificationContext,
    backoff_after_failure, claim_expires_at, claim_is_available, is_dead_letter,
    slack_webhook_payload_json, unique_delivery_key, webhook_payload_json,
};
pub use state_machine::{
    AlertComparator, AlertMeasurement, AlertSeverity, AlertTransition, EvaluationOutcome,
    NoDataBehavior, RuleEvalConfig, RuleEvalState, evaluate_rule,
};
