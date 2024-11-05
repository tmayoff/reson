use anyhow::Result;
use ast::{Function, Node, Program};
use reson::Builder;

pub mod ast;

pub struct Interpreter {
    builder: Builder,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            builder: Builder::default(),
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<()> {
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
                ast::Node::Function(function) => self.interpret_function(function),
                ast::Node::Program(_) => todo!(),
                ast::Node::Codeblock(_) => todo!(),
            }
        }

        Ok(())
    }

    fn interpret_function(&mut self, func: &Function) {
        println!("Interpret function");
        match func.name.as_str() {
            "project" => self.interpret_project(func),
            _ => todo!("Unknown function"),
        }
    }

    fn interpret_project(&mut self, func: &Function) {
        let args = &func.args;

        self.builder.project.name = if let Some(n) = args.args.get(0) {
            if let Node::String(str) = n {
                str.clone()
            } else {
                panic!("Fail here")
            }
        } else {
            panic!("Fail here")
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use anyhow::Result;

    #[test]
    fn project() -> Result<()> {
        let input = "project('hello world', 'cpp')";
        let prog = parser::parse(input)?;

        let mut interpreter = Interpreter::new();
        interpreter.interpret(&prog)?;

        assert_eq!(interpreter.builder.project.name, "hello world".to_string());

        Ok(())
    }
}
