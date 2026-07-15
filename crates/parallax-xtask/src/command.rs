use std::{path::Path, process::Command as Process};

use anyhow::{Context, Result, bail};

use crate::cli::{Cli, Command, DocsAction, FacadeAction, Output, SemconvAction};
use crate::dependencies::{self, Selection};
use crate::docs_links;
use crate::facade;
use crate::nextest_evidence;
use crate::policy;
use crate::release;
use crate::semconv;

pub(crate) fn execute(cli: Cli) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    match cli.command {
        Command::Ci { fast, full } => execute_ci(&root, fast, full, cli.output),
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
        Command::Semconv {
            action,
            playground_root,
        } => execute_semconv(&root, action, playground_root.as_deref(), cli.output),
        release_command @ (Command::ReleaseValidate { .. }
        | Command::ReleasePackage { .. }
        | Command::ReleaseRehearse { .. }
        | Command::ReleaseVerify { .. }) => execute_release(release_command),
    }
}

fn execute_semconv(
    root: &Path,
    action: SemconvAction,
    playground_root: Option<&Path>,
    output: Output,
) -> Result<()> {
    match action {
        SemconvAction::Check => {
            let report = match semconv::check(root, playground_root) {
                Ok(report) => report,
                Err(error) => {
                    if matches!(output, Output::Json) {
                        println!(
                            "{}",
                            serde_json::json!({
                                "schema_version": 1,
                                "status": "error",
                                "reason": format!("{error:#}"),
                            })
                        );
                    }
                    return Err(error);
                }
            };
            match output {
                Output::Json => println!(
                    "{}",
                    serde_json::json!({
                        "schema_version": report.schema_version,
                        "status": "ok",
                        "artifacts": report.artifacts,
                    })
                ),
                Output::Human | Output::Github => {
                    println!("semantic-convention artifacts are deterministic and current");
                }
            }
            Ok(())
        }
        SemconvAction::Generate => match playground_root {
            Some(playground_root) => semconv::generate_with_playground(root, playground_root),
            None => semconv::generate(root),
        },
    }
}

fn execute_ci(root: &Path, fast: bool, full: bool, output: Output) -> Result<()> {
    debug_assert!(fast ^ full);
    for partition in ci_partitions(full) {
        match partition {
            "lint" => lint(root)?,
            "policy" => policy::run(root, None, output)?,
            "facade" => facade::check(root)?,
            "docs-links" => docs_links::run(root, output)?,
            "ui" => ui(root)?,
            "test" => test(root)?,
            "integration" => integration(root)?,
            "dependencies" => dependencies::run(root, Selection::All, output)?,
            unknown => bail!("internal unknown CI partition `{unknown}`"),
        }
    }
    Ok(())
}

fn execute_release(command: Command) -> Result<()> {
    match command {
        Command::ReleaseValidate { version, channel } => {
            release::validate_channel_version(&version, channel)
        }
        Command::ReleasePackage {
            binary,
            archive,
            target,
            version,
            channel,
            source_epoch,
        } => release::package(&binary, &archive, &target, &version, channel, source_epoch),
        Command::ReleaseRehearse {
            binary,
            target,
            version,
            channel,
            source_epoch,
            output_dir,
        } => release::rehearse(
            &binary,
            &target,
            &version,
            channel,
            source_epoch,
            &output_dir,
        ),
        Command::ReleaseVerify {
            archive,
            target,
            version,
            source_epoch,
            source_commit,
            source_ref,
            repository,
            signer_identity,
            signer_workflow,
        } => release::verify(release::VerifySpec {
            archive,
            target,
            version,
            source_epoch,
            source_commit,
            source_ref,
            repository,
            signer_identity,
            signer_workflow,
        }),
        _ => unreachable!("execute_release receives only release commands"),
    }
}

fn execute_docs(root: &Path, action: DocsAction, output: Output) -> Result<()> {
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
    nextest_evidence::run(root, "ci", Output::Human)
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
