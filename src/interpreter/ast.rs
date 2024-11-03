use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug)]
pub struct Program {
    pub nodes: Vec<Node>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub kwargs: HashMap<String, String>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Node {
    None, // For debugging only
    Function(Function),
    Program(Program),
}
