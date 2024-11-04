use anyhow::Result;
use ast::Program;

pub mod ast;

pub fn interpret(program: &Program) -> Result<()> {
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
            ast::Node::Function(_) => todo!(),
            ast::Node::Program(_) => todo!(),
            ast::Node::Codeblock(_) => todo!(),
        }
    }

    Ok(())
}
