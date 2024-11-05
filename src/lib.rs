use std::path::PathBuf;

#[derive(Default)]
pub struct Builder {
    pub project: Project,
    pub compiler: Compiler,
}

#[derive(Default)]
pub struct Project {
    pub name: String,
    pub version: String,
}

#[derive(Default)]
pub struct Compiler {
    pub name: String,
    pub path: PathBuf,
}
