mod architecture;
mod config;

use std::path::Path;

use anyhow::{Result, bail};

use crate::{
    cli::Output,
    diagnostic::{Format, Severity, render},
};

pub fn run(root: &Path, only: Option<&str>, output: Output) -> Result<()> {
    if let Some(rule) = only
        && rule != "architecture"
    {
        bail!("unknown policy family `{rule}`; available: architecture");
    }
    let ratchet = config::Ratchet::load(&root.join("ratchet.toml"))?;
    let findings = architecture::check_workspace(root, &ratchet)?;
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
