//! Parallax storage adapters.
//!
//! Everything engine-specific lives behind the adapter boundary defined here:
//! the `TelemetryStore` trait (in-memory now, GreptimeDB next), the Turso
//! metadata store for mutable product state, and the NDJSON ingest spool.

pub mod adapter;
pub mod greptime;
pub mod memory;
pub mod metadata;
pub mod model;
pub mod spool;
