mod cli;
mod command;
pub mod diagnostic;
mod facade;
mod policy;

use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<()> {
    command::execute(cli::Cli::parse())
}
