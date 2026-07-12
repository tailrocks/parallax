use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};

use crate::diagnostic::Finding;

use super::{config::Ratchet, rust, typescript};

pub(super) fn check_workspace(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut health = rust::health(root, ratchet)?;
    health.extend(typescript::health(root, ratchet)?);
    health.extend(self::health(root)?);
    let mut measured = BTreeMap::new();
    for finding in health {
        let metric = finding
            .rule_id
            .strip_prefix("health.")
            .context("health rule prefix")?
            .to_owned();
        let scope = if metric.starts_with("rust.function-") {
            format!(
                "{}::{}",
                finding.file,
                finding
                    .reason
                    .split_whitespace()
                    .next()
                    .context("Rust function name")?
            )
        } else if metric.starts_with("typescript.function-") {
            format!("{}:{}", finding.file, finding.line)
        } else {
            finding.file.clone()
        };
        let value = finding
            .reason
            .split_whitespace()
            .find_map(|word| word.parse::<usize>().ok())
            .context("health finding must contain measurement")?;
        measured.insert((metric, scope), value);
    }
    let limits: BTreeMap<_, _> = ratchet
        .limits
        .iter()
        .map(|limit| ((limit.metric.clone(), limit.scope.clone()), limit.ceiling))
        .collect();
    let mut findings = evaluate(&measured, &limits);
    findings.extend(check_generated(root, ratchet)?);
    Ok(findings)
}

pub(super) fn health(root: &Path) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            if path.is_dir() {
                if !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some(".git" | "node_modules" | "target")
                ) {
                    stack.push(path);
                }
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if path.file_name().and_then(|name| name.to_str()) == Some("AGENTS.md") {
                let bytes = fs::metadata(&path)?.len() as usize;
                findings.push(Finding::warning(
                    "health.agent-doc-bytes",
                    &relative,
                    1,
                    &format!("count {bytes} exceeds target 0"),
                    "keep durable agent rules compact and lower the ratchet after shrinkage",
                    "cargo xtask health",
                ));
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("facade.toml") {
                let value: toml::Value = toml::from_str(&fs::read_to_string(&path)?)?;
                let count = value["roots"]
                    .as_table()
                    .into_iter()
                    .flat_map(|table| table.values())
                    .filter_map(toml::Value::as_array)
                    .map(Vec::len)
                    .sum::<usize>();
                if count > 0 {
                    findings.push(Finding::warning(
                        "health.rust.public-root-items",
                        &relative,
                        1,
                        &format!("count {count} exceeds target 0"),
                        "review every root export and lower the ratchet after removal",
                        "cargo xtask health",
                    ));
                }
            }
            if is_legacy_module(&path, &relative) {
                findings.push(Finding::warning(
                    "health.rust.mod-rs",
                    &relative,
                    1,
                    "count 1 exceeds target 0",
                    "use Rust 2024 self-named module files",
                    "cargo xtask health",
                ));
            }
        }
    }
    Ok(findings)
}

fn is_legacy_module(path: &Path, relative: &str) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("mod.rs")
        && relative.starts_with("crates/")
}

fn check_generated(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    let owned: BTreeSet<_> = ratchet
        .generated
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    for entry in &ratchet.generated {
        if [&entry.generator, &entry.owner, &entry.drift_check]
            .iter()
            .any(|value| value.trim().is_empty())
            || !root.join(&entry.path).is_file()
        {
            findings.push(error(
                "structural.generated.invalid",
                &entry.path,
                "generated ownership is incomplete or its path is missing",
            ));
        }
    }
    let mut stack = vec![root.join("ui")];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if relative.contains(".gen.") && !owned.contains(relative.as_str()) {
                findings.push(error(
                    "structural.generated.unowned",
                    &relative,
                    "generated-looking file has no exact ownership row",
                ));
            }
        }
    }
    Ok(findings)
}

fn evaluate(
    measured: &BTreeMap<(String, String), usize>,
    limits: &BTreeMap<(String, String), usize>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for ((metric, scope), value) in measured {
        match limits.get(&(metric.clone(), scope.clone())) {
            None => findings.push(error(
                "structural.limit.missing",
                scope,
                &format!(
                    "{metric} scope {scope} measurement {value} exceeds target without an exact ratchet row"
                ),
            )),
            Some(ceiling) if value > ceiling => findings.push(error(
                "structural.limit.growth",
                scope,
                &format!("{metric} grew to {value} above ceiling {ceiling}"),
            )),
            Some(ceiling) if value < ceiling => findings.push(error(
                "structural.limit.stale",
                scope,
                &format!("{metric} shrank to {value}; lower stale ceiling {ceiling}"),
            )),
            _ => {}
        }
    }
    let keys: BTreeSet<_> = measured.keys().cloned().collect();
    for ((metric, scope), ceiling) in limits {
        if !keys.contains(&(metric.clone(), scope.clone())) {
            findings.push(error(
                "structural.limit.stale",
                scope,
                &format!("{metric} no longer exceeds its target; remove ceiling {ceiling}"),
            ));
        }
    }
    findings
}

fn error(rule: &str, scope: &str, reason: &str) -> Finding {
    let (file, line) = if let Some((file, _name)) = scope.split_once("::") {
        (file, 1)
    } else {
        scope
            .rsplit_once(':')
            .and_then(|(file, line)| line.parse().ok().map(|line| (file, line)))
            .unwrap_or((scope, 1))
    };
    Finding::error(
        rule,
        file,
        line,
        reason,
        "lower or remove the ratchet in a separate policy change; never refresh it upward",
        "cargo xtask policy --only structural",
    )
}

#[cfg(test)]
mod tests;
