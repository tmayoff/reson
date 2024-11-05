use anyhow::Result;
use clap::{Parser, Subcommand};

use std::path::PathBuf;

#[derive(Subcommand)]
enum Commands {
    Setup {
        #[arg(short = 'C')]
        source_dir: Option<PathBuf>,
        build_dir: PathBuf,
    },
    Build,
}

#[derive(Parser)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = CliArgs::parse();

    match cli.command {
        Commands::Setup {
            build_dir,
            source_dir,
        } => todo!(),
        Commands::Build => todo!(),
    }

    Ok(())
}
