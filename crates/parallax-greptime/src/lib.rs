//! GreptimeDB native-table telemetry adapter.
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]

mod adapter {
    pub(crate) use parallax_storage::adapter::*;
}
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
mod greptime;
mod greptime_sql;
mod model {
    pub(crate) use parallax_model::*;
}
mod outcomes;

pub use greptime::GreptimeStore;
