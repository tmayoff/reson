mod lexer;
mod node;
mod token;

use std::collections::HashMap;

use crate::parser::node::Node;
use crate::parser::token::TokenType;
use lazy_static::lazy_static;

use self::{
    lexer::Lexer,
    node::NodeKind,
    token::{Token, TokenValue},
};

pub struct Parser {
    lexer: Lexer,

    current_tok: Token,

    in_ternary: bool,
}

lazy_static! {
    static ref COMPARISON_MAP: HashMap<TokenType, &'static str> = HashMap::from([
        (TokenType::Equal, "=="),
        (TokenType::NEqual, "!="),
        (TokenType::LT, "<"),
        (TokenType::LE, "<="),
        (TokenType::GT, ">"),
        (TokenType::GE, ">="),
        (TokenType::In, "in"),
        (TokenType::NotIn, "not in"),
    ]);
}

impl Parser {
    pub fn new(code: String, filename: String) -> Self {
        Parser {
            lexer: Lexer::new(code, filename),
            current_tok: Token::new(TokenType::EOF, "", 0, 0, 0, None),
            in_ternary: false,
        }
    }

    fn getsym(&mut self) {
        let t = self.lexer.next();
        self.current_tok = match t {
            Some(t) => t,
            None => Token::new(
                TokenType::EOF,
                "",
                self.current_tok.line_start,
                self.current_tok.lineno,
                self.current_tok.colno,
                None,
            ),
        }
    }

    fn accept(&mut self, tok: TokenType) -> bool {
        if self.current_tok.tid == tok {
            self.getsym();
            return true;
        }
        false
    }

    fn accept_any(&mut self, tids: Vec<TokenType>) -> Option<TokenType> {
        let tid = self.current_tok.tid;
        if tids.contains(&tid) {
            self.getsym();
            return Some(tid);
        }

        return None;
    }

    fn expect(&mut self, tok: TokenType) {
        assert!(self.accept(tok));
    }

    fn block_expect(&mut self, tok: TokenType, _block_start: Token) {
        assert!(self.accept(tok));
    }

    pub fn parse(&mut self) -> Node {
        let n = self.codeblock();
        self.expect(TokenType::EOF);
        return n;
    }

    fn statement(&mut self) -> Node {
        self.e1()
    }

    fn e1(&mut self) -> Node {
        let left = self.e2();
        if self.accept(TokenType::PlusAssign) {
            let value = self.e1();
            if left.node_kind != NodeKind::IDNode {
                panic!("Plus Assignment target must be an ID");
            }

            assert!(matches!(left.value, Some(TokenValue::Str(_))));

            let var_name = if let Some(TokenValue::Str(value)) = left.value {
                value
            } else {
                "".to_string()
            };

            return Node::plusassignment_node(
                left.filename,
                left.lineno,
                left.colno,
                var_name,
                value,
            );
        } else if self.accept(TokenType::Assign) {
            let value = self.e1();
            if left.node_kind != NodeKind::IDNode {
                panic!("Plus Assignment target must be an ID");
            }

            assert!(matches!(left.value, Some(TokenValue::Str(_))));

            let var_name = if let Some(TokenValue::Str(value)) = left.value {
                value
            } else {
                "".to_string()
            };

            return Node::assignment_node(left.filename, left.lineno, left.colno, var_name, value);
        } else if self.accept(TokenType::QuestionMark) {
            if self.in_ternary {
                panic!("Nested ternary operators are not allowed");
            }

            self.in_ternary = true;
            let trueblock = self.e1();
            self.expect(TokenType::Colon);
            let falseblock = self.e1();
            self.in_ternary = false;
            return Node::ternary_node(left, trueblock, falseblock);
        }

        left
    }

    fn e2(&mut self) -> Node {
        let mut left = self.e3();

        while self.accept(TokenType::Or) {
            if left.node_kind == NodeKind::EmptyNode {
                panic!("Invalid and clause.");
            }

            left = Node::or_node(left, self.e3());
        }

        left
    }

    fn e3(&mut self) -> Node {
        let mut left = self.e4();

        while self.accept(TokenType::And) {
            if left.node_kind == NodeKind::EmptyNode {
                panic!("Invalid and clause.");
            }

            left = Node::and_node(left, self.e4());
        }

        left
    }

    fn e4(&mut self) -> Node {
        let left = self.e5();

        for (nodename, operator_type) in COMPARISON_MAP.clone() {
            if self.accept(nodename) {
                return Node::comparison_node(operator_type, left, self.e5());
            }
        }

        if self.accept(TokenType::Not) && self.accept(TokenType::In) {
            return Node::comparison_node("notin", left, self.e5());
        }

        left
    }

