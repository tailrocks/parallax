//! Parallax domain logic.
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]
//!
//! Graduates the mechanisms proven in `poc/evidence-loop` (error derivation,
//! fingerprinting, grouping, bundle assembly, bounding, redaction,
//! hypotheses) onto the real OTLP protocol types. Filled milestone by
//! milestone; M0 ships the crate skeleton.

pub mod agent_session;
#[expect(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "bounded bundle estimates"
)]
pub mod bundle;
pub mod gaps;
pub mod story;
