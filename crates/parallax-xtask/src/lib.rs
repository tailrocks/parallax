mod browser_contracts;
mod browser_foundation;
mod cli;
mod closure_final;
mod command;
mod dependencies;
pub mod diagnostic;
mod docs_links;
mod facade;
mod nextest_evidence;
mod policy;
mod release;
mod semconv;
mod ui_graphql;

use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    command::execute(cli::Cli::parse())
}
