use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use serde::Deserialize;

use crate::diagnostic::Finding;

use super::config::Ratchet;

#[derive(Debug, Deserialize)]
struct CrateDoc {
    schema_version: u32,
    package: String,
    class: String,
    tier: Option<u8>,
    dependencies: Vec<String>,
    facade_roots: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Facade {
    roots: BTreeMap<String, Vec<String>>,
}

pub fn check_workspace(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let metadata = MetadataCommand::new().current_dir(root).no_deps().exec()?;
    let members: BTreeSet<_> = metadata.workspace_members.iter().collect();
    let classes: BTreeMap<_, _> = ratchet
        .architecture
        .packages
        .iter()
        .map(|package| (&package.name, package))
        .collect();
    let mut findings = Vec::new();
    for package in metadata
        .packages
        .iter()
        .filter(|package| members.contains(&package.id))
    {
        let directory = package.manifest_path.parent().context("manifest parent")?;
        let readme = directory.join("README.md");
        let doc = match parse(
            &fs::read_to_string(&readme).with_context(|| format!("missing {}", readme))?,
        ) {
            Ok(doc) => doc,
            Err(error) => {
                findings.push(finding(
                    &readme,
                    &format!("invalid crate-doc front matter: {error:#}"),
                ));
                continue;
            }
        };
        let class = classes.get(&package.name.to_string());
        let mut dependencies: Vec<_> = package
            .dependencies
            .iter()
            .filter(|dependency| dependency.path.is_some())
            .map(|dependency| dependency.name.to_string())
            .collect();
        dependencies.sort();
        dependencies.dedup();
        let facade: Facade = toml::from_str(&fs::read_to_string(directory.join("facade.toml"))?)?;
        let mut roots: Vec<_> = facade.roots.keys().cloned().collect();
        roots.sort();
        if doc.schema_version != 1
            || package.name != doc.package
            || class.is_none_or(|class| class.class != doc.class || class.tier != doc.tier)
            || doc.dependencies != dependencies
            || doc.facade_roots != roots
        {
            findings.push(finding(&readme, "crate documentation disagrees with Cargo metadata, architecture class/tier, or facade roots"));
        }
    }
    Ok(findings)
}

fn parse(source: &str) -> Result<CrateDoc> {
    let rest = source
        .strip_prefix("+++\n")
        .context("missing opening +++")?;
    let (front, body) = rest.split_once("\n+++\n").context("missing closing +++")?;
    anyhow::ensure!(!body.trim().is_empty(), "README body is empty");
    Ok(toml::from_str(front)?)
}

fn finding(path: impl AsRef<Path>, reason: &str) -> Finding {
    Finding::error(
        "docs.crate-semantic",
        &path.as_ref().to_string_lossy(),
        1,
        reason,
        "update README front matter and prose from current crate ownership",
        "cargo xtask policy",
    )
}

#[cfg(test)]
mod tests;
