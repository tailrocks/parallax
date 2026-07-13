use std::path::Path;

use anyhow::Result;

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
        if entry
            .and_then(|entry| entry.get("integrity"))
            .and_then(toml::Value::as_str)
            .is_none_or(|integrity| !integrity.starts_with("sha512-"))
            || entry
                .and_then(|entry| entry.get("platform-packages"))
                .and_then(toml::Value::as_array)
                .is_none_or(|platforms| platforms.len() != 4)
        {
            findings.push(contract_failure(&format!(
                "`prestable.{name}` requires exact integrity and four supported platform packages"
            )));
        }
    }
    findings.extend(check_handoffs(policy));
    findings
}

fn check_handoffs(policy: &toml::Value) -> Vec<Finding> {
    let mut findings = Vec::new();
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
    for (path, expected) in [
        (
            &["handoff", "plan-132", "transitive-playwright"][..],
            "1.61.1",
        ),
        (
            &["handoff", "plan-132", "transitive-playwright-core"][..],
            "1.61.1",
        ),
    ] {
        if path
            .iter()
            .try_fold(policy, |value, key| value.get(*key))
            .and_then(toml::Value::as_str)
            != Some(expected)
        {
            findings.push(failure(
                "dependencies.ui.playwright-handoff",
                "dependency-policy.toml",
                "Playwright runner, package, and core versions must remain identical",
                "cargo xtask dependencies --ui",
            ));
        }
    }
    let oxlint = policy
        .get("handoff")
        .and_then(|value| value.get("plan-131"));
    if oxlint
        .and_then(|value| value.get("oxlint-integrity"))
        .and_then(toml::Value::as_str)
        .is_none_or(|integrity| !integrity.starts_with("sha512-"))
        || oxlint
            .and_then(|value| value.get("oxlint-platform-packages"))
            .and_then(toml::Value::as_array)
            .is_none_or(|platforms| platforms.len() != 4)
    {
        findings.push(failure(
            "dependencies.ui.handoff",
            "dependency-policy.toml",
            "stable Oxlint requires exact integrity and four supported platform packages",
            "cargo xtask dependencies --ui",
        ));
    }
    findings
}

pub(super) fn blocked_lifecycle(directory: &Path, report: &str) -> Result<Vec<Finding>> {
    let lock = std::fs::read_to_string(directory.join("bun.lock"))?;
    let valid = !report.contains("[postinstall]:")
        && !lock.contains("unrs-resolver@")
        && !directory.join("node_modules/unrs-resolver").exists();
    Ok(if valid {
        Vec::new()
    } else {
        vec![failure(
            "dependencies.ui.lifecycle",
            "dependency-policy.toml",
            "blocked lifecycle set, locked integrity, or reviewed script behavior drifted",
            "cargo xtask dependencies --ui",
        )]
    })
}

pub(super) fn registry_and_license(directory: &Path) -> Result<Vec<Finding>> {
    let lock = std::fs::read_to_string(directory.join("bun.lock"))?;
    let missing_integrity = lock
        .lines()
        .any(|line| registry_entry(line) && !line.contains("sha512-"));
    let mut manifests = Vec::new();
    collect_manifests(&directory.join("node_modules"), &mut manifests)?;
    let allowed = [
        "0BSD",
        "Apache-2.0",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "BlueOak-1.0.0",
        "CC-BY-4.0",
        "CC0-1.0",
        "ISC",
        "MIT",
        "MIT AND ISC",
        "MIT-0",
        "MPL-2.0",
        "OFL-1.1",
        "Python-2.0",
        "Unlicense",
    ];
    let incompatible = manifests
        .into_iter()
        .filter_map(|path| {
            let package: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).ok()?).ok()?;
            package.get("name")?;
            package.get("version")?;
            let license = package.get("license").and_then(serde_json::Value::as_str)?;
            (!allowed.contains(&license)).then_some(format!("{} ({license})", path.display()))
        })
        .collect::<Vec<_>>();
    Ok(
        if missing_integrity
            || !incompatible.is_empty()
            || lock.contains("git+")
            || lock.contains("http://")
        {
            vec![failure(
                "dependencies.ui.registry-license",
                "ui/bun.lock",
                &format!(
                    "registry integrity/source or Apache-compatible license coverage failed: {}",
                    incompatible.join(", ")
                ),
                "cargo xtask dependencies --ui",
            )]
        } else {
            Vec::new()
        },
    )
}

fn registry_entry(line: &str) -> bool {
    if !line.starts_with("    \"") {
        return false;
    }
    let Some(package) = line
        .split(": [\"")
        .nth(1)
        .and_then(|value| value.split('\"').next())
    else {
        return false;
    };
    package
        .rsplit_once('@')
        .and_then(|(_, version)| version.chars().next())
        .is_some_and(|character| character.is_ascii_digit())
}

fn collect_manifests(directory: &Path, manifests: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_manifests(&path, manifests)?;
        } else if path.file_name().and_then(|name| name.to_str()) == Some("package.json") {
            manifests.push(path);
        }
    }
    Ok(())
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
