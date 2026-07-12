use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Ratchet {
    pub schema_version: u32,
    pub architecture: Architecture,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
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
