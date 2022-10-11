#[macro_use]
extern crate log;

use std::path::Path;

use clap::Parser;
use chrono::prelude::*;

#[derive(Debug, Clone, clap::ValueEnum)]
enum Commands {
    Setup,
    Compile
}

#[derive(Parser)]
struct CliArgs {
    #[clap(value_enum)]
    command: Commands,

    #[arg(short='C')]
    build_dir: String
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();

    let args = CliArgs::parse();
    
    let binding = std::env::current_dir().expect("Can't get source directory");
    let source_directory = binding.as_path();

    match args.command {
        Commands::Setup => {
            let build_directory = source_directory.join(args.build_dir);

            debug!("Build Started at {}", Local::now());
            info!("Reson Build System");
            info!("Version: {}", VERSION);
            info!("Source Directory {:?}", source_directory.to_str());
            info!("Build Directory {:?}", build_directory.to_str());
        },
        Commands::Compile => {

        },
    }
}
