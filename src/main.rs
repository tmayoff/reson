use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, str::FromStr};

mod parser;

#[derive(Subcommand)]
enum Commands {
    Setup,
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
        Commands::Setup => {
            parser::parse_file(&PathBuf::from_str(".")?)?;
        }
        Commands::Build => todo!(),
    }

    Ok(())
}
