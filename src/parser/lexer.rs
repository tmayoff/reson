use core::panic;
use std::collections::BTreeMap;

use regex::Regex;

use super::token::{Token, TokenType, TokenValue};

macro_rules! regex {
    ($reg:expr) => {
        Regex::new($reg).expect("Regex should work")
    };
}

pub struct Lexer {
    filename: String,
    code: String,
    keywords: Vec<String>,
    futur_keywords: Vec<String>,
    token_specification: BTreeMap<TokenType, Regex>,
    line_start: usize,
    lineno: i32,
    loc: usize,

    paren_count: i32,
    brace_count: i32,
    bracket_count: i32,
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.loc > self.code.len() {
            return None;
        }

        Some(self.lex())
    }
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

        let token_specification = BTreeMap::from([
            (TokenType::Ignore, regex!(r"[\t]")),
            (TokenType::MultilineFstring, regex!(r"(?m)f'''(.|\n)*?'''")),
            (TokenType::FString, regex!(r"f'([^'\\]|(\\.))*'")),
            (TokenType::ID, regex!(r"[_a-zA-Z][_0-9a-zA-Z]*")),
            (
                TokenType::Number,
                regex!(r"0[bB][01]+|0[oO][0-7]+|0[xX][0-9a-fA-F]+|0|[1-9]\d*"),
            ),
            (TokenType::EolCont, regex!(r"\\\n")),
            (TokenType::Eol, regex!(r"\n")),
            (TokenType::MultilineString, regex!(r"(?m)'''(.|\n)*?'''")),
            (TokenType::Comment, regex!(r"#.*")),
            (TokenType::LParen, regex!(r"\(")),
            (TokenType::RParen, regex!(r"\)")),
            (TokenType::LBracket, regex!(r"\[")),
            (TokenType::RBracket, regex!(r"\]")),
            (TokenType::LBrace, regex!(r"\{")),
            (TokenType::RBrace, regex!(r"\}")),
            (TokenType::DBLQuote, regex!(r#"""#)),
            (TokenType::String, regex!(r"'([^'\\]|(\\.))*'")),
            (TokenType::Comma, regex!(r",")),
            (TokenType::PlusAssign, regex!(r"\+=")),
            (TokenType::Dot, regex!(r"\.")),
            (TokenType::Plus, regex!(r"\+")),
            (TokenType::Dash, regex!(r"-")),
            (TokenType::Star, regex!(r"\*")),
            (TokenType::Percent, regex!(r"%")),
            (TokenType::FSlash, regex!(r"/")),
            (TokenType::Colon, regex!(r":")),
            (TokenType::Equal, regex!(r"==")),
            (TokenType::NEqual, regex!(r"!=")),
            (TokenType::Assign, regex!(r"=")),
            (TokenType::LE, regex!(r"<=")),
            (TokenType::LT, regex!(r"<")),
            (TokenType::GE, regex!(r">=")),
            (TokenType::GT, regex!(r">")),
            (TokenType::QuestionMark, regex!(r"\?")),
        ]);

        Lexer {
            filename,
            code,
            keywords,
            futur_keywords,
            token_specification,
            line_start: 0,
            lineno: 1,
            loc: 0,
            paren_count: 0,
            brace_count: 0,
            bracket_count: 0,
        }
    }

    pub fn lex(&mut self) -> Token {
        let mut matched = false;
        let mut value = None;

        for (tid, reg) in &self.token_specification {
            let tid = tid.clone();

            let m = reg.find_at(&self.code, self.loc as usize);
            if let Some(m) = m {
                let curline = self.lineno;
                let curline_start = self.line_start;
                let col = (m.start() - self.line_start) as i32;
                self.loc = m.end();
                let match_text = m.as_str();
                matched = true;

                match tid {
                    TokenType::Ignore | TokenType::Comment => break,

                    TokenType::LParen => self.paren_count += 1,
                    TokenType::RParen => self.paren_count -= 1,
                    TokenType::LBracket => self.bracket_count += 1,
                    TokenType::RBracket => self.bracket_count -= 1,
                    TokenType::LBrace => self.brace_count += 1,
                    TokenType::RBrace => self.brace_count += 1,
                    TokenType::DBLQuote => panic!("Double quotes are not supported"),
                    TokenType::FString | TokenType::String => {
                        if match_text.contains('\n') {
                            panic!("New line detected in string");
                        }

                        value = Some(TokenValue::Str(
                            match_text[if tid == TokenType::FString { 2 } else { 1 }
                                ..match_text.len() - 1]
                                .to_string(),
                        ));
                    }
                    TokenType::MultilineFstring | TokenType::MultilineString => todo!(),
                    TokenType::Number => {
                        value = Some(TokenValue::Int(
                            match_text.parse::<i32>().expect("Failed to parse number"),
                        ))
                    }
                    TokenType::EolCont => {
                        self.lineno += 1;
                        self.line_start = self.loc;
                        break;
                    }
                    TokenType::Eol => {
                        self.lineno += 1;
                        self.line_start = self.loc;
                        if self.paren_count > 0 || self.bracket_count > 0 || self.brace_count > 0 {
                            break;
                        }
                    }
                    TokenType::ID => {
                        if self.keywords.contains(&match_text.to_string()) {
                            // tid = TokenType::ID(Some(match_text.to_string()));
                        } else {
                            value = Some(TokenValue::Str(match_text.to_string()));
                        }
                    }

                    _ => {
                        continue;
                    }
                }

                return Token::new(
                    tid,
                    &self.filename,
                    curline_start as i32,
                    curline,
                    col,
                    value,
                );
            }
        }

        if !matched {
            panic!("Failed to match with Token");
        }

        Token::new(TokenType::Ignore, "", 0, 0, 0, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut l = Lexer::new("123".to_string(), "".to_string());
        let t = l.next();
        assert!(t.is_some());
        let t = t.unwrap();
        assert_eq!(t.tid, TokenType::Number);
        assert!(t.value.is_some());
        assert_eq!(t.value.unwrap(), TokenValue::Int(123));

        let mut l = Lexer::new("'hello world'".to_string(), "".to_string());
        let t = l.next();
        assert!(t.is_some());
        let t = t.unwrap();
        assert_eq!(t.tid, TokenType::String);
        assert!(t.value.is_some());
        assert_eq!(t.value.unwrap(), TokenValue::Str("hello world".to_string()));
    }
}
