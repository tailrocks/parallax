//! Plan 144 browser product-contract policy: matrix ownership, locator rules,
//! Bun-only runtime, contracts project, and no happy-path interception.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::diagnostic::Finding;

const RERUN: &str = "cargo xtask policy --only ui.browser-contracts";
const PACKAGE: &str = "ui/package.json";
const CONFIG: &str = "ui/playwright.config.ts";
const MATRIX: &str = "ui/test-matrix.json";

#[derive(Debug, Deserialize)]
struct Matrix {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    scenario_owner: String,
    lane_owner: String,
    delivery_plan: Option<u16>,
    layer: String,
    test_file: String,
    status: String,
}

pub(super) fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    check_package(root, &mut findings)?;
    check_config(root, &mut findings)?;
    check_matrix(root, &mut findings)?;
    check_specs(root, &mut findings)?;
    Ok(findings)
}

fn check_package(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(PACKAGE);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let package: serde_json::Value = serde_json::from_str(&source)?;
    let scripts = package
        .get("scripts")
        .and_then(serde_json::Value::as_object);
    let browser = scripts
        .and_then(|scripts| scripts.get("test:browser"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if browser != "bunx --bun --no-install playwright test --project=contracts-chromium" {
        findings.push(error(
            "ui.browser-contracts.scripts",
            PACKAGE,
            "script `test:browser` must be exact Bun-forced contracts-chromium project",
        ));
    }
    Ok(())
}

fn check_config(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(CONFIG);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    for needle in [
        "name: \"contracts-chromium\"",
        "cargo xtask browser-contracts-serve",
        "retries: 0",
        "testMatch: \"**/contracts/**/*.spec.ts\"",
        "timezoneId: \"UTC\"",
        "locale: \"en-US\"",
        "reducedMotion: \"reduce\"",
    ] {
        if !source.contains(needle) {
            findings.push(error(
                "ui.browser-contracts.config",
                CONFIG,
                &format!("playwright.config.ts missing `{needle}`"),
            ));
        }
    }
    Ok(())
}

fn check_matrix(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(MATRIX);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let matrix: Matrix = serde_json::from_str(&source)?;
    let mut contract_owners = std::collections::BTreeSet::new();
    let mut has_shell = false;
    let mut has_investigations = false;
    for entry in &matrix.entries {
        if entry.lane_owner != "playwright/contracts" {
            continue;
        }
        if entry.layer != "browser-contract" {
            findings.push(error(
                "ui.browser-contracts.matrix",
                MATRIX,
                &format!("entry `{}` must use layer browser-contract", entry.id),
            ));
        }
        if entry.scenario_owner.trim().is_empty() {
            findings.push(error(
                "ui.browser-contracts.matrix",
                MATRIX,
                &format!("entry `{}` missing scenario_owner", entry.id),
            ));
        }
        contract_owners.insert(entry.scenario_owner.clone());
        if entry.scenario_owner == "layout/shell" && entry.status == "implemented" {
            has_shell = true;
        }
        if entry.scenario_owner == "features/investigations" && entry.status == "implemented" {
            has_investigations = true;
        }
        if entry.status == "reserved" && entry.delivery_plan.is_none() {
            findings.push(error(
                "ui.browser-contracts.matrix",
                MATRIX,
                &format!("reserved entry `{}` needs delivery_plan", entry.id),
            ));
        }
        if entry.status == "implemented" && !root.join(&entry.test_file).is_file() {
            findings.push(error(
                "ui.browser-contracts.matrix",
                MATRIX,
                &format!("implemented entry `{}` missing spec {}", entry.id, entry.test_file),
            ));
        }
    }
    for required in [
        "layout/shell",
        "features/investigations",
        "features/sql",
        "features/ecosystem",
        "features/dashboards",
        "features/services",
        "features/issues",
        "features/runs",
        "features/logs",
        "features/traces",
        "features/overview",
    ] {
        if !contract_owners.contains(required) {
            findings.push(error(
                "ui.browser-contracts.matrix",
                MATRIX,
                &format!("missing playwright/contracts owner for `{required}`"),
            ));
        }
    }
    if !has_shell || !has_investigations {
        findings.push(error(
            "ui.browser-contracts.matrix",
            MATRIX,
            "shell and investigations must have implemented contract rows",
        ));
    }
    Ok(())
}

fn check_specs(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let e2e = root.join("ui/tests/e2e");
    if !e2e.is_dir() {
        findings.push(error(
            "ui.browser-contracts.specs",
            "ui/tests/e2e",
            "e2e tree is required",
        ));
        return Ok(());
    }
    let mut files = Vec::new();
    collect_ts(&e2e, &mut files)?;
    let mut has_shell = false;
    let mut has_investigations = false;
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&path)?;
        if relative.ends_with("contracts/shell.spec.ts") {
            has_shell = true;
        }
        if relative.ends_with("contracts/investigations.spec.ts") {
            has_investigations = true;
        }
        for forbidden in [
            "test.only",
            "test.describe.only",
            "page.waitForTimeout(",
            "page.$(",
            "page.route(",
            "test.skip(",
            "test.fix(",
        ] {
            if source.contains(forbidden) {
                findings.push(error(
                    "ui.browser-contracts.locator-policy",
                    &relative,
                    &format!("forbidden pattern `{forbidden}`"),
                ));
            }
        }
        if source.contains("page.locator(")
            && (source.contains("css=") || source.contains("xpath="))
        {
            findings.push(error(
                "ui.browser-contracts.locator-policy",
                &relative,
                "CSS/XPath locators are forbidden; use role/name/label/text",
            ));
        }
    }
    if !has_shell || !has_investigations {
        findings.push(error(
            "ui.browser-contracts.specs",
            "ui/tests/e2e/contracts",
            "shell and investigations contract specs are required",
        ));
    }
    Ok(())
}

fn collect_ts(root: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_ts(&path, files)?;
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "ts" || ext == "tsx")
        {
            files.push(path);
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
        "restore the plan 144 browser product-contract policy",
        RERUN,
    )
}
