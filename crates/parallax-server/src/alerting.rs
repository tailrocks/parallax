//! Alerting v1 pure evaluation + delivery surface (plan 167).
//!
//! Preliminary: consecutive-breach state machine + delivery payload/backoff/
//! claim helpers. Peer must wire Turso schema/CRUD, evaluator/delivery I/O
//! loops, GraphQL, UI, and playground breach scenarios. Do not treat as Done.

mod delivery;
mod delivery_worker;
mod evaluator;
#[cfg(test)]
#[path = "alerting/evaluator_bundle_tests.rs"]
mod evaluator_bundle_tests;
mod incident_bundle;
#[cfg(test)]
#[path = "alerting/incident_bundle_tests.rs"]
mod incident_bundle_tests;
mod measurement;
mod measurement_source;
mod preview;
#[cfg(test)]
#[path = "alerting/preview_tests.rs"]
mod preview_tests;

pub(crate) use measurement_source::AdapterMeasurementSource;

pub(crate) use measurement::{
    ServiceWindowStats, SignalType, groups_by_service, scalar_measurement, service_in_scope,
    span_measurements,
};

pub(crate) use delivery_worker::deliver_due_once;
mod state_machine;

pub(crate) use evaluator::{GroupMeasurement, MeasurementSource, eval_config, tick_once};
pub(crate) use preview::preview_rule;

pub(crate) use delivery::{
    DeliveryEventType, NotificationContext, backoff_after_failure, is_dead_letter,
    slack_webhook_payload_json, unique_delivery_key, webhook_payload_json,
};
pub(crate) use state_machine::{
    AlertComparator, AlertMeasurement, AlertSeverity, AlertTransition, EvaluationOutcome,
    NoDataBehavior, RuleEvalConfig, RuleEvalState, evaluate_rule,
};
