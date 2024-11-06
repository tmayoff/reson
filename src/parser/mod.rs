mod tokens;

use crate::interpreter::ast::{Arguments, Arithmetic, Assignment, Function, MathOp, Node, Program};
use logos::{Lexer, Logos};
use std::{collections::HashMap, fs::read_to_string, path::PathBuf};
use thiserror::Error;
use tokens::Token;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read file")]
    ReadError(PathBuf),
    #[error("IO Error")]
    Io(#[from] std::io::Error),
}

pub fn parse_file(path: &PathBuf) -> Result<Program, Error> {
    let content = read_to_string(path).map_err(|_| Error::ReadError(path.into()))?;
    parse(&content)
}

struct Parser<'source> {
    lexer: Lexer<'source, Token>,
    current: Token,
    // peeked: Token,
}

impl<'source> Parser<'source> {
    fn new(input: &'source str) -> Self {
        let mut l = Token::lexer(input);
        let t = l
            .next()
            .map(|e| e.expect("Failed to lex"))
            .unwrap_or(Token::EOF);
        // let c = l
        //     .next()
        //     .map(|e| e.expect("Failed to lex"))
        //     .unwrap_or(Token::EOF);

        Self {
            lexer: l,
            current: t,
            // peeked: c,
        }
    }

    pub fn accept(&mut self, tok: &Token) -> bool {
        if std::mem::discriminant(&self.current) == std::mem::discriminant(&tok) {
            self.advance();
            return true;
        }

        false
    }

    pub fn accept_any(&mut self, toks: &Vec<Token>) -> bool {
        for tok in toks {
            if self.accept(tok) {
                return true;
            }
        }

        false
    }

    pub fn expect(&mut self, tok: Token) -> Result<(), Error> {
        if !self.accept(&tok) {
            // TODO panic here
        }

        Ok(())
    }

    fn advance(&mut self) {
        self.current = self
            .lexer
            .next()
            .map(|t| t.expect("failed to lex"))
            .unwrap_or(Token::EOF);
        // self.peeked = self
        //     .lexer
        //     .next()
        //     .map(|e| e.expect("failed to lex"))
        //     .unwrap_or(Token::EOF);
    }

    fn curr(&self) -> Token {
        self.current.clone()
    }

    // fn peek(&self) -> Token {
    //     self.peeked.clone()
    // }

    fn statement(&mut self) -> Result<Node, Error> {
        return self.e1();
    }

    // Recursive descent parser for Meson's definition language.
    // Very basic apart from the fact that we have many precedence
    // levels so there are not enough words to describe them all.
    // Enter numbering:
    //
    // 1 assignment
    // 2 or
    // 3 and
    // 4 comparison
    // 5 arithmetic
    // 6 negation
    // 7 funcall, method call
    // 8 parentheses
    // 9 plain token

    // Assignment
    fn e1(&mut self) -> Result<Node, Error> {
        let left = self.e2()?;

        if self.accept(&Token::PlusAssign) {
        } else if self.accept(&Token::Assign) {
            let value = self.e1()?;

            return Ok(Node::Assignment(Assignment {
                left: Box::new(left),
                right: Box::new(value),
            }));
        }

        Ok(left)
    }

    // or
    fn e2(&mut self) -> Result<Node, Error> {
        let left = self.e3()?;

        Ok(left)
    }

    // and
    fn e3(&mut self) -> Result<Node, Error> {
        let left = self.e4()?;

        Ok(left)
    }

    // comparison
    fn e4(&mut self) -> Result<Node, Error> {
        let left = self.e5()?;

        Ok(left)
    }

    // arithmetic
    fn e5(&mut self) -> Result<Node, Error> {
        self.e5addsub()
    }

    fn e5addsub(&mut self) -> Result<Node, Error> {
        let mut left = self.e5muldiv()?;

        loop {
            let tok = self.curr();
            let op = self.accept_any(&vec![Token::Plus, Token::Minus]);
            if op {
                let operator = if tok == Token::Plus {
                    MathOp::Add
                } else {
                    MathOp::Sub
                };

                left = Node::Arithmetic(Arithmetic {
                    left: Box::new(left),
                    right: Box::new(self.e5muldiv()?),
                    op: operator,
                });
            } else {
                break;
            }
        }

        Ok(left)
    }
    fn e5muldiv(&mut self) -> Result<Node, Error> {
        let left = self.e6()?;

        Ok(left)
    }

    // negation
    fn e6(&mut self) -> Result<Node, Error> {
        self.e7()
    }

