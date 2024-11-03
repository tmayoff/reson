mod tokens;

use crate::interpreter::ast::{Function, Node, Program};
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
    current: Option<Token>,
    peeked: Option<Token>,
}

impl<'source> Parser<'source> {
    fn new(input: &'source str) -> Self {
        let mut l = Token::lexer(input);
        let t = l.next().map(|e| e.expect("Failed to lex"));
        let c = l.next().map(|e| e.expect("Failed to lex"));

        Self {
            lexer: l,
            current: t,
            peeked: c,
        }
    }

    fn advance(&mut self) {
        self.current = self.peeked.clone();
        self.peeked = self.lexer.next().map(|e| e.expect("failed to lex"));
    }

    fn curr(&self) -> Option<Token> {
        self.current.clone()
    }

    fn peek(&self) -> Option<Token> {
        self.peeked.clone()
    }
}

fn parse(input: &str) -> Result<Program> {
    let mut parser = Parser::new(input);

    let mut program = Program { nodes: vec![] };

    while let Some(_) = parser.curr() {
        let node = parse_tok(&mut parser)?;
        program.nodes.push(node);

        parser.advance();
    }

    Ok(program)
}

fn parse_func(parser: &mut Parser, name: &str) -> Result<Function> {
    parser.advance();

    let mut args = Vec::new();
    let mut kwargs = HashMap::new();

    let mut token = parser.curr();
    while let Some(tok) = &token {
        if tok == &Token::RParen {
            break;
        }

        match tok {
            Token::StringLiteral(literal) => {
                args.push(literal.to_owned());
            }
            Token::Identifier(ident) => {
                parser.advance();
                if let Some(peek) = parser.peek() {
                    if let Token::StringLiteral(val) = peek {
                        kwargs.insert(ident.to_owned(), val);
                        parser.advance();
                    }
                    // TODO Error
                }
                // TODO error
            }
            _ => {
                // TODO Error
            }
        }

        parser.advance();
        token = parser.curr();
    }

    Ok(Function {
        name: name.to_owned(),
        args,
        kwargs,
    })
}

fn parse_tok(parser: &mut Parser) -> Result<Node> {
    if let None = parser.current {
        return Ok(Node::None);
    }

    match parser.curr().unwrap() {
        Token::LParen => todo!(),
        Token::RParen => todo!(),
        Token::Comma => todo!(),
        Token::Colon => todo!(),
        Token::StringLiteral(_) => todo!(),
        Token::Identifier(ident) => {
            if let Some(p) = parser.peek() {
                if p == Token::LParen {
                    // TODO Function here
                    return Ok(Node::Function(parse_func(parser, &ident)?));
                }
            }

            // TODO identifier
        }
    }

    Ok(Node::None)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
                args: vec!["hello world".to_string(), "cpp".to_string()],
                kwargs: HashMap::from([("version".to_string(), "0.1.0".to_string())]),
            })],
        }];

        for test in tests {
            let program = parse(test.input)?;

            assert_eq!(program.nodes.len(), test.expected.len());

            for i in 0..program.nodes.len() {
                let got = program.nodes.get(i).unwrap();
                let exp = test.expected.get(i).unwrap();

                assert_eq!(got, exp);
            }
        }

        Ok(())
    }
}
