use crate::release::Channel;
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
    /// Install and run every required Bun UI gate, or GraphQL schema tools.
    Ui {
        #[command(subcommand)]
        action: Option<UiAction>,
    },
    /// Serve the built UI for Playwright foundation smoke (plan 132).
    BrowserFoundationServe,
    /// Serve fixture-backed product contracts (plan 144) via injected test adapter.
    BrowserContractsServe,
    /// Serve/attach managed GreptimeDB + Turso full-stack browser lane (plan 145).
    BrowserFullStackServe,
    /// Production UI bundle analysis / two-clean-build gates (plan 148).
    UiBundle {
        #[command(subcommand)]
        action: UiBundleAction,
    },
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
    /// Verify the final mechanical closure commit and auditor attestations.
    ClosureFinal {
        /// Exercise passing and tampered fixtures before the closure commit exists.
        #[arg(long)]
        dry_run: bool,
    },
    /// Refresh or verify syntax-derived crate facade manifests.
    Facade {
        #[command(subcommand)]
        action: FacadeAction,
    },
    /// Generate or verify checked-in semantic-convention artifacts.
    Semconv {
        #[command(subcommand)]
        action: SemconvAction,
        /// Optional checkout of the linked telemetry playground repository.
        #[arg(long)]
        playground_root: Option<PathBuf>,
    },
    /// Validate a release channel/version identity before an expensive build.
    ReleaseValidate {
        #[arg(long)]
        version: String,
        #[arg(long, value_enum)]
        channel: Channel,
    },
    /// Package one built binary through the deterministic release implementation.
    ReleasePackage {
        #[arg(long)]
        binary: PathBuf,
        #[arg(long)]
        archive: PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        version: String,
        #[arg(long, value_enum)]
        channel: Channel,
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
        #[arg(long, value_enum)]
        channel: Channel,
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
pub(crate) enum UiAction {
    /// GraphQL schema export / drift / contract gates (Plan 152).
    Graphql {
        #[command(subcommand)]
        action: UiGraphqlAction,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum UiGraphqlAction {
    /// Write deterministic `ui/graphql/schema.graphql` from parallax-api.
    Export,
    /// Fail when the checked-in schema (and codegen artifacts) drift.
    Check,
}

#[derive(Debug, Subcommand)]
pub(crate) enum UiBundleAction {
    /// Analyze the current (or freshly built) production client against budgets.
    Analyze,
    /// Build production UI twice and require identical normalized inventories.
    BuildTwice,
}

#[derive(Debug, Subcommand)]
pub(crate) enum FacadeAction {
    Refresh,
    Check,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SemconvAction {
    /// Compare deterministic generated output with the checked-in artifacts.
    Check,
    /// Explicitly refresh checked-in artifacts from the registry.
    Generate,
}

#[cfg(test)]
mod tests;
