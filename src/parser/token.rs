use logos_derive::Logos;

#[derive(Logos, Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Token {
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("elif")]
    ElIf,
    #[token("endif")]
    EndIf,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("foreach")]
    Foreach,
    #[token("endforeach")]
    EndForeach,
    #[token("in")]
    In,
    #[token("continue")]
    Continue,
    #[token("break")]
    Break,
    #[token("ignore")]
    Ignore,
    //    #[regex("")]
    //    MultilineFstring,
    //    #[regex("f'")]
    //    FString,
    #[regex(r#"'([^'\\]|(\\.))*'"#, |lex| {
        let len = lex.slice().len();
        lex.slice()[1..len-1].to_owned()
    })]
    String(String),
    #[regex("([A-Za-z_-])+", |lex| lex.slice().to_owned())]
    ID(String),
    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    Number(i32),
    //    #[regex("")]
    //    EolCont,
    #[regex("\n")]
    EOL,
    //    #[regex("")]
    //    MultilineString,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LCurly,
    #[token("}")]
    RCurly,
    #[token("[")]
    LBrace,
    #[token("]")]
    RBrace,
    #[token(",")]
    Comma,
    #[token("+=")]
    PlusAssign,
    #[token(".")]
    Dot,
    #[token("+")]
    Plus,
    #[token("-")]
    Dash,
    #[token("*")]
    Star,
    #[token("%")]
    Percent,
    #[token("/")]
    FSlash,
    #[token(":")]
    Colon,
    #[token("==")]
    Equal,
    #[token("!=")]
    NEqual,
    #[token("=")]
    Assign,
    #[token("<=")]
    LE,
    #[token("<")]
    LT,
    #[token(">=")]
    GE,
    #[token(">")]
    GT,
    #[token("notin")]
    NotIn,
    #[token("?")]
    QuestionMark,

    #[regex(r"[ \t]+", logos::skip)]
    #[error]
    EOF,
}

mod tests {
    use super::Token;
    use logos::Logos;

    #[test]
    fn lexer_test() {
        let input = r#"
        true false if else elif endif and or not foreach endforeach in continue break ignore meson_version
        '' 'Hello World'
        ()
        "#;
        let expected_tokens: Vec<Token> = vec![
            Token::EOL,
            Token::True,
            Token::False,
            Token::If,
            Token::Else,
            Token::ElIf,
            Token::EndIf,
            Token::And,
            Token::Or,
            Token::Not,
            Token::Foreach,
            Token::EndForeach,
            Token::In,
            Token::Continue,
            Token::Break,
            Token::Ignore,
            Token::ID("meson_version".to_string()),
            Token::EOL,
            Token::String("".to_string()),
            Token::String("Hello World".to_string()),
            Token::EOL,
            Token::LParen,
            Token::RParen,
        ];

        let mut lexer = Token::lexer(input);

        expected_tokens.iter().for_each(|expected| {
            let next = lexer.next().unwrap();

            assert!(
                next.clone() == expected.clone(),
                "Got {:?}, expected {:?}",
                next,
                expected
            );
        });
    }
}
