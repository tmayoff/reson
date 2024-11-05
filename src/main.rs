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
        } => {
            // TODO interpret function
            // let mut _interpreter = Interpreter::new(&source_dir.unwrap_or(".".into()), &build_dir);
            // let root_meson = source_dir.unwrap_or(".".into()).join("meson.build");

            // let prog = parser::parse_file(&root_meson)?;
            // interpreter.interpret(&prog)?;
        }
        Commands::Build => todo!(),
    }

    Ok(())
}
