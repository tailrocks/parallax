//! Parallax storage adapters.
//!
//! Everything engine-specific lives behind the adapter boundary defined here.
//! M0 ships the spool (NDJSON write-ahead landing zone for raw OTLP export
//! requests); the GreptimeDB, Turso, and in-memory adapters arrive in M1.

pub mod spool;
