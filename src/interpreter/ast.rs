use enum_as_inner::EnumAsInner;
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
pub struct IfClause {
    pub ifs: Vec<If>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct If {
    pub condition: Node,
    pub block: Node,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CompareOp {
    Equal,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Comparison {
    pub left: Box<Node>,
    pub op: CompareOp,
    pub right: Box<Node>,
}

#[derive(PartialEq, Eq, Debug, Clone, EnumAsInner)]
pub enum Node {
    None, // For debugging only

    Boolean(bool),
    Number(i64),
    String(String),

    Identifier(String),

    IfClause(IfClause),

    Assignment(Assignment),
    Comparison(Comparison),
    Arithmetic(Arithmetic),
    Or,
    And,
    Function(Function),
    Program(Program),
    Codeblock(Vec<Node>),
}
