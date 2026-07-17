mod architecture;
mod browser_breadth;
mod browser_contracts;
mod browser_foundation;
mod browser_full_stack;
mod config;
mod docs;
mod product;
mod rust;
mod structural;
#[expect(
    clippy::excessive_nesting,
    clippy::too_many_arguments,
    reason = "compiler analysis"
)]
#[expect(clippy::too_many_lines, reason = "compiler analysis")]
mod typescript;
mod ui_architecture;
mod ui_graphql_contract;
mod ui_runtime_boundaries;
mod ui_tests;

use std::path::Path;

use anyhow::{Result, bail};

use crate::{
    cli::Output,
    diagnostic::{Format, Severity, render},
};

pub(crate) fn run(root: &Path, only: Option<&str>, output: Output) -> Result<()> {
    if let Some(rule) = only
        && !matches!(
            rule,
            "architecture"
                | "typescript"
                | "product"
                | "structural"
                | "ui.tests"
                | "ui.architecture"
                | "ui.ratchets"
                | "ui.browser-foundation"
                | "ui.browser-contracts"
                | "ui.browser-full-stack"
                | "ui.browser-breadth"
                | "ui.graphql-contract"
                | "ui.runtime-boundaries"
        )
    {
        bail!(
            "unknown policy family `{rule}`; available: architecture, product, structural, typescript, ui.architecture, ui.browser-breadth, ui.browser-contracts, ui.browser-foundation, ui.browser-full-stack, ui.graphql-contract, ui.ratchets, ui.runtime-boundaries, ui.tests"
        );
    }
    let ratchet = config::Ratchet::load(&root.join("ratchet.toml"))?;
    let mut findings = Vec::new();
    if only.is_none() || only == Some("architecture") {
        findings.extend(architecture::check_workspace(root, &ratchet)?);
    }
    if only.is_none() || only == Some("typescript") {
        findings.extend(typescript::check_workspace(root)?);
    }
    if only.is_none() || only == Some("ui.architecture") {
        findings.extend(ui_architecture::check_workspace(root, &ratchet)?);
    }
    if only.is_none() || only == Some("ui.ratchets") {
        findings.extend(ui_architecture::check_ratchets(root, &ratchet)?);
    }
    if only.is_none() || only == Some("product") {
        findings.extend(product::check_workspace(root, &ratchet)?);
    }
    if only.is_none() || only == Some("structural") {
        findings.extend(structural::check_workspace(root, &ratchet)?);
    }
    if only.is_none() || only == Some("ui.tests") {
        findings.extend(ui_tests::check_workspace(root)?);
    }
    if only.is_none() || only == Some("ui.browser-foundation") {
        findings.extend(browser_foundation::check_workspace(root)?);
    }
    if only.is_none() || only == Some("ui.browser-contracts") {
        findings.extend(browser_contracts::check_workspace(root)?);
    }
    if only.is_none() || only == Some("ui.browser-full-stack") {
        findings.extend(browser_full_stack::check_workspace(root)?);
    }
    if only.is_none() || only == Some("ui.browser-breadth") {
        findings.extend(browser_breadth::check_workspace(root)?);
    }
    if only.is_none() || only == Some("ui.graphql-contract") {
        findings.extend(ui_graphql_contract::check_workspace(root)?);
    }
    if only.is_none() || only == Some("ui.runtime-boundaries") {
        findings.extend(ui_runtime_boundaries::check_workspace(root)?);
    }
    if only.is_none() {
        findings.extend(docs::check_workspace(root, &ratchet)?);
    }
    let format = match output {
        Output::Human => Format::Human,
        Output::Json => Format::Json,
        Output::Github => Format::Github,
    };
    print!("{}", render(&findings, format)?);
    if findings
        .iter()
        .any(|finding| finding.severity == Severity::Error)
    {
        bail!("policy found {} violation(s)", findings.len());
    }
    Ok(())
}

pub(crate) fn health(root: &Path, output: Output) -> Result<()> {
    let ratchet = config::Ratchet::load(&root.join("ratchet.toml"))?;
    let mut findings = rust::health(root, &ratchet)?;
    findings.extend(typescript::health(root, &ratchet)?);
    findings.extend(structural::health(root)?);
    let format = match output {
        Output::Human => Format::Human,
        Output::Json => Format::Json,
        Output::Github => Format::Github,
    };
    print!("{}", render(&findings, format)?);
    Ok(())
}

pub(crate) fn typescript_package_imports(
    root: &Path,
) -> Result<std::collections::BTreeSet<String>> {
    typescript::packages::package_imports(root)
}