    fn e5(&mut self) -> Node {
        self.e5addsub()
    }

    fn e5addsub(&mut self) -> Node {
        let op_map = HashMap::from([(TokenType::Plus, "add"), (TokenType::Dash, "sub")]);
        let mut left = self.e5muldiv();
        loop {
            let op = self.accept_any(vec![TokenType::Plus, TokenType::Dash]);
            if let Some(op) = op {
                left = Node::arithmetic_node(op_map[&op], left, self.e5muldiv());
            } else {
                break;
            }
        }

        left
    }

    fn e5muldiv(&mut self) -> Node {
        let op_map = HashMap::from([
            (TokenType::Percent, "mod"),
            (TokenType::Star, "mul"),
            (TokenType::FSlash, "div"),
        ]);

        let mut left = self.e6();
        loop {
            let op = self.accept_any(vec![TokenType::Percent, TokenType::Star, TokenType::FSlash]);
            if let Some(op) = op {
                left = Node::arithmetic_node(op_map[&op], left, self.e6());
            } else {
                break;
            }
        }

        left
    }

    fn e6(&mut self) -> Node {
        if self.accept(TokenType::Not) {
            return Node::not_node(self.current_tok, self.e7());
        }
        if self.accept(TokenType::Dash) {
            return Node::uminus_node(self.current_tok, self.e7());
        }

        self.e7()
    }

    fn e7(&mut self) -> Node {
        let mut left = self.e8();
        let block_start = self.current_tok;
        if self.accept(TokenType::LParen) {
            let args = self.args();
            self.block_expect(TokenType::RParen, block_start);
            if left.node_kind != NodeKind::IDNode {
                panic!("Function call must be applied to plain id");
            }

            assert!(matches!(left.value, Some(TokenValue::Str(_))));

            if let Some(TokenValue::Str(value)) = left.value {
                left = Node::function_node(
                    left.filename,
                    left.lineno,
                    left.colno,
                    Some(self.current_tok.lineno),
                    Some(self.current_tok.colno),
                    value,
                    args,
                );
            }
        }

        let go_again = true;
        while go_again {
            go_again = false;
            if self.accept(TokenType::Dot) {
                go_again = true;
                left = self.method_call(left);
            }
            if self.accept(TokenType::LBracket) {
                go_again = true;
                left = self.index_call(left);
            }
        }

        left
    }

    fn e8(&mut self) -> Node {
        let block_start = self.current_tok;
        if self.accept(TokenType::LParen) {
            let e = self.statement();
            self.block_expect(TokenType::RParen, block_start);
            return e;
        } else if self.accept(TokenType::LBracket) {
            let args = self.args();
            self.block_expect(TokenType::RBracket, block_start);
            return Node::array_node(
                args,
                block_start.lineno,
                block_start.colno,
                Some(self.current_tok.lineno),
                Some(self.current_tok.colno),
            );
        } else if self.accept(TokenType::LBrace) {
            let key_values = self.key_values();
            self.block_expect(TokenType::RBrace, block_start);

            return Node::dict_node(
                key_values,
                block_start.lineno,
                block_start.colno,
                Some(self.current_tok.lineno),
                Some(self.current_tok.colno),
            );
        } else {
            return self.e9();
        }
    }

    fn e9(&mut self) -> Node {
        let mut t = self.current_tok;
        if self.accept(TokenType::True) {
            t.value = Some(TokenValue::Bool(true));
            return Node::boolean_node(t);
        }

        if self.accept(TokenType::False) {
            t.value = Some(TokenValue::Bool(false));
            return Node::boolean_node(t);
        }

        if self.accept(TokenType::ID) {
            return Node::id_node(t);
        }

        if self.accept(TokenType::Number) {
            return Node::number_node(t);
        }

        if self.accept(TokenType::String) {
            return Node::string_node(t);
        }

        if self.accept(TokenType::FString) {
            return Node::fstring_node(t);
        }

        if self.accept(TokenType::MultilineFstring) {
            return Node::multiline_fstring_node(t);
        }

        Node::empty_node(
            self.current_tok.lineno,
            self.current_tok.colno,
            self.current_tok.filename,
        )
    }

