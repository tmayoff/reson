use anyhow::Result;
use clap::{Parser, Subcommand};
use interpreter::Interpreter;
use std::path::PathBuf;

mod interpreter;
mod parser;

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
        } => {
            // TODO interpret function
            let mut interpreter = Interpreter::new();
            let root_meson = source_dir.unwrap_or(".".into()).join("meson.build");

            let prog = parser::parse_file(&root_meson)?;
            interpreter.interpret(&prog)?;
        }
        Commands::Build => todo!(),
    }

    Ok(())
}
