use clap::Parser;

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

fn main() {
    let args = CliArgs::parse();

    match args.command {
        Commands::Setup => {
            
        },
        Commands::Compile => {

        },
    }
}
