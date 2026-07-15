use std::fs;

use super::{check_workspace, test_id};

#[test]
fn extracts_only_named_top_level_test_calls() {
    assert_eq!(
        test_id("  it(\"keeps behavior\", () => {}"),
        Some("keeps behavior".into())
    );
    assert_eq!(
        test_id("test(\"maps value\", () => {})"),
        Some("maps value".into())
    );
    assert_eq!(test_id("describe(\"group\", () => {})"), None);
    assert_eq!(test_id("it.each([])(\"variant\", () => {})"), None);
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
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "ui.tests.ids")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "ui.tests.harness")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "ui.tests.antipattern")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "ui.tests.private-route")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "ui.tests.contract")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_id == "ui.tests.catalog")
    );
    Ok(())
}
