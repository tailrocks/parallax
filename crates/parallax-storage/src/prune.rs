//! Query-neutral deterministic prune-plan contract.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

mod metadata;
pub use metadata::MetadataPruneStore;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PruneStore {
    Greptime,
    Turso,
    LocalDisk,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PruneClass {
    RawTraces,
    RawLogs,
    RawMetrics,
    ErrorEvents,
    InvocationMetricPoints,
    MetricExemplars,
    Issues,
    IssueBuckets,
    IssueOccurrences,
    Invocations,
    Dashboards,
    Investigations,
    SavedViews,
    AlertRules,
    AlertRuleStates,
    AlertIncidents,
    AlertDestinations,
    AlertDeliveryEvents,
    AlertChecks,
    Spool,
    PinnedEvidence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PruneEstimate {
    pub rows: Option<u64>,
    pub objects: Option<u64>,
    pub bytes: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PruneExclusionKind {
    Active,
    Unresolved,
    Pinned,
    NotExpired,
    RetainedByPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PruneExclusion {
    pub kind: PruneExclusionKind,
    pub count: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PruneItem {
    pub store: PruneStore,
    pub class: PruneClass,
    pub target: String,
    #[serde(with = "u128_string")]
    pub cutoff_nanos: u128,
    pub estimate: PruneEstimate,
    pub exclusions: Vec<PruneExclusion>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PruneSnapshot {
    pub config_generation: String,
    pub protection_generation: String,
    pub catalog_fingerprint: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrunePlanLimits {
    pub max_items: usize,
    pub max_annotations_per_item: usize,
    pub max_text_bytes: usize,
}

impl Default for PrunePlanLimits {
    fn default() -> Self {
        Self {
            max_items: 512,
            max_annotations_per_item: 64,
            max_text_bytes: 256,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PrunePlan {
    contract_version: u8,
    plan_id: String,
    #[serde(with = "u128_string")]
    cutoff_nanos: u128,
    snapshot: PruneSnapshot,
    items: Vec<PruneItem>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrunePlanWire {
    contract_version: u8,
    plan_id: String,
    #[serde(with = "u128_string")]
    cutoff_nanos: u128,
    snapshot: PruneSnapshot,
    items: Vec<PruneItem>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PruneExecutionMode {
    DryRun,
    Execute,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PruneExecutionRequest {
    plan_id: String,
    mode: PruneExecutionMode,
}

impl PruneExecutionRequest {
    #[must_use]
    pub fn dry_run(plan_id: String) -> Self {
        Self {
            plan_id,
            mode: PruneExecutionMode::DryRun,
        }
    }

    pub fn execute(plan_id: String, confirmed: bool) -> Result<Self, PrunePlanError> {
        if !confirmed {
            return Err(PrunePlanError::ConfirmationRequired);
        }
        Ok(Self {
            plan_id,
            mode: PruneExecutionMode::Execute,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PruneAuthorization {
    DryRun,
    Execute,
}

#[derive(Debug, Error)]
pub enum PrunePlanError {
    #[error("prune snapshot field {0} must not be empty")]
    EmptySnapshotField(&'static str),
    #[error("prune item cutoff {item} differs from plan cutoff {plan}")]
    CutoffMismatch { item: u128, plan: u128 },
    #[error("prune item {target:?} has no row or object estimate")]
    MissingEstimate { target: String },
    #[error("prune plan contains {actual} items; limit is {limit}")]
    TooManyItems { actual: usize, limit: usize },
    #[error("prune item {target:?} has {actual} annotations; limit is {limit}")]
    TooManyAnnotations {
        target: String,
        actual: usize,
        limit: usize,
    },
    #[error("prune {field} contains {actual} bytes; limit is {limit}")]
    TextTooLong {
        field: &'static str,
        actual: usize,
        limit: usize,
    },
    #[error("prune item target must not be empty")]
    EmptyTarget,
    #[error("duplicate prune item for {target:?}")]
    DuplicateItem { target: String },
    #[error("prune plan is stale: {field} changed after planning")]
    StaleSnapshot { field: &'static str },
    #[error("execution request does not name this prune plan")]
    PlanIdentityMismatch,
    #[error("prune plan contents do not match its identity")]
    PlanIntegrityMismatch,
    #[error("unsupported prune contract version {0}")]
    UnsupportedContractVersion(u8),
    #[error("destructive prune execution requires explicit confirmation")]
    ConfirmationRequired,
    #[error("failed to encode prune plan identity: {0}")]
    Identity(#[from] serde_json::Error),
}

impl PrunePlan {
    /// Decode persisted machine output only after rebuilding its canonical
    /// identity and reapplying the caller's current safety bounds.
    pub fn decode(encoded: &str, limits: PrunePlanLimits) -> Result<Self, PrunePlanError> {
        let wire: PrunePlanWire = serde_json::from_str(encoded)?;
        if wire.contract_version != 1 {
            return Err(PrunePlanError::UnsupportedContractVersion(
                wire.contract_version,
            ));
        }
        let plan = Self::build(wire.cutoff_nanos, wire.snapshot, wire.items, limits)?;
        if plan.plan_id != wire.plan_id {
            return Err(PrunePlanError::PlanIntegrityMismatch);
        }
        Ok(plan)
    }

    pub fn build(
        cutoff_nanos: u128,
        snapshot: PruneSnapshot,
        mut items: Vec<PruneItem>,
        limits: PrunePlanLimits,
    ) -> Result<Self, PrunePlanError> {
        for (field, value) in [
            ("config_generation", snapshot.config_generation.as_str()),
            (
                "protection_generation",
                snapshot.protection_generation.as_str(),
            ),
            ("catalog_fingerprint", snapshot.catalog_fingerprint.as_str()),
        ] {
            if value.is_empty() {
                return Err(PrunePlanError::EmptySnapshotField(field));
            }
            validate_text(field, value, limits.max_text_bytes)?;
        }
        if items.len() > limits.max_items {
            return Err(PrunePlanError::TooManyItems {
                actual: items.len(),
                limit: limits.max_items,
            });
        }
        for item in &items {
            if item.target.is_empty() {
                return Err(PrunePlanError::EmptyTarget);
            }
            validate_text("target", &item.target, limits.max_text_bytes)?;
            if item.cutoff_nanos != cutoff_nanos {
                return Err(PrunePlanError::CutoffMismatch {
                    item: item.cutoff_nanos,
                    plan: cutoff_nanos,
                });
            }
            if item.estimate.rows.is_none() && item.estimate.objects.is_none() {
                return Err(PrunePlanError::MissingEstimate {
                    target: item.target.clone(),
                });
            }
            let annotation_count = item.exclusions.len().saturating_add(item.warnings.len());
            if annotation_count > limits.max_annotations_per_item {
                return Err(PrunePlanError::TooManyAnnotations {
                    target: item.target.clone(),
                    actual: annotation_count,
                    limit: limits.max_annotations_per_item,
                });
            }
            for warning in &item.warnings {
                validate_text("warning", warning, limits.max_text_bytes)?;
            }
        }
        for item in &mut items {
            item.exclusions.sort();
            item.exclusions.dedup();
            item.warnings.sort();
            item.warnings.dedup();
        }
        items.sort_by(|left, right| {
            (&left.store, &left.class, &left.target, left.cutoff_nanos).cmp(&(
                &right.store,
                &right.class,
                &right.target,
                right.cutoff_nanos,
            ))
        });
        for pair in items.windows(2) {
            if item_identity(&pair[0]) == item_identity(&pair[1]) {
                return Err(PrunePlanError::DuplicateItem {
                    target: pair[0].target.clone(),
                });
            }
        }
        let plan_id = compute_plan_id(1, cutoff_nanos, &snapshot, &items)?;
        Ok(Self {
            contract_version: 1,
            plan_id,
            cutoff_nanos,
            snapshot,
            items,
        })
    }

    pub fn validate_snapshot(&self, current: &PruneSnapshot) -> Result<(), PrunePlanError> {
        for (field, unchanged) in [
            (
                "config_generation",
                self.snapshot.config_generation == current.config_generation,
            ),
            (
                "protection_generation",
                self.snapshot.protection_generation == current.protection_generation,
            ),
            (
                "catalog_fingerprint",
                self.snapshot.catalog_fingerprint == current.catalog_fingerprint,
            ),
        ] {
            if !unchanged {
                return Err(PrunePlanError::StaleSnapshot { field });
            }
        }
        Ok(())
    }

    pub fn authorize(
        &self,
        request: &PruneExecutionRequest,
        current: &PruneSnapshot,
    ) -> Result<PruneAuthorization, PrunePlanError> {
        if compute_plan_id(
            self.contract_version,
            self.cutoff_nanos,
            &self.snapshot,
            &self.items,
        )? != self.plan_id
        {
            return Err(PrunePlanError::PlanIntegrityMismatch);
        }
        if request.plan_id != self.plan_id {
            return Err(PrunePlanError::PlanIdentityMismatch);
        }
        self.validate_snapshot(current)?;
        match request.mode {
            PruneExecutionMode::DryRun => Ok(PruneAuthorization::DryRun),
            PruneExecutionMode::Execute => Ok(PruneAuthorization::Execute),
        }
    }

    #[must_use]
    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }

    #[must_use]
    pub const fn contract_version(&self) -> u8 {
        self.contract_version
    }

    #[must_use]
    pub const fn cutoff_nanos(&self) -> u128 {
        self.cutoff_nanos
    }

    #[must_use]
    pub fn snapshot(&self) -> &PruneSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn items(&self) -> &[PruneItem] {
        &self.items
    }
}

fn compute_plan_id(
    contract_version: u8,
    cutoff_nanos: u128,
    snapshot: &PruneSnapshot,
    items: &[PruneItem],
) -> Result<String, PrunePlanError> {
    let identity =
        serde_json::to_vec(&(contract_version, cutoff_nanos.to_string(), snapshot, items))?;
    Ok(format!("{:x}", Sha256::digest(identity)))
}

fn validate_text(field: &'static str, value: &str, limit: usize) -> Result<(), PrunePlanError> {
    if value.len() > limit {
        return Err(PrunePlanError::TextTooLong {
            field,
            actual: value.len(),
            limit,
        });
    }
    Ok(())
}

fn item_identity(item: &PruneItem) -> (PruneStore, PruneClass, &str, u128) {
    (item.store, item.class, &item.target, item.cutoff_nanos)
}

mod u128_string {
    use serde::{Deserialize, Deserializer, Serializer, de::Error as _};

    pub(super) fn serialize<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests;
