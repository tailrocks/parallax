//! Alerting v1 pure evaluation + delivery surface (plan 167).
//!
//! Preliminary: consecutive-breach state machine + delivery payload/backoff/
//! claim helpers. Peer must wire Turso schema/CRUD, evaluator/delivery I/O
//! loops, GraphQL, UI, and playground breach scenarios. Do not treat as Done.

mod delivery;
mod delivery_worker;
mod evaluator;
mod measurement;
mod measurement_source;

pub use measurement_source::{AdapterMeasurementSource, LOG_COUNT_SEVERITY_FLOOR};

pub use measurement::{
    ServiceWindowStats, SignalType, groups_by_service, scalar_measurement, service_in_scope,
    span_measurements,
};

pub use delivery_worker::{DELIVERY_LEASE_SECS, DeliveryReport, deliver_due_once};
mod state_machine;

pub use evaluator::{GroupMeasurement, MeasurementSource, TickReport, eval_config, tick_once};

pub use delivery::{
    CLAIM_LEASE_SECS, DeliveryEventType, MAX_DELIVERY_ATTEMPTS, NotificationContext,
    backoff_after_failure, claim_expires_at, claim_is_available, is_dead_letter,
    slack_webhook_payload_json, unique_delivery_key, webhook_payload_json,
};
pub use state_machine::{
    AlertComparator, AlertMeasurement, AlertSeverity, AlertTransition, EvaluationOutcome,
    NoDataBehavior, RuleEvalConfig, RuleEvalState, evaluate_rule,
};
