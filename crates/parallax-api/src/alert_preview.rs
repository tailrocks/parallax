//! Alert-rule preview contract (plan 171 feature 1).
//!
//! The live evaluator lives in `parallax-server`. The GraphQL crate only owns
//! the request/response shape and a trait the server implements with the
//! existing measurement path + pure state machine.

use anyhow::Result;
use parallax_metadata::AlertRuleRecord;
use std::future::Future;
use std::pin::Pin;

/// One measured bucket in a preview series.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreviewPointData {
    pub ts_nanos: String,
    pub value: Option<f64>,
    pub sample_count: u64,
    pub would_fire: bool,
}

/// Per-group preview outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreviewGroupData {
    pub group_key: String,
    pub points: Vec<AlertPreviewPointData>,
    pub samples_sufficient: bool,
}

/// Full preview payload (no persistence).
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreviewData {
    pub window_minutes: u32,
    pub groups: Vec<AlertPreviewGroupData>,
}

/// Server-provided preview runner. Implementations must reuse the live
/// measurement source and state machine and must not write incidents.
pub trait AlertPreviewer: Send + Sync {
    fn preview(
        &self,
        rule: AlertRuleRecord,
        window_minutes: u32,
        now_nanos: u128,
    ) -> Pin<Box<dyn Future<Output = Result<AlertPreviewData>> + Send + '_>>;
}
