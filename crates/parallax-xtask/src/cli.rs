use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Output {
    Human,
    Json,
    Github,
}

#[derive(Debug, Parser)]
#[command(name = "cargo xtask", about = "Parallax repository control plane")]
pub struct Cli {
    #[arg(long, global = true, value_enum, default_value_t = Output::Human)]
    pub output: Output,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
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
    /// Report noisy structural metrics without failing policy.
    Health,
    /// Refresh or verify syntax-derived crate facade manifests.
    Facade {
        #[command(subcommand)]
        action: FacadeAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum FacadeAction {
    Refresh,
    Check,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command};

    #[test]
    fn parses_every_initial_command() {
        for args in [
            vec!["xtask", "ci", "--fast"],
            vec!["xtask", "ci", "--full"],
            vec!["xtask", "lint"],
            vec!["xtask", "test"],
            vec!["xtask", "ui"],
            vec!["xtask", "integration"],
            vec!["xtask", "policy"],
            vec!["xtask", "policy", "--only", "architecture"],
            vec!["xtask", "arch"],
            vec!["xtask", "health"],
            vec!["xtask", "facade", "refresh"],
            vec!["xtask", "facade", "check"],
        ] {
            Cli::try_parse_from(args).expect("documented command should parse");
        }
    }

    #[test]
    fn ci_requires_exactly_one_partition() {
        assert!(Cli::try_parse_from(["xtask", "ci"]).is_err());
        assert!(Cli::try_parse_from(["xtask", "ci", "--fast", "--full"]).is_err());
        let cli = Cli::try_parse_from(["xtask", "lint"]).expect("lint should parse");
        assert!(matches!(cli.command, Command::Lint));
    }
}
