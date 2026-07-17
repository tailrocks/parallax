//! Alerting v1 pure evaluation surface (plan 167).
//!
//! Preliminary: the consecutive-breach state machine only. Peer must wire
//! Turso schema/CRUD, evaluator/delivery loops, GraphQL, UI, and playground
//! breach scenarios. Do not treat this module as plan Done.

mod state_machine;

pub use state_machine::{
    AlertComparator, AlertMeasurement, AlertSeverity, AlertTransition, EvaluationOutcome,
    NoDataBehavior, RuleEvalConfig, RuleEvalState, evaluate_rule,
};
