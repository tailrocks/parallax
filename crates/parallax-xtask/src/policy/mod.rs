mod architecture;
mod config;
mod typescript;

use std::path::Path;

use anyhow::{Result, bail};

use crate::{
    cli::Output,
    diagnostic::{Format, Severity, render},
};

pub fn run(root: &Path, only: Option<&str>, output: Output) -> Result<()> {
    if let Some(rule) = only
        && !matches!(rule, "architecture" | "typescript")
    {
        bail!("unknown policy family `{rule}`; available: architecture, typescript");
    }
    let ratchet = config::Ratchet::load(&root.join("ratchet.toml"))?;
    let mut findings = Vec::new();
    if only.is_none() || only == Some("architecture") {
        findings.extend(architecture::check_workspace(root, &ratchet)?);
    }
    if only.is_none() || only == Some("typescript") {
        findings.extend(typescript::check_workspace(root)?);
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
