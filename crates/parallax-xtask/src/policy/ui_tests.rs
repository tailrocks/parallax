use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::diagnostic::Finding;

const MATRIX_PATH: &str = "ui/test-matrix.json";
const RERUN: &str = "cargo xtask policy --only ui.tests";

#[derive(Debug, Deserialize)]
struct Matrix {
    schema_version: u32,
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    surface: String,
    risk: String,
    scenario_owner: String,
    lane_owner: String,
    delivery_plan: Option<u16>,
    layer: String,
    test_file: String,
    test_ids: Vec<String>,
    required_environment: String,
    status: String,
    legacy_handoff: Option<LegacyHandoff>,
}

#[derive(Debug, Deserialize)]
struct LegacyHandoff {
    current_path: String,
    destination_owner: String,
    removal_plan: u16,
    created: String,
    expires: String,
}

pub(super) fn check_workspace(root: &Path) -> Result<Vec<Finding>> {
    let matrix_path = root.join(MATRIX_PATH);
    let source = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: Matrix = serde_json::from_str(&source)
        .with_context(|| format!("parse {}", matrix_path.display()))?;
    let mut findings = Vec::new();
    if matrix.schema_version != 1 {
        findings.push(finding("ui.tests.schema", "schema_version must equal 1"));
    }

    let mut ids = BTreeSet::new();
    let mut represented = BTreeMap::new();
    for entry in &matrix.entries {
        validate_entry(root, entry, &mut ids, &mut represented, &mut findings)?;
    }

    let discovered = discover_tests(root)?;
    for (path, test_ids) in &discovered {
        if path.starts_with("ui/src/test/") {
            findings.push(finding(
                "ui.tests.topology",
                &format!("test body `{path}` is inside the harness-only src/test directory"),
            ));
        }
        check_local_harness(root, path, &mut findings)?;
        match represented.get(path) {
            Some(expected) if expected == test_ids => {}
            Some(expected) => findings.push(finding(
                "ui.tests.ids",
                &format!(
                    "matrix IDs for `{path}` differ: expected {test_ids:?}, recorded {expected:?}"
                ),
            )),
            None => findings.push(finding(
                "ui.tests.file",
                &format!("test file `{path}` has no matrix owner"),
            )),
        }
    }
    for path in represented.keys() {
        if !discovered.contains_key(path) {
            findings.push(finding(
                "ui.tests.file",
                &format!("matrix references missing or empty test file `{path}`"),
            ));
        }
    }
    Ok(findings)
}

fn validate_entry(
    root: &Path,
    entry: &Entry,
    ids: &mut BTreeSet<String>,
    represented: &mut BTreeMap<String, BTreeSet<String>>,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    if !ids.insert(entry.id.clone()) {
        findings.push(finding(
            "ui.tests.id",
            &format!("duplicate matrix ID `{}`", entry.id),
        ));
    }
    let valid_owner = [
        "features/",
        "layout/",
        "platform/",
        "shared/",
        "capabilities/",
    ]
    .iter()
    .any(|prefix| entry.scenario_owner.starts_with(prefix));
    if !valid_owner || entry.surface != entry.scenario_owner {
        findings.push(finding(
            "ui.tests.owner",
            &format!(
                "entry `{}` has an invalid or mismatched scenario owner",
                entry.id
            ),
        ));
    }
    if !matches!(
        entry.layer.as_str(),
        "model" | "component" | "route-contract" | "platform-contract"
    ) || !entry.lane_owner.starts_with("vitest/")
        || entry.risk.trim().is_empty()
        || entry.required_environment.trim().is_empty()
        || entry.status != "implemented"
    {
        findings.push(finding(
            "ui.tests.contract",
            &format!("entry `{}` has an invalid required field", entry.id),
        ));
    }
    if entry.test_file.contains("/__tests__/") {
        let valid_handoff = entry
            .delivery_plan
            .zip(entry.legacy_handoff.as_ref())
            .is_some_and(|(plan, handoff)| {
                handoff.current_path == entry.test_file
                    && handoff.destination_owner == entry.scenario_owner
                    && handoff.removal_plan == plan
                    && handoff.created == "2026-07-15"
                    && handoff.expires == format!("plan-{plan}-completion")
            });
        if !valid_handoff {
            findings.push(finding(
                "ui.tests.handoff",
                &format!(
                    "entry `{}` has a broad or inconsistent legacy handoff",
                    entry.id
                ),
            ));
        }
    } else if !entry.test_file.contains("/tests/")
        || entry.legacy_handoff.is_some()
        || entry.delivery_plan.is_some()
    {
        findings.push(finding(
            "ui.tests.topology",
            &format!("entry `{}` is outside the final tests/ topology", entry.id),
        ));
    }
    if !root.join(&entry.test_file).is_file() || entry.test_ids.is_empty() {
        findings.push(finding(
            "ui.tests.file",
            &format!(
                "entry `{}` does not resolve to non-empty evidence",
                entry.id
            ),
        ));
    }
    let target = represented.entry(entry.test_file.clone()).or_default();
    for test_id in &entry.test_ids {
        if !target.insert(test_id.clone()) {
            findings.push(finding(
                "ui.tests.ids",
                &format!("duplicate test ID `{test_id}` in `{}`", entry.test_file),
            ));
        }
    }
    Ok(())
}

fn check_local_harness(root: &Path, path: &str, findings: &mut Vec<Finding>) -> Result<()> {
    let source = fs::read_to_string(root.join(path))?;
    for forbidden in [
        "window.scrollTo =",
        "window.matchMedia =",
        "globalThis.ResizeObserver =",
        "HTMLElement.prototype.scrollIntoView =",
    ] {
        if source.contains(forbidden) {
            findings.push(finding(
                "ui.tests.harness",
                &format!("`{path}` duplicates shared browser shim `{forbidden}`"),
            ));
        }
    }
    Ok(())
}

fn discover_tests(workspace: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut files = Vec::new();
    collect_files(&workspace.join("ui/src"), &mut files)?;
    collect_files(&workspace.join("ui/tests/harness"), &mut files)?;
    let mut tests = BTreeMap::new();
    for path in files {
        let name = path.to_string_lossy();
        if !(name.ends_with(".test.ts") || name.ends_with(".test.tsx")) {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        let ids = source.lines().filter_map(test_id).collect::<BTreeSet<_>>();
        if !ids.is_empty() {
            let relative = path.strip_prefix(workspace).with_context(|| {
                format!("{} is outside {}", path.display(), workspace.display())
            })?;
            tests.insert(relative.to_string_lossy().replace('\\', "/"), ids);
        }
    }
    Ok(tests)
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn test_id(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("it(\"")
        .or_else(|| trimmed.strip_prefix("test(\""))?;
    Some(rest.split_once('"')?.0.to_owned())
}

fn finding(rule_id: &str, reason: &str) -> Finding {
    Finding::error(
        rule_id,
        MATRIX_PATH,
        1,
        reason,
        "update the exact test evidence and ownership in ui/test-matrix.json",
        RERUN,
    )
}

#[cfg(test)]
mod tests;
