use std::collections::HashMap;

use crate::parser::token::TokenValue;

use super::token::Token;

#[derive(PartialEq, Eq)]
pub struct ArgumentNode {
    args: Vec<Node>,
    pub commas: Vec<Token>,
    kwargs: HashMap<Node, Node>,
    order_error: bool,
}

impl ArgumentNode {
    fn new() -> Self {
        Self {
            args: Vec::new(),
            commas: Vec::new(),
            kwargs: HashMap::new(),
            order_error: false,
        }
    }

    pub fn append(&self, statement: Node) {
        if self.kwargs.len() > 0 {
            self.order_error = true;
        }

        if statement.node_kind != NodeKind::EmptyNode {
            self.args.push(statement);
        }
    }

    pub fn set_kwarg(&self, name: Node, value: Node) {
        // TODO warning about duplicate kwargs
        self.kwargs.insert(name, value);
    }

    pub fn set_kwarg_no_check(&self, name: Node, value: Node) {
        self.kwargs.insert(name, value);
    }
}

#[derive(PartialEq, Eq)]
pub struct MethodNode {
    source_object: Box<Node>,
    name: String,
    args: Box<Node>,
}

#[derive(PartialEq, Eq)]
pub struct IndexNode {
    iobject: Box<Node>,
    index: Box<Node>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Node {
    pub lineno: i32,
    pub colno: i32,
    pub filename: String,
    pub end_lineno: Option<i32>,
    pub end_colno: Option<i32>,

    pub node_kind: NodeKind,

    pub value: Option<TokenValue>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            lineno: Default::default(),
            colno: Default::default(),
            filename: Default::default(),
            end_lineno: Default::default(),
            end_colno: Default::default(),
            node_kind: NodeKind::EmptyNode,
            value: Default::default(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum NodeKind {
    EmptyNode,
    BoolNode {
        value: bool,
    },
    CodeBlock {
        lines: Vec<Node>,
    },
    IDNode,
    NumberNode,
    StringNode,
    FStringNode,
    MultilineFStringNode,
    ArgumentNode(ArgumentNode),
    ArrayNode {
        args: Box<Node>,
    },
    DictNode {
        args: Box<Node>,
    },
    FunctionNode {
        args: Box<Node>,
    },
    MethodNode(MethodNode),
    IndexNode(IndexNode),
    UMinusNode {
        value: Box<Node>,
    },
    NotNode {
        value: Box<Node>,
    },
    ArithmeticNode {
        left: Box<Node>,
        right: Box<Node>,
        operation: String,
    },

    ComparisonNode {
        left: Box<Node>,
        right: Box<Node>,
        ctype: String,
    },

    AndNode {
        left: Box<Node>,
        right: Box<Node>,
    },

    OrNode {
        left: Box<Node>,
        right: Box<Node>,
    },

    PlusAssignmentNode {
        var_name: String,
        value: Box<Node>,
    },

    AssignmentNode {
        var_name: String,
        value: Box<Node>,
    },

    TernaryNode {
        condition: Box<Node>,
        trueblock: Box<Node>,
        falseblock: Box<Node>,
    },

    IfNode {
        condition: Box<Node>,
        block: Box<Node>,
    },

    IfClauseNode {
        ifs: Vec<Node>,
        elseblock: Option<Box<Node>>,
    },
}

impl std::hash::Hash for NodeKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Node {
    pub fn new(tok: Token, node_kind: NodeKind) -> Self {
        Self {
            lineno: tok.lineno,
            colno: tok.colno,
            filename: tok.filename,
            end_lineno: None,
            end_colno: None,
            node_kind,
            value: tok.value,
        }
    }

    pub fn boolean_node(tok: Token) -> Self {
        let v = tok.value.expect("Boolean node must contain a value");
        assert!(matches!(v, TokenValue::Bool(_)));
        let TokenValue::Bool(v) = v;
        Node::new(tok, NodeKind::BoolNode { value: v })
    }

    pub fn id_node(tok: Token) -> Self {
        Node::new(tok, NodeKind::IDNode)
    }

    pub fn number_node(tok: Token) -> Self {
        Node::new(tok, NodeKind::NumberNode)
    }

    pub fn string_node(tok: Token) -> Self {
        Node::new(tok, NodeKind::StringNode)
    }

    pub fn fstring_node(tok: Token) -> Self {
        Node::new(tok, NodeKind::FStringNode)
    }

    pub fn multiline_fstring_node(tok: Token) -> Self {
        Node::new(tok, NodeKind::MultilineFStringNode)
    }

    pub fn argument_node(tok: Token) -> Self {
        let kind = NodeKind::ArgumentNode(ArgumentNode::new());
        Node::new(tok, kind)
    }

    pub fn array_node(
        args: Node,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
    ) -> Self {
        let kind = NodeKind::ArrayNode {
            args: Box::from(args),
        };
        Self {
            lineno,
            colno,
            filename: args.filename,
            end_lineno,
            end_colno,
            node_kind: kind,
            value: None,
        }
    }

    pub fn dict_node(
        args: Node,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
    ) -> Self {
        let kind = NodeKind::DictNode {
            args: Box::from(args),
        };
        Self {
            lineno,
            colno,
            filename: args.filename,
            end_lineno,
            end_colno,
            node_kind: kind,
            value: None,
        }
    }

    pub fn function_node(
        filename: String,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
        func_name: String,
        args: Node,
    ) -> Self {
        let kind = NodeKind::FunctionNode {
            args: Box::new(args),
        };

        Self {
            lineno,
            colno,
            filename,
            end_lineno,
            end_colno,
            node_kind: kind,
            value: None,
        }
    }

    pub fn method_node(
        filename: String,
        lineno: i32,
        colno: i32,
        source_object: Node,
        name: String,
        args: Node,
    ) -> Self {
        let kind = NodeKind::MethodNode(MethodNode {
            source_object: Box::new(source_object),
            name,
            args: Box::new(args),
        });

        Self {
            lineno,
            colno,
            filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
            value: None,
        }
    }

    pub fn if_node(linenode: Node, condition: Node, block: Node) -> Node {
        let kind = NodeKind::IfNode {
            condition: Box::new(condition),
            block: Box::new(block),
        };

        Self {
            lineno: linenode.lineno,
            colno: linenode.colno,
            filename: linenode.filename,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn ifclause_node(linenode: Node) -> Node {
        let kind = NodeKind::IfClauseNode {
            ifs: Vec::new(),
            elseblock: None,
        };

        Self {
            lineno: linenode.lineno,
            colno: linenode.colno,
            filename: linenode.filename,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn index_node(iobject: Node, index: Node) -> Self {
        let kind = NodeKind::IndexNode(IndexNode {
            iobject: Box::new(iobject),
            index: Box::new(index),
        });

        Self {
            lineno: iobject.lineno,
            colno: iobject.colno,
            filename: iobject.filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
            value: None,
        }
    }

    pub fn uminus_node(current_location: Token, value: Node) -> Self {
        let kind = NodeKind::UMinusNode {
            value: Box::new(value),
        };

        Self {
            lineno: current_location.lineno,
            colno: current_location.colno,
            filename: current_location.filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
            value: None,
        }
    }

    pub fn not_node(token: Token, value: Node) -> Self {
        let kind = NodeKind::NotNode {
            value: Box::new(value),
        };

        Self {
            lineno: token.lineno,
            colno: token.colno,
            filename: token.filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
            value: None,
        }
    }

    pub fn arithmetic_node(operation: &str, left: Node, right: Node) -> Self {
        let kind = NodeKind::ArithmeticNode {
            left: Box::new(left),
            right: Box::new(right),
            operation: operation.to_string(),
        };

        Node {
            lineno: left.lineno,
            colno: left.colno,
            ..Default::default()
        }
    }

    pub fn comparison_node(ctype: &str, left: Node, right: Node) -> Self {
        let kind = NodeKind::ComparisonNode {
            left: Box::new(left),
            right: Box::new(right),
            ctype: ctype.to_string(),
        };

        Node {
            lineno: left.lineno,
            colno: left.colno,
            filename: left.filename,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn and_node(left: Node, right: Node) -> Self {
        let kind = NodeKind::AndNode {
            left: Box::new(left),
            right: Box::new(right),
        };

        Self {
            lineno: left.lineno,
            colno: left.colno,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn or_node(left: Node, right: Node) -> Self {
        let kind = NodeKind::OrNode {
            left: Box::new(left),
            right: Box::new(right),
        };

        Self {
            lineno: left.lineno,
            colno: left.colno,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn plusassignment_node(
        filename: String,
        lineno: i32,
        colno: i32,
        var_name: String,
        value: Node,
    ) -> Self {
        let kind = NodeKind::PlusAssignmentNode {
            var_name: var_name.to_string(),
            value: Box::new(value),
        };

        Self {
            lineno,
            colno,
            filename,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn assignment_node(
        filename: String,
        lineno: i32,
        colno: i32,
        var_name: String,
        value: Node,
    ) -> Self {
        let kind = NodeKind::AssignmentNode {
            var_name: var_name.to_string(),
            value: Box::new(value),
        };

        Self {
            lineno,
            colno,
            filename,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn ternary_node(condition: Node, trueblock: Node, falseblock: Node) -> Self {
        let kind = NodeKind::TernaryNode {
            condition: Box::new(condition),
            trueblock: Box::new(trueblock),
            falseblock: Box::new(falseblock),
        };

        Self {
            lineno: condition.lineno,
            colno: condition.colno,
            filename: condition.filename,
            node_kind: kind,
            ..Default::default()
        }
    }

    pub fn empty_node(lineno: i32, colno: i32, filename: String) -> Self {
        Self {
            node_kind: NodeKind::EmptyNode,
            ..Default::default()
        }
    }
}
