mod tokens;

use anyhow::Result;
use logos::Logos;
use std::{fs::read_to_string, path::PathBuf};
use tokens::Token;

pub fn parse_file(path: &PathBuf) -> Result<()> {
    let content = read_to_string(path)?;

    parse(&content)?;

    Ok(())
}

// struct Lexer {

// }

fn parse(input: &str) -> Result<()> {
    let lex = Token::lexer(input);
    let mut peek_itr = lex.clone();
    let mut peek_tok = peek_itr.next();

    for tok in lex {
        let token = tok.expect("Failed to tokenize the input");

        match token {
            Token::LParen => todo!(),
            Token::RParen => todo!(),
            Token::Comma => todo!(),
            Token::Colon => todo!(),
            Token::StringLiteral(_) => todo!(),
            Token::Identifier(_) => {
                if let Some(p) = peek_tok {
                    if let Ok(tok) = p {
                        if let Token::LParen = tok {
                            // Function
                            // TODO if the next token is a ( it's a function
                        }
                    }
                }

                // TODO identifier
            }
        }

        peek_tok = peek_itr.next();
    }

    Ok(())
}

fn parse_tok(tok: &Token, peek: Option<&Token>) {}

#[cfg(test)]
mod tests {}
