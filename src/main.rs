// prep - A repository health auditor and build error parser
// Version: 0.100.0
// License: MIT

mod build;
mod cache;
mod checks;
mod cli;
mod config;
mod core;
mod git;
mod output;
mod report;
mod utils;
mod validate;

use anyhow::Result;
use clap::Parser;
use cli::commands::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
