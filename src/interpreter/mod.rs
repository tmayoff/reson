use std::path::{Path, PathBuf};

use crate::{parser, BuildTarget, Builder, Project};
use ast::{Function, Node, Program};
use thiserror::Error;

pub mod ast;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't find a meson.build file at the path {0}")]
    MesonBuildNotFound(PathBuf),

    #[error("Arguments passed to function don't match with required")]
    InvalidArguments(String),

    #[error("Parse error")]
    Parse(#[from] parser::Error),

    #[error("Invalid node expected: `{0:?}`")]
    InvalidNode(Node),

    #[error("Expected {}, got {:?}", expected, got)]
    Expected { expected: String, got: Node },
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
        let meson_build = self.builder.project.source_dir.join("meson.build");
        if !meson_build.exists() {
            return Err(Error::MesonBuildNotFound(meson_build));
        }

        // TODO check if file exists
        let prog = parser::parse_file(&meson_build)?;

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
                ast::Node::Comparison(_) => todo!(),
                ast::Node::Arithmetic(_) => todo!(),
                ast::Node::Or => todo!(),
                ast::Node::IfClause(_) => todo!(),
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
            "project" => self.project(func)?,
            "executable" => self.executable(func)?,
            _ => todo!("Unknown function"),
        }

        Ok(())
    }

    fn project(&mut self, func: &Function) -> Result<(), Error> {
        let args = &func.args.args;
        let kwargs = &func.args.kwargs;

        self.builder.project.name = if let Some(n) = args.get(0) {
            n.clone().into_string().map_err(|e| Error::Expected {
                expected: "String".to_string(),
                got: e,
            })?
        } else {
            return Err(Error::InvalidArguments(
                "Project requires arguments".to_string(),
            ));
        };

        Ok(())
    }

    fn executable(&mut self, func: &Function) -> Result<(), Error> {
        println!("Add executable build target");

        let args = &func.args.args;
        let kwargs = &func.args.kwargs;

        let target_name = if let Some(n) = args.get(0) {
            n.clone().into_string().map_err(|e| Error::Expected {
                expected: "String".to_string(),
                got: e,
            })?
        } else {
            return Err(Error::InvalidArguments(
                "Executable requires arguments".to_string(),
            ));
        };

        self.builder.build_targets.push(BuildTarget {
            name: target_name,
            files: vec![],
        });

        Ok(())
    }
}
