use super::*;
use crate::policy::config::Ratchet;

fn ratchet(root: &Path) -> Ratchet {
    fs::write(
        root.join("ratchet.toml"),
        r#"
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
[[rust_suppressions]]
crate_name = "fixture"
lint = "clippy::unwrap_used"
ceiling = 1
"#,
    )
    .expect("ratchet fixture");
    Ratchet::load(&root.join("ratchet.toml")).expect("load ratchet fixture")
}

#[test]
fn rejects_missing_reasons_and_suppression_growth() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let crate_dir = directory.path().join("crates/fixture/src");
    fs::create_dir_all(&crate_dir).expect("fixture crate");
    let source = crate_dir.join("lib.rs");
    fs::write(
        &source,
        "#![expect(clippy::unwrap_used, reason = \"fixture assertion\")]\n",
    )
    .expect("positive fixture");
    let ratchet = ratchet(directory.path());
    if !check(directory.path(), &ratchet)
        .expect("positive check")
        .is_empty()
    {
        panic!("reasoned suppression at its ceiling must pass");
    }

    fs::write(&source, "#![allow(clippy::unwrap_used)]\n").expect("reason fixture");
    let missing_reason = check(directory.path(), &ratchet).expect("reason check");
    if !missing_reason
        .iter()
        .any(|finding| finding.rule_id == "rust.suppression.reason")
    {
        panic!("missing suppression reason must fail");
    }

    fs::write(
        &source,
        "#![expect(clippy::unwrap_used, reason = \"one\")]\n#[expect(clippy::unwrap_used, reason = \"two\")] fn f() {}\n",
    )
    .expect("growth fixture");
    let growth = check(directory.path(), &ratchet).expect("growth check");
    if !growth.iter().any(|finding| {
        finding.rule_id == "rust.suppression.ratchet" && finding.reason.contains("grew")
    }) {
        panic!("suppression growth must fail");
    }
}
