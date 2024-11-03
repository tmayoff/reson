use logos::Logos;

#[derive(Logos, Clone, Debug, PartialEq)]
// #[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[regex(r#"'([^'\\]|)*'"#, |lex| lex.slice().to_owned().replace("'", ""))]
    StringLiteral(String),

    #[regex("[a-zA-Z]+", |lex| lex.slice().to_owned())]
    Identifier(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn lexer() -> Result<()> {
        #[derive(Clone, Debug)]
        struct Test<'a> {
            input: &'a str,
            expected: Vec<Token>,
        }

        let tests: Vec<Test> = vec![
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
            // Test {
            //     input: "project('hello world', 'cpp', version: '0.1.0')",
            //     expected: vec![
            //         Token::Identifier("project".to_string()),
            //         Token::LParen,
            //         Token::StringLiteral("hello_world".to_string()),
            //         Token::Comma,
            //     ],
            // },
        ];

        for test in tests {
            let mut lex = Token::lexer(test.input);
            let mut exp = test.expected.iter();

            assert_eq!(
                lex.clone().count(),
                exp.clone().count(),
                "Lexed count the expected count should be the same"
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
