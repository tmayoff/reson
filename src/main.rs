#[macro_use]
extern crate log;

use snafu::{prelude::*, Whatever};
use std::{env::current_dir, path::Path, path::PathBuf};

use chrono::prelude::*;
use clap::{Parser, Subcommand};

mod build;
mod environment;
mod interpreter;
use crate::build::Build;
use crate::environment::Environment;
use crate::interpreter::Interpreter;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_FILE_NAME: &str = "meson.build";

fn has_build_file(dir: &PathBuf) -> bool {
    let mut d = PathBuf::new();
    d.push(dir);
    d.push(BUILD_FILE_NAME);
    d.exists()
}

fn validate_core_dirs(
    dir1: Option<PathBuf>,
    dir2: Option<PathBuf>,
) -> Result<(PathBuf, PathBuf), Whatever> {
    let mut _dir1 = PathBuf::new();
    let mut _dir2 = PathBuf::new();
    match dir1 {
        Some(dir1) => {
            _dir1 = dir1;
        }
        None => {
            if dir2.is_none() {
                if !Path::new("meson.build").exists() && Path::new("../meson.build").exists() {
                    _dir2.push("..");
                } else {
                    whatever!("Must specify at least one directory")
                }
            }
            let cwd = std::env::current_dir().expect("Failed to get current working directory");
            _dir1 = cwd;
        }
    }

    if dir2.is_none() {
        _dir2 = current_dir().expect("Failed to get current working directory");
    }

    // let _dir1 = _dir1
    //     .canonicalize()
    //     .expect("Failed to create absolute path");
    // let _dir2 = _dir2
    //     .canonicalize()
    //     .expect("Failed to create absolute path");

    let err = std::fs::create_dir_all(&_dir1);
    if let Err(e) = err {
        error!("Failed to create directories {}", e);
    }

    let err = std::fs::create_dir_all(&_dir2);
    if let Err(e) = err {
        error!("Failed to create directories {}", e);
    }

    // TODO Make sure the dirs are not the same

    if has_build_file(&_dir1) {
        if has_build_file(&_dir2) {
            whatever!("Both directories contain a build file.")
        }

        return Ok((_dir1, _dir2));
    }

    if has_build_file(&_dir2) {
        return Ok((_dir2, _dir1));
    }

    whatever!("Neither directory contains a build file")
}

fn validate_dirs(
    dir1: Option<PathBuf>,
    dir2: Option<PathBuf>,
    _reconfigure: bool,
    _wipe: bool,
) -> Result<(PathBuf, PathBuf), Whatever> {
    // validate core dirs
    let (src_dir, build_dir) = validate_core_dirs(dir1, dir2)?;

    Ok((src_dir, build_dir))
}

#[derive(Subcommand)]
enum Commands {
    Setup { build_dir: Option<PathBuf> },
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
            let (source_dir, build_dir) = validate_dirs(build_dir.to_owned(), None, false, false)
                .expect("Failed to validate dirs");

            let env = Environment::new(Some(source_dir.clone()), Some(build_dir.clone()));

            debug!("Build Started at {}", Local::now());
            info!("Reson Build System");
            info!("Version: {}", VERSION);
            info!("Source Directory {:?}", source_dir);
            info!("Build Directory {:?}", build_dir);

            let build = Build::new(env);

            let interpreter = Interpreter::new(build, None, None, None, None);
        }

        Commands::Compile {} => todo!(),
        //     Commands::Setup => {

        //
        //     }
        //     Commands::Compile => {}
    }
}
