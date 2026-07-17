//! Plan 100 UI architecture control plane: ownership ledger, layer graph, and
//! migration contracts.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{Context, Result};

use crate::diagnostic::Finding;

use super::{
    config::{Ratchet, UiOwnership},
    typescript::{self, collect_source_files},
};

const RERUN: &str = "cargo xtask policy --only ui.architecture";
const RATCHET_RERUN: &str = "cargo xtask policy --only ui.ratchets";

/// Enforce ownership ledger completeness plus the Oxc-backed TypeScript layer graph.
pub(super) fn check_workspace(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let reexports = compatibility_reexport_paths(ratchet);
    let exceptions = layer_exception_set(ratchet);
    let mut findings = typescript::check_workspace(root)?;
    findings.retain(|finding| {
        !is_allowed_layer_finding(root, finding, &reexports, &exceptions)
    });
    findings.extend(check_ownership_ledger(root, ratchet)?);
    findings.extend(check_feature_edges(ratchet)?);
    findings.extend(check_layer_exceptions(ratchet)?);
    Ok(findings)
}

/// TypeScript-only structural ratchets (file/function size and complexity ceilings).
pub(super) fn check_ratchets(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let health = typescript::health(root, ratchet)?;
    let mut measured = BTreeMap::new();
    for finding in health {
        let Some(metric) = finding.rule_id.strip_prefix("health.") else {
            continue;
        };
        if !metric.starts_with("typescript.") {
            continue;
        }
        let scope = if metric.starts_with("typescript.function-") {
            format!("{}:{}", finding.file, finding.line)
        } else {
            finding.file.clone()
        };
        let value = finding
            .reason
            .split_whitespace()
            .find_map(|word| word.parse::<usize>().ok())
            .context("typescript health finding must contain a measurement")?;
        measured.insert((metric.to_owned(), scope), value);
    }

    let limits: BTreeMap<_, _> = ratchet
        .limits
        .iter()
        .filter(|limit| limit.metric.starts_with("typescript."))
        .map(|limit| ((limit.metric.clone(), limit.scope.clone()), limit.ceiling))
        .collect();

    let mut findings = Vec::new();
    for ((metric, scope), value) in &measured {
        match limits.get(&(metric.clone(), scope.clone())) {
            None => findings.push(Finding::error(
                "ui.ratchets.missing",
                scope,
                1,
                &format!(
                    "{metric} scope {scope} measurement {value} exceeds target without an exact ratchet row"
                ),
                "add a shrink-only [[limits]] row or reduce the measurement",
                RATCHET_RERUN,
            )),
            Some(ceiling) if value > ceiling => findings.push(Finding::error(
                "ui.ratchets.growth",
                scope,
                1,
                &format!("{metric} grew to {value} above ceiling {ceiling}"),
                "split the module; never refresh ceilings upward",
                RATCHET_RERUN,
            )),
            Some(ceiling) if value < ceiling => findings.push(Finding::error(
                "ui.ratchets.stale",
                scope,
                1,
                &format!("{metric} shrank to {value}; lower stale ceiling {ceiling}"),
                "lower the ratchet.toml ceiling to the live measurement",
                RATCHET_RERUN,
            )),
            _ => {}
        }
    }
    for ((metric, scope), ceiling) in &limits {
        if !measured.contains_key(&(metric.clone(), scope.clone())) {
            findings.push(Finding::error(
                "ui.ratchets.orphan",
                scope,
                1,
                &format!("{metric} no longer exceeds its target; remove ceiling {ceiling}"),
                "remove the stale ceiling after the live measurement dropped",
                RATCHET_RERUN,
            ));
        }
    }
    Ok(findings)
}

