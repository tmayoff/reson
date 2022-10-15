use core::panic;
use std::iter::Peekable;
use std::str::Chars;

use super::token::{Token, TokenType, TokenValue};



pub struct Lexer {
    filename: String,
    code: String,
    keywords: Vec<String>,
    futur_keywords: Vec<String>,
    line_start: usize,
    lineno: i32,
    loc: usize,

    paren_count: i32,
    brace_count: i32,
    bracket_count: i32,
}

impl Lexer {
    pub fn new(code: String, filename: String) -> Self {
        let keywords = vec![
            "true".to_string(),
            "false".to_string(),
            "if".to_string(),
            "else".to_string(),
            "elif".to_string(),
            "endif".to_string(),
            "and".to_string(),
            "or".to_string(),
            "not".to_string(),
            "foreach".to_string(),
            "endforeach".to_string(),
            "in".to_string(),
            "continue".to_string(),
            "break".to_string(),
        ];

        let futur_keywords = vec!["return".to_string()];


        Lexer {
            filename,
            code,
            keywords,
            futur_keywords,
            line_start: 0,
            lineno: 1,
            loc: 0,
            paren_count: 0,
            brace_count: 0,
            bracket_count: 0,
        }
    }

    pub fn next(&mut self) -> Token {
        if self.loc >= self.code.len() {
            return Token::new(
                TokenType::EOF,
                &self.filename,
                self.line_start as i32,
                self.lineno,
                0,
                TokenValue::None,
            );
        }

        self.lex()
    }

    pub fn lex(&mut self) -> Token {
        let mut matched = false;
        let mut value = TokenValue::None;

        let binding = self.code[self.loc..].to_string();
        let mut code_slice = binding.chars().peekable();

        while let Some(&c) = code_slice.peek() {
            match c {
                '\t' | '#' => {
                    // Ignore / Comment
                    // TODO skip line
                }
                '(' => self.paren_count += 1,
                ')' => self.paren_count -= 1,
                '[' => self.bracket_count += 1,
                ']' => self.bracket_count -= 1,
                '{' => self.brace_count += 1,
                '}' => self.brace_count -= 1,
                '=' => {
                    self.advance(&mut code_slice);
                    if code_slice.peek() == Some(&'=') {
                        self.advance(&mut code_slice);
                        return Token::new(
                            TokenType::Equal,
                            &self.filename.clone(),
                            self.line_start as i32,
                            self.lineno,
                            0,
                            TokenValue::None,
                        );
                    }

                    return Token::new(
                        TokenType::Assign,
                        &self.filename.clone(),
                        self.line_start as i32,
                        self.lineno,
                        0,
                        TokenValue::None,
                    );
                }
                '\'' => {
                    self.advance(&mut code_slice);
                    let str = self.get_string(&mut code_slice);
                    return Token::new(
                        TokenType::String,
                        &self.filename.clone(),
                        self.line_start as i32,
                        self.lineno,
                        0,
                        TokenValue::Str(str),
                    );
                }
                c if c.is_whitespace() => {
                    self.consume_whitespace(&mut code_slice);
                }
                c if c.is_alphabetic() => {
                    // Peek further
                    let mut n = code_slice.clone();
                    n.next();
                    if c == 'f' && n.peek() == Some(&'\'') {
                        self.advance(&mut code_slice);
                        self.advance(&mut code_slice);
                        let str = self.get_string(&mut code_slice);
                        return Token::new(
                            TokenType::FString,
                            &self.filename.clone(),
                            self.line_start as i32,
                            self.lineno,
                            0,
                            TokenValue::Str(str),
                        );
                    } else {
                        // ID
                        let str = self.get_id(&mut code_slice);
                        return Token::new(
                            TokenType::ID,
                            &self.filename.clone(),
                            self.line_start as i32,
                            self.lineno,
                            0,
                            TokenValue::Str(str),
                        );
                    }
                }
                c if c.is_numeric() => {
                    let num = self.get_int(&mut code_slice);
                    return Token::new(
                        TokenType::Number,
                        &self.filename.clone(),
                        self.line_start as i32,
                        self.lineno,
                        0,
                        TokenValue::Int(num),
                    );
                }

                _ => panic!("Unrecognized token"),
            }
        }


        if !matched {
            panic!("Failed to match with Token");
        }

        Token::new(TokenType::Ignore, "", 0, 0, 0, TokenValue::None)
    }

