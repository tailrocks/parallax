use clap::{ArgGroup, Parser, Subcommand, ValueEnum};

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
}

#[derive(Debug, Subcommand)]
pub(crate) enum FacadeAction {
    Refresh,
    Check,
}

#[cfg(test)]
mod tests;
