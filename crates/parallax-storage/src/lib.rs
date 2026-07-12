//! Parallax storage adapters.
//!
//! Everything engine-specific lives behind the adapter boundary defined here:
//! the `TelemetryStore` trait, the production GreptimeDB adapter, the Turso
//! metadata store for mutable product state, and the ingest spool. The
//! in-memory adapter is compiled only for tests and explicit test support.

pub mod adapter;
mod arrow_sql;
#[cfg(any(test, feature = "test-support"))]
pub mod conformance;
pub mod greptime;
#[cfg(any(test, feature = "test-support"))]
pub mod memory;
pub mod metadata;
pub mod model;
pub mod spool;
