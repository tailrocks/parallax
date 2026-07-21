use super::*;

#[test]
fn dependency_policy_helpers_preserve_diagnostics_and_reject_unsafe_ui() {
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
        (
            rules,
            combined_output(b"unused dependency: example\n", b"scan failed\n"),
        ),
        (
            [
                "dependencies.ui.executable",
                "dependencies.ui.integrity",
                "dependencies.ui.mutable-executable",
                "dependencies.ui.runtime",
                "dependencies.ui.trust",
            ]
            .into_iter()
            .collect(),
            "unused dependency: example\nscan failed".to_owned(),
        )
    );
}
