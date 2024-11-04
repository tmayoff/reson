mod tokens;

use crate::interpreter::ast::{Arguments, Function, Node, Program};
use anyhow::Result;
use logos::{Lexer, Logos};
use std::{collections::HashMap, fs::read_to_string, path::PathBuf};
use tokens::Token;

pub fn parse_file(path: &PathBuf) -> Result<()> {
    let content = read_to_string(path)?;

    parse(&content)?;

    Ok(())
}

struct Parser<'source> {
    lexer: Lexer<'source, Token>,
    current: Token,
    peeked: Token,
}

impl<'source> Parser<'source> {
    fn new(input: &'source str) -> Self {
        let mut l = Token::lexer(input);
        let t = l
            .next()
            .map(|e| e.expect("Failed to lex"))
            .unwrap_or(Token::EOF);
        let c = l
            .next()
            .map(|e| e.expect("Failed to lex"))
            .unwrap_or(Token::EOF);

        Self {
            lexer: l,
            current: t,
            peeked: c,
        }
    }

    pub fn accept(&mut self, tok: Token) -> bool {
        if std::mem::discriminant(&self.current) == std::mem::discriminant(&tok) {
            self.advance();
            return true;
        }

        false
    }

    fn advance(&mut self) {
        self.current = self.peeked.clone();
        self.peeked = self
            .lexer
            .next()
            .map(|e| e.expect("failed to lex"))
            .unwrap_or(Token::EOF);
    }

    fn curr(&self) -> Token {
        self.current.clone()
    }

    fn peek(&self) -> Token {
        self.peeked.clone()
    }

    fn statement(&mut self) -> Result<Node> {
        return self.e1();
    }

    // Assignment
    fn e1(&mut self) -> Result<Node> {
        let left = self.e2()?;

        Ok(left)
    }

    // or
    fn e2(&mut self) -> Result<Node> {
        let left = self.e3()?;

        Ok(left)
    }

    // and
    fn e3(&mut self) -> Result<Node> {
        let left = self.e4()?;

        Ok(left)
    }

    // comparison
    fn e4(&mut self) -> Result<Node> {
        let left = self.e5()?;

        Ok(left)
    }

    // arithmetic
    fn e5(&mut self) -> Result<Node> {
        self.e5addsub()
    }

    fn e5addsub(&mut self) -> Result<Node> {
        let left = self.e5muldiv()?;

        Ok(left)
    }
    fn e5muldiv(&mut self) -> Result<Node> {
        let left = self.e6()?;

        Ok(left)
    }

    // negation
    fn e6(&mut self) -> Result<Node> {
        self.e7()
    }

    // funcall, method call
    fn e7(&mut self) -> Result<Node> {
        let left = self.e8()?;

        if self.accept(Token::LParen) {
            // Get ident
            // TODO throw if left isn't an IdentNode
            if let Node::Identifier(ident) = left {
                return Ok(Node::Function(Function {
                    name: ident,
                    args: self.args()?,
                }));
            } else {
                // TODO error
            }
        }

        Ok(left)
    }

    // parentheses
    fn e8(&mut self) -> Result<Node> {
        let start = self.current.clone();

        if self.accept(Token::LParen) {
            // TODO another statment
        }

        self.e9()
    }

    // plain
    fn e9(&mut self) -> Result<Node> {
        let tok = self.curr().clone();
        if self.accept(Token::True) {
            return Ok(Node::Boolean(true));
        } else if self.accept(Token::False) {
            return Ok(Node::Boolean(false));
        } else if self.accept(Token::Identifier(String::new())) {
            if let Token::Identifier(ident) = tok {
                return Ok(Node::Identifier(ident));
            }
        }

        Ok(Node::None)
    }

    fn line(&mut self) -> Result<Node> {
        // TODO check for reserved words
        self.statement()
    }

    fn codeblock(&mut self) -> Result<Node> {
        // TODO parse codeblock

        let mut block = vec![];
        loop {
            let curr_line = self.line()?;
            if curr_line != Node::None {
                block.push(curr_line);
            }

            if !self.accept(Token::EOF) {
                break;
            }
        }

        Ok(Node::Codeblock(block))
    }

    fn args(&mut self) -> Result<Arguments> {
        let args = Arguments {};
        Ok(args)
    }
}

fn parse(input: &str) -> Result<Program> {
    let mut parser = Parser::new(input);

    let block = parser.codeblock()?;
    let mut prog = Program { nodes: vec![] };
    if let Node::Codeblock(nodes) = block {
        prog.nodes = nodes;
    }

    Ok(prog)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::{Function, Node};
    use anyhow::Result;

    #[test]
    fn parser() -> Result<()> {
        struct Test<'a> {
            input: &'a str,
            expected: Vec<Node>,
        }
        let tests = vec![Test {
            input: "project('hello world', 'cpp', version: '0.1.0')",
            expected: vec![Node::Function(Function {
                name: "project".to_string(),
                args: Arguments {},
            })],
        }];

        for test in tests {
            let program = parse(test.input)?;

            assert_eq!(
                program.nodes.len(),
                test.expected.len(),
                "Not enough nodes parsed"
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
