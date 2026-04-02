mod cli;
mod logger;
mod utils;
mod core;


use cli::{Cli, Commands };
use clap::Parser;
use anyhow::Result;
use crate::{core::{cmd_run, cmd_undo}, logger::cmd_log};


fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            source,
            dest,
            move_files,
            dry_run,
            conflict,
            cleanup,
        } => cmd_run(&source, &dest, move_files, dry_run, conflict, cleanup)?,
        Commands::Undo => cmd_undo()?,
        Commands::Log => cmd_log()?,
    }
    Ok(())
}














