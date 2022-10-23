use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::parser::node::Node;
use crate::parser::token::TokenType;
use lazy_static::lazy_static;

use super::lexer::Lexer;
use super::node::NodeType;
use super::token::{Token, TokenValue};

#[derive(Debug)]
enum ParserError {}

pub struct Parser {
    lexer: Lexer,

    current_tok: Token,

    in_ternary: bool,
}

lazy_static! {
    static ref COMPARISON_MAP: BTreeMap<TokenType, &'static str> = BTreeMap::from([
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
        let lexer = Lexer::new(code, filename);

        let current_tok = Token::new(TokenType::EOF, "", 0, 0, 0, TokenValue::None);

        let mut p = Parser {
            lexer,
            current_tok,
            in_ternary: false,
        };

        p.getsym();

        p
    }

    fn getsym(&mut self) {
        let t = self.lexer.next();
        match t {
            Ok(t) => self.current_tok = t,
            Err(_) => todo!(),
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
        let tid = self.current_tok.tid.clone();
        if tids.contains(&tid) {
            self.getsym();
            return Some(tid);
        }

        None
    }

    fn expect(&mut self, tok: TokenType) {
        assert!(self.accept(tok));
    }

    fn block_expect(&mut self, tok: TokenType, _block_start: &Token) {
        assert!(self.accept(tok));
    }

    pub fn parse(&mut self) -> Node {
        let n = self.codeblock();
        self.expect(TokenType::EOF);
        n
    }

    // # Recursive descent parser for Meson's definition language.
    // # Very basic apart from the fact that we have many precedence
    // # levels so there are not enough words to describe them all.
    // # Enter numbering:
    // #
    // # 1 assignment
    // # 2 or
    // # 3 and
    // # 4 comparison
    // # 5 arithmetic
    // # 6 negation
    // # 7 funcall, method call
    // # 8 parentheses
    // # 9 plain token

    fn statement(&mut self) -> Node {
        self.e1()
    }

    fn e1(&mut self) -> Node {
        let left = self.e2();
        if self.accept(TokenType::PlusAssign) {
            let value = self.e1();
            if !matches!(left.node_type, NodeType::IDNode { .. }) {
                panic!("Plus Assignment target must be an ID");
            }

            if let NodeType::IDNode { value: var_name } = &left.node_type {
                return Node::plusassignment_node(
                    left.filename.to_owned(),
                    left.lineno,
                    left.colno,
                    var_name.to_owned(),
                    &value,
                );
            }
        } else if self.accept(TokenType::Assign) {
            let value = self.e1();
            if !matches!(left.node_type, NodeType::IDNode { .. }) {
                panic!("Assignment target must be an ID");
            }

            if let NodeType::IDNode { value: var_name } = &left.node_type {
                return Node::assignment_node(
                    left.filename.to_owned(),
                    left.lineno,
                    left.colno,
                    var_name.to_string(),
                    &value,
                );
            }
        } else if self.accept(TokenType::QuestionMark) {
            if self.in_ternary {
                panic!("Nested ternary operators are not allowed");
            }

            self.in_ternary = true;
            let trueblock = self.e1();
            self.expect(TokenType::Colon);
            let falseblock = self.e1();
            self.in_ternary = false;
            return Node::ternary_node(&left, &trueblock, &falseblock);
        }

        left
    }

    fn e2(&mut self) -> Node {
        let mut left = self.e3();

        while self.accept(TokenType::Or) {
            if left.node_type == NodeType::EmptyNode {
                panic!("Invalid and clause.");
            }

            left = Node::or_node(&left, &self.e3());
        }

        left
    }

    fn e3(&mut self) -> Node {
        let mut left = self.e4();

        while self.accept(TokenType::And) {
            if left.node_type == NodeType::EmptyNode {
                panic!("Invalid and clause.");
            }

            left = Node::and_node(&left, &self.e4());
        }

        left
    }

    fn e4(&mut self) -> Node {
        let left = self.e5();

        for (nodename, operator_type) in COMPARISON_MAP.clone() {
            if self.accept(nodename) {
                return Node::comparison_node(operator_type, &left, &self.e5());
            }
        }

        if self.accept(TokenType::Not) && self.accept(TokenType::In) {
            return Node::comparison_node("notin", &left, &self.e5());
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
                left = Node::arithmetic_node(op_map[&op], &left, &self.e5muldiv());
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
                left = Node::arithmetic_node(op_map[&op], &left, &self.e6());
            } else {
                break;
            }
        }

        left
    }

    fn e6(&mut self) -> Node {
        if self.accept(TokenType::Not) {
            return Node::not_node(self.current_tok.clone(), &self.e7());
        }
        if self.accept(TokenType::Dash) {
            return Node::uminus_node(self.current_tok.clone(), &self.e7());
        }

        self.e7()
    }

    fn e7(&mut self) -> Node {
        let mut left = self.e8();
        let block_start = self.current_tok.clone();
        if self.accept(TokenType::LParen) {
            let args = self.args();
            self.block_expect(TokenType::RParen, &block_start);

            if !matches!(left.node_type, NodeType::IDNode { .. }) {
                panic!("Function call must be applied to plain ID");
            }

            if let NodeType::IDNode { value: var_name } = &left.node_type {
                left = Node::function_node(
                    left.filename.to_string(),
                    left.lineno,
                    left.colno,
                    Some(self.current_tok.lineno),
                    Some(self.current_tok.colno),
                    var_name.to_string(),
                    &args,
                );
            }
        }

        let mut go_again = true;
        while go_again {
            go_again = false;
            if self.accept(TokenType::Dot) {
                go_again = true;
                left = self.method_call(&left);
            }
            if self.accept(TokenType::LCurly) {
                go_again = true;
                left = self.index_call(&left);
            }
        }

        left
    }

    fn e8(&mut self) -> Node {
        let block_start = self.current_tok.clone();
        if self.accept(TokenType::LParen) {
            let e = self.statement();
            self.block_expect(TokenType::RParen, &block_start);
            e
        } else if self.accept(TokenType::LBrace) {
            let args = self.args();
            self.block_expect(TokenType::RBrace, &block_start);
            Node::array_node(
                &args,
                block_start.lineno,
                block_start.colno,
                Some(self.current_tok.lineno),
                Some(self.current_tok.colno),
            )
        } else if self.accept(TokenType::LBrace) {
            let key_values = self.key_values();
            self.block_expect(TokenType::RBrace, &block_start);

            Node::dict_node(
                &key_values,
                block_start.lineno,
                block_start.colno,
                Some(self.current_tok.lineno),
                Some(self.current_tok.colno),
            )
        } else {
            self.e9()
        }
    }

    fn e9(&mut self) -> Node {
        let mut t = self.current_tok.clone();
        if self.accept(TokenType::True) {
            t.value = TokenValue::Bool(true);
            return Node::boolean_node(t);
        }

        if self.accept(TokenType::False) {
            t.value = TokenValue::Bool(false);
            return Node::boolean_node(t);
        }

        if self.accept(TokenType::ID) {
            let mut value = String::new();
            if let TokenValue::Str(s) = &t.value {
                value = s.clone();
            };

            return Node::id_node(t, value);
        }

        if self.accept(TokenType::Number) {
            let mut value = 0;
            if let TokenValue::Int(n) = t.value {
                value = n
            };
            return Node::number_node(t, value);
        }

        if self.accept(TokenType::String) {
            let mut value = String::new();
            if let TokenValue::Str(s) = &t.value {
                value = s.clone();
            };
            return Node::string_node(t, value);
        }

        if self.accept(TokenType::FString) {
            let mut value = String::new();
            if let TokenValue::Str(s) = &t.value {
                value = s.clone();
            };

            return Node::fstring_node(t, value);
        }

        if self.accept(TokenType::MultilineFstring) {
            let mut value = String::new();
            if let TokenValue::Str(s) = &t.value {
                value = s.clone();
            };
            return Node::multiline_fstring_node(t, value);
        }

        Node::empty_node(
            self.current_tok.lineno,
            self.current_tok.colno,
            self.current_tok.filename.clone(),
        )
    }

    fn key_values(&mut self) -> Node {
        let mut s = Rc::from(self.statement());
        let mut a = Node::argument_node(self.current_tok.clone());

        while s.node_type != NodeType::EmptyNode {
            if self.accept(TokenType::Colon) {
                if let NodeType::ArgumentNode(ref mut arg_node) = a.node_type {
                    arg_node.set_kwarg_no_check(s, Rc::from(self.statement()));
                    let potential = self.current_tok.clone();
                    if !self.accept(TokenType::Comma) {
                        return a;
                    }

                    arg_node.commas.push(potential);
                }
            } else {
                panic!("Only key:value pairs are valid in dict construction");
            }

            s = Rc::from(self.statement());
        }

        a
    }

    fn args(&mut self) -> Node {
        let mut s = Rc::from(self.statement());
        let mut a = Node::argument_node(self.current_tok.clone());

        while s.node_type != NodeType::EmptyNode {
            let mut potential = self.current_tok.clone();
            if self.accept(TokenType::Comma) {
                if let NodeType::ArgumentNode(arg_node) = &mut a.node_type {
                    arg_node.commas.push(potential);
                    arg_node.append(s);
                }
            } else if self.accept(TokenType::Colon) {
                if !matches!(s.node_type, NodeType::IDNode { .. }) {
                    panic!("Dictionary key must be a plain identifier");
                }

                if let NodeType::ArgumentNode(arg_node) = &mut a.node_type {
                    arg_node.set_kwarg(s, Rc::from(self.statement()));
                    potential = self.current_tok.clone();

                    if !self.accept(TokenType::Comma) {
                        return a;
                    }

                    arg_node.commas.push(potential);
                }
            } else if let NodeType::ArgumentNode(arg_node) = &mut a.node_type {
                arg_node.append(s);
                return a;
            }

            s = Rc::from(self.statement());
        }

        a
    }

    fn method_call(&mut self, source: &Node) -> Node {
        let methodname = self.e9();
        if !matches!(methodname.node_type, NodeType::IDNode { .. }) {
            panic!("Method name must be plain ID");
        }

        self.expect(TokenType::LParen);
        let args = self.args();
        self.expect(TokenType::RParen);

        let method = if let NodeType::IDNode { value: method_name } = methodname.node_type {
            Node::method_node(
                methodname.filename,
                methodname.lineno,
                methodname.colno,
                source,
                method_name,
                &args,
            )
        } else {
            unreachable!();
        };

        if self.accept(TokenType::Dot) {
            return self.method_call(&method);
        }

        method
    }

    fn index_call(&mut self, source_object: &Node) -> Node {
        let index_statement = self.statement();
        self.expect(TokenType::RCurly);
        Node::index_node(
            &Rc::from(source_object.to_owned()),
            &Rc::from(index_statement),
        )
    }

    fn ifblock(&mut self) -> Node {
        let condition = self.statement();
        let clause = Node::ifclause_node(&condition);
        self.expect(TokenType::EOL);
        let block = self.codeblock();
        if let NodeType::IfClauseNode {
            ref mut ifs,
            elseblock: _,
        } = clause.node_type.clone()
        {
            ifs.push(Rc::from(Node::if_node(&clause, &condition, &block)));
        }

        self.elseifblock(&clause);
        if let NodeType::IfClauseNode {
            ifs: _,
            mut elseblock,
        } = clause.node_type.clone()
        {
            elseblock = Some(Rc::from(self.elseblock()));
        }

        clause
    }

    fn elseifblock(&mut self, clause: &Node) {
        while self.accept(TokenType::ElIf) {
            let s = self.statement();
            self.expect(TokenType::EOL);
            let b = self.codeblock();

            if let NodeType::IfClauseNode {
                mut ifs,
                elseblock: _,
            } = clause.node_type.clone()
            {
                ifs.push(Rc::from(Node::if_node(&s, &s, &b)));
            }
        }
    }

    fn elseblock(&mut self) -> Node {
        if self.accept(TokenType::Else) {
            self.expect(TokenType::EOL);
            return self.codeblock();
        }

        Node::empty_node(
            self.current_tok.lineno,
            self.current_tok.colno,
            self.current_tok.filename.clone(),
        )
    }

    fn line(&mut self) -> Node {
        let block_start = self.current_tok.clone();

        if self.current_tok.tid == TokenType::EOL {
            return Node::new(self.current_tok.clone(), NodeType::EmptyNode);
        }

        if self.accept(TokenType::If) {
            let ifblock = self.ifblock();
            self.block_expect(TokenType::EndIf, &block_start);
            return ifblock;
        }

        self.statement()
    }

    fn codeblock(&mut self) -> Node {
        let mut lines: Vec<Rc<Node>> = Vec::new();
        let mut cond = true;

        while cond {
            let curline = self.line();
            if curline.node_type != NodeType::EmptyNode {
                lines.push(Rc::from(curline));
            }

            cond = self.accept(TokenType::EOL);
        }

        Node::new(self.current_tok.clone(), NodeType::CodeBlock { lines })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_test() {
        let code = r#"
            project('test_proj', ['cpp'])
            
            a = dependency('dep')
            deps = [a, b, c]
            executable('exec', dependencies: deps)
        "#;

        let mut p = Parser::new(code.to_string(), "parser_test".to_string());
        let c = p.parse();

        // First node from parse should be codeblock node
        assert!(matches!(c.node_type, NodeType::CodeBlock { .. }));
        if let NodeType::CodeBlock { lines } = c.node_type {
            assert_eq!(lines.len(), 4);

            let mut it = lines.into_iter();
            let l = it.next().expect("Failed to get first line");

            // First line should be project() function call
            assert!(matches!(l.node_type, NodeType::FunctionNode { .. }));
            if let NodeType::FunctionNode { func_name, args } = &l.node_type {
                assert_eq!(func_name, "project");

                // Arguments should be project name and language
                assert!(matches!(args.node_type, NodeType::ArgumentNode { .. }));
                if let NodeType::ArgumentNode(args_node) = &args.node_type {
                    assert_eq!(args_node.commas.len(), 1);
                    assert_eq!(args_node.arguments.len(), 2);

                    let it = &mut args_node.clone().arguments.into_iter();

                    let arg = it.next().expect("Expected a node");
                    assert!(matches!(arg.node_type, NodeType::StringNode { .. }));
                    if let NodeType::StringNode { value } = &arg.node_type {
                        assert_eq!(value, "test_proj");
                    }

                    let arg = it.next().expect("Expected another node");
                    assert!(matches!(arg.node_type, NodeType::ArrayNode { .. }));
                    if let NodeType::ArrayNode { args } = &arg.node_type {
                        let args = if let NodeType::ArgumentNode(args) = &args.node_type {
                            args
                        } else {
                            unreachable!()
                        };

                        assert_eq!(args.commas.len(), 0);

                        assert_eq!(args.arguments.len(), 1);
                        let arg = &args.arguments[0];
                        assert!(matches!(arg.node_type, NodeType::StringNode { .. }));
                        if let NodeType::StringNode { value } = &arg.node_type {
                            assert_eq!(value, "cpp");
                        }
                    }
                }
            }

            let l = it.next().expect("Expected another line");
            assert!(matches!(l.node_type, NodeType::AssignmentNode { .. }));
        }
    }
}
