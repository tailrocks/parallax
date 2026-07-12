use std::{collections::BTreeMap, fs, path::Path};

use anyhow::Result;
use syn::{Meta, Token, punctuated::Punctuated};

use crate::diagnostic::Finding;

use super::{analyze, collect as collect_files};
use crate::policy::config::Ratchet;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Suppression {
    pub(super) lint: String,
    pub(super) reason: Option<String>,
}

pub(super) fn check(root: &Path, ratchet: &Ratchet) -> Result<Vec<Finding>> {
    let mut files = Vec::new();
    collect_files(&root.join("crates"), &mut files)?;
    let mut counts = BTreeMap::<(String, String), usize>::new();
    let mut findings = Vec::new();
    for path in files {
        let metric = analyze(&fs::read_to_string(&path)?)?;
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let crate_name = relative.split('/').nth(1).unwrap_or("unknown").to_owned();
        for suppression in metric.suppression_details {
            if suppression.reason.as_deref().is_none_or(str::is_empty) {
                findings.push(Finding::error(
                    "rust.suppression.reason",
                    &relative,
                    1,
                    &format!(
                        "suppression for `{}` has no nonempty reason",
                        suppression.lint
                    ),
                    "use a narrow reason-bearing expect/allow attribute",
                    "cargo xtask policy --only structural",
                ));
            }
            *counts
                .entry((crate_name.clone(), suppression.lint))
                .or_default() += 1;
        }
    }
    let ceilings: BTreeMap<_, _> = ratchet
        .rust_suppressions
        .iter()
        .map(|row| ((row.crate_name.clone(), row.lint.clone()), row.ceiling))
        .collect();
    for (key, count) in &counts {
        match ceilings.get(key) {
            None => findings.push(finding(key, &format!("count {count} has no exact ceiling"))),
            Some(ceiling) if count > ceiling => findings.push(finding(
                key,
                &format!("count {count} grew above ceiling {ceiling}"),
            )),
            Some(ceiling) if count < ceiling => findings.push(finding(
                key,
                &format!("count shrank to {count}; lower stale ceiling {ceiling}"),
            )),
            Some(_) => {}
        }
    }
    for (key, ceiling) in ceilings {
        if !counts.contains_key(&key) {
            findings.push(finding(
                &key,
                &format!("count is zero; remove ceiling {ceiling}"),
            ));
        }
    }
    Ok(findings)
}

fn finding((crate_name, lint): &(String, String), reason: &str) -> Finding {
    Finding::error(
        "rust.suppression.ratchet",
        &format!("crates/{crate_name}"),
        1,
        &format!("`{lint}` {reason}"),
        "lower or remove the per-crate/per-lint ceiling; never refresh it upward",
        "cargo xtask policy --only structural",
    )
}

pub(super) fn collect(meta: &Meta, output: &mut Vec<Suppression>) {
    let Meta::List(list) = meta else { return };
    if list.path.is_ident("cfg_attr") {
        if let Ok(items) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            for item in items.iter().skip(1) {
                collect(item, output);
            }
        }
        return;
    }
    if !list.path.is_ident("allow") && !list.path.is_ident("expect") {
        return;
    }
    let Ok(items) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) else {
        return;
    };
    let reason = items.iter().find_map(reason);
    for item in items {
        if let Meta::Path(path) = item {
            output.push(Suppression {
                lint: path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::"),
                reason: reason.clone(),
            });
        }
    }
}

fn reason(meta: &Meta) -> Option<String> {
    let Meta::NameValue(name_value) = meta else {
        return None;
    };
    let syn::Expr::Lit(expression) = &name_value.value else {
        return None;
    };
    let syn::Lit::Str(literal) = &expression.lit else {
        return None;
    };
    name_value.path.is_ident("reason").then(|| literal.value())
}

#[cfg(test)]
mod tests;
