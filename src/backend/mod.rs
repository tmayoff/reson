use std::path::PathBuf;

use crate::build::Build;

pub mod ninja;

mod build_element;

pub trait Backend {
    fn new(build: &Build) -> Self;

    fn generate(&mut self);

    fn get_name(&self) -> &String;

    fn get_build_to_src(&self) -> &PathBuf;
}
