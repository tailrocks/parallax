//! Plan 148 bundle policy: budgets file, scripts, analyze wiring, matrix @bundle rows.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::diagnostic::Finding;

const RERUN: &str = "cargo xtask policy --only ui.bundles";
const PACKAGE: &str = "ui/package.json";
const BUDGETS: &str = "ui/bundle-budgets.json";
const ANALYZE: &str = "ui/scripts/bundle-analyze.ts";
const MATRIX: &str = "ui/test-matrix.json";
const SPEC: &str = "ui/tests/e2e/contracts/bundle-resources.spec.ts";
const VITE: &str = "ui/vite.config.ts";

#[derive(Debug, Deserialize)]
struct Matrix {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    scenario_owner: String,
    lane_owner: String,
    layer: String,
    test_file: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct Budgets {
    schema_version: u32,
    total_raw_ceiling: u64,
    total_gzip_ceiling: u64,
    file_count_ceiling: u64,
    largest_gzip_ceiling: u64,
    source_map_files_ceiling: u64,
    clean_build_tolerance_bytes: u64,
}

pub(super) fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    check_scripts(root, &mut findings)?;
    check_budgets(root, &mut findings)?;
    check_analyze_script(root, &mut findings)?;
    check_vite(root, &mut findings)?;
    check_matrix(root, &mut findings)?;
    check_spec(root, &mut findings)?;
    Ok(findings)
}

fn check_scripts(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(PACKAGE);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let package: Value = serde_json::from_str(&source)?;
    let scripts = package
        .get("scripts")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for key in [
        "bundle:analyze",
        "bundle:analyze:build",
        "bundle:build-twice",
    ] {
        let value = scripts.get(key).and_then(Value::as_str).unwrap_or_default();
        if !value.contains("bundle-analyze.ts") {
            findings.push(error(
                "ui.bundles.scripts",
                PACKAGE,
                &format!("script `{key}` must invoke bundle-analyze.ts"),
            ));
        }
    }
    Ok(())
}

fn check_budgets(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(BUDGETS);
    if !path.is_file() {
        findings.push(error(
            "ui.bundles.budgets",
            BUDGETS,
            "checked-in bundle-budgets.json is required",
        ));
        return Ok(());
    }
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let budgets: Budgets =
        serde_json::from_str(&source).with_context(|| format!("parse {}", path.display()))?;
    if budgets.schema_version != 1 {
        findings.push(error(
            "ui.bundles.budgets",
            BUDGETS,
            "schema_version must be 1",
        ));
    }
    if budgets.source_map_files_ceiling != 0 {
        findings.push(error(
            "ui.bundles.budgets",
            BUDGETS,
            "source_map_files_ceiling must be 0",
        ));
    }
    for (name, value) in [
        ("total_raw_ceiling", budgets.total_raw_ceiling),
        ("total_gzip_ceiling", budgets.total_gzip_ceiling),
        ("file_count_ceiling", budgets.file_count_ceiling),
        ("largest_gzip_ceiling", budgets.largest_gzip_ceiling),
    ] {
        if value == 0 {
            findings.push(error(
                "ui.bundles.budgets",
                BUDGETS,
                &format!("{name} must be positive"),
            ));
        }
    }
    let _ = budgets.clean_build_tolerance_bytes;
    Ok(())
}

fn check_analyze_script(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(ANALYZE);
    if !path.is_file() {
        findings.push(error(
            "ui.bundles.analyze",
            ANALYZE,
            "bundle analyze script is required",
        ));
        return Ok(());
    }
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    for needle in [
        "target/ui-bundle",
        "sourceMapFiles",
        "--build-twice",
        "totalGzip",
        "bundle-budgets.json",
    ] {
        if !source.contains(needle) {
            findings.push(error(
                "ui.bundles.analyze",
                ANALYZE,
                &format!("analyze script missing `{needle}`"),
            ));
        }
    }
    Ok(())
}

fn check_vite(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(VITE);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    if !source.contains("sourcemap: false") {
        findings.push(error(
            "ui.bundles.vite",
            VITE,
            "production sourcemap must be false",
        ));
    }
    Ok(())
}

fn check_matrix(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(MATRIX);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let matrix: Matrix = serde_json::from_str(&source)?;
    let mut bundle_rows = 0u32;
    for entry in &matrix.entries {
        if entry.lane_owner != "performance/bundle" {
            continue;
        }
        bundle_rows += 1;
        if entry.layer != "browser-contracts" {
            findings.push(error(
                "ui.bundles.matrix",
                MATRIX,
                &format!("entry `{}` must use layer browser-contracts", entry.id),
            ));
        }
        if entry.scenario_owner.trim().is_empty() {
            findings.push(error(
                "ui.bundles.matrix",
                MATRIX,
                &format!("entry `{}` missing scenario_owner", entry.id),
            ));
        }
        if entry.status == "implemented" && !root.join(&entry.test_file).is_file() {
            findings.push(error(
                "ui.bundles.matrix",
                MATRIX,
                &format!(
                    "implemented entry `{}` missing spec {}",
                    entry.id, entry.test_file
                ),
            ));
        }
    }
    if bundle_rows == 0 {
        findings.push(error(
            "ui.bundles.matrix",
            MATRIX,
            "at least one performance/bundle matrix row is required",
        ));
    }
    Ok(())
}

fn check_spec(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(SPEC);
    if !path.is_file() {
        findings.push(error(
            "ui.bundles.spec",
            SPEC,
            "bundle resource Playwright spec is required",
        ));
        return Ok(());
    }
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    for needle in ["@bundle", "@pw-bundle-shell-entry", ".map"] {
        if !source.contains(needle) {
            findings.push(error(
                "ui.bundles.spec",
                SPEC,
                &format!("bundle spec missing `{needle}`"),
            ));
        }
    }
    Ok(())
}

fn error(rule: &str, path: &str, message: &str) -> Finding {
    Finding::error(
        rule,
        path,
        1,
        message,
        "restore the plan 148 UI bundle policy",
        RERUN,
    )
}
