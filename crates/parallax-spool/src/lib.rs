//! Raw OTLP frame durability, rotation, retention, and recovery.

mod spool;

pub use spool::{Signal, Spool, SpoolReclaim, SpoolRetention};