fn check_ownership_ledger(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let live = collect_ui_files(root)?;
    let mut by_path: BTreeMap<&str, &UiOwnership> = BTreeMap::new();
    for entry in &ratchet.ui.ownership {
        if by_path.insert(entry.path.as_str(), entry).is_some() {
            findings.push(Finding::error(
                "ui.architecture.ownership.duplicate",
                &entry.path,
                1,
                "ownership ledger lists this path more than once",
                "keep exactly one ownership row per live file",
                RERUN,
            ));
        }
        if entry.current_owner.trim().is_empty()
            || entry.target_owner.trim().is_empty()
            || entry.migration_plan.trim().is_empty()
            || entry.kind.trim().is_empty()
        {
            findings.push(Finding::error(
                "ui.architecture.ownership.incomplete",
                &entry.path,
                1,
                "ownership row is missing current_owner, target_owner, migration_plan, or kind",
                "fill every required ownership field",
                RERUN,
            ));
        }
        if entry.kind == "compatibility-reexport" && entry.migration_plan == "none" {
            findings.push(Finding::error(
                "ui.architecture.ownership.reexport",
                &entry.path,
                1,
                "compatibility reexport must name a removal migration_plan",
                "set migration_plan to the plan that deletes the reexport",
                RERUN,
            ));
        }
        let is_feature_facade = entry.path.contains("/features/")
            && entry
                .path
                .rsplit('/')
                .next()
                .is_some_and(|name| name == "index.ts" || name == "index.tsx");
        if entry.facade && !is_feature_facade {
            findings.push(Finding::error(
                "ui.architecture.ownership.facade",
                &entry.path,
                1,
                "facade=true is reserved for features/<name>/index.ts public entries",
                "set facade=false or move the public surface to the feature index",
                RERUN,
            ));
        }
        if is_feature_facade && !entry.facade {
            findings.push(Finding::error(
                "ui.architecture.ownership.facade",
                &entry.path,
                1,
                "feature index.ts must be marked facade=true in the ownership ledger",
                "set facade = true on the feature public entry row",
                RERUN,
            ));
        }
    }

    for path in &live {
        if by_path.contains_key(path.as_str()) {
            continue;
        }
        findings.push(Finding::error(
            "ui.architecture.ownership.unclassified",
            path,
            1,
            "handwritten UI file has no ownership-ledger row",
            &format!(
                "add [[ui.ownership]] for `{path}` with current_owner, target_owner, migration_plan, and kind; place under an existing feature/domain/platform/shared/app/layout owner from ui/AGENTS.md"
            ),
            RERUN,
        ));
    }

    for entry in &ratchet.ui.ownership {
        if live.contains(&entry.path) {
            continue;
        }
        findings.push(Finding::error(
            "ui.architecture.ownership.stale",
            &entry.path,
            1,
            "ownership ledger row has no live file",
            "delete the stale row when the file moves or is removed",
            RERUN,
        ));
    }

    Ok(findings)
}

fn check_feature_edges(ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let mut seen = BTreeSet::new();
    for edge in &ratchet.ui.feature_edges {
        let key = (edge.from_feature.as_str(), edge.to_feature.as_str());
        if !seen.insert(key) {
            findings.push(Finding::error(
                "ui.architecture.feature-edge.duplicate",
                "ratchet.toml",
                1,
                &format!(
                    "duplicate feature facade edge {} -> {}",
                    edge.from_feature, edge.to_feature
                ),
                "keep one exact feature-edge row with reason and owner",
                RERUN,
            ));
        }
        if edge.from_feature == edge.to_feature
            || edge.reason.trim().is_empty()
            || edge.owner.trim().is_empty()
        {
            findings.push(Finding::error(
                "ui.architecture.feature-edge.invalid",
                "ratchet.toml",
                1,
                &format!(
                    "invalid feature facade edge {} -> {}",
                    edge.from_feature, edge.to_feature
                ),
                "cross-feature edges require distinct features, reason, and owner",
                RERUN,
            ));
        }
    }
    Ok(findings)
}

fn check_layer_exceptions(ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    for exception in &ratchet.ui.layer_exceptions {
        if [
            exception.source.as_str(),
            exception.target.as_str(),
            exception.rule.as_str(),
            exception.owner.as_str(),
            exception.removal_plan.as_str(),
            exception.reason.as_str(),
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            findings.push(Finding::error(
                "ui.architecture.layer-exception.incomplete",
                &exception.source,
                1,
                "layer exception is missing a required field",
                "fill source, target, rule, owner, removal_plan, and reason",
                RERUN,
            ));
        }
        if exception.removal_plan == "none" {
            findings.push(Finding::error(
                "ui.architecture.layer-exception.permanent",
                &exception.source,
                1,
                "layer exceptions cannot be permanent",
                "name the migration plan that removes this edge",
                RERUN,
            ));
        }
    }
    Ok(findings)
}

