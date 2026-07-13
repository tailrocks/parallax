use std::{collections::BTreeSet, fs, path::Path};

use anyhow::Result;

use crate::diagnostic::Finding;

use super::{Ratchet, error};

fn count(source: &str) -> usize {
    [
        "anyhow::Result",
        "anyhow::Error",
        "anyhow::bail!",
        "anyhow::ensure!",
        "anyhow::anyhow!",
        ".context(",
    ]
    .iter()
    .map(|needle| source.matches(needle).count())
    .sum()
}

fn rust_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

pub(super) fn check(root: &Path, ratchet: &Ratchet, findings: &mut Vec<Finding>) -> Result<()> {
    let configured: BTreeSet<_> = ratchet
        .product
        .anyhow_edges
        .iter()
        .map(|edge| edge.path.as_str())
        .collect();
    for edge in &ratchet.product.anyhow_edges {
        let path = root.join(&edge.path);
        let actual = count(&fs::read_to_string(&path)?);
        if edge.reason.trim().is_empty() || edge.ceiling == 0 || actual != edge.ceiling {
            findings.push(error(
                "product.anyhow-edge",
                &path,
                &format!(
                    "anyhow edge count {actual} differs from exact ceiling {} or lacks a reason",
                    edge.ceiling
                ),
            ));
        }
    }
    for package in ratchet
        .architecture
        .packages
        .iter()
        .filter(|package| package.class == "product" && package.name != "parallax-cli")
    {
        let mut files = Vec::new();
        rust_files(
            &root.join("crates").join(&package.name).join("src"),
            &mut files,
        )?;
        for path in files {
            if path.file_name().is_some_and(|name| name == "tests.rs") {
                continue;
            }
            let relative = path.strip_prefix(root)?.to_string_lossy();
            if count(&fs::read_to_string(&path)?) > 0 && !configured.contains(relative.as_ref()) {
                findings.push(error(
                    "product.anyhow-edge.unapproved",
                    &path,
                    "product library anyhow use is not enumerated in product.anyhow_edges",
                ));
            }
        }
    }
    Ok(())
}
