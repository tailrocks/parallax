//! Cycle-safe reusable fakes, builders, and conformance scenarios.
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]

pub mod builders;
pub mod conformance;
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "bounded analytics math"
)]
mod memory;
mod normalizers;

fn warn_error<E: std::fmt::Display>(result: Result<(), E>, operation: &str) {
    if let Err(error) = result {
        tracing::warn!(%error, operation, "test-support operation failed");
    }
}
