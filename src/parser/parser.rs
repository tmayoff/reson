use super::node::ArgumentNode;
use super::node::IndexNode;
use super::node::MethodNode;
use super::node::Node;
use super::token::Token;
use lazy_static::lazy_static;
use logos::{Logos, SpannedIter};
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::rc::Rc;

#[derive(Debug)]
enum ParserError {}

pub struct Parser<'a> {
    lexer: SpannedIter<'a, Token>,
    source: &'a str,
    current_tok: (Token, Range<usize>),
    in_ternary: bool,
}

lazy_static! {
    static ref COMPARISON_MAP: BTreeMap<Token, &'static str> = BTreeMap::from([
        (Token::Equal, "=="),
        (Token::NEqual, "!="),
        (Token::LT, "<"),
        (Token::LE, "<="),
        (Token::GT, ">"),
        (Token::GE, ">="),
        (Token::In, "in"),
        (Token::NotIn, "not in"),
    ]);
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, _filename: &str) -> Self {
        let lex = Token::lexer(code).spanned();

        let mut p = Parser {
            lexer: lex,
            source: code,
            current_tok: (Token::EOF, Range::default()),
            in_ternary: false,
        };

        p.getsym();

        p
    }

    fn getsym(&mut self) {
        let t = self.lexer.next();
        match t {
            Some(t) => self.current_tok = t,
            None => self.current_tok = (Token::EOF, Range::default()),
        }
    }

    fn accept(&mut self, tok: Token) -> bool {
        if std::mem::discriminant(&self.current_tok.0) == std::mem::discriminant(&tok) {
            self.getsym();
            return true;
        }
        false
    }

    fn accept_any(&mut self, tids: Vec<Token>) -> Option<Token> {
        let tid = self.current_tok.0.clone();
        if tids.contains(&tid) {
            self.getsym();
            return Some(tid);
        }

        None
    }

    fn expect(&mut self, tok: Token) {
        assert!(self.accept(tok.clone()), "Expected token {:?}", &tok);
    }

    fn block_expect(&mut self, tok: Token, _block_start: &Token) {
        assert!(
            self.accept(tok.clone()),
            "Expected token {:?}, block started: {:?}",
            &tok,
            &_block_start
        );
    }

    pub fn parse(&mut self) -> Node {
        let n = self.codeblock();
        self.expect(Token::EOF);
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
        if self.accept(Token::PlusAssign) {
            let value = self.e1();
            if !matches!(left, Node::IDNode { .. }) {
                panic!("Plus Assignment target must be an ID");
            }

            if let Node::IDNode { value: var_name } = &left {
                return Node::PlusAssignmentNode {
                    var_name: var_name.to_owned(),
                    value: Rc::from(value),
                };
            }
        } else if self.accept(Token::Assign) {
            let value = self.e1();
            if !matches!(left, Node::IDNode { .. }) {
                panic!("Assignment target must be an ID");
            }

            if let Node::IDNode { value: var_name } = &left {
                return Node::AssignmentNode {
                    var_name: var_name.to_owned(),
                    value: Rc::from(value),
                };
            }
        } else if self.accept(Token::QuestionMark) {
            if self.in_ternary {
                panic!("Nested ternary operators are not allowed");
            }

            self.in_ternary = true;
            let trueblock = self.e1();
            self.expect(Token::Colon);
            let falseblock = self.e1();
            self.in_ternary = false;
            return Node::TernaryNode {
                condition: Rc::from(left),
                trueblock: Rc::from(trueblock),
                falseblock: Rc::from(falseblock),
            };
        }

        left
    }

    fn e2(&mut self) -> Node {
        let mut left = self.e3();

        while self.accept(Token::Or) {
            if left == Node::Empty {
                panic!("Invalid and clause.");
            }

            left = Node::OrNode {
                left: Rc::from(left),
                right: Rc::from(self.e3()),
            }
        }

        left
    }

    fn e3(&mut self) -> Node {
        let mut left = self.e4();

        while self.accept(Token::And) {
            if left == Node::Empty {
                panic!("Invalid and clause.");
            }

            left = Node::AndNode {
                left: Rc::from(left),
                right: Rc::from(self.e4()),
            };
        }

        left
    }

    fn e4(&mut self) -> Node {
        let left = self.e5();

        for (nodename, operator_type) in COMPARISON_MAP.clone() {
            if self.accept(nodename) {
                return Node::ComparisonNode {
                    left: Rc::from(left),
                    right: Rc::from(self.e5()),
                    ctype: operator_type.to_string(),
                };
            }
        }

        if self.accept(Token::Not) && self.accept(Token::In) {
            return Node::ComparisonNode {
                left: Rc::from(left),
                right: Rc::from(self.e5()),
                ctype: "notin".to_string(),
            };
        }

        left
    }

    fn e5(&mut self) -> Node {
        self.e5addsub()
    }

    fn e5addsub(&mut self) -> Node {
        let op_map = HashMap::from([(Token::Plus, "add"), (Token::Dash, "sub")]);
        let mut left = self.e5muldiv();
        loop {
            let op = self.accept_any(vec![Token::Plus, Token::Dash]);
            if let Some(op) = op {
                left = Node::ArithmeticNode {
                    left: Rc::from(left),
                    right: Rc::from(self.e5muldiv()),
                    operation: op_map[&op].to_string(),
                };
            } else {
                break;
            }
        }

        left
    }

    fn e5muldiv(&mut self) -> Node {
        let op_map = HashMap::from([
            (Token::Percent, "mod"),
            (Token::Star, "mul"),
            (Token::FSlash, "div"),
        ]);

        let mut left = self.e6();
        loop {
            let op = self.accept_any(vec![Token::Percent, Token::Star, Token::FSlash]);
            if let Some(op) = op {
                left = Node::ArithmeticNode {
                    left: Rc::from(left),
                    right: Rc::from(self.e6()),
                    operation: op_map[&op].to_owned(),
                };
            } else {
                break;
            }
        }

        left
    }

    fn e6(&mut self) -> Node {
        if self.accept(Token::Not) {
            return Node::NotNode {
                value: Rc::from(self.e7()),
            };
        }
        if self.accept(Token::Dash) {
            return Node::UMinusNode {
                value: Rc::from(self.e7()),
            };
        }

        self.e7()
    }
    // Func Call
    fn e7(&mut self) -> Node {
        let mut left = self.e8();
        let block_start = self.current_tok.clone();
        if self.accept(Token::LParen) {
            let args = self.args();
            self.block_expect(Token::RParen, &block_start.0);

            if !matches!(left, Node::IDNode { .. }) {
                panic!("Function call must be applied to plain ID");
            }

            if let Node::IDNode { value: var_name } = &left {
                left = Node::FunctionNode {
                    func_name: var_name.to_string(),
                    args: Rc::from(args),
                };
            }
        }

        let mut go_again = true;
        while go_again {
            go_again = false;
            if self.accept(Token::Dot) {
                go_again = true;
                left = self.method_call(&left);
            }
            if self.accept(Token::LCurly) {
                go_again = true;
                left = self.index_call(&left);
            }
        }

        left
    }

    fn e8(&mut self) -> Node {
        let block_start = self.current_tok.clone();
        if self.accept(Token::LParen) {
            let e = self.statement();
            self.block_expect(Token::RParen, &block_start.0);
            e
        } else if self.accept(Token::LBrace) {
            let args = self.args();
            self.block_expect(Token::RBrace, &block_start.0);
            Node::ArrayNode {
                args: Rc::from(args),
            }
        } else if self.accept(Token::LBrace) {
            let key_values = self.key_values();
            self.block_expect(Token::RBrace, &block_start.0);

            Node::DictNode {
                args: Rc::from(key_values),
            }
        } else {
            self.e9()
        }
    }

    fn e9(&mut self) -> Node {
        let mut t = self.current_tok.clone();
        if self.accept(Token::True) {
            return Node::BoolNode { value: true };
        }

        if self.accept(Token::False) {
            return Node::BoolNode { value: false };
        }

        if self.accept(Token::ID("".to_string())) {
            if let Token::ID(id) = t.0 {
                return Node::IDNode { value: id };
            }
        }

        if self.accept(Token::Number(0)) {
            if let Token::Number(num) = t.0 {
                return Node::NumberNode { value: num };
            }
        }

        if self.accept(Token::String("".to_string())) {
            if let Token::String(str) = t.0 {
                return Node::StringNode { value: str };
            }
        }

        //        if self.accept(Token::FString) {
        //            let mut value = String::new();
        //            if let TokenValue::Str(s) = &t.value {
        //                value = s.clone();
        //            };
        //
        //            return Node::fstring_node(t, value);
        //        }
        //
        //        if self.accept(Token::MultilineFstring) {
        //            let mut value = String::new();
        //            if let TokenValue::Str(s) = &t.value {
        //                value = s.clone();
        //            };
        //            return Node::multiline_fstring_node(t, value);
        //        }
        //
        Node::Empty
    }

    fn key_values(&mut self) -> Node {
        let mut s = Rc::from(self.statement());
        let mut a = Node::ArgumentNode(ArgumentNode::default());

        while *s != Node::Empty {
            if self.accept(Token::Colon) {
                if let Node::ArgumentNode(ref mut arg_node) = a {
                    arg_node.set_kwarg_no_check(s, Rc::from(self.statement()));
                    let potential = self.current_tok.clone();
                    if !self.accept(Token::Comma) {
                        return a;
                    }

                    arg_node.commas.push(potential.0);
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
        let mut a = Node::ArgumentNode(ArgumentNode::default());

        while *s != Node::Empty {
            let mut potential = self.current_tok.clone();
            if self.accept(Token::Comma) {
                if let Node::ArgumentNode(arg_node) = &mut a {
                    arg_node.commas.push(potential.0);
                    arg_node.append(s);
                }
            } else if self.accept(Token::Colon) {
                if !matches!(*s, Node::IDNode { .. }) {
                    panic!("Dictionary key must be a plain identifier");
                }

                if let Node::ArgumentNode(arg_node) = &mut a {
                    arg_node.set_kwarg(s, Rc::from(self.statement()));
                    potential = self.current_tok.clone();

                    //                    if !self.accept(Token::Comma) {
                    //                        return a;
                    //                    }

                    arg_node.commas.push(potential.0);
                }
            } else if let Node::ArgumentNode(arg_node) = &mut a {
                arg_node.append(s);
                return a;
            }

            s = Rc::from(self.statement());
        }

        a
    }

    fn method_call(&mut self, source: &Node) -> Node {
        let methodname = self.e9();
        if !matches!(methodname, Node::IDNode { .. }) {
            panic!("Method name must be plain ID");
        }

        self.expect(Token::LParen);
        let args = self.args();
        self.expect(Token::RParen);

        let method = if let Node::IDNode { value: method_name } = methodname {
            Node::MethodNode(MethodNode {
                source_object: Rc::from(source.to_owned()),
                name: method_name,
                args: Rc::from(args),
            })
        } else {
            unreachable!();
        };

        if self.accept(Token::Dot) {
            return self.method_call(&method);
        }

        method
    }

    fn index_call(&mut self, source_object: &Node) -> Node {
        let index_statement = self.statement();
        self.expect(Token::RCurly);
        Node::IndexNode(IndexNode {
            iobject: Rc::from(source_object.to_owned()),
            index: Rc::from(index_statement),
        })
    }

    fn ifblock(&mut self) -> Node {
        let condition = self.statement();
        let clause = Node::IfClauseNode {
            ifs: Vec::new(),
            elseblock: None,
        };
        self.expect(Token::EOL);
        let block = self.codeblock();
        if let Node::IfClauseNode {
            ref mut ifs,
            elseblock: _,
        } = clause.clone()
        {
            ifs.push(Rc::from(Node::IfNode {
                condition: Rc::from(condition),
                block: Rc::from(block),
            }));
        }

        self.elseifblock(&clause);
        if let Node::IfClauseNode {
            ifs: _,
            mut elseblock,
        } = clause.clone()
        {
            elseblock = Some(Rc::from(self.elseblock()));
        }

        clause
    }

    fn elseifblock(&mut self, clause: &Node) {
        while self.accept(Token::ElIf) {
            let s = self.statement();
            self.expect(Token::EOL);
            let b = self.codeblock();

            if let Node::IfClauseNode {
                mut ifs,
                elseblock: _,
            } = clause.clone()
            {
                ifs.push(Rc::from(Node::IfNode {
                    condition: Rc::from(s),
                    block: Rc::from(b),
                }));
            }
        }
    }

    fn elseblock(&mut self) -> Node {
        if self.accept(Token::Else) {
            self.expect(Token::EOL);
            return self.codeblock();
        }

        Node::Empty
    }

    fn line(&mut self) -> Node {
        let block_start = self.current_tok.clone();

        if self.current_tok.0 == Token::EOL {
            return Node::Empty;
        }

        if self.accept(Token::If) {
            let ifblock = self.ifblock();
            self.block_expect(Token::EndIf, &block_start.0);
            return ifblock;
        }

        self.statement()
    }

    fn codeblock(&mut self) -> Node {
        let mut lines: Vec<Rc<Node>> = Vec::new();
        let mut cond = true;

        while cond {
            let curline = self.line();
            if curline != Node::Empty {
                lines.push(Rc::from(curline));
            }

            cond = self.accept(Token::EOL);
        }

        Node::CodeBlock { lines: lines }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        let expected = Node::CodeBlock {
            lines: vec![Rc::from(Node::FunctionNode {
                func_name: "project".to_string(),
                args: Rc::from(Node::ArgumentNode(ArgumentNode {
                    arguments: vec![Rc::from(Node::StringNode {
                        value: "simple".to_string(),
                    })],
                    commas: Vec::new(),
                    kwargs: Default::default(),
                    ..Default::default()
                })),
            })],
        };

        let code = r#"
            project('simple')
        "#;

        let mut p = Parser::new(code, "simple_test");
        let ast = p.parse();

        assert_eq!(ast, expected);
    }

    #[test]
    fn kwargs_test() {
        let expected = Node::CodeBlock {
            lines: vec![Rc::from(Node::FunctionNode {
                func_name: "project".to_string(),
                args: Rc::from(Node::ArgumentNode(ArgumentNode {
                    arguments: vec![
                        Rc::from(Node::StringNode {
                            value: "kwargs_test".to_string(),
                        }),
                        Rc::from(Node::StringNode {
                            value: "cpp".to_string(),
                        }),
                    ],
                    commas: vec![Token::Comma, Token::Comma, Token::RParen],
                    kwargs: BTreeMap::from([
                        (
                            Rc::from(Node::IDNode {
                                value: "version".to_string(),
                            }),
                            Rc::from(Node::StringNode {
                                value: "0.1.1".to_string(),
                            }),
                        ),
                        (
                            Rc::from(Node::IDNode {
                                value: "meson_version".to_string(),
                            }),
                            Rc::from(Node::StringNode {
                                value: "1.0.0".to_string(),
                            }),
                        ),
                    ]),
                    ..Default::default()
                })),
            })],
        };

        let code = r#"
            project('kwargs_test', 'cpp', version: '0.1.1', meson_version: '1.0.0')
        "#;

        let mut p = Parser::new(code, "simple_test");
        let ast = p.parse();

        assert_eq!(ast, expected);
    }
}
