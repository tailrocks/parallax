//! Plan 132 browser-foundation policy: Bun-only Playwright runtime, lock alignment,
//! config/project contract, and locator/artifact invariants.

use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::diagnostic::Finding;

const RERUN: &str = "cargo xtask policy --only ui.browser-foundation";
const PACKAGE: &str = "ui/package.json";
const CONFIG: &str = "ui/playwright.config.ts";
const LOCK: &str = "ui/bun.lock";
const POLICY: &str = "dependency-policy.toml";

pub(super) fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    check_package(root, &mut findings)?;
    check_config(root, &mut findings)?;
    check_lock_alignment(root, &mut findings)?;
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
    for (name, needle) in [
        (
            "test:browser:list",
            "bunx --bun --no-install playwright test --list",
        ),
        (
            "test:browser:foundation",
            "bunx --bun --no-install playwright test --project=foundation-chromium",
        ),
    ] {
        let value = scripts
            .and_then(|scripts| scripts.get(name))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if value != needle {
            findings.push(error(
                "ui.browser-foundation.scripts",
                PACKAGE,
                &format!("script `{name}` must be exact `{needle}`"),
            ));
        }
    }
    let dev = package
        .get("devDependencies")
        .and_then(serde_json::Value::as_object);
    let playwright = dev
        .and_then(|deps| deps.get("@playwright/test"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if playwright != "1.61.1" {
        findings.push(error(
            "ui.browser-foundation.dependency",
            PACKAGE,
            "devDependency `@playwright/test` must be exact 1.61.1",
        ));
    }
    for forbidden in ["playwright", "playwright-core"] {
        if dev.is_some_and(|deps| deps.contains_key(forbidden))
            || package
                .get("dependencies")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|deps| deps.contains_key(forbidden))
        {
            findings.push(error(
                "ui.browser-foundation.forbidden-direct",
                PACKAGE,
                &format!("direct package `{forbidden}` is forbidden; only `@playwright/test`"),
            ));
        }
    }
    Ok(())
}

fn check_config(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let path = root.join(CONFIG);
    if !path.is_file() {
        findings.push(error(
            "ui.browser-foundation.config",
            CONFIG,
            "playwright.config.ts is required",
        ));
        return Ok(());
    }
    let source = fs::read_to_string(&path)?;
    for needle in [
        "testDir: \"./tests/e2e\"",
        "forbidOnly",
        "baseURL",
        "timezoneId: \"UTC\"",
        "locale: \"en-US\"",
        "reducedMotion: \"reduce\"",
        "colorScheme: \"dark\"",
        "contextOptions",
        "name: \"foundation-chromium\"",
        "cargo xtask browser-foundation-serve",
        "reuseExistingServer: false",
        "screenshot: \"only-on-failure\"",
        "video: \"retain-on-failure\"",
        "trace: \"on-first-retry\"",
    ] {
        if !source.contains(needle) {
            findings.push(error(
                "ui.browser-foundation.config",
                CONFIG,
                &format!("playwright.config.ts missing required contract fragment `{needle}`"),
            ));
        }
    }
    Ok(())
}

fn check_lock_alignment(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let lock = fs::read_to_string(root.join(LOCK))?;
    let policy = fs::read_to_string(root.join(POLICY))?;
    for needle in [
        "\"@playwright/test@1.61.1\"",
        "\"playwright@1.61.1\"",
        "\"playwright-core@1.61.1\"",
    ] {
        if !lock.contains(needle) {
            findings.push(error(
                "ui.browser-foundation.lock",
                LOCK,
                &format!("bun.lock missing exact package pin fragment {needle}"),
            ));
        }
    }
    for needle in [
        "playwright-test = \"1.61.1\"",
        "transitive-playwright = \"1.61.1\"",
        "transitive-playwright-core = \"1.61.1\"",
    ] {
        if !policy.contains(needle) {
            findings.push(error(
                "ui.browser-foundation.policy",
                POLICY,
                &format!("dependency-policy.toml missing `{needle}`"),
            ));
        }
    }
    Ok(())
}

fn check_specs(root: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    let e2e = root.join("ui/tests/e2e");
    if !e2e.is_dir() {
        findings.push(error(
            "ui.browser-foundation.specs",
            "ui/tests/e2e",
            "e2e tree is required",
        ));
        return Ok(());
    }
    let mut files = Vec::new();
    collect_ts(&e2e, &mut files)?;
    if files.is_empty() {
        findings.push(error(
            "ui.browser-foundation.specs",
            "ui/tests/e2e",
            "e2e tree has no TypeScript files",
        ));
    }
    let mut has_smoke = false;
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&path)?;
        if relative.contains("/smoke/") && relative.ends_with(".spec.ts") {
            has_smoke = true;
            if !source.contains("@pw-foundation-") {
                findings.push(error(
                    "ui.browser-foundation.stable-id",
                    &relative,
                    "foundation smoke must declare a stable @pw-foundation-* id",
                ));
            }
        }
        for forbidden in [
            "test.only",
            "test.describe.only",
            "page.waitForTimeout(",
            "page.$(",
        ] {
            if source.contains(forbidden) {
                findings.push(error(
                    "ui.browser-foundation.locator-policy",
                    &relative,
                    &format!("forbidden pattern `{forbidden}`"),
                ));
            }
        }
        if source.contains("page.locator(")
            && (source.contains("css=") || source.contains("xpath="))
        {
            findings.push(error(
                "ui.browser-foundation.locator-policy",
                &relative,
                "CSS/XPath locators are forbidden; use role/name/label/text",
            ));
        }
    }
    if !has_smoke {
        findings.push(error(
            "ui.browser-foundation.specs",
            "ui/tests/e2e/smoke",
            "at least one foundation smoke spec is required",
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
        "restore the plan 132 browser foundation contract",
        RERUN,
    )
}
