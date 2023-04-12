use std::{collections::BTreeMap, rc::Rc};

use super::token::Token;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArgumentNode {
    pub arguments: Vec<Rc<Node>>,
    pub commas: Vec<Token>,
    pub kwargs: BTreeMap<Rc<Node>, Rc<Node>>,
    pub order_error: bool,
}

impl ArgumentNode {
    pub fn append(&mut self, statement: Rc<Node>) {
        if !self.kwargs.is_empty() {
            self.order_error = true;
        }

        if *statement != Node::Empty {
            self.arguments.push(statement);
        }
    }

    pub fn set_kwarg(&mut self, name: Rc<Node>, value: Rc<Node>) {
        // TODO warning about duplicate kwargs
        self.kwargs.insert(name, value);
    }

    pub fn set_kwarg_no_check(&mut self, name: Rc<Node>, value: Rc<Node>) {
        self.kwargs.insert(name, value);
    }

    pub fn incorrect_order(&self) -> bool {
        self.order_error
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MethodNode {
    pub source_object: Rc<Node>,
    pub name: String,
    pub args: Rc<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexNode {
    pub indexed_node: Rc<Node>,
    pub index: Rc<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Node {
    #[default]
    Empty,

    BoolNode {
        value: bool,
    },
    ID {
        value: String,
    },
    Number {
        value: i32,
    },
    String {
        value: String,
    },
    FStringNode {
        value: String,
    },
    MultilineFStringNode {
        value: String,
    },
    ContinueNode,
    BreakNode,
    Argument(ArgumentNode),
    Array {
        args: Rc<Node>,
    },
    Dict {
        args: Rc<Node>,
    },
    OrNode {
        left: Rc<Node>,
        right: Rc<Node>,
    },
    AndNode {
        left: Rc<Node>,
        right: Rc<Node>,
    },
    Comparison {
        left: Rc<Node>,
        right: Rc<Node>,
        ctype: String,
    },
    Arithmetic {
        left: Rc<Node>,
        right: Rc<Node>,
        operation: String,
    },
    NotNode {
        value: Rc<Node>,
    },
    CodeBlock {
        lines: Vec<Rc<Node>>,
    },
    Index(IndexNode),
    Method(MethodNode),
    Function {
        func_name: String,
        args: Rc<Node>,
    },
    Assignment {
        var_name: String,
        value: Rc<Node>,
    },
    PlusAssignmentNode {
        var_name: String,
        value: Rc<Node>,
    },
    ForeachClauseNode {
        varname: Vec<String>,
        items: Rc<Node>,
        block: Rc<Node>,
    },
    IfNode {
        condition: Rc<Node>,
        block: Rc<Node>,
    },
    IfClauseNode {
        ifs: Vec<Rc<Node>>,
        elseblock: Option<Rc<Node>>,
    },
    UMinusNode {
        value: Rc<Node>,
    },
    Ternary {
        condition: Rc<Node>,
        trueblock: Rc<Node>,
        falseblock: Rc<Node>,
    },
}
