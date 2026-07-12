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
