//! Evidence projection, gap detection, and bounded bundle assembly.
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]

pub mod agent_session;
#[expect(
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    reason = "bounded bundle estimates"
)]
pub mod bundle;
/// Claude Code stream-json / hook normalizer (plan 120).
pub mod claude_code;
pub mod envelope;
pub mod gaps;
/// GitHub Actions CI evidence normalizer (plan 124).
pub mod github_actions;
/// GitHub deploy/change webhook verify + normalize (plan 121).
pub mod github_deploy;
pub mod redaction_policy;
pub mod story;

pub use redaction_policy::{
    DETECTOR_POLICY_VERSION, EvidenceField, SOURCE_POLICY_VERSION, SourceAction, SourceDecision,
    decide, project_text, sanitize_text,
};