    fn advance(&mut self, it: &mut Peekable<Chars>) -> Option<char> {
        self.loc += 1;
        let c = it.next();

        if let Some(c) = c {
            if c == '\n' {
                self.lineno += 1;
            }
        }

        c
    }

    fn get_int(&mut self, it: &mut Peekable<Chars>) -> i32 {
        let mut str = String::new();
        while let Some(c) = self.advance(it) {
            if c.is_numeric() {
                str.push(c);
            } else {
                break;
            }
        }

        str.parse::<i32>().expect("Failed to parse integer")
    }

    fn get_id(&mut self, it: &mut Peekable<Chars>) -> String {
        let mut str = String::new();
        while let Some(c) = self.advance(it) {
            if c == ' ' {
                break;
            }

            str.push(c);
        }

        str
    }

    fn get_string(&mut self, it: &mut Peekable<Chars>) -> String {
        let mut str = String::new();
        while let Some(c) = self.advance(it) {
            if c == '\'' {
                break;
            }

            str.push(c);
        }

        str
    }

    fn consume_whitespace(&mut self, it: &mut Peekable<Chars>) {
        while let Some(c) = it.peek() {
            if !c.is_whitespace() {
                break;
            }

            it.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_token() {
        struct Test<'a> {
            input: &'a str,
            expected: Token,
        }

        let tests = vec![
            Test {
                input: "'hello world'",
                expected: Token {
                    tid: TokenType::String,
                    value: TokenValue::Str("hello world".to_string()),
                    ..Default::default()
                },
            },
            Test {
                input: "f'hello world'",
                expected: Token {
                    tid: TokenType::FString,
                    value: TokenValue::Str("hello world".to_string()),
                    ..Default::default()
                },
            },
            Test {
                input: "123",
                expected: Token {
                    tid: TokenType::Number,
                    value: TokenValue::Int(123),
                    ..Default::default()
                },
            },
        ];

        for (index, test) in tests.iter().enumerate() {
            let mut l = Lexer::new(test.input.to_string(), "".to_string());
            let t = l.next();
            assert_eq!(
                t.tid, test.expected.tid,
                "Test {}, TokenType ({:?}) mismatch  expected {:?}",
                index, t.tid, test.expected.tid
            );
            assert_eq!(
                t.value, test.expected.value,
                "Test {}, TokenValue ({:?}) mismatch expected ({:?})",
                index, t.value, test.expected.value
            );
        }
    }

    #[test]
    fn line_test() {
        struct Test<'a> {
            input: &'a str,
            expected: Vec<Token>,
        }

        let tests = vec![
            Test {
                input: "a = 123",
                expected: vec![
                    Token {
                        tid: TokenType::ID,
                        value: TokenValue::Str("a".to_string()),
                        ..Default::default()
                    },
                    Token {
                        tid: TokenType::Assign,
                        value: TokenValue::None,
                        ..Default::default()
                    },
                    Token {
                        tid: TokenType::Number,
                        value: TokenValue::Int(123),
                        ..Default::default()
                    },
                ],
            },
            Test {
                input: "10 == 10",
                expected: vec![
                    Token {
                        tid: TokenType::Number,
                        value: TokenValue::Int(10),
                        ..Default::default()
                    },
                    Token {
                        tid: TokenType::Equal,
                        ..Default::default()
                    },
                    Token {
                        tid: TokenType::Number,
                        value: TokenValue::Int(10),
                        ..Default::default()
                    },
                ],
            },
        ];

        for (index, test) in tests.into_iter().enumerate() {
            let mut l = Lexer::new(test.input.to_string(), "".to_string());

            let mut it = test.expected.into_iter();
            loop {
                let t = l.next();
                let expected = it.next();
                if t.tid == TokenType::EOF {
                    break;
                }

                assert!(expected.is_some());
                match expected {
                    Some(e) => {
                        assert_eq!(
                            t.tid, e.tid,
                            "Test {}, TokenType ({:?}) mismatch  expected {:?}",
                            index, t.tid, e.tid
                        );
                        assert_eq!(
                            t.value, e.value,
                            "Test {}, TokenValue ({:?}) mismatch expected ({:?})",
                            index, t.value, e.value
                        );
                    }
                    None => {
                        unreachable!();
                    }
                }
            }
        }
    }
}
