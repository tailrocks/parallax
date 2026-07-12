use crate::diagnostic::Finding;

use super::failure;

pub(super) fn check(policy: &toml::Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    let prestable = policy.get("prestable").and_then(toml::Value::as_table);
    let actual = prestable
        .into_iter()
        .flat_map(|table| table.keys())
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let expected = ["oxfmt", "oxlint-tsgolint"].into_iter().collect();
    if actual != expected {
        findings.push(contract_failure(
            "pre-stable exceptions must contain exactly Oxfmt and oxlint-tsgolint",
        ));
    }
    for (name, version, owner) in [
        ("oxfmt", "0.58.0", "Plan 130"),
        ("oxlint-tsgolint", "0.24.0", "Plan 131"),
    ] {
        let entry = prestable.and_then(|table| table.get(name));
        for (field, expected) in [("version", version), ("owner", owner)] {
            if entry
                .and_then(|entry| entry.get(field))
                .and_then(toml::Value::as_str)
                != Some(expected)
            {
                findings.push(contract_failure(&format!(
                    "`prestable.{name}.{field}` must be `{expected}`"
                )));
            }
        }
        if entry
            .and_then(|entry| entry.get("expiry"))
            .and_then(toml::Value::as_str)
            .is_none_or(str::is_empty)
        {
            findings.push(contract_failure(&format!(
                "`prestable.{name}` requires a stable-release expiry"
            )));
        }
    }
    for (path, expected) in [
        (&["handoff", "plan-131", "oxlint"][..], "1.73.0"),
        (&["handoff", "plan-131", "typescript"][..], "7.0.2"),
        (
            &["handoff", "plan-129", "testing-library-user-event"][..],
            "14.6.1",
        ),
        (&["handoff", "plan-132", "playwright-test"][..], "1.61.1"),
    ] {
        let value = path.iter().try_fold(policy, |value, key| value.get(*key));
        if value.and_then(toml::Value::as_str) != Some(expected) {
            findings.push(failure(
                "dependencies.ui.handoff",
                "dependency-policy.toml",
                &format!("`{}` must be exact-pinned to `{expected}`", path.join(".")),
                "cargo xtask dependencies --ui",
            ));
        }
    }
    findings
}

fn contract_failure(reason: &str) -> Finding {
    failure(
        "dependencies.ui.prestable",
        "dependency-policy.toml",
        reason,
        "cargo xtask dependencies --ui",
    )
}

#[cfg(test)]
mod tests;
