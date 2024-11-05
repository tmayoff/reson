use std::path::{Path, PathBuf};

use crate::{parser, Builder, Project};
use ast::{Function, Node, Program};
use thiserror::Error;

pub mod ast;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Arguments passed to function don't match with required")]
    InvalidArguments(String),

    #[error("Parse error")]
    Parse(#[from] parser::Error),
}

pub struct Interpreter {
    builder: Builder,
}

impl Interpreter {
    pub fn new(source_dir: &Path, build_dir: &Path) -> Self {
        Self {
            builder: Builder {
                project: Project {
                    source_dir: PathBuf::from(source_dir),
                    build_dir: PathBuf::from(build_dir),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    pub fn interpret(&mut self) -> Result<(), Error> {
        // TODO check if file exists
        let prog = parser::parse_file(&self.builder.project.source_dir.join("meson.build"))?;

        self.interpret_program(&prog)?;

        Ok(())
    }

    fn interpret_program(&mut self, program: &Program) -> Result<(), Error> {
        for node in &program.nodes {
            match node {
                ast::Node::None => break,
                ast::Node::Boolean(_) => todo!(),
                ast::Node::Number(_) => todo!(),
                ast::Node::String(_) => todo!(),
                ast::Node::Identifier(_) => todo!(),
                ast::Node::Assignment(_) => todo!(),
                ast::Node::Arithmetic(_) => todo!(),
                ast::Node::Or => todo!(),
                ast::Node::And => todo!(),
                ast::Node::Function(function) => self.interpret_function(function)?,
                ast::Node::Program(_) => todo!(),
                ast::Node::Codeblock(_) => todo!(),
            }
        }

        Ok(())
    }

    fn interpret_function(&mut self, func: &Function) -> Result<(), Error> {
        println!("Interpret function");
        match func.name.as_str() {
            "project" => self.interpret_project(func)?,
            _ => todo!("Unknown function"),
        }

        Ok(())
    }

    fn interpret_project(&mut self, func: &Function) -> Result<(), Error> {
        let args = &func.args;

        self.builder.project.name = if let Some(n) = args.args.get(0) {
            if let Node::String(str) = n {
                str.clone()
            } else {
                return Err(Error::InvalidArguments(
                    "First argument to project must be a project name string".to_string(),
                ));
            }
        } else {
            return Err(Error::InvalidArguments(
                "Project requires arguments".to_string(),
            ));
        };
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::parser;
//     use anyhow::Result;

//     #[test]
//     fn project() -> Result<()> {
//         let input = "project('hello world', 'cpp')";
//         let prog = parser::parse(input)?;

//         let mut interpreter = Interpreter::new();
//         interpreter.interpret(&prog)?;

//         assert_eq!(interpreter.builder.project.name, "hello world".to_string());

//         Ok(())
//     }
// }
