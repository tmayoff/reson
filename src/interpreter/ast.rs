use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Program {
    pub nodes: Vec<Node>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Arguments {
    pub args: Vec<Node>,
    pub kwargs: HashMap<String, Node>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Arguments,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Arithmetic {
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub op: MathOp,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Assignment {
    pub left: Box<Node>,
    pub right: Box<Node>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Node {
    None, // For debugging only

    Boolean(bool),
    Number(i64),
    String(String),

    Identifier(String),

    Assignment(Assignment),
    Arithmetic(Arithmetic),
    Or,
    And,
    Function(Function),
    Program(Program),
    Codeblock(Vec<Node>),
}