    fn key_values(&mut self) -> Node {
        let s = self.statement();
        let a = Node::argument_node(self.current_tok);

        while s.node_kind != NodeKind::EmptyNode {
            if self.accept(TokenType::Colon) {
                if let NodeKind::ArgumentNode(arg_node) = a.node_kind {
                    arg_node.set_kwarg_no_check(s, self.statement());
                    let potential = self.current_tok;
                    if !self.accept(TokenType::Comma) {
                        return a;
                    }

                    arg_node.commas.push(potential);
                }
            } else {
                panic!("Only key:value pairs are valid in dict construction");
            }
        }

        a
    }

    fn args(&mut self) -> Node {
        let s = self.statement();
        let a = Node::argument_node(self.current_tok);

        while s.node_kind != NodeKind::EmptyNode {
            let potential = self.current_tok;
            if self.accept(TokenType::Comma) {
                if let NodeKind::ArgumentNode(a) = a.node_kind {
                    a.commas.push(potential);
                    a.append(s);
                }
            } else if self.accept(TokenType::Colon) {
                if s.node_kind != NodeKind::IDNode {
                    panic!("Dictionary key must be a plain identifier");
                }

                if let NodeKind::ArgumentNode(arg_node) = a.node_kind {
                    arg_node.set_kwarg(s, self.statement());
                    potential = self.current_tok;

                    if !self.accept(TokenType::Comma) {
                        return a;
                    }

                    arg_node.commas.push(potential);
                }
            } else if let NodeKind::ArgumentNode(arg_node) = a.node_kind {
                arg_node.append(s);
                return a;
            }

            s = self.statement();
        }

        a
    }

    fn method_call(&mut self, source: Node) -> Node {
        let methodname = self.e9();
        if methodname.node_kind != NodeKind::IDNode {
            panic!("Method name must be plain ID");
        }

        assert!(matches!(methodname.value, Some(TokenValue::Str(_))));

        self.expect(TokenType::LParen);
        let args = self.args();
        self.expect(TokenType::RParen);

        let name = if let Some(TokenValue::Str(name)) = methodname.value {
            name
        } else {
            "".to_string()
        };

        let method = Node::method_node(
            methodname.filename,
            methodname.lineno,
            methodname.colno,
            source,
            name,
            args,
        );

        if self.accept(TokenType::Dot) {
            return self.method_call(method);
        }

        return method;
    }

    fn index_call(&mut self, source_object: Node) -> Node {
        let index_statement = self.statement();
        self.expect(TokenType::RBracket);
        Node::index_node(source_object, index_statement)
    }

    fn ifblock(&mut self) -> Node {
        let condition = self.statement();
        let clause = Node::ifclause_node(condition);
        self.expect(TokenType::Eol);
        let block = self.codeblock();
        if let NodeKind::IfClauseNode {
            mut ifs,
            elseblock: _,
        } = clause.node_kind
        {
            ifs.push(Node::if_node(clause, condition, block));
        }

        self.elseifblock(clause);
        if let NodeKind::IfClauseNode { ifs, elseblock } = clause.node_kind {
            elseblock = Some(Box::new(self.elseblock()));
        }

        return clause;
    }

    fn elseifblock(&mut self, clause: Node) {
        while self.accept(TokenType::ElIf) {
            let s = self.statement();
            self.expect(TokenType::Eol);
            let b = self.codeblock();

            if let NodeKind::IfClauseNode { ifs, elseblock } = clause.node_kind {
                ifs.push(Node::if_node(s, s, b));
            }
        }
    }

    fn elseblock(&mut self) -> Node {
        if self.accept(TokenType::Else) {
            self.expect(TokenType::Eol);
            return self.codeblock();
        }

        Node::empty_node(
            self.current_tok.lineno,
            self.current_tok.colno,
            self.current_tok.filename,
        )
    }

    fn line(&mut self) -> Node {
        let block_start = self.current_tok;

        if self.current_tok.tid == TokenType::Eol {
            return Node::new(self.current_tok, NodeKind::EmptyNode);
        }

        if self.accept(TokenType::If) {
            let ifblock = self.ifblock();
            self.block_expect(TokenType::EndIf, block_start);
            return ifblock;
        }

        Node::empty_node(
            self.current_tok.lineno,
            self.current_tok.colno,
            self.current_tok.filename,
        )
    }

    fn codeblock(&mut self) -> Node {
        let mut lines: Vec<Node> = Vec::new();
        let cond = true;

        while cond {
            let curline = self.line();
            if curline.node_kind != NodeKind::EmptyNode {
                lines.push(curline);
            }

            cond = self.accept(TokenType::Eol);
        }

        Node::new(self.current_tok, NodeKind::CodeBlock { lines })
    }
}
