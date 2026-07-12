use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Ratchet {
    pub schema_version: u32,
    pub architecture: Architecture,
    pub budgets: Budgets,
    pub product: Product,
    #[serde(default)]
    pub limits: Vec<Limit>,
    #[serde(default)]
    pub generated: Vec<Generated>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

#[derive(Debug, Deserialize)]
pub struct Limit {
    pub metric: String,
    pub scope: String,
    pub ceiling: usize,
}

#[derive(Debug, Deserialize)]
pub struct Generated {
    pub path: String,
    pub generator: String,
    pub owner: String,
    pub drift_check: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Product {
    #[serde(default)]
    pub clone_floors: Vec<CloneFloor>,
}

#[derive(Debug, Deserialize)]
pub struct CloneFloor {
    pub path: String,
    pub ceiling: usize,
}

#[derive(Debug, Default, Deserialize)]
pub struct Budgets {
    pub rust: RustBudgets,
    pub typescript: TypeScriptBudgets,
}

#[derive(Debug, Default, Deserialize)]
pub struct RustBudgets {
    pub root_file_lines: usize,
    pub production_file_lines: usize,
    pub test_file_lines: usize,
    pub function_lines: usize,
    pub cognitive_complexity: usize,
}

#[derive(Debug, Default, Deserialize)]
pub struct TypeScriptBudgets {
    pub route_file_lines: usize,
    pub module_lines: usize,
    pub test_file_lines: usize,
    pub function_lines: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
}

#[derive(Debug, Deserialize)]
pub struct Architecture {
    pub packages: Vec<PackageClass>,
}

#[derive(Debug, Deserialize)]
pub struct PackageClass {
    pub name: String,
    pub class: String,
    pub tier: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Exception {
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
    pub fn load(path: &Path) -> Result<Self> {
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
