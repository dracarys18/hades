mod ident;
mod op;

pub use ident::*;
pub use op::*;

use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn is_kind(&self, kind: &TokenKind) -> bool {
        &self.kind == kind
    }

    pub fn matches(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.kind)
    }
}

#[macro_export]
macro_rules! tok {
    // Case 1: Just a TokenKind, with start..end
    ($kind:expr, $start:expr, $end:expr) => {
        Token::new($kind, Span::new($start, $end))
    };

    // Case 2: TokenKind constructor with arguments, like Ident("foo")
    ($kind:path, $arg:expr, $start:expr, $end:expr) => {
        Token::new($kind($arg), Span::new($start, $end))
    };
}

#[macro_export]
macro_rules! token_matches {
    ($token:expr, $($pattern:pat_param)|+) => {
        matches!($token.kind(), $($pattern)|+)
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    RightBracket,
    LeftBracket,
    Comma,
    Assign,
    Dot,
    Range,
    Minus,
    Plus,
    Multiply,
    Divide,
    MinusEqual,
    PlusEqual,
    Colon,
    Semicolon,
    Newline,
    // One or two character tokens.
    Bang,
    BangEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Ident(Ident),
    String(String),
    Number(i64),
    Float(f64),
    // Keywords.
    And,
    BoleanAnd,
    Struct,
    Else,
    False,
    For,
    If,
    Return,
    Break,
    Continue,
    Or,
    BooleanOr,
    True,
    While,
    Fn,
    Let,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Range => write!(f, ".."),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Multiply => write!(f, "*"),
            TokenKind::Divide => write!(f, "/"),
            TokenKind::MinusEqual => write!(f, "-="),
            TokenKind::PlusEqual => write!(f, "+="),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Ident(s) => write!(f, "{s}"),
            TokenKind::String(s) => write!(f, "\"{s}\""),
            TokenKind::Number(n) => write!(f, "{n}"),
            TokenKind::Float(n) => write!(f, "{n}"),
            TokenKind::And => write!(f, "and"),
            TokenKind::BoleanAnd => write!(f, "&&"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::False => write!(f, "false"),
            TokenKind::For => write!(f, "for"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::BooleanOr => write!(f, "||"),
            TokenKind::True => write!(f, "true"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
        }
    }
}
