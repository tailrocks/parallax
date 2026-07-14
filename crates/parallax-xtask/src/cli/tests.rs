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
        vec!["xtask", "docs", "links"],
        vec!["xtask", "policy"],
        vec!["xtask", "policy", "--only", "architecture"],
        vec!["xtask", "arch"],
        vec!["xtask", "dependencies", "--rust"],
        vec!["xtask", "dependencies", "--ui"],
        vec!["xtask", "dependencies", "--all"],
        vec!["xtask", "nextest-evidence", "--profile", "ci"],
        vec!["xtask", "health"],
        vec!["xtask", "facade", "refresh"],
        vec!["xtask", "facade", "check"],
        vec!["xtask", "semconv", "check"],
        vec!["xtask", "semconv", "generate"],
        vec![
            "xtask",
            "release-package",
            "--binary",
            "target/release/parallax",
            "--archive",
            "target/dist/parallax.tar.gz",
            "--source-epoch",
            "1",
        ],
        vec![
            "xtask",
            "release-rehearse",
            "--binary",
            "target/release/parallax",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--version",
            "0.1.0",
            "--source-epoch",
            "1",
        ],
        vec![
            "xtask",
            "release-verify",
            "--archive",
            "parallax-x86_64-unknown-linux-gnu.tar.gz",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--version",
            "0.1.0-preview.1+abcdef0",
            "--source-epoch",
            "1",
            "--source-commit",
            "abcdef0123456789abcdef0123456789abcdef01",
            "--signer-identity",
            "https://github.com/tailrocks/parallax/.github/workflows/preview.yml@refs/heads/main",
            "--signer-workflow",
            "tailrocks/parallax/.github/workflows/preview.yml",
        ],
    ] {
        Cli::try_parse_from(args).expect("documented command should parse");
    }
}

#[test]
fn dependencies_requires_exactly_one_scope() {
    Cli::try_parse_from(["xtask", "dependencies"]).unwrap_err();
    Cli::try_parse_from(["xtask", "dependencies", "--rust", "--ui"]).unwrap_err();
}

#[test]
fn ci_requires_exactly_one_partition() {
    Cli::try_parse_from(["xtask", "ci"]).unwrap_err();
    Cli::try_parse_from(["xtask", "ci", "--fast", "--full"]).unwrap_err();
    let cli = Cli::try_parse_from(["xtask", "lint"]).expect("lint should parse");
    assert!(matches!(cli.command, Command::Lint));
}
