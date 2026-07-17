//! Plan 146 browser breadth policy: engines, mobile, a11y, visual projects.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::diagnostic::Finding;

const RERUN: &str = "cargo xtask policy --only ui.browser-breadth";
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
    lane_owner: String,
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
    check_axe_dependency(root, &mut findings)?;
    Ok(findings)
}

fn check_package(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(PACKAGE);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let package: serde_json::Value = serde_json::from_str(&source)?;
    let scripts = package
        .get("scripts")
        .and_then(serde_json::Value::as_object);
    for (name, needles) in [
        (
            "test:browser:cross",
            &[
                "bunx --bun --no-install",
                "cross-firefox",
                "cross-webkit",
                "mobile-chromium",
                "mobile-webkit",
            ][..],
        ),
        (
            "test:browser:a11y",
            &["bunx --bun --no-install", "accessibility-chromium"][..],
        ),
        (
            "test:browser:visual",
            &["bunx --bun --no-install", "visual-chromium-linux"][..],
        ),
    ] {
        let script = scripts
            .and_then(|s| s.get(name))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        for needle in needles {
            if !script.contains(needle) {
                findings.push(error(
                    "ui.browser-breadth.scripts",
                    PACKAGE,
                    &format!("script `{name}` must include `{needle}`"),
                ));
            }
        }
    }
    Ok(())
}

fn check_config(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(CONFIG);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    for needle in [
        "name: \"cross-firefox\"",
        "name: \"cross-webkit\"",
        "name: \"mobile-chromium\"",
        "name: \"mobile-webkit\"",
        "name: \"accessibility-chromium\"",
        "name: \"visual-chromium-linux\"",
        "devices[",
        "retries: 0",
    ] {
        if !source.contains(needle) {
            findings.push(error(
                "ui.browser-breadth.config",
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
    let mut has_a11y = false;
    let mut has_mobile = false;
    let mut has_visual = false;
    let mut has_cross = false;
    for entry in &matrix.entries {
        if entry.lane_owner != "playwright/breadth" {
            continue;
        }
        if entry.layer != "browser-breadth" {
            findings.push(error(
                "ui.browser-breadth.matrix",
                MATRIX,
                &format!("entry `{}` must use layer browser-breadth", entry.id),
            ));
        }
        if entry.status == "implemented" && !root.join(&entry.test_file).is_file() {
            findings.push(error(
                "ui.browser-breadth.matrix",
                MATRIX,
                &format!(
                    "implemented entry `{}` missing spec {}",
                    entry.id, entry.test_file
                ),
            ));
        }
        if entry.id.contains("a11y") || entry.id.contains("accessibility") {
            has_a11y = true;
        }
        if entry.id.contains("mobile") {
            has_mobile = true;
        }
        if entry.id.contains("visual") {
            has_visual = true;
        }
        if entry.id.contains("cross") {
            has_cross = true;
        }
    }
    if !has_a11y || !has_mobile || !has_visual || !has_cross {
        findings.push(error(
            "ui.browser-breadth.matrix",
            MATRIX,
            "expected implemented breadth rows for cross, mobile, a11y, and visual pilots",
        ));
    }
    Ok(())
}

fn check_specs(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    for required in [
        "ui/tests/e2e/accessibility/shell-accessibility.spec.ts",
        "ui/tests/e2e/accessibility/investigations-accessibility.spec.ts",
        "ui/tests/e2e/mobile/shell-mobile.spec.ts",
        "ui/tests/e2e/mobile/investigations-mobile.spec.ts",
        "ui/tests/e2e/visual/shell.visual.spec.ts",
        "ui/tests/e2e/visual/investigations.visual.spec.ts",
        "ui/tests/e2e/fixtures/accessibility-fixture.ts",
        "ui/tests/e2e/support/visual-manifest.ts",
    ] {
        if !root.join(required).is_file() {
            findings.push(error(
                "ui.browser-breadth.specs",
                required,
                "plan 146 breadth pilot artifact is required",
            ));
        }
    }
    Ok(())
}

fn check_axe_dependency(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(PACKAGE);
    let source = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let package: serde_json::Value = serde_json::from_str(&source)?;
    let deps = package
        .get("devDependencies")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .chain(
            package
                .get("dependencies")
                .and_then(serde_json::Value::as_object),
        );
    let version = deps
        .filter_map(|map| map.get("@axe-core/playwright"))
        .find_map(serde_json::Value::as_str)
        .unwrap_or_default();
    if version.is_empty() || version.starts_with('^') || version.starts_with('~') {
        findings.push(error(
            "ui.browser-breadth.axe",
            PACKAGE,
            "@axe-core/playwright must be exact-pinned (no caret/tilde range)",
        ));
    }
    Ok(())
}

fn error(rule: &str, path: &str, message: &str) -> Finding {
    Finding::error(
        rule,
        path,
        1,
        message,
        "restore the plan 146 browser breadth policy",
        RERUN,
    )
}
