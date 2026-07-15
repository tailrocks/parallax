use std::fs;

use super::{check_workspace, test_id};

#[test]
fn extracts_only_named_top_level_test_calls() -> Result<(), String> {
    let actual = (
        test_id("  it(\"keeps behavior\", () => {}"),
        test_id("test(\"maps value\", () => {})"),
        test_id("describe(\"group\", () => {})"),
        test_id("it.each([])(\"variant\", () => {})"),
    );
    let expected = (
        Some("keeps behavior".into()),
        Some("maps value".into()),
        None,
        None,
    );
    if actual != expected {
        return Err(format!("test ID extraction mismatch: {actual:?}"));
    }
    Ok(())
}

#[test]
fn rejects_test_id_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let test_path = temp.path().join("ui/src/routes/__tests__/sample.test.ts");
    fs::create_dir_all(test_path.parent().expect("test path has parent"))?;
    fs::write(
        &test_path,
        "import { privateHelper } from \"@/routes/sample\";\nwindow.scrollTo = () => {};\nsetTimeout(() => {}, 1);\nit(\"actual behavior\", () => {});\n",
    )?;
    fs::write(
        temp.path().join("ui/test-matrix.json"),
        r#"{
  "schema_version": 1,
  "ratchets": {
    "fire_event_calls": 0,
    "legacy_handoffs": 1,
    "raw_router_builders": 0,
    "test_cases": 1,
    "test_files": 1
  },
  "private_route_imports": [],
  "entries": [{
    "id": "vitest-001",
    "surface": "features/sample",
    "risk": "Behavior characterized by sample.test.ts",
    "scenario_owner": "features/sample",
    "lane_owner": "vitest/route",
    "delivery_plan": 134,
    "layer": "route-contract",
    "test_file": "ui/src/routes/__tests__/sample.test.ts",
    "test_ids": ["stale behavior"],
    "required_environment": "bun",
    "status": "implemented",
    "legacy_handoff": {
      "current_path": "ui/src/routes/__tests__/sample.test.ts",
      "destination_owner": "features/sample",
      "removal_plan": 134,
      "created": "2026-07-15",
      "expires": "plan-134-completion"
    }
  }]
}"#,
    )?;

    let findings = check_workspace(temp.path())?;
    let actual = findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        "ui.tests.ids",
        "ui.tests.harness",
        "ui.tests.antipattern",
        "ui.tests.private-route",
        "ui.tests.contract",
        "ui.tests.catalog",
        "ui.tests.lint",
    ] {
        if !actual.contains(required) {
            return Err(format!("missing expected finding `{required}`").into());
        }
    }
    Ok(())
}
