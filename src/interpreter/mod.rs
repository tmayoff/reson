use std::collections::HashMap;

use crate::build::Build;

use self::objects::BuiltinTypes;

pub mod file;
pub mod interpreter;
mod objects;

pub trait InterpreterTrait {
    fn new(
        build: Build,
        backend: Option<String>,
        subdir: Option<&str>,
        subproject: Option<String>,
        subproject_dir: Option<String>,
    ) -> Result<Self, std::io::Error>
    where
        Self: std::marker::Sized;

    fn get_builtin(&self) -> &HashMap<String, BuiltinTypes>;
    fn get_sourceroot(&self) -> String;
    fn get_funcs(&self) -> Vec<String>;

    fn run(&mut self);

    fn load_root_meson_file(&mut self) -> Result<(), std::io::Error>;

    fn parse_project(&mut self);
}
