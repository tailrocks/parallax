use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum Output {
    Human,
    Json,
    Github,
}

#[derive(Debug, Parser)]
#[command(name = "cargo xtask", about = "Parallax repository control plane")]
pub(crate) struct Cli {
    #[arg(long, global = true, value_enum, default_value_t = Output::Human)]
    pub output: Output,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Run the deterministic fast or full repository partition.
    Ci {
        #[arg(long, conflicts_with = "full", required_unless_present = "full")]
        fast: bool,
        #[arg(long, conflicts_with = "fast", required_unless_present = "fast")]
        full: bool,
    },
    /// Run Rust formatting and strict Clippy.
    Lint,
    /// Run non-doctest Rust tests through nextest.
    Test,
    /// Install and run every required Bun UI gate.
    Ui,
    /// Run the distinct Rust doctest integration partition.
    Integration,
    /// Validate repository documentation.
    Docs {
        #[command(subcommand)]
        action: DocsAction,
    },
    /// Enforce all required repository policies or one named rule family.
    Policy {
        #[arg(long)]
        only: Option<String>,
    },
    /// Enforce the staged Cargo workspace architecture graph.
    Arch,
    /// Enforce Rust and Bun dependency policy.
    #[command(group(ArgGroup::new("scope").required(true).multiple(false).args(["rust", "ui", "all"])))]
    Dependencies {
        #[arg(long)]
        rust: bool,
        #[arg(long)]
        ui: bool,
        #[arg(long)]
        all: bool,
    },
    /// Validate nextest's structured JUnit evidence for one profile.
    NextestEvidence {
        #[arg(long)]
        profile: String,
    },
    /// Report noisy structural metrics without failing policy.
    Health,
    /// Refresh or verify syntax-derived crate facade manifests.
    Facade {
        #[command(subcommand)]
        action: FacadeAction,
    },
    /// Package one built binary through the deterministic release implementation.
    ReleasePackage {
        #[arg(long)]
        binary: PathBuf,
        #[arg(long)]
        archive: PathBuf,
        #[arg(long)]
        source_epoch: u64,
    },
    /// Package the same binary twice and require byte-identical release output.
    ReleaseRehearse {
        #[arg(long)]
        binary: PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        source_epoch: u64,
        #[arg(long, default_value = "target/dist")]
        output_dir: PathBuf,
    },
    /// Verify one complete archive, checksum, SBOM, signature, and provenance set.
    ReleaseVerify {
        #[arg(long)]
        archive: PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        version: String,
        #[arg(long)]
        source_epoch: u64,
        #[arg(long)]
        source_commit: String,
        #[arg(long, default_value = "refs/heads/main")]
        source_ref: String,
        #[arg(long, default_value = "tailrocks/parallax")]
        repository: String,
        #[arg(long)]
        signer_identity: String,
        #[arg(long)]
        signer_workflow: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DocsAction {
    /// Validate every tracked internal Markdown link and fragment.
    Links,
}

#[derive(Debug, Subcommand)]
pub(crate) enum FacadeAction {
    Refresh,
    Check,
}

#[cfg(test)]
mod tests;
