//! Deploy events and the Reconciler's recurrence kernel.
//!
//! Deploy events follow the `parallax.deploy.v0` shape from
//! docs/research/architecture/integration-contract.md. Two loop mechanisms are
//! proven here (docs/research/architecture/autonomous-fix-loop.md):
//!
//! * Detect — `deploy_adjacent_regression`: a new error fingerprint shortly
//!   after a deploy escalates the trigger, with edge strength upgraded to
//!   strong when the deployed SHA matches the erroring service's
//!   `vcs.ref.head.revision` resource attribute.
//! * Validate — `reconcile_recurrence`: given a fix deploy and the observed
//!   event times for a fingerprint, decide whether the fix held. No wall
//!   clock: the observation horizon is passed in explicitly, so the verdict
//!   is reproducible.

use serde::{Deserialize, Serialize};

/// Default adjacency window: an error within 30 minutes of a deploy is
/// deploy-adjacent.
pub const ADJACENCY_WINDOW_NANOS: u128 = 30 * 60 * 1_000_000_000;

#[derive(Debug, Deserialize)]
pub struct DeploysData {
    #[serde(default)]
    pub deploys: Vec<DeployEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeployEvent {
    pub schema: String,
    pub project_id: String,
    pub release: String,
    pub vcs_sha: String,
    pub environment: String,
    pub status: String,
    pub finished_at_unix_nano: String,
}

impl DeployEvent {
    pub fn finished_nanos(&self) -> u128 {
        self.finished_at_unix_nano.parse().unwrap_or(0)
    }
}

/// Latest succeeded deploy that finished at or before the first error and
/// within the adjacency window.
pub fn find_adjacent_deploy<'a>(
    deploys: &'a [DeployEvent],
    first_error_nanos: u128,
    window_nanos: u128,
) -> Option<&'a DeployEvent> {
    deploys
        .iter()
        .filter(|d| d.status == "succeeded")
        .filter(|d| {
            let finished = d.finished_nanos();
            finished <= first_error_nanos && first_error_nanos - finished <= window_nanos
        })
        .max_by_key(|d| d.finished_nanos())
}

/// Outcome of a recurrence watch (the Reconciler's Stage-5 verdict).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RecurrenceVerdict {
    /// Fingerprint fired again inside the watch window: the fix did not hold.
    /// Feeds a `fix_worsened_issue`/recurrence outcome row and a
    /// `fix_regression` trigger.
    Recurred { events_in_window: usize },
    /// Watch window fully elapsed with no recurrence: `fix_addressed_issue`
    /// may be recorded as strong.
    Silent,
    /// Horizon has not reached the end of the window; no verdict yet.
    WindowOpen,
}

/// Decide recurrence for one fingerprint after a fix deploy.
///
/// `event_times` are the observed occurrence times for the fingerprint;
/// `observation_horizon_nanos` is the latest telemetry timestamp seen (passed
/// in, never read from the clock).
pub fn reconcile_recurrence(
    fix_deploy_finished_nanos: u128,
    event_times: &[u128],
    window_nanos: u128,
    observation_horizon_nanos: u128,
) -> RecurrenceVerdict {
    let window_end = fix_deploy_finished_nanos.saturating_add(window_nanos);
    let events_in_window = event_times
        .iter()
        .filter(|&&t| t > fix_deploy_finished_nanos && t <= window_end)
        .count();
    if events_in_window > 0 {
        RecurrenceVerdict::Recurred { events_in_window }
    } else if observation_horizon_nanos >= window_end {
        RecurrenceVerdict::Silent
    } else {
        RecurrenceVerdict::WindowOpen
    }
}
