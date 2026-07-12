mod architecture;
mod config;
mod docs;
mod product;
mod rust;
mod structural;
mod typescript;

use std::path::Path;

use anyhow::{Result, bail};

use crate::{
    cli::Output,
    diagnostic::{Format, Severity, render},
};

pub fn run(root: &Path, only: Option<&str>, output: Output) -> Result<()> {
    if let Some(rule) = only
        && !matches!(
            rule,
            "architecture" | "typescript" | "product" | "structural"
        )
    {
        bail!(
            "unknown policy family `{rule}`; available: architecture, product, structural, typescript"
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
    if only.is_none() || only == Some("product") {
        findings.extend(product::check_workspace(root, &ratchet)?);
    }
    if only.is_none() || only == Some("structural") {
        findings.extend(structural::check_workspace(root, &ratchet)?);
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

pub fn health(root: &Path, output: Output) -> Result<()> {
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
