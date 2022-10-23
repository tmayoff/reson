use std::{collections::BTreeMap, rc::Rc};

use crate::parser::token::TokenValue;

use super::token::Token;

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct ArgumentNode {
    pub arguments: Vec<Rc<Node>>,
    pub commas: Vec<Token>,
    pub kwargs: BTreeMap<Rc<Node>, Rc<Node>>,
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

    pub fn append(&mut self, statement: Rc<Node>) {
        if !self.kwargs.is_empty() {
            self.order_error = true;
        }

        if statement.node_type != NodeType::EmptyNode {
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

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct MethodNode {
    source_object: Rc<Node>,
    name: String,
    args: Rc<Node>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct IndexNode {
    iobject: Rc<Node>,
    index: Rc<Node>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub enum NodeType {
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
        args: Rc<Node>,
    },
    DictNode {
        args: Rc<Node>,
    },
    EmptyNode,
    OrNode {
        left: Rc<Node>,
        right: Rc<Node>,
    },
    AndNode {
        left: Rc<Node>,
        right: Rc<Node>,
    },
    ComparisonNode {
        left: Rc<Node>,
        right: Rc<Node>,
        ctype: String,
    },
    ArithmeticNode {
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
    IndexNode(IndexNode),
    MethodNode(MethodNode),
    FunctionNode {
        func_name: String,
        args: Rc<Node>,
    },
    AssignmentNode {
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
    TernaryNode {
        condition: Rc<Node>,
        trueblock: Rc<Node>,
        falseblock: Rc<Node>,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    pub lineno: i32,
    pub colno: i32,
    pub filename: String,
    pub end_lineno: Option<i32>,
    pub end_colno: Option<i32>,
    pub node_type: NodeType,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            lineno: Default::default(),
            colno: Default::default(),
            filename: Default::default(),
            end_lineno: Default::default(),
            end_colno: Default::default(),
            node_type: NodeType::EmptyNode,
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
    pub fn new(tok: Token, node_kind: NodeType) -> Self {
        Self {
            lineno: tok.lineno,
            colno: tok.colno,
            filename: tok.filename,
            end_lineno: None,
            end_colno: None,
            node_type: node_kind,
        }
    }

    pub fn boolean_node(tok: Token) -> Self {
        let v = tok.value.clone();
        assert!(matches!(v, TokenValue::Bool(_)));
        if let TokenValue::Bool(v) = v {
            return Node::new(tok.clone(), NodeType::BoolNode { value: v });
        }

        panic!("Value not a boolean");
    }

    pub fn id_node(tok: Token, id: String) -> Self {
        Node::new(tok, NodeType::IDNode { value: id })
    }

    pub fn number_node(tok: Token, num: i32) -> Self {
        Node::new(tok, NodeType::NumberNode { value: num })
    }

    pub fn string_node(tok: Token, str: String) -> Self {
        Node::new(tok, NodeType::StringNode { value: str })
    }

    pub fn fstring_node(tok: Token, str: String) -> Self {
        Node::new(tok, NodeType::FStringNode { value: str })
    }

    pub fn multiline_fstring_node(tok: Token, str: String) -> Self {
        Node::new(tok, NodeType::MultilineFStringNode { value: str })
    }

    pub fn argument_node(tok: Token) -> Self {
        let kind = NodeType::ArgumentNode(ArgumentNode::new());
        Node::new(tok, kind)
    }

    pub fn array_node(
        args: &Node,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
    ) -> Self {
        let filename = args.filename.clone();
        let kind = NodeType::ArrayNode {
            args: Rc::from(args.to_owned()),
        };
        Self {
            lineno,
            colno,
            filename,
            end_lineno,
            end_colno,
            node_type: kind,
        }
    }

    pub fn dict_node(
        args: &Node,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
    ) -> Self {
        let filename = args.filename.clone();
        let kind = NodeType::DictNode {
            args: Rc::from(args.to_owned()),
        };
        Self {
            lineno,
            colno,
            filename,
            end_lineno,
            end_colno,
            node_type: kind,
        }
    }

    pub fn function_node(
        filename: String,
        lineno: i32,
        colno: i32,
        end_lineno: Option<i32>,
        end_colno: Option<i32>,
        func_name: String,
        args: &Node,
    ) -> Self {
        let kind = NodeType::FunctionNode {
            func_name,
            args: Rc::from(args.to_owned()),
        };
        Self {
            lineno,
            colno,
            filename,
            end_lineno,
            end_colno,
            node_type: kind,
        }
    }

    pub fn method_node(
        filename: String,
        lineno: i32,
        colno: i32,
        source_object: &Node,
        name: String,
        args: &Node,
    ) -> Self {
        let kind = NodeType::MethodNode(MethodNode {
            source_object: Rc::from(source_object.to_owned()),
            name,
            args: Rc::from(args.to_owned()),
        });

        Self {
            lineno,
            colno,
            filename,
            end_lineno: None,
            end_colno: None,
            node_type: kind,
        }
    }

    pub fn if_node(linenode: &Node, condition: &Node, block: &Node) -> Self {
        let kind = NodeType::IfNode {
            condition: Rc::from(condition.to_owned()),
            block: Rc::from(block.to_owned()),
        };

        Self {
            lineno: linenode.lineno,
            colno: linenode.colno,
            filename: linenode.filename.clone(),
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn ifclause_node(linenode: &Node) -> Self {
        let kind = NodeType::IfClauseNode {
            ifs: Vec::new(),
            elseblock: None,
        };

        Self {
            lineno: linenode.lineno,
            colno: linenode.colno,
            filename: linenode.filename.clone(),
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn index_node(iobject: &Node, index: &Node) -> Self {
        let filename = iobject.filename.clone();
        let lineno = iobject.lineno;
        let colno = iobject.colno;

        let kind = NodeType::IndexNode(IndexNode {
            iobject: Rc::from(iobject.to_owned()),
            index: Rc::from(index.to_owned()),
        });
        Self {
            lineno,
            colno,
            filename,
            end_lineno: None,
            end_colno: None,
            node_type: kind,
        }
    }

    pub fn uminus_node(current_location: Token, value: &Node) -> Self {
        let kind = NodeType::UMinusNode {
            value: Rc::from(value.to_owned()),
        };
        Self {
            lineno: current_location.lineno,
            colno: current_location.colno,
            filename: current_location.filename,
            end_lineno: None,
            end_colno: None,
            node_type: kind,
        }
    }

    pub fn not_node(token: Token, value: &Node) -> Self {
        let kind = NodeType::NotNode {
            value: Rc::from(value.to_owned()),
        };
        Self {
            lineno: token.lineno,
            colno: token.colno,
            filename: token.filename,
            end_lineno: None,
            end_colno: None,
            node_type: kind,
        }
    }

    pub fn arithmetic_node(operation: &str, left: &Node, right: &Node) -> Self {
        let lineno = left.lineno;
        let colno = left.colno;

        let kind = NodeType::ArithmeticNode {
            left: Rc::from(left.to_owned()),
            right: Rc::from(right.to_owned()),
            operation: operation.to_string(),
        };

        Self {
            lineno,
            colno,
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn comparison_node(ctype: &str, left: &Node, right: &Node) -> Self {
        let filename = left.filename.clone();
        let lineno = left.lineno;
        let colno = left.colno;
        let kind = NodeType::ComparisonNode {
            left: Rc::from(left.to_owned()),
            right: Rc::from(right.to_owned()),
            ctype: ctype.to_string(),
        };

        Self {
            lineno,
            colno,
            filename,
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn and_node(left: &Node, right: &Node) -> Self {
        let lineno = left.lineno;
        let colno = left.colno;
        let kind = NodeType::AndNode {
            left: Rc::from(left.to_owned()),
            right: Rc::from(right.to_owned()),
        };

        Self {
            lineno,
            colno,
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn or_node(left: &Node, right: &Node) -> Self {
        let lineno = left.lineno;
        let colno = left.colno;

        let kind = NodeType::OrNode {
            left: Rc::from(left.to_owned()),
            right: Rc::from(right.to_owned()),
        };

        Self {
            lineno,
            colno,
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn plusassignment_node(
        filename: String,
        lineno: i32,
        colno: i32,
        var_name: String,
        value: &Node,
    ) -> Self {
        let kind = NodeType::PlusAssignmentNode {
            var_name,
            value: Rc::from(value.to_owned()),
        };

        Self {
            lineno,
            colno,
            filename,
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn assignment_node(
        filename: String,
        lineno: i32,
        colno: i32,
        var_name: String,
        value: &Node,
    ) -> Self {
        let kind = NodeType::AssignmentNode {
            var_name,
            value: Rc::from(value.to_owned()),
        };

        Self {
            lineno,
            colno,
            filename,
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn ternary_node(condition: &Node, trueblock: &Node, falseblock: &Node) -> Self {
        let filename = condition.filename.clone();
        let lineno = condition.lineno;
        let colno = condition.colno;

        let kind = NodeType::TernaryNode {
            condition: Rc::from(condition.to_owned()),
            trueblock: Rc::from(trueblock.to_owned()),
            falseblock: Rc::from(falseblock.to_owned()),
        };

        Self {
            lineno,
            colno,
            filename,
            node_type: kind,
            ..Default::default()
        }
    }

    pub fn empty_node(lineno: i32, colno: i32, filename: String) -> Self {
        Self {
            lineno,
            colno,
            filename,
            node_type: NodeType::EmptyNode,
            ..Default::default()
        }
    }
}
