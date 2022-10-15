use std::collections::BTreeMap;

use crate::parser::token::TokenValue;

use super::token::Token;

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct ArgumentNode {
    arguments: Vec<Box<Node>>,
    pub commas: Vec<Token>,
    kwargs: BTreeMap<Box<Node>, Box<Node>>,
    order_error: bool,
}

impl ArgumentNode {
    fn new() -> Self {
        Self {
            arguments: Vec::new(),
            commas: Vec::new(),
            kwargs: BTreeMap::new(),
            order_error: false,
        }
    }

    pub fn append(&mut self, statement: Box<Node>) {
        if self.kwargs.len() > 0 {
            self.order_error = true;
        }

        if statement.node_kind != NodeKind::EmptyNode {
            self.arguments.push(statement);
        }
    }

    pub fn set_kwarg(&mut self, name: Box<Node>, value: Box<Node>) {
        // TODO warning about duplicate kwargs
        self.kwargs.insert(name, value);
    }

    pub fn set_kwarg_no_check(&mut self, name: Box<Node>, value: Box<Node>) {
        self.kwargs.insert(name, value);
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct MethodNode {
    source_object: Box<Node>,
    name: String,
    args: Box<Node>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct IndexNode {
    iobject: Box<Node>,
    index: Box<Node>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub enum NodeKind {
    BoolNode {
        value: bool,
    },
    IDNode {
        value: String,
    },
    NumberNode {
        value: i32,
    },
    StringNode {
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
    ArgumentNode(ArgumentNode),
    ArrayNode {
        args: Box<Node>,
    },
    DictNode {
        args: Box<Node>,
    },
    EmptyNode,
    OrNode {
        left: Box<Node>,
        right: Box<Node>,
    },
    AndNode {
        left: Box<Node>,
        right: Box<Node>,
    },
    ComparisonNode {
        left: Box<Node>,
        right: Box<Node>,
        ctype: String,
    },
    ArithmeticNode {
        left: Box<Node>,
        right: Box<Node>,
        operation: String,
    },
    NotNode {
        value: Box<Node>,
    },
    CodeBlock {
        lines: Vec<Box<Node>>,
    },
    IndexNode(IndexNode),
    MethodNode(MethodNode),
    FunctionNode {
        func_name: String,
        args: Box<Node>,
    },
    AssignmentNode {
        var_name: String,
        value: Box<Node>,
    },
    PlusAssignmentNode {
        var_name: String,
        value: Box<Node>,
    },
    ForeachClauseNode {
        varname: Vec<String>,
        items: Box<Node>,
        block: Box<Node>,
    },
    IfNode {
        condition: Box<Node>,
        block: Box<Node>,
    },
    IfClauseNode {
        ifs: Vec<Box<Node>>,
        elseblock: Option<Box<Node>>,
    },
    UMinusNode {
        value: Box<Node>,
    },
    TernaryNode {
        condition: Box<Node>,
        trueblock: Box<Node>,
        falseblock: Box<Node>,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    pub lineno: i32,
    pub colno: i32,
    pub filename: String,
    pub end_lineno: Option<i32>,
    pub end_colno: Option<i32>,
    pub node_kind: NodeKind,
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
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(c) => c,
            None => std::cmp::Ordering::Equal,
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.lineno.partial_cmp(&other.lineno) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.colno.partial_cmp(&other.colno) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.filename.partial_cmp(&other.filename) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.end_lineno.partial_cmp(&other.end_lineno) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.end_colno.partial_cmp(&other.end_colno) {
            Some(core::cmp::Ordering::Equal) => return None,
            ord => return ord,
        }
    }
}

impl Node {
    pub fn new(tok: Token, node_kind: NodeKind) -> Box<Self> {
        Box::new(Self {
            lineno: tok.lineno,
            colno: tok.colno,
            filename: tok.filename,
            end_lineno: None,
            end_colno: None,
            node_kind,
        })
    }

    pub fn boolean_node(tok: Token) -> Box<Self> {
        let v = tok.value.clone();
        assert!(matches!(v, TokenValue::Bool(_)));
        if let TokenValue::Bool(v) = v {
            return Node::new(tok.clone(), NodeKind::BoolNode { value: v });
        }

        panic!("Value not a boolean");
    }

    pub fn id_node(tok: Token, id: String) -> Box<Self> {
        Node::new(tok, NodeKind::IDNode { value: id })
    }

    pub fn number_node(tok: Token, num: i32) -> Box<Self> {
        Node::new(tok, NodeKind::NumberNode { value: num })
    }

    pub fn string_node(tok: Token, str: String) -> Box<Self> {
        Node::new(tok, NodeKind::StringNode { value: str })
    }

    pub fn fstring_node(tok: Token, str: String) -> Box<Self> {
        Node::new(tok, NodeKind::FStringNode { value: str })
    }

    pub fn multiline_fstring_node(tok: Token, str: String) -> Box<Self> {
        Node::new(tok, NodeKind::MultilineFStringNode { value: str })
    }

    pub fn argument_node(tok: Token) -> Box<Self> {
        let kind = NodeKind::ArgumentNode(ArgumentNode::new());
        Node::new(tok, kind)
    }

    pub fn array_node(
        args: Box<Node>,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
    ) -> Box<Self> {
        let filename = args.filename.clone();
        let kind = NodeKind::ArrayNode { args };
        Box::new(Self {
            lineno,
            colno,
            filename,
            end_lineno,
            end_colno,
            node_kind: kind,
        })
    }

    pub fn dict_node(
        args: Box<Node>,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
    ) -> Box<Self> {
        let filename = args.filename.clone();
        let kind = NodeKind::DictNode { args };
        Box::new(Self {
            lineno,
            colno,
            filename,
            end_lineno,
            end_colno,
            node_kind: kind,
        })
    }

    pub fn function_node(
        filename: String,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
        func_name: String,
        args: Box<Node>,
    ) -> Box<Self> {
        let kind = NodeKind::FunctionNode { func_name, args };
        Box::new(Self {
            lineno,
            colno,
            filename,
            end_lineno,
            end_colno,
            node_kind: kind,
        })
    }

    pub fn method_node(
        filename: String,
        lineno: i32,
        colno: i32,
        source_object: Box<Node>,
        name: String,
        args: Box<Node>,
    ) -> Box<Self> {
        let kind = NodeKind::MethodNode(MethodNode {
            source_object,
            name,
            args,
        });

        Box::new(Self {
            lineno,
            colno,
            filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
        })
    }

    pub fn if_node(linenode: Box<Node>, condition: Box<Node>, block: Box<Node>) -> Box<Node> {
        let kind = NodeKind::IfNode { condition, block };
        Box::new(Self {
            lineno: linenode.lineno,
            colno: linenode.colno,
            filename: linenode.filename,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn ifclause_node(linenode: &Box<Node>) -> Box<Self> {
        let kind = NodeKind::IfClauseNode {
            ifs: Vec::new(),
            elseblock: None,
        };

        Box::new(Self {
            lineno: linenode.lineno,
            colno: linenode.colno,
            filename: linenode.filename,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn index_node(iobject: Box<Node>, index: Box<Node>) -> Box<Self> {
        let filename = iobject.filename.clone();
        let lineno = iobject.lineno;
        let colno = iobject.colno;

        let kind = NodeKind::IndexNode(IndexNode { iobject, index });
        Box::new(Self {
            lineno,
            colno,
            filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
        })
    }

    pub fn uminus_node(current_location: Token, value: Box<Node>) -> Box<Self> {
        let kind = NodeKind::UMinusNode { value };
        Box::new(Self {
            lineno: current_location.lineno,
            colno: current_location.colno,
            filename: current_location.filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
        })
    }

    pub fn not_node(token: Token, value: Box<Node>) -> Box<Self> {
        let kind = NodeKind::NotNode { value };
        Box::new(Self {
            lineno: token.lineno,
            colno: token.colno,
            filename: token.filename,
            end_lineno: None,
            end_colno: None,
            node_kind: kind,
        })
    }

    pub fn arithmetic_node(operation: &str, left: Box<Node>, right: Box<Node>) -> Box<Self> {
        let lineno = left.lineno;
        let colno = left.colno;

        let kind = NodeKind::ArithmeticNode {
            left,
            right,
            operation: operation.to_string(),
        };

        Box::new(Self {
            lineno,
            colno,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn comparison_node(ctype: &str, left: Box<Node>, right: Box<Node>) -> Box<Self> {
        let filename = left.filename.clone();
        let lineno = left.lineno;
        let colno = left.colno;
        let kind = NodeKind::ComparisonNode {
            left,
            right,
            ctype: ctype.to_string(),
        };

        Box::new(Self {
            lineno,
            colno,
            filename,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn and_node(left: Box<Node>, right: Box<Node>) -> Box<Self> {
        let lineno = left.lineno;
        let colno = left.colno;
        let kind = NodeKind::AndNode { left, right };

        Box::new(Self {
            lineno,
            colno,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn or_node(left: Box<Node>, right: Box<Node>) -> Box<Self> {
        let lineno = left.lineno;
        let colno = left.colno;

        let kind = NodeKind::OrNode { left, right };

        Box::new(Self {
            lineno,
            colno,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn plusassignment_node(
        filename: String,
        lineno: i32,
        colno: i32,
        var_name: String,
        value: Box<Node>,
    ) -> Box<Self> {
        let kind = NodeKind::PlusAssignmentNode { var_name, value };

        Box::new(Self {
            lineno,
            colno,
            filename,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn assignment_node(
        filename: String,
        lineno: i32,
        colno: i32,
        var_name: String,
        value: Box<Node>,
    ) -> Box<Self> {
        let kind = NodeKind::AssignmentNode { var_name, value };

        Box::new(Self {
            lineno,
            colno,
            filename,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn ternary_node(
        condition: Box<Node>,
        trueblock: Box<Node>,
        falseblock: Box<Node>,
    ) -> Box<Self> {
        let filename = condition.filename.clone();
        let lineno = condition.lineno;
        let colno = condition.colno;

        let kind = NodeKind::TernaryNode {
            condition,
            trueblock,
            falseblock,
        };

        Box::new(Self {
            lineno,
            colno,
            filename,
            node_kind: kind,
            ..Default::default()
        })
    }

    pub fn empty_node(lineno: i32, colno: i32, filename: String) -> Box<Self> {
        Box::new(Self {
            lineno,
            colno,
            filename,
            node_kind: NodeKind::EmptyNode,
            ..Default::default()
        })
    }
}
