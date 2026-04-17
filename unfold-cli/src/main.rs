use clap::Parser;
use anyhow::Result;

mod cli;

use cli::{Cli, Commands};
use unfold_core::{cmd_run, cmd_undo, cmd_log, ConflictStrategy};

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
