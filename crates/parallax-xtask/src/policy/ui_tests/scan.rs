use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::{Matrix, REQUIRED_SURFACES, REQUIRED_VITEST_RULES, finding};
use crate::diagnostic::Finding;

pub(super) fn validate_oxlint(root: &Path, findings: &mut Vec<Finding>) {
    let source = fs::read_to_string(root.join("ui/.oxlintrc.jsonc")).unwrap_or_default();
    for rule in REQUIRED_VITEST_RULES {
        let required = format!(r#""vitest/{rule}": "error""#);
        if !source.contains(&required) {
            findings.push(finding(
                "ui.tests.lint",
                &format!("stable native Vitest rule `{rule}` must be an error"),
            ));
        }
    }
}

pub(super) fn validate_catalog(matrix: &Matrix, findings: &mut Vec<Finding>) {
    let surfaces = matrix
        .entries
        .iter()
        .map(|entry| entry.surface.as_str())
        .collect::<BTreeSet<_>>();
    for required in REQUIRED_SURFACES {
        if !surfaces.contains(required) {
            findings.push(finding(
                "ui.tests.catalog",
                &format!("required product surface `{required}` has no risk owner"),
            ));
        }
    }
}

pub(super) fn discover_private_route_imports(
    root: &Path,
) -> Result<BTreeSet<(String, String, String)>> {
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

pub(super) fn check_test_source(
    root: &Path,
    path: &str,
    findings: &mut Vec<Finding>,
) -> Result<usize> {
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

pub(super) fn discover_tests(workspace: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
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

pub(super) fn test_id(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("it(\"")
        .or_else(|| trimmed.strip_prefix("test(\""))?;
    Some(rest.split_once('"')?.0.to_owned())
}