    // function all, method call
    fn e7(&mut self) -> Result<Node, Error> {
        let left = self.e8()?;

        if self.accept(&Token::LParen) {
            // Get ident
            // TODO throw if left isn't an IdentNode
            if let Node::Identifier(ident) = left {
                let func = Node::Function(Function {
                    name: ident,
                    args: self.args()?,
                });
                self.expect(Token::RParen)?;
                return Ok(func);
            } else {
                // TODO error
            }
        }

        Ok(left)
    }

    // parentheses
    fn e8(&mut self) -> Result<Node, Error> {
        let start = self.current.clone();

        if self.accept(&Token::LParen) {
            // TODO another statment
        }

        self.e9()
    }

    // plain
    fn e9(&mut self) -> Result<Node, Error> {
        let tok = self.curr().clone();
        if self.accept(&Token::True) {
            return Ok(Node::Boolean(true));
        } else if self.accept(&Token::False) {
            return Ok(Node::Boolean(false));
        } else if self.accept(&Token::Identifier(String::new())) {
            if let Token::Identifier(ident) = tok {
                return Ok(Node::Identifier(ident));
            }
        } else if self.accept(&Token::NumberLiteral(0)) {
            if let Token::NumberLiteral(num) = tok {
                return Ok(Node::Number(num));
            }
        } else if self.accept(&Token::StringLiteral(String::new())) {
            if let Token::StringLiteral(str) = tok {
                return Ok(Node::String(str));
            }
        }

        Ok(Node::None)
    }

    fn line(&mut self) -> Result<Node, Error> {
        // TODO check for reserved words
        self.statement()
    }

    fn code_block(&mut self) -> Result<Node, Error> {
        // TODO parse code block

        let mut block = vec![];
        loop {
            let curr_line = self.line()?;
            if curr_line != Node::None {
                block.push(curr_line);
            }

            if self.accept(&Token::EOF) {
                break;
            }
        }

        Ok(Node::Codeblock(block))
    }

    fn args(&mut self) -> Result<Arguments, Error> {
        let mut s = self.statement()?;
        let mut args = Arguments {
            args: vec![],
            kwargs: HashMap::new(),
        };

        loop {
            if let Node::None = s {
                break;
            }

            if self.accept(&Token::Comma) {
                args.args.push(s);
            } else if self.accept(&Token::Colon) {
                if let Node::Identifier(ident) = s {
                    args.kwargs.insert(ident, self.statement()?);
                }
            } else {
                args.args.push(s);
                return Ok(args);
            }

            s = self.statement()?;
        }

        Ok(args)
    }
}

pub fn parse(input: &str) -> Result<Program, Error> {
    let mut parser = Parser::new(input);

    let block = parser.code_block()?;
    let mut prog = Program { nodes: vec![] };
    if let Node::Codeblock(nodes) = block {
        prog.nodes = nodes;
    }

    Ok(prog)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::{Assignment, Function, Node};
    use anyhow::Result;

    #[test]
    fn parser() -> Result<()> {
        struct Test<'a> {
            input: &'a str,
            expected: Vec<Node>,
        }
        let tests = vec![
            Test {
                input: "true false",
                expected: vec![Node::Boolean(true), Node::Boolean(false)],
            },
            Test {
                input: "1 + 2",
                expected: vec![Node::Arithmetic(Arithmetic {
                    left: Box::new(Node::Number(1)),
                    right: Box::new(Node::Number(2)),
                    op: MathOp::Add,
                })],
            },
            Test {
                input: "ident = dependency()",
                expected: vec![Node::Assignment(Assignment {
                    left: Box::new(Node::Identifier("ident".to_string())),
                    right: Box::new(Node::Function(Function {
                        name: "dependency".to_string(),
                        args: Arguments {
                            args: vec![],
                            kwargs: HashMap::new(),
                        },
                    })),
                })],
            },
            // Test {
            //     input: "if get_option('buildtype') == 'debug'\nendif",
            //     expected: vec![],
            // },
            Test {
                input: "project('hello world', 'cpp', version: '0.1.0')",
                expected: vec![Node::Function(Function {
                    name: "project".to_string(),
                    args: Arguments {
                        args: vec![
                            Node::String("hello world".to_string()),
                            Node::String("cpp".to_string()),
                        ],
                        kwargs: HashMap::from([(
                            "version".to_string(),
                            Node::String("0.1.0".to_string()),
                        )]),
                    },
                })],
            },
            Test {
                input: "",
                expected: vec![],
            },
        ];

        for test in tests {
            let program = parse(test.input)?;

            assert_eq!(
                program.nodes.len(),
                test.expected.len(),
                "Not enough nodes parsed: {:?}",
                program.nodes
            );

            for i in 0..program.nodes.len() {
                let got = program.nodes.get(i).unwrap();
                let exp = test.expected.get(i).unwrap();

                assert_eq!(got, exp);
            }
        }

        Ok(())
    }
}
