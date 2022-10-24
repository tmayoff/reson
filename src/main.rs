#[macro_use]
extern crate log;

use snafu::Whatever;
use std::{path::Path, path::PathBuf};

use chrono::prelude::*;
use clap::{Parser, Subcommand};

mod backend;
mod build;
mod compiler;
mod coredata;
mod environment;
mod interpreter;
mod parser;
mod utils;
use crate::backend::ninja::NinjaBackend;
use crate::backend::Backend;
use crate::build::Build;
use crate::environment::Environment;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::InterpreterTrait;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_FILE_NAME: &str = "meson.build";

fn has_build_file(dir: &Path) -> bool {
    let d = dir.join(BUILD_FILE_NAME);
    d.exists()
}

fn validate_core_dirs(
    source_dir: &Option<PathBuf>,
    build_dir: &Path,
) -> Result<(PathBuf, PathBuf), Whatever> {
    let source_dir = match source_dir {
        Some(dir) => dir.to_path_buf(),
        None => {
            std::env::current_dir().expect("Failed to get current working directory as source dir")
        }
    };

    std::fs::create_dir_all(&source_dir)
        .unwrap_or_else(|_| panic!("Failed to create source directory {:?}", &source_dir));

    std::fs::create_dir_all(build_dir)
        .unwrap_or_else(|_| panic!("Failed to create directory {:?}", &build_dir));

    let build_dir = build_dir
        .canonicalize()
        .expect("Failed to create absolute path for source dir");
    let source_dir = source_dir
        .canonicalize()
        .expect("Failed to create absolute path for source dir");

    if has_build_file(&source_dir) {}

    assert!(
        has_build_file(&source_dir),
        "Source dir doesn't contain a meson.build file"
    );

    Ok((source_dir, build_dir))
}

fn validate_dirs(
    source_dir: &Option<PathBuf>,
    build_dir: &Path,
    _reconfigure: bool,
    _wipe: bool,
) -> Result<(PathBuf, PathBuf), Whatever> {
    let (src_dir, build_dir) = validate_core_dirs(source_dir, build_dir)?;

    Ok((src_dir, build_dir))
}

#[derive(Subcommand)]
enum Commands {
    Setup { build_dir: PathBuf },
    Compile {},
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    match &args.command {
        Commands::Setup { build_dir } => {
            let (source_dir, build_dir) =
                validate_dirs(&None, build_dir, false, false).expect("Failed to validate dirs");

            let env =
                Environment::new(&source_dir, &build_dir).expect("Failed to setup environment");

            debug!("Build Started at {}", Local::now());
            debug!("Main Binary {:?}", std::env::current_exe());
            debug!("Build Options: ");
            info!("Reson Build System");
            info!("Version: {}", VERSION);
            info!("Source Directory {:?}", source_dir);
            info!("Build Directory {:?}", build_dir);

            let build = Build::new(env);

            let mut interpreter =
                Interpreter::new(build, None, None, None, None).expect("Should be constructed");

            // This files the build struct based on the parsed file
            interpreter.run();

            // Generate the build definitions for the specified backend
            let mut backend = NinjaBackend::new(&interpreter.build);
            backend.generate();
        }

        Commands::Compile {} => todo!("Wrap ninja"),
    }
}
