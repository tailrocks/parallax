use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::diagnostic::Finding;

mod scan;

#[cfg(test)]
use scan::test_id;
use scan::{
    check_test_source, discover_private_route_imports, discover_tests, validate_catalog,
    validate_oxlint,
};

const MATRIX_PATH: &str = "ui/test-matrix.json";
const RERUN: &str = "cargo xtask policy --only ui.tests";
pub(super) const REQUIRED_SURFACES: [&str; 21] = [
    "capabilities/time-range",
    "features/dashboards",
    "features/ecosystem",
    "features/investigations",
    "features/issues",
    "features/logs",
    "features/overview",
    "features/runs",
    "features/services",
    "features/sql",
    "features/traces",
    "layout/shell",
    "platform/browser",
    "platform/graphql",
    "platform/live",
    "platform/test-harness",
    "shared/charts",
    "shared/console",
    "shared/format",
    "shared/ui",
    "shared/visualization",
];
pub(super) const REQUIRED_VITEST_RULES: [&str; 12] = [
    "expect-expect",
    "no-conditional-expect",
    "no-conditional-tests",
    "no-disabled-tests",
    "no-duplicate-hooks",
    "no-focused-tests",
    "no-identical-title",
    "no-standalone-expect",
    "no-test-prefixes",
    "valid-describe-callback",
    "valid-expect",
    "warn-todo",
];

#[derive(Debug, Deserialize)]
pub(super) struct Matrix {
    schema_version: u32,
    ratchets: Ratchets,
    private_route_imports: Vec<PrivateRouteImport>,
    pub(super) entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Ratchets {
    fire_event_calls: usize,
    legacy_handoffs: usize,
    raw_router_builders: usize,
    test_cases: usize,
    test_files: usize,
}

#[derive(Debug, Deserialize)]
pub(super) struct Entry {
    id: String,
    pub(super) surface: String,
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
    raw_router_reason: Option<String>,
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
    validate_oxlint(root, &mut findings);

    let mut ids = BTreeSet::new();
    let mut represented = BTreeMap::new();
    for entry in &matrix.entries {
        validate_entry(root, entry, &mut ids, &mut represented, &mut findings)?;
    }
    validate_catalog(&matrix, &mut findings);

    let discovered = discover_tests(root)?;
    let test_cases = discovered.values().map(BTreeSet::len).sum::<usize>();
    let legacy_handoffs = matrix
        .entries
        .iter()
        .filter(|entry| entry.legacy_handoff.is_some())
        .count();
    let mut fire_event_calls = 0;
    let mut raw_router_builders = 0;
    for (path, test_ids) in &discovered {
        if path.starts_with("ui/src/test/") {
            findings.push(finding(
                "ui.tests.topology",
                &format!("test body `{path}` is inside the harness-only src/test directory"),
            ));
        }
        fire_event_calls += check_test_source(root, path, &mut findings)?;
        raw_router_builders += fs::read_to_string(root.join(path))?
            .match_indices("createMemoryHistory(")
            .count();
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
    for (actual, expected, label) in [
        (discovered.len(), matrix.ratchets.test_files, "test files"),
        (test_cases, matrix.ratchets.test_cases, "test cases"),
        (
            legacy_handoffs,
            matrix.ratchets.legacy_handoffs,
            "legacy handoffs",
        ),
    ] {
        if actual != expected {
            findings.push(finding(
                "ui.tests.inventory-ratchet",
                &format!("{label} count is {actual}, matrix ratchet is {expected}"),
            ));
        }
    }
    if fire_event_calls != matrix.ratchets.fire_event_calls {
        findings.push(finding(
            "ui.tests.fire-event-ratchet",
            &format!(
                "fireEvent call count is {fire_event_calls}, matrix ratchet is {}",
                matrix.ratchets.fire_event_calls
            ),
        ));
    }
    if raw_router_builders != matrix.ratchets.raw_router_builders {
        findings.push(finding(
            "ui.tests.router-ratchet",
            &format!(
                "raw router builder count is {raw_router_builders}, matrix ratchet is {}",
                matrix.ratchets.raw_router_builders
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

    let is_browser = entry.lane_owner.starts_with("playwright/");
    if is_browser {
        validate_browser_entry(root, entry, findings);
    } else {
        validate_vitest_entry(root, entry, findings);
    }

    validate_fire_event_reason(root, entry, findings);
    validate_raw_router_reason(root, entry, findings);
    // Reserved browser inventory rows may share a future file path; only
    // implemented evidence contributes to the represented-id set.
    if entry.status == "implemented" {
        let target = represented.entry(entry.test_file.clone()).or_default();
        for test_id in &entry.test_ids {
            if !target.insert(test_id.clone()) {
                findings.push(finding(
                    "ui.tests.ids",
                    &format!("duplicate test ID `{test_id}` in `{}`", entry.test_file),
                ));
            }
        }
    }
    Ok(())
}

fn validate_vitest_entry(root: &Path, entry: &Entry, findings: &mut Vec<Finding>) {
    if !matches!(
        entry.layer.as_str(),
        "model" | "component" | "route-contract" | "platform-contract"
    ) || !entry.lane_owner.starts_with("vitest/")
        || entry.risk.trim().is_empty()
        || entry.risk.starts_with("Behavior characterized by")
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
}

fn validate_browser_entry(root: &Path, entry: &Entry, findings: &mut Vec<Finding>) {
    let valid_lane = matches!(
        entry.lane_owner.as_str(),
        "playwright/contracts"
            | "playwright/foundation"
            | "playwright/full-stack"
            | "playwright/breadth"
    );
    let valid_layer = matches!(
        entry.layer.as_str(),
        "browser-contract" | "browser-full-stack" | "browser-breadth"
    );
    let valid_status = matches!(entry.status.as_str(), "implemented" | "reserved");
    if !valid_layer
        || !valid_lane
        || !valid_status
        || entry.risk.trim().is_empty()
        || entry.required_environment.trim().is_empty()
        || entry.test_ids.is_empty()
        || !entry.test_file.contains("ui/tests/e2e/")
    {
        findings.push(finding(
            "ui.tests.contract",
            &format!("browser entry `{}` has an invalid required field", entry.id),
        ));
    }
    if entry.legacy_handoff.is_some() {
        findings.push(finding(
            "ui.tests.handoff",
            &format!(
                "browser entry `{}` must not use vitest legacy handoff fields",
                entry.id
            ),
        ));
    }
    if entry.status == "implemented" && !root.join(&entry.test_file).is_file() {
        findings.push(finding(
            "ui.tests.file",
            &format!(
                "implemented browser entry `{}` does not resolve to evidence",
                entry.id
            ),
        ));
    }
    if entry.status == "reserved" && entry.delivery_plan.is_none() {
        findings.push(finding(
            "ui.tests.contract",
            &format!(
                "reserved browser entry `{}` requires a delivery_plan owner",
                entry.id
            ),
        ));
    }
}

fn validate_raw_router_reason(root: &Path, entry: &Entry, findings: &mut Vec<Finding>) {
    let Ok(source) = fs::read_to_string(root.join(&entry.test_file)) else {
        return;
    };
    let has_builder = source.contains("createMemoryHistory(");
    let has_reason = entry
        .raw_router_reason
        .as_deref()
        .is_some_and(|reason| !reason.trim().is_empty());
    if has_builder != has_reason {
        findings.push(finding(
            "ui.tests.router-reason",
            &format!(
                "entry `{}` must own an exact reason iff it builds a raw router",
                entry.id
            ),
        ));
    }
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