fn collect_ui_files(root: &Path) -> Result<BTreeSet<String>> {
    let mut files = Vec::new();
    let ui = root.join("ui");
    // Plan 100 owns src + harness + tooling. Playwright trees are Plan 132+ and
    // register ownership when those plans land their first real files.
    for relative in ["src", "tests/harness", "scripts"] {
        let directory = ui.join(relative);
        if directory.is_dir() {
            collect_source_files(&directory, &mut files)?;
        }
    }
    let mut out = BTreeSet::new();
    for path in files {
        out.insert(normalize_path(root, &path));
    }
    Ok(out)
}

fn compatibility_reexport_paths(ratchet: &Ratchet) -> BTreeSet<String> {
    ratchet
        .ui
        .ownership
        .iter()
        .filter(|entry| entry.kind == "compatibility-reexport")
        .map(|entry| entry.path.clone())
        .collect()
}

fn layer_exception_set(ratchet: &Ratchet) -> BTreeSet<(String, String, String)> {
    ratchet
        .ui
        .layer_exceptions
        .iter()
        .map(|entry| {
            (
                entry.source.clone(),
                entry.target.clone(),
                entry.rule.clone(),
            )
        })
        .collect()
}

fn normalize_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_allowed_layer_finding(
    root: &Path,
    finding: &Finding,
    reexports: &BTreeSet<String>,
    exceptions: &BTreeSet<(String, String, String)>,
) -> bool {
    if finding.rule_id != "typescript.layer" && finding.rule_id != "typescript.feature-facade" {
        return false;
    }
    let source = normalize_path(root, Path::new(&finding.file));
    if reexports.contains(&source)
        && (finding.reason.contains("Shared -> Platform")
            || finding.reason.contains("Shared -> Domain")
            || finding.reason.contains("Routes -> Platform")
            || finding.reason.contains("Layout -> Platform"))
    {
        return true;
    }
    // Exact exception rows match source path embedded in the finding file field.
    exceptions.iter().any(|(exception_source, _, rule)| {
        rule == &finding.rule_id && (exception_source == &source || source.ends_with(exception_source))
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::policy::config::Ratchet;

    fn minimal_ratchet(ownership: Vec<UiOwnership>) -> Ratchet {
        let source = r#"
schema_version = 1
[architecture]
packages = []
[budgets.rust]
root_file_lines = 200
production_file_lines = 400
test_file_lines = 600
function_lines = 100
cognitive_complexity = 25
[budgets.typescript]
route_file_lines = 150
module_lines = 300
test_file_lines = 500
function_lines = 60
cyclomatic_complexity = 12
cognitive_complexity = 15
[product]
"#;
        let mut ratchet: Ratchet = toml::from_str(source).expect("ratchet");
        ratchet.ui.ownership = ownership;
        ratchet
    }

    #[test]
    fn rejects_unclassified_and_stale_ownership_rows() {
        let directory = tempfile::tempdir().expect("temp");
        let root = directory.path();
        fs::create_dir_all(root.join("ui/src/lib")).expect("dirs");
        fs::write(root.join("ui/src/lib/present.ts"), "export const x = 1").expect("write");
        let ratchet = minimal_ratchet(vec![UiOwnership {
            path: "ui/src/lib/missing.ts".into(),
            current_owner: "legacy-lib".into(),
            target_owner: "shared".into(),
            migration_plan: "100".into(),
            kind: "handwritten".into(),
            facade: false,
        }]);
        let findings = check_ownership_ledger(root, &ratchet).expect("ledger");
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule_id == "ui.architecture.ownership.unclassified")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule_id == "ui.architecture.ownership.stale")
        );
    }

    #[test]
    fn accepts_complete_live_ownership_row() {
        let directory = tempfile::tempdir().expect("temp");
        let root = directory.path();
        fs::create_dir_all(root.join("ui/src/lib")).expect("dirs");
        fs::write(root.join("ui/src/lib/present.ts"), "export const x = 1").expect("write");
        let ratchet = minimal_ratchet(vec![UiOwnership {
            path: "ui/src/lib/present.ts".into(),
            current_owner: "legacy-lib".into(),
            target_owner: "shared".into(),
            migration_plan: "100".into(),
            kind: "handwritten".into(),
            facade: false,
        }]);
        let findings = check_ownership_ledger(root, &ratchet).expect("ledger");
        assert!(findings.is_empty(), "{findings:?}");
    }
}
