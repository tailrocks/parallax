use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Ratchet {
    pub schema_version: u32,
    pub architecture: Architecture,
    pub budgets: Budgets,
    pub product: Product,
    #[serde(default)]
    pub ui: UiPolicy,
    #[serde(default)]
    pub limits: Vec<Limit>,
    #[serde(default)]
    pub generated: Vec<Generated>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
    #[serde(default)]
    pub rust_suppressions: Vec<RustSuppression>,
}

/// Plan-100 UI architecture control plane.
#[derive(Debug, Default, Deserialize)]
pub(super) struct UiPolicy {
    /// Exact current→target ownership for every handwritten UI source file.
    #[serde(default)]
    pub ownership: Vec<UiOwnership>,
    /// Shrink-only approved cross-feature facade edges (`feature-a -> feature-b`).
    #[serde(default)]
    pub feature_edges: Vec<UiFeatureEdge>,
    /// Exact current layer-graph exceptions with a removal plan.
    #[serde(default)]
    pub layer_exceptions: Vec<UiLayerException>,
}

#[derive(Debug, Deserialize)]
pub(super) struct UiOwnership {
    pub path: String,
    pub current_owner: String,
    pub target_owner: String,
    pub migration_plan: String,
    pub kind: String,
    #[serde(default)]
    pub facade: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct UiFeatureEdge {
    pub from_feature: String,
    pub to_feature: String,
    pub reason: String,
    pub owner: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct UiLayerException {
    pub source: String,
    pub target: String,
    pub rule: String,
    pub owner: String,
    pub removal_plan: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct Limit {
    pub metric: String,
    pub scope: String,
    pub ceiling: usize,
}

#[derive(Debug, Deserialize)]
pub(super) struct RustSuppression {
    pub crate_name: String,
    pub lint: String,
    pub ceiling: usize,
}

#[derive(Debug, Deserialize)]
pub(super) struct Generated {
    pub path: String,
    pub generator: String,
    pub owner: String,
    pub drift_check: String,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct Product {
    #[serde(default)]
    pub clone_floors: Vec<CloneFloor>,
    #[serde(default)]
    pub anyhow_edges: Vec<AnyhowEdge>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CloneFloor {
    pub path: String,
    pub ceiling: usize,
}

#[derive(Debug, Deserialize)]
pub(super) struct AnyhowEdge {
    pub path: String,
    pub ceiling: usize,
    pub reason: String,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct Budgets {
    pub rust: RustBudgets,
    pub typescript: TypeScriptBudgets,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct RustBudgets {
    pub root_file_lines: usize,
    pub production_file_lines: usize,
    pub test_file_lines: usize,
    pub function_lines: usize,
    pub cognitive_complexity: usize,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct TypeScriptBudgets {
    pub route_file_lines: usize,
    pub module_lines: usize,
    pub test_file_lines: usize,
    pub function_lines: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
}

#[derive(Debug, Deserialize)]
pub(super) struct Architecture {
    pub packages: Vec<PackageClass>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PackageClass {
    pub name: String,
    pub class: String,
    pub tier: Option<u8>,
    #[serde(default)]
    pub agent_context: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct Exception {
    pub rule: String,
    pub scope: String,
    pub evidence: String,
    pub owner: String,
    pub created: String,
    pub expires: String,
    pub removal_condition: String,
    pub replacement: String,
}

impl Ratchet {
    pub(super) fn load(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let ratchet: Self = toml::from_str(&source)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        anyhow::ensure!(
            ratchet.schema_version == 1,
            "unsupported ratchet schema version"
        );
        Ok(ratchet)
    }
}

#[cfg(test)]
mod tests;
