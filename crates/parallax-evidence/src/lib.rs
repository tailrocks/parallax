//! Evidence projection, gap detection, and bounded bundle assembly.
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]

pub mod agent_session;
#[expect(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "bounded bundle estimates"
)]
pub mod bundle;
pub mod envelope;
pub mod gaps;
pub mod redaction_policy;
pub mod story;
