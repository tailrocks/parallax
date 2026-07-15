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
    ratchets: Ratchets,
    private_route_imports: Vec<PrivateRouteImport>,
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Ratchets {
    fire_event_calls: usize,
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
    fire_event_reason: Option<String>,
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

#[derive(Debug, Deserialize)]
struct PrivateRouteImport {
    test_file: String,
    module: String,
    symbols: Vec<String>,
    removal_plan: u16,
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
    let mut fire_event_calls = 0;
    for (path, test_ids) in &discovered {
        if path.starts_with("ui/src/test/") {
            findings.push(finding(
                "ui.tests.topology",
                &format!("test body `{path}` is inside the harness-only src/test directory"),
            ));
        }
        fire_event_calls += check_test_source(root, path, &mut findings)?;
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
    validate_private_route_imports(root, &matrix, &mut findings)?;
    if fire_event_calls != matrix.ratchets.fire_event_calls {
        findings.push(finding(
            "ui.tests.fire-event-ratchet",
            &format!(
                "fireEvent call count is {fire_event_calls}, matrix ratchet is {}",
                matrix.ratchets.fire_event_calls
            ),
        ));
    }
    Ok(findings)
}

fn validate_private_route_imports(
    root: &Path,
    matrix: &Matrix,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    let plans = matrix
        .entries
        .iter()
        .filter_map(|entry| entry.delivery_plan.map(|plan| (&entry.test_file, plan)))
        .collect::<BTreeMap<_, _>>();
    let mut expected = BTreeSet::new();
    for import in &matrix.private_route_imports {
        let valid = plans.get(&import.test_file) == Some(&import.removal_plan)
            && import.module.starts_with("@/routes/")
            && !import.symbols.is_empty();
        if !valid {
            findings.push(finding(
                "ui.tests.private-route",
                &format!(
                    "private route handoff for `{}` is invalid",
                    import.test_file
                ),
            ));
        }
        for symbol in &import.symbols {
            if !expected.insert((
                import.test_file.clone(),
                import.module.clone(),
                symbol.clone(),
            )) {
                findings.push(finding(
                    "ui.tests.private-route",
                    &format!(
                        "duplicate private route symbol `{symbol}` in `{}`",
                        import.test_file
                    ),
                ));
            }
        }
    }
    let actual = discover_private_route_imports(root)?;
    if actual != expected {
        findings.push(finding(
            "ui.tests.private-route",
            &format!("private route imports differ: expected {expected:?}, discovered {actual:?}"),
        ));
    }
    Ok(())
}

fn discover_private_route_imports(root: &Path) -> Result<BTreeSet<(String, String, String)>> {
    let mut discovered = BTreeSet::new();
    for path in discover_tests(root)?.keys() {
        let source = fs::read_to_string(root.join(path))?;
        let mut statement = String::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                statement.clear();
            }
            if !statement.is_empty() || trimmed.starts_with("import ") {
                statement.push_str(trimmed);
                statement.push(' ');
            }
            if statement.contains(" from \"@/routes/") {
                collect_private_symbols(path, &statement, &mut discovered);
                statement.clear();
            }
        }
    }
    Ok(discovered)
}

fn collect_private_symbols(
    path: &str,
    statement: &str,
    discovered: &mut BTreeSet<(String, String, String)>,
) {
    let Some(module) = statement
        .split_once(" from \"")
        .and_then(|(_, tail)| tail.split_once('"').map(|(module, _)| module))
    else {
        return;
    };
    let Some((_, symbols)) = statement.split_once('{') else {
        return;
    };
    let Some((symbols, _)) = symbols.split_once('}') else {
        return;
    };
    for symbol in symbols
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        discovered.insert((path.to_owned(), module.to_owned(), symbol.to_owned()));
    }
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
    validate_fire_event_reason(root, entry, findings);
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

fn validate_fire_event_reason(root: &Path, entry: &Entry, findings: &mut Vec<Finding>) {
    let Ok(source) = fs::read_to_string(root.join(&entry.test_file)) else {
        return;
    };
    let fire_events = source.match_indices("fireEvent.").count();
    let has_reason = entry
        .fire_event_reason
        .as_deref()
        .is_some_and(|reason| !reason.trim().is_empty());
    if (fire_events > 0) != has_reason {
        findings.push(finding(
            "ui.tests.fire-event-reason",
            &format!(
                "entry `{}` must own an exact reason iff its file uses fireEvent",
                entry.id
            ),
        ));
    }
}

fn check_test_source(root: &Path, path: &str, findings: &mut Vec<Finding>) -> Result<usize> {
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
    for forbidden in [
        (".only(", "focused test"),
        (".skip(", "skipped test"),
        ("toMatchSnapshot(", "snapshot assertion"),
        ("toMatchInlineSnapshot(", "inline snapshot assertion"),
        ("setTimeout(", "real-time sleep/timer"),
        ("setInterval(", "real-time interval/timer"),
        ("diagnosticAllowlist", "diagnostic allowlist"),
    ] {
        if source.contains(forbidden.0) {
            findings.push(finding(
                "ui.tests.antipattern",
                &format!(
                    "`{path}` contains forbidden {} `{}`",
                    forbidden.1, forbidden.0
                ),
            ));
        }
    }
    Ok(source.match_indices("fireEvent.").count())
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
