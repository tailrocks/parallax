use std::{path::Path, process::Command as Process};

use anyhow::{Context, Result, bail};

use crate::cli::{Cli, Command, DocsAction, FacadeAction};
use crate::dependencies::{self, Selection};
use crate::docs_links;
use crate::facade;
use crate::nextest_evidence;
use crate::policy;

pub(crate) fn execute(cli: Cli) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    match cli.command {
        Command::Ci { fast, full } => {
            debug_assert!(fast ^ full);
            for partition in ci_partitions(full) {
                match partition {
                    "lint" => lint(&root)?,
                    "policy" => policy::run(&root, None, cli.output)?,
                    "facade" => facade::check(&root)?,
                    "docs-links" => docs_links::run(&root, cli.output)?,
                    "ui" => ui(&root)?,
                    "test" => test(&root)?,
                    "integration" => integration(&root)?,
                    "dependencies" => dependencies::run(&root, Selection::All, cli.output)?,
                    unknown => bail!("internal unknown CI partition `{unknown}`"),
                }
            }
            Ok(())
        }
        Command::Lint => lint(&root),
        Command::Test => test(&root),
        Command::Ui => ui(&root),
        Command::Integration => integration(&root),
        Command::Docs { action } => execute_docs(&root, action, cli.output),
        Command::Policy { only } => policy::run(&root, only.as_deref(), cli.output),
        Command::Arch => policy::run(&root, Some("architecture"), cli.output),
        Command::Dependencies { rust, ui, all } => {
            dependencies::run(&root, dependency_selection(rust, ui, all), cli.output)
        }
        Command::NextestEvidence { profile } => nextest_evidence::run(&root, &profile, cli.output),
        Command::Health => policy::health(&root, cli.output),
        Command::Facade { action } => execute_facade(&root, action),
    }
}

fn execute_docs(root: &Path, action: DocsAction, output: crate::cli::Output) -> Result<()> {
    match action {
        DocsAction::Links => docs_links::run(root, output),
    }
}

fn execute_facade(root: &Path, action: FacadeAction) -> Result<()> {
    match action {
        FacadeAction::Refresh => facade::refresh(root),
        FacadeAction::Check => facade::check(root),
    }
}

fn dependency_selection(rust: bool, ui: bool, all: bool) -> Selection {
    match (rust, ui, all) {
        (true, false, false) => Selection::Rust,
        (false, true, false) => Selection::Ui,
        (false, false, true) => Selection::All,
        _ => unreachable!("clap requires exactly one dependency scope"),
    }
}

fn ci_partitions(full: bool) -> Vec<&'static str> {
    let mut partitions = vec!["lint", "policy", "facade", "docs-links", "ui"];
    if full {
        partitions.extend(["test", "integration", "dependencies"]);
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
            "--profile",
            "ci",
            "--no-tests=fail",
            "--color=always",
        ],
    )?;
    nextest_evidence::run(root, "ci", crate::cli::Output::Human)
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
