//! Parallax storage adapters.
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]
//!
//! Everything engine-specific lives behind the adapter boundary defined here:
//! the `TelemetryStore` trait, the production `GreptimeDB` adapter, the Turso
//! metadata store for mutable product state, and the ingest spool. The
//! in-memory adapter is compiled only for tests and explicit test support.

#[expect(clippy::too_many_arguments, reason = "adapter contract")]
pub mod adapter;
#[expect(clippy::cast_precision_loss, reason = "bounded analytics ratios")]
mod adapter_math;
mod adapter_rules;
mod arrow_sql;
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "telemetry math"
)]
#[expect(
    clippy::cast_sign_loss,
    clippy::excessive_nesting,
    reason = "checked queries"
)]
pub mod greptime;
mod greptime_sql;
#[expect(clippy::excessive_nesting, reason = "transaction flow")]
pub mod metadata;
pub use parallax_model as model;
mod outcomes;
