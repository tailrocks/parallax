use super::*;

#[test]
fn fails_closed_on_broad_or_drifting_exceptions() -> Result<()> {
    let policy: toml::Value = toml::from_str(
        r#"
        [prestable.oxfmt]
        version = "latest"
        owner = "Plan 130"
        expiry = "stable"
        [prestable.oxlint-tsgolint]
        version = "0.24.0"
        owner = "Plan 131"
        expiry = "stable"
        [prestable.extra-plugin]
        version = "1.0.0-beta"
        "#,
    )
    .expect("policy");
    let findings = check(&policy);
    let actual = findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let expected = [
        "dependencies.ui.handoff",
        "dependencies.ui.playwright-handoff",
        "dependencies.ui.prestable",
    ]
    .into_iter()
    .collect();
    anyhow::ensure!(actual == expected, "unexpected findings: {findings:?}");
    Ok(())
}

#[test]
fn registry_policy_rejects_missing_integrity_and_license() -> Result<()> {
    let root = tempfile::tempdir()?;
    let package = root.path().join("node_modules/bad");
    std::fs::create_dir_all(&package)?;
    std::fs::write(
        root.path().join("bun.lock"),
        "    \"bad\": [\"bad@1.0.0\", \"\", {}],\n",
    )?;
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"bad","version":"1.0.0","license":"Proprietary"}"#,
    )?;
    let findings = registry_and_license(root.path())?;
    anyhow::ensure!(
        findings.len() == 1 && findings[0].rule_id == "dependencies.ui.registry-license"
    );
    Ok(())
}
