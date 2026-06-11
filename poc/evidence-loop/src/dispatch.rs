//! Dispatch: the `parallax.fix_candidate.v0` wake payload.
//!
//! Stage 3 of docs/research/architecture/autonomous-fix-loop.md: how a fixer
//! learns there is work. The payload references the bundle (never inlines it —
//! redaction/projection stay on one audited path), carries the computed
//! autonomy budget, and is idempotent per (issue, bundle). `expires_at` is
//! derived from the bundle's telemetry-anchored generation time plus a TTL —
//! no wall clock, so emission stays reproducible.

use crate::budget::AutonomyBudget;
use crate::bundle::Bundle;
use serde::Serialize;

pub const EVENT_TYPE: &str = "parallax.fix_candidate.v0";

/// Dispatch TTL: 24 hours, in nanoseconds.
pub const DISPATCH_TTL_NANOS: u128 = 24 * 60 * 60 * 1_000_000_000;

#[derive(Debug, Serialize)]
pub struct FixCandidate {
    pub event_type: String,
    pub project_id: String,
    pub issue_id: String,
    pub trigger: String,
    pub bundle_ref: String,
    pub canonical_bundle_hash: String,
    pub autonomy_budget: AutonomyBudget,
    pub required_validation: Vec<String>,
    pub expires_at_unix_nano: String,
    pub idempotency_key: String,
}

/// Build the wake payload for one bundle under a computed budget.
pub fn build_fix_candidate(
    bundle: &Bundle,
    budget: AutonomyBudget,
    required_validation: Vec<String>,
) -> FixCandidate {
    let issue_id = format!("iss_{}", bundle.anchor.fingerprint);
    let generated_at: u128 = bundle.generated_at_unix_nano.parse().unwrap_or(0);
    FixCandidate {
        event_type: EVENT_TYPE.to_string(),
        project_id: bundle.project.clone(),
        issue_id: issue_id.clone(),
        trigger: bundle.trigger.r#type.clone(),
        bundle_ref: bundle.bundle_id.clone(),
        canonical_bundle_hash: bundle
            .canonical_hash
            .clone()
            .unwrap_or_else(|| "sha256:missing".to_string()),
        autonomy_budget: budget,
        required_validation,
        expires_at_unix_nano: generated_at.saturating_add(DISPATCH_TTL_NANOS).to_string(),
        idempotency_key: format!("{issue_id}:{}", bundle.bundle_id),
    }
}
