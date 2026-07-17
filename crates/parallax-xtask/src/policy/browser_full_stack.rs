//! Plan 145 browser full-stack policy: managed storage, one worker, matrix,
//! foundation specs, and no fixture/memory substitution.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::diagnostic::Finding;

const RERUN: &str = "cargo xtask policy --only ui.browser-full-stack";
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
    let full = scripts
        .and_then(|scripts| scripts.get("test:browser:full"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if !full.contains("playwright test")
        || !full.contains("--project=full-stack-chromium")
        || !full.contains("bunx --bun --no-install")
    {
        findings.push(error(
            "ui.browser-full-stack.scripts",
            PACKAGE,
            "script `test:browser:full` must be Bun-forced full-stack-chromium project",
        ));
    }
    Ok(())
}

fn check_config(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(CONFIG);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    for needle in [
        "name: \"full-stack-chromium\"",
        "cargo xtask browser-full-stack-serve",
        "testMatch: \"**/full-stack/**/*.spec.ts\"",
        "retries: 0",
    ] {
        if !source.contains(needle) {
            findings.push(error(
                "ui.browser-full-stack.config",
                CONFIG,
                &format!("playwright.config.ts missing `{needle}`"),
            ));
        }
    }
    if source.contains("storage.mode = \"memory\"") || source.contains("MemoryStore") {
        findings.push(error(
            "ui.browser-full-stack.config",
            CONFIG,
            "full-stack lane must not configure memory storage",
        ));
    }
    Ok(())
}

fn check_matrix(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(MATRIX);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let matrix: Matrix = serde_json::from_str(&source)?;
    let mut has_discovery = false;
    let mut has_storage = false;
    let mut has_live = false;
    // Feature owners 134-143 + 150: reserved until materialization, then implemented.
    let mut feature_rows = 0u32;
    for entry in &matrix.entries {
        if entry.lane_owner != "playwright/full-stack" {
            continue;
        }
        if entry.layer != "browser-full-stack" {
            findings.push(error(
                "ui.browser-full-stack.matrix",
                MATRIX,
                &format!("entry `{}` must use layer browser-full-stack", entry.id),
            ));
        }
        if entry.scenario_owner.trim().is_empty() {
            findings.push(error(
                "ui.browser-full-stack.matrix",
                MATRIX,
                &format!("entry `{}` missing scenario_owner", entry.id),
            ));
        }
        if entry.status == "implemented" && !root.join(&entry.test_file).is_file() {
            findings.push(error(
                "ui.browser-full-stack.matrix",
                MATRIX,
                &format!(
                    "implemented entry `{}` missing spec {}",
                    entry.id, entry.test_file
                ),
            ));
        }
        if entry.status == "reserved" && entry.delivery_plan.is_none() {
            findings.push(error(
                "ui.browser-full-stack.matrix",
                MATRIX,
                &format!("reserved entry `{}` needs delivery_plan", entry.id),
            ));
        }
        if entry.id.contains("telemetry-discovery") && entry.status == "implemented" {
            has_discovery = true;
        }
        if entry.id.contains("storage-composition") && entry.status == "implemented" {
            has_storage = true;
        }
        if entry.id.contains("live-transport") && entry.status == "implemented" {
            has_live = true;
        }
        // Count durable feature full-stack rows (investigations…overview).
        if (entry.status == "reserved" || entry.status == "implemented")
            && (entry.id.contains("investigations")
                || entry.id.contains("sql")
                || entry.id.contains("ecosystem")
                || entry.id.contains("dashboards")
                || entry.id.contains("services")
                || entry.id.contains("issues")
                || entry.id.contains("runs")
                || entry.id.contains("logs")
                || entry.id.contains("traces")
                || entry.id.contains("shell")
                || entry.id.contains("overview"))
            && !entry.id.contains("telemetry")
            && !entry.id.contains("live-transport")
            && !entry.id.contains("storage-composition")
        {
            feature_rows += 1;
        }
    }
    if !has_discovery || !has_storage || !has_live {
        findings.push(error(
            "ui.browser-full-stack.matrix",
            MATRIX,
            "foundation rows telemetry-discovery, storage-composition, live-transport must be implemented",
        ));
    }
    if feature_rows < 11 {
        findings.push(error(
            "ui.browser-full-stack.matrix",
            MATRIX,
            "expected 11 feature full-stack rows (plans 134-143 and 150) reserved or implemented",
        ));
    }
    Ok(())
}

fn check_specs(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    for required in [
        "ui/tests/e2e/full-stack/telemetry-discovery.spec.ts",
        "ui/tests/e2e/full-stack/storage-composition.spec.ts",
        "ui/tests/e2e/full-stack/live-transport.spec.ts",
    ] {
        if !root.join(required).is_file() {
            findings.push(error(
                "ui.browser-full-stack.specs",
                required,
                "foundation full-stack spec is required",
            ));
        }
    }
    let full_stack = root.join("ui/tests/e2e/full-stack");
    if full_stack.is_dir() {
        let mut files = Vec::new();
        collect_ts(&full_stack, &mut files)?;
        for path in files {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let source = fs::read_to_string(&path)?;
            for forbidden in [
                "test.only",
                "page.waitForTimeout(",
                "page.route(",
                "test.skip(",
                "test.fix(",
            ] {
                if source.contains(forbidden) {
                    findings.push(error(
                        "ui.browser-full-stack.locator-policy",
                        &relative,
                        &format!("forbidden pattern `{forbidden}`"),
                    ));
                }
            }
        }
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
        "restore the plan 145 browser full-stack policy",
        RERUN,
    )
}
