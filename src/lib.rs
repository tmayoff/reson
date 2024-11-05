use std::path::PathBuf;

pub mod interpreter;
pub mod parser;

#[derive(Default)]
pub struct Builder {
    pub project: Project,
    pub compiler: Compiler,

    pub build_targets: Vec<BuildTarget>,
}

#[derive(Default)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub source_dir: PathBuf,
    pub build_dir: PathBuf,
}

#[derive(Default)]
pub struct Compiler {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Default)]
pub struct BuildTarget {
    pub name: String,
}