//! Parallax storage adapters.
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
pub mod metadata;
pub use parallax_model as model;
