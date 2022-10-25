use std::path::PathBuf;

use crate::{build::Build, environment::Environment};

pub mod ninja;

mod build_element;

pub trait Backend {
    fn new(env: &Environment, build: &Build) -> Self;

    fn generate(&mut self);

    fn get_name(&self) -> &String;

    fn get_build_to_src(&self) -> &PathBuf;
}
