//! Incident-anchored bundle window (plan 173). Shared by fire-time
//! assembly and GraphQL re-read so preview/incident windows cannot drift.

/// Identifier + rule snapshot for an alert-incident bundle anchor.
#[derive(Debug, Clone)]
pub struct IncidentAnchor {
    pub incident_id: String,
    pub rule_name: String,
    pub signal_type: String,
    pub severity: String,
    pub group_key: String,
    pub window_minutes: u32,
    pub last_value: Option<f64>,
}

const MINUTE_NS: u128 = 60 * 1_000_000_000;
/// Existing issue/trace pad is ±5 minutes; incident windows may be wider
/// but never exceed 60 minutes either side (metric-summary bound).
const MAX_HALF_MINUTES: u32 = 60;

/// Breach-centered window: ± `window_minutes`, clamped to 1..=60.
#[must_use]
pub fn incident_bundle_window(breach_nanos: u128, window_minutes: u32) -> (u128, u128) {
    let half = u128::from(window_minutes.clamp(1, MAX_HALF_MINUTES)) * MINUTE_NS;
    (
        breach_nanos.saturating_sub(half),
        breach_nanos.saturating_add(half),
    )
}
