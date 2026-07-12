use std::{collections::BTreeSet, fs, path::Path};

use anyhow::Result;
use cargo_metadata::Metadata;

use crate::diagnostic::Finding;

use super::error;

const RUST_VERSION: &str = "1.97.0";
const TARGETS: [&str; 4] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
];

pub(super) fn check(root: &Path, metadata: &Metadata, findings: &mut Vec<Finding>) -> Result<()> {
    check_files(root, findings)?;
    let members: BTreeSet<_> = metadata.workspace_members.iter().collect();
    for package in metadata
        .packages
        .iter()
        .filter(|package| members.contains(&package.id))
    {
        let manifest_path = package.manifest_path.as_std_path();
        let valid_metadata = package.version.to_string() == "0.1.0-dev"
            && package.edition.to_string() == "2024"
            && package
                .rust_version
                .as_ref()
                .is_some_and(|v| v.to_string() == RUST_VERSION)
            && package.license.as_deref() == Some("Apache-2.0")
            && package.authors == ["Tailrocks"]
            && package.repository.as_deref() == Some("https://github.com/tailrocks/parallax");
        if !valid_metadata {
            findings.push(error(
                "product.rust-package-metadata",
                manifest_path,
                "workspace package metadata does not match the pinned workspace contract",
            ));
        }
        let manifest: toml::Value = toml::from_str(&fs::read_to_string(manifest_path)?)?;
        if manifest["lints"]["workspace"].as_bool() != Some(true) {
            findings.push(error(
                "product.rust-lint-inheritance",
                manifest_path,
                "workspace package must set [lints] workspace = true",
            ));
        }
        for key in [
            "version",
            "edition",
            "rust-version",
            "license",
            "authors",
            "repository",
        ] {
            if manifest["package"][key]["workspace"].as_bool() != Some(true) {
                findings.push(error(
                    "product.rust-package-inheritance",
                    manifest_path,
                    &format!("package metadata `{key}` must inherit from the workspace"),
                ));
            }
        }
    }
    Ok(())
}

fn check_files(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let toolchain: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("rust-toolchain.toml"))?)?;
    let components = string_set(&toolchain["toolchain"]["components"]);
    let targets = string_set(&toolchain["toolchain"]["targets"]);
    if toolchain["toolchain"]["channel"].as_str() != Some(RUST_VERSION)
        || toolchain["toolchain"]["profile"].as_str() != Some("minimal")
        || components != BTreeSet::from(["clippy", "rustfmt"])
        || targets != BTreeSet::from(TARGETS)
    {
        findings.push(error(
            "product.rust-toolchain",
            root.join("rust-toolchain.toml"),
            "toolchain version, profile, components, or release targets drifted",
        ));
    }
    let mise: toml::Value = toml::from_str(&fs::read_to_string(root.join("mise.toml"))?)?;
    let cargo: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;
    if mise["tools"]["rust"].as_str() != Some(RUST_VERSION)
        || cargo["workspace"]["package"]["rust-version"].as_str() != Some(RUST_VERSION)
        || cargo["workspace"]["package"]["edition"].as_str() != Some("2024")
    {
        findings.push(error(
            "product.rust-toolchain-agreement",
            root.join("Cargo.toml"),
            "mise, Cargo, and rust-toolchain.toml must agree on exact Rust and edition",
        ));
    }
    Ok(())
}

fn string_set(value: &toml::Value) -> BTreeSet<&str> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .collect()
}

#[cfg(test)]
mod tests;
