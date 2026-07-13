mod cli;
mod command;
mod dependencies;
pub mod diagnostic;
mod docs_links;
mod facade;
mod nextest_evidence;
mod policy;

use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    command::execute(cli::Cli::parse())
}
