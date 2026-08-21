// prep - A repository health auditor and build error parser
// Version: 0.100.0
// License: MIT

mod cli;
mod core;
mod checks;
mod build;
mod report;
mod git;
mod config;
mod cache;
mod output;
mod validate;
mod utils;

use clap::Parser;
use cli::commands::Cli;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute()
}