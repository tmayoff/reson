use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug)]
pub struct Program {
    pub nodes: Vec<Node>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Arguments {}

#[derive(PartialEq, Eq, Debug)]
pub struct Function {
    pub name: String,
    pub args: Arguments,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Node {
    None, // For debugging only

    Boolean(bool),

    Identifier(String),

    Assignment,
    Or,
    And,
    Function(Function),
    Program(Program),
    Codeblock(Vec<Node>),
}
