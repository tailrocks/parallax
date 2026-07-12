#![expect(
    clippy::excessive_nesting,
    reason = "measured handoff document validation"
)]

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

pub(super) fn check_workspace(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
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
        let doc =
            match parse(&fs::read_to_string(&readme).with_context(|| format!("missing {readme}"))?)
            {
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
            .map(|dependency| dependency.name.clone())
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
    findings.extend(check_handoffs(root)?);
    Ok(findings)
}

fn check_handoffs(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let mut stable_ids = BTreeSet::new();
    for relative in [
        "plans/097-model-test-support-and-dependency-direction.md",
        "plans/101-dependencies-nextest-and-hygiene.md",
    ] {
        let path = root.join(relative);
        let source = fs::read_to_string(&path)?;
        match parse_handoff(&source) {
            Ok(rows) => {
                for row in rows {
                    if !stable_ids.insert(row[0].clone()) {
                        findings.push(handoff_finding(&path, "stable ID is reused"));
                    }
                }
            }
            Err(error) => findings.push(handoff_finding(
                &path,
                &format!("invalid Plan 127 handoff: {error:#}"),
            )),
        }
    }
    let plan126 = root.join("plans/126-rust-workspace-decomposition.md");
    match parse_owned_handoff(
        &fs::read_to_string(&plan126)?,
        "## Incoming Handoff From 097",
        "097-",
    ) {
        Ok(rows) => {
            for row in rows {
                if !stable_ids.insert(row[0].clone()) {
                    findings.push(handoff_finding(&plan126, "stable ID is reused"));
                }
            }
        }
        Err(error) => findings.push(handoff_finding(
            &plan126,
            &format!("invalid Plan 097 extraction handoff: {error:#}"),
        )),
    }
    Ok(findings)
}

fn parse_handoff(source: &str) -> Result<Vec<Vec<String>>> {
    parse_owned_handoff(source, "## Incoming Handoff From 127", "127-")
}

fn parse_owned_handoff(
    source: &str,
    heading: &str,
    stable_id_prefix: &str,
) -> Result<Vec<Vec<String>>> {
    let section = source
        .split_once(heading)
        .context("missing handoff heading")?
        .1;
    let table: Vec<_> = section
        .lines()
        .skip_while(|line| !line.starts_with("| Stable ID |"))
        .take_while(|line| line.starts_with('|'))
        .collect();
    anyhow::ensure!(table.len() >= 3, "handoff table has no data rows");
    let mut rows = Vec::new();
    for line in table.into_iter().skip(2) {
        let cells: Vec<_> = line
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().trim_matches('`').to_owned())
            .collect();
        anyhow::ensure!(cells.len() == 5, "handoff row must contain five columns");
        anyhow::ensure!(
            cells[0].starts_with(stable_id_prefix) && !cells[0].ends_with("pending"),
            "handoff row needs a stable owned ID"
        );
        anyhow::ensure!(
            cells[1..4].iter().all(|cell| !cell.is_empty()),
            "handoff ownership columns cannot be empty"
        );
        anyhow::ensure!(cells[4] == "OWNED", "handoff status must be OWNED");
        anyhow::ensure!(
            !cells.iter().any(|cell| {
                let lower = cell.to_ascii_lowercase();
                lower.contains("pending")
                    || lower.contains("unowned")
                    || lower.contains("populate during")
            }),
            "handoff contains placeholder ownership"
        );
        rows.push(cells);
    }
    Ok(rows)
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

fn handoff_finding(path: impl AsRef<Path>, reason: &str) -> Finding {
    Finding::error(
        "docs.plan-127-handoff",
        &path.as_ref().to_string_lossy(),
        1,
        reason,
        "populate every handoff row with a stable ID, explicit owner, and OWNED status",
        "cargo xtask policy",
    )
}

#[cfg(test)]
mod tests;
