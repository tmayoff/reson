use clap::{Parser, Subcommand};

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

fn main() {
    let cli = CliArgs::parse();


    match cli.command {
        Commands::Setup => todo!(),
        Commands::Build => todo!(),
    }
}
