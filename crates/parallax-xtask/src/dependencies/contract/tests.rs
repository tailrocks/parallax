use super::*;

#[test]
fn fails_closed_on_broad_or_drifting_exceptions() -> anyhow::Result<()> {
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
    let expected = ["dependencies.ui.handoff", "dependencies.ui.prestable"]
        .into_iter()
        .collect();
    anyhow::ensure!(actual == expected, "unexpected findings: {findings:?}");
    Ok(())
}
