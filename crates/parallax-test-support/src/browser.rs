//! Browser product-contract seed/reset/readiness facade (plan 144).
//!
//! These APIs are internal test harness calls — never product HTTP endpoints
//! and never a product storage mode. Xtask/browser harness injects the
//! in-memory adapter at composition and drives reset through this module.

mod datasets;
mod real_stack;
mod seed;
mod seed_builders;

pub use datasets::{
    ALERT_DEST_PILOT_ID, ALERT_DEST_PILOT_NAME, ALERT_INCIDENT_PILOT_ID, ALERT_RULE_PILOT_ID,
    ALERT_RULE_PILOT_NAME, ANCHOR_TS_NANOS, CONTRACTS_TS_NANOS, DASHBOARD_PILOT_ID,
    DASHBOARD_PILOT_NAME,
    DASHBOARD_PILOT_WIDGET, DatasetId, INVESTIGATION_PILOT_ID, INVESTIGATION_PILOT_NAME,
    LOGS_PILOT_BODY, LOGS_PILOT_COUNT, LOGS_PILOT_SERVICE_A, LOGS_PILOT_SERVICE_B,
    METRICS_PILOT_GAUGE, METRICS_PILOT_HISTOGRAM, ScenarioManifest, TRACES_PILOT_CHILD_NAME,
    TRACES_PILOT_ERROR_NAME, TRACES_PILOT_ROOT_NAME, TRACES_PILOT_TRACE_ID, catalog, dataset_ids,
    manifest_for, pilot_investigation_state_json,
};
pub use real_stack::{
    RealStackIds, live_followup_log, live_followup_logs, live_followup_span, live_followup_spans,
    logs_request, metrics_request, traces_request,
};
pub use seed::{
    InvestigationSnapshot, clear_metadata, investigation_snapshot, postconditions_hold,
    reset_and_seed, seed_dataset,
};
