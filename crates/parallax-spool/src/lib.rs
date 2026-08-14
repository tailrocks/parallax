//! Raw OTLP frame durability, rotation, retention, and recovery.

mod spool;

pub use spool::{
    Signal, Spool, SpoolHealth, SpoolPruneEstimate, SpoolReclaim, SpoolRetention, count_pspl_frames,
};
