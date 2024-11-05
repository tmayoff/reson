use logos::Logos;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"([ \t\n\f]+)|(#[^\n]*\n?)")]
pub enum Token {
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[token("-")]
    Minus,
    #[token(".")]
    Period,
    #[token("+")]
    Plus,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("==")]
    Equal,

    #[regex(r#"'([^'\\]|)*'"#, |lex| lex.slice().to_owned().replace("'", ""))]
    StringLiteral(String),

    #[regex("[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    NumberLiteral(i64),

    #[regex("[a-zA-Z]+", |lex| lex.slice().to_owned())]
    Identifier(String),

    EOF,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use coverage_helper::test;

    #[test]
    fn lexer() -> Result<()> {
        #[derive(Clone, Debug)]
        struct Test<'a> {
            input: &'a str,
            expected: Vec<Token>,
        }

        let tests: Vec<Test> = vec![
            Test {
                input: "1234 'hello world' +-(): hello true false",
                expected: vec![
                    Token::NumberLiteral(1234),
                    Token::StringLiteral("hello world".to_string()),
                    Token::Plus,
                    Token::Minus,
                    Token::LParen,
                    Token::RParen,
                    Token::Colon,
                    Token::Identifier("hello".to_string()),
                    Token::True,
                    Token::False,
                ],
            },
            Test {
                input: "project",
                expected: vec![Token::Identifier("project".to_string())],
            },
            Test {
                input: "project()",
                expected: vec![
                    Token::Identifier("project".to_string()),
                    Token::LParen,
                    Token::RParen,
                ],
            },
            Test {
                input: "project('hello world')",
                expected: vec![
                    Token::Identifier("project".to_string()),
                    Token::LParen,
                    Token::StringLiteral("hello world".to_string()),
                    Token::RParen,
                ],
            },
            Test {
                input: "project('hello world', 'cpp', version: '0.1.0')",
                expected: vec![
                    Token::Identifier("project".to_string()),
                    Token::LParen,
                    Token::StringLiteral("hello world".to_string()),
                    Token::Comma,
                    Token::StringLiteral("cpp".to_string()),
                    Token::Comma,
                    Token::Identifier("version".to_string()),
                    Token::Colon,
                    Token::StringLiteral("0.1.0".to_string()),
                    Token::RParen,
                ],
            },
        ];

        for test in tests {
            let mut lex = Token::lexer(test.input);
            let mut exp = test.expected.iter();

            assert_eq!(
                lex.clone().count(),
                exp.clone().count(),
                "Lexed count the expected count should be the same: {:?}",
                lex.collect::<Vec<_>>()
            );

            for _ in 0..lex.clone().count() {
                let l = lex.next();
                let e = exp.next();

                assert!(l.is_some(), "Lexer ran out");

                let l = l.unwrap().expect("Lexer failed");
                let e = e.unwrap();
                assert_eq!(&l, e);
            }
        }

        Ok(())
    }
}
