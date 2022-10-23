use crate::build::Build;

pub mod ninja;

mod build_element;

pub trait Backend {
    fn new(build: &Build) -> Self;

    fn generate(&mut self);

    fn get_name(&self) -> &String;
}
