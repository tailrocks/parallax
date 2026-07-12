use super::*;

#[test]
fn ui_policy_rejects_trust_and_mutable_executables() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        root.path().join("package.json"),
        r#"{"packageManager":"npm@1","trustedDependencies":["x"],"scripts":{"x":"bunx tool@latest"}}"#,
    )
    .expect("package");
    std::fs::write(root.path().join("bun.lock"), "empty").expect("lock");
    let findings = ui_manifest_policy(root.path()).expect("policy");
    let rules = findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        rules,
        [
            "dependencies.ui.executable",
            "dependencies.ui.integrity",
            "dependencies.ui.mutable-executable",
            "dependencies.ui.runtime",
            "dependencies.ui.trust",
        ]
        .into_iter()
        .collect()
    );
}
