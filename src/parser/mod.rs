mod tokens;

use crate::interpreter::ast::{
    Arguments, Arithmetic, Assignment, CompareOp, Comparison, Function, If, IfClause, MathOp, Node,
    Program,
};
use logos::{Lexer, Logos};
use std::{collections::HashMap, fs::read_to_string, path::PathBuf};
use thiserror::Error;
use tokens::Token;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to lex {0}")]
    LexError(String),
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
}

impl<'source> Parser<'source> {
    fn new(input: &'source str) -> Result<Self, Error> {
        let mut l = Token::lexer(input);
        let t = match l.next().unwrap_or(Ok(Token::EOF)) {
            Ok(t) => Ok(t),
            Err(_) => Err(Error::LexError(l.slice().to_string())),
        }?;

        Ok(Self {
            lexer: l,
            current: t,
        })
    }

    pub fn accept(&mut self, tok: &Token) -> Result<bool, Error> {
        if std::mem::discriminant(&self.current) == std::mem::discriminant(&tok) {
            self.advance()?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn accept_any(&mut self, toks: &Vec<Token>) -> Result<bool, Error> {
        for tok in toks {
            if self.accept(tok)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn expect(&mut self, tok: Token) -> Result<(), Error> {
        if !self.accept(&tok)? {
            // TODO panic here
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<(), Error> {
        self.current = match self.lexer.next().unwrap_or(Ok(Token::EOF)) {
            Ok(t) => Ok(t),
            Err(_) => Err(Error::LexError(self.lexer.slice().to_string())),
        }?;
        Ok(())
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

        if self.accept(&Token::PlusAssign)? {
        } else if self.accept(&Token::Assign)? {
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

        let comparison_ops = [Token::Equal];
        for compare in comparison_ops {
            let op = self.curr();
            if self.accept(&compare)? {
                let op = match op {
                    Token::Equal => CompareOp::Equal,
                    _ => todo!("Compare op doesn't exist {:?}", self.curr()),
                };

                return Ok(Node::Comparison(Comparison {
                    left: Box::new(left),
                    op,
                    right: Box::new(self.e5()?),
                }));
            }
        }

        // TODO check for Token::Not

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
            let op = self.accept_any(&vec![Token::Plus, Token::Minus])?;
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

        if self.accept(&Token::LParen)? {
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
        let _start = self.current.clone();

        if self.accept(&Token::LParen)? {
            // TODO another statment
        }

        self.e9()
    }

    // plain
    fn e9(&mut self) -> Result<Node, Error> {
        let tok = self.curr().clone();
        if self.accept(&Token::True)? {
            return Ok(Node::Boolean(true));
        } else if self.accept(&Token::False)? {
            return Ok(Node::Boolean(false));
        } else if self.accept(&Token::Identifier(String::new()))? {
            if let Token::Identifier(ident) = tok {
                return Ok(Node::Identifier(ident));
            }
        } else if self.accept(&Token::NumberLiteral(0))? {
            if let Token::NumberLiteral(num) = tok {
                return Ok(Node::Number(num));
            }
        } else if self.accept(&Token::StringLiteral(String::new()))? {
            if let Token::StringLiteral(str) = tok {
                return Ok(Node::String(str));
            }
        }

        Ok(Node::None)
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

            if self.accept(&Token::Comma)? {
                args.args.push(s);
            } else if self.accept(&Token::Colon)? {
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

    fn ifblock(&mut self) -> Result<IfClause, Error> {
        // let if_node = If {};
        let condition = self.statement()?;
        let mut clause = IfClause { ifs: vec![] };
        self.expect(Token::EOL)?;

        let block = self.code_block()?;

        clause.ifs.push(If { condition, block });

        // Elseif blocks

        return Ok(clause);
    }

    fn line(&mut self) -> Result<Node, Error> {
        let _block_start = self.curr();

        if self.curr() == Token::EOL {
            return Ok(Node::None);
        }
        if self.accept(&Token::If)? {
            let ifblock = self.ifblock()?;
            self.expect(Token::Endif)?;
            return Ok(Node::IfClause(ifblock));
        }

        self.statement()
    }

    fn code_block(&mut self) -> Result<Node, Error> {
        let mut block = vec![];
        let mut cond = true;
        while cond {
            let curr_line = self.line()?;
            if curr_line != Node::None {
                block.push(curr_line);
            }

            cond = self.accept(&Token::EOL)?;
        }

        Ok(Node::Codeblock(block))
    }
}

pub fn parse(input: &str) -> Result<Program, Error> {
    let mut parser = Parser::new(input)?;

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
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn parser() -> Result<()> {
        struct Test<'a> {
            input: &'a str,
            expected: Vec<Node>,
        }
        let tests = vec![
            Test {
                input: "true\nfalse",
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
            Test {
                input: r#"
                if get_option('buildtype') == 'debug'
                endif"#,
                expected: vec![Node::IfClause(IfClause {
                    ifs: vec![If {
                        condition: Node::Comparison(Comparison {
                            left: Box::new(Node::Function(Function {
                                name: "get_option".to_string(),
                                args: Arguments {
                                    args: vec![Node::String("buildtype".to_string())],
                                    kwargs: HashMap::new(),
                                },
                            })),
                            op: CompareOp::Equal,
                            right: Box::new(Node::String("debug".to_string())),
                        }),
                        block: Node::Codeblock(vec![]),
                    }],
                })],
            },
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
                "Not enough nodes parsed (got: {}, expected: {}): {:?}\n{:?}",
                program.nodes.len(),
                test.expected.len(),
                program.nodes,
                test.input
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
