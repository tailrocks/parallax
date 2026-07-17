//! Browser product-contract seed/reset/readiness facade (plan 144).
//!
//! These APIs are internal test harness calls — never product HTTP endpoints
//! and never a product storage mode. Xtask/browser harness injects the
//! in-memory adapter at composition and drives reset through this module.

mod datasets;
mod seed;

pub use datasets::{
    ANCHOR_TS_NANOS, DatasetId, INVESTIGATION_PILOT_ID, INVESTIGATION_PILOT_NAME, ScenarioManifest,
    catalog, dataset_ids, manifest_for, pilot_investigation_state_json,
};
pub use seed::{
    InvestigationSnapshot, clear_metadata, investigation_snapshot, postconditions_hold,
    reset_and_seed, seed_dataset,
};
