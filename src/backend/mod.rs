use crate::{build::Build, interpreter::Interpreter};

pub mod ninja;

mod build_element;

pub trait Backend {
    fn new(build: &Build, interpreter: &Interpreter) -> Self;

    fn generate(&mut self);

    fn get_name(&self) -> &String;
}
