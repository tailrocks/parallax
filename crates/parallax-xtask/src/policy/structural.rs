use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{Context, Result};

use crate::diagnostic::Finding;

use super::{config::Ratchet, rust, typescript};

pub fn check_workspace(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut health = rust::health(root, ratchet)?;
    health.extend(typescript::health(root, ratchet)?);
    let mut measured = BTreeMap::new();
    for finding in health {
        let metric = finding
            .rule_id
            .strip_prefix("health.")
            .context("health rule prefix")?
            .to_owned();
        let scope = if metric.ends_with("function-lines") {
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
    Ok(evaluate(&measured, &limits))
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
                    "{metric} measurement {value} exceeds target without an exact ratchet row"
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
    let (file, line) = scope
        .rsplit_once(':')
        .and_then(|(file, line)| line.parse().ok().map(|line| (file, line)))
        .unwrap_or((scope, 1));
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
mod tests {
    use super::*;

    #[test]
    fn scope_line_split_preserves_plain_files() {
        let finding = error("x", "ui/src/a.ts:42", "reason");
        assert_eq!((finding.file.as_str(), finding.line), ("ui/src/a.ts", 42));
        let finding = error("x", "Cargo.toml", "reason");
        assert_eq!((finding.file.as_str(), finding.line), ("Cargo.toml", 1));
    }

    #[test]
    fn rejects_missing_growth_shrink_and_stale_rows() {
        let key = ("rust.file-lines".to_owned(), "a.rs".to_owned());
        let measured = BTreeMap::from([(key.clone(), 12)]);
        assert_eq!(
            evaluate(&measured, &BTreeMap::new())[0].rule_id,
            "structural.limit.missing"
        );
        assert_eq!(
            evaluate(&measured, &BTreeMap::from([(key.clone(), 11)]))[0].rule_id,
            "structural.limit.growth"
        );
        assert_eq!(
            evaluate(&measured, &BTreeMap::from([(key.clone(), 13)]))[0].rule_id,
            "structural.limit.stale"
        );
        assert_eq!(
            evaluate(&BTreeMap::new(), &BTreeMap::from([(key, 12)]))[0].rule_id,
            "structural.limit.stale"
        );
    }
}
