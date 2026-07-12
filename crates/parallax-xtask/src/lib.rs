mod cli;
mod command;
pub mod diagnostic;

use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    command::execute(cli::Cli::parse())
}
