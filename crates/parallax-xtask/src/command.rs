use std::{path::Path, process::Command as Process};

use anyhow::{Context, Result, bail};

use crate::cli::{Cli, Command, FacadeAction};
use crate::facade;
use crate::policy;

pub fn execute(cli: Cli) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    match cli.command {
        Command::Ci { fast, full } => {
            debug_assert!(fast ^ full);
            for partition in ci_partitions(full) {
                match partition {
                    "lint" => lint(&root)?,
                    "policy" => policy::run(&root, None, cli.output)?,
                    "facade" => facade::check(&root)?,
                    "ui" => ui(&root)?,
                    "test" => test(&root)?,
                    "integration" => integration(&root)?,
                    "audit" => run(&root, "cargo", &["audit"])?,
                    unknown => bail!("internal unknown CI partition `{unknown}`"),
                }
            }
            Ok(())
        }
        Command::Lint => lint(&root),
        Command::Test => test(&root),
        Command::Ui => ui(&root),
        Command::Integration => integration(&root),
        Command::Policy { only } => policy::run(&root, only.as_deref(), cli.output),
        Command::Arch => policy::run(&root, Some("architecture"), cli.output),
        Command::Health => policy::health(&root, cli.output),
        Command::Facade { action } => match action {
            FacadeAction::Refresh => facade::refresh(&root),
            FacadeAction::Check => facade::check(&root),
        },
    }
}

fn ci_partitions(full: bool) -> Vec<&'static str> {
    let mut partitions = vec!["lint", "policy", "facade", "ui"];
    if full {
        partitions.extend(["test", "integration", "audit"]);
    }
    partitions
}

fn lint(root: &Path) -> Result<()> {
    run(root, "cargo", &["fmt", "--all", "--", "--check"])?;
    run(
        root,
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    )
}

fn test(root: &Path) -> Result<()> {
    run(
        root,
        "cargo",
        &[
            "nextest",
            "run",
            "--workspace",
            "--all-targets",
            "--color=always",
        ],
    )
}

fn integration(root: &Path) -> Result<()> {
    run(root, "cargo", &["test", "--workspace", "--doc", "--locked"])
}

fn ui(root: &Path) -> Result<()> {
    let directory = root.join("ui");
    run(&directory, "bun", &["ci"])?;
    for script in ["check", "typecheck", "lint", "test:ci", "build"] {
        run(&directory, "bun", &["run", script])?;
    }
    Ok(())
}

fn run(directory: &Path, program: &str, arguments: &[&str]) -> Result<()> {
    println!("==> {program} {}", arguments.join(" "));
    let status = Process::new(program)
        .args(arguments)
        .current_dir(directory)
        .status()
        .with_context(|| format!("failed to start {program}"))?;
    if !status.success() {
        bail!("{program} {} exited with {status}", arguments.join(" "));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
