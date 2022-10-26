use std::hash::Hash;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenValue {
    None,
    Str(String),
    Bool(bool),
    Int(i32),
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum TokenType {
    True,
    False,
    If,
    Else,
    ElIf,
    EndIf,
    And,
    Or,
    Not,
    Foreach,
    EndForeach,
    In,
    Continue,
    Break,

    Ignore,
    MultilineFstring,
    FString,
    String,
    ID,
    Number,
    EolCont,
    EOL,
    MultilineString,
    LParen,
    RParen,
    LCurly,
    RCurly,
    LBrace,
    RBrace,
    Comma,
    PlusAssign,
    Dot,
    Plus,
    Dash,
    Star,
    Percent,
    FSlash,
    Colon,
    Equal,
    NEqual,
    Assign,
    LE,
    LT,
    GE,
    GT,
    NotIn,
    QuestionMark,
    EOF,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Token {
    pub tid: TokenType,
    pub filename: String,
    pub line_start: i32,
    pub lineno: i32,
    pub colno: i32,
    pub value: TokenValue,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            tid: TokenType::EOF,
            filename: "".to_string(),
            line_start: 0,
            lineno: 0,
            colno: 0,
            value: TokenValue::None,
        }
    }
}

impl Token {
    pub fn new(
        tid: TokenType,
        filename: &str,
        line_start: i32,
        lineno: i32,
        colno: i32,
        value: TokenValue,
    ) -> Self {
        Token {
            tid,
            filename: filename.to_string(),
            line_start,
            lineno,
            colno,
            value,
        }
    }
}
