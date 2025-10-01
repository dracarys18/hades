mod error;

use crate::error::Span;
use crate::tok;
use crate::tokens::{Assoc, Ident, OpInfo, Token, TokenKind};
pub use error::{LexError, LexResult};

use phf::phf_map;

static KEYWORDS: phf::Map<&'static str, TokenKind> = phf_map! {
    "let" => TokenKind::Let,
    "if" => TokenKind::If,
    "else" => TokenKind::Else,
    "while" => TokenKind::While,
    "for" => TokenKind::For,
    "fn" => TokenKind::Fn,
    "struct" => TokenKind::Struct,
    "return" => TokenKind::Return,
    "break" => TokenKind::Break,
    "continue" => TokenKind::Continue,
    "true" => TokenKind::True,
    "false" => TokenKind::False,
};

pub struct Lexer<'a> {
    input: &'a str,
    source_id: String,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, source_id: String) -> Self {
        Self {
            input: source,
            source_id,
            tokens: Vec::new(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        if let Some('\n') = c {
            self.push_token(tok!(TokenKind::Newline, self.pos, self.pos + 1));
        }
        self.pos += 1;
        c
    }

    fn move_next(&mut self) {
        self.pos += 1;
    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn consume_while<F>(&mut self, mut f: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let start_pos = self.pos;
        while let Some(c) = self.peek() {
            if f(c) {
                self.pos += 1;
            } else {
                break;
            }
        }
        // Use slice to avoid character-by-character allocation
        self.input[start_pos..self.pos].to_string()
    }

    fn parse_plus_equal(&mut self) {
        let start_pos = self.pos;

        if self.peek() == Some('+') {
            self.next();
            if self.peek() == Some('=') {
                self.next();
                self.push_token(tok!(TokenKind::PlusEqual, start_pos, self.pos));
            } else {
                self.push_token(tok!(TokenKind::Plus, start_pos, self.pos));
            }
        }
    }

    fn parse_minus_equal(&mut self) {
        let start_pos = self.pos;
        if self.peek() == Some('-') {
            self.next();
            if self.peek() == Some('=') {
                self.next();
                self.push_token(tok!(TokenKind::MinusEqual, start_pos, self.pos));
            } else {
                self.push_token(tok!(TokenKind::Minus, start_pos, self.pos));
            }
        }
    }

    fn parse_string(&mut self) -> LexResult<()> {
        if self.peek() == Some('"') {
            let start_pos = self.pos;
            self.next();

            // Pre-allocate with estimated capacity to reduce reallocations
            let mut s = String::with_capacity(32);

            loop {
                match self.next() {
                    Some('"') => break,
                    Some('\\') => match self.next() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('"') => s.push('"'),
                        Some('\\') => s.push('\\'),
                        Some(other) => s.push(other),
                        None => {
                            let span = start_pos..self.pos;
                            return Err(LexError::unterminated_string(
                                span,
                                self.source_id.clone(),
                            ));
                        }
                    },
                    Some(c) => s.push(c),
                    None => {
                        let span = start_pos..self.pos;
                        return Err(LexError::unterminated_string(span, self.source_id.clone()));
                    }
                }
            }

            self.push_token(tok!(TokenKind::String(s), start_pos, self.pos));
        }
        Ok(())
    }

    fn parse_keyword_or_identifier(&mut self) {
        let start_pos = self.pos;

        if let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() || c == '_' {
                let ident = self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_');
                if let Some(keyword_token) = KEYWORDS.get(ident.as_str()) {
                    self.push_token(tok!(keyword_token.clone(), start_pos, self.pos));
                } else {
                    self.push_token(tok!(
                        TokenKind::Ident(Ident::new(ident)),
                        start_pos,
                        self.pos
                    ));
                }
            }
        }
    }

    fn parse_number(&mut self) -> LexResult<()> {
        let start_pos = self.pos;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                self.next();
            } else {
                break;
            }
        }

        // Use slice to avoid allocation until needed for parsing
        let num_str = &self.input[start_pos..self.pos];
        let span = start_pos..self.pos;

        if num_str.contains('.') {
            match num_str.parse::<f64>() {
                Ok(num) => {
                    self.push_token(tok!(TokenKind::Float(num), start_pos, self.pos));
                    Ok(())
                }
                Err(err) => Err(LexError::invalid_number(
                    &num_str,
                    &err.to_string(),
                    span,
                    self.source_id.clone(),
                )),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(n) => {
                    self.push_token(tok!(TokenKind::Number(n), start_pos, self.pos));
                    Ok(())
                }
                Err(err) => Err(LexError::invalid_number(
                    &num_str,
                    &err.to_string(),
                    span,
                    self.source_id.clone(),
                )),
            }
        }
    }

    fn parse_operator(&mut self) {
        let start_pos = self.pos;
        if let Some(c) = self.peek() {
            match c {
                '+' => self.parse_plus_equal(),
                '-' => self.parse_minus_equal(),
                '*' => {
                    self.next();
                    self.push_token(tok!(TokenKind::Multiply, start_pos, self.pos));
                }
                '/' => {
                    self.next();
                    self.push_token(tok!(TokenKind::Divide, start_pos, self.pos));
                }
                '=' => {
                    self.next();
                    if self.peek() == Some('=') {
                        self.next();

                        self.push_token(tok!(TokenKind::EqualEqual, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Assign, start_pos, self.pos));
                    }
                }
                '!' => {
                    self.move_next();
                    if self.peek() == Some('=') {
                        self.next();
                        self.push_token(tok!(TokenKind::BangEqual, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Bang, start_pos, self.pos));
                    }
                }
                '|' => {
                    self.move_next();
                    if self.peek() == Some('|') {
                        self.next();
                        self.push_token(tok!(TokenKind::Or, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::BooleanOr, start_pos, self.pos));
                    }
                }
                '&' => {
                    self.move_next();
                    if self.peek() == Some('&') {
                        self.next();
                        self.push_token(tok!(TokenKind::And, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::BoleanAnd, start_pos, self.pos));
                    }
                }
                '>' => {
                    self.move_next();
                    if self.peek() == Some('=') {
                        self.next();
                        self.push_token(tok!(TokenKind::GreaterEqual, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Greater, start_pos, self.pos));
                    }
                }
                '<' => {
                    self.move_next();
                    if self.peek() == Some('=') {
                        self.next();
                        self.push_token(tok!(TokenKind::LessEqual, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Less, start_pos, self.pos));
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_punctuation(&mut self) {
        let start_pos = self.pos;
        if let Some(c) = self.peek() {
            match c {
                '(' => {
                    self.next();
                    self.push_token(tok!(TokenKind::LeftParen, start_pos, self.pos));
                }
                ')' => {
                    self.next();
                    self.push_token(tok!(TokenKind::RightParen, start_pos, self.pos));
                }
                '{' => {
                    self.next();
                    self.push_token(tok!(TokenKind::LeftBrace, start_pos, self.pos));
                }
                '}' => {
                    self.next();
                    self.push_token(tok!(TokenKind::RightBrace, start_pos, self.pos));
                }
                '[' => {
                    self.next();
                    self.push_token(tok!(TokenKind::LeftBracket, start_pos, self.pos));
                }
                ']' => {
                    self.next();
                    self.push_token(tok!(TokenKind::RightBracket, start_pos, self.pos));
                }
                ',' => {
                    self.next();
                    self.push_token(tok!(TokenKind::Comma, start_pos, self.pos));
                }
                '.' => {
                    self.next();
                    if self.peek() == Some('.') {
                        self.next();
                        self.push_token(tok!(TokenKind::Range, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Dot, start_pos, self.pos));
                    }
                }
                ';' => {
                    self.next();
                    self.push_token(tok!(TokenKind::Semicolon, start_pos, self.pos));
                }
                ':' => {
                    self.next();
                    self.push_token(tok!(TokenKind::Colon, start_pos, self.pos));
                }
                _ => {}
            }
        }
    }

    fn is_operator_char(c: char) -> bool {
        matches!(c, '+' | '-' | '*' | '/' | '=' | '!' | '>' | '<' | '&' | '|')
    }

    fn is_punctuation_char(c: char) -> bool {
        matches!(c, '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | '.' | ':')
    }

    fn is_whitespace(c: char) -> bool {
        c.is_whitespace()
    }

    fn is_string_start(c: char) -> bool {
        c == '"'
    }

    pub fn tokenize(&mut self) -> LexResult<()> {
        while let Some(c) = self.peek() {
            match c {
                ch if ch.is_ascii_digit() => self.parse_number()?,
                ch if (ch.is_alphabetic() || ch.eq(&'_')) => self.parse_keyword_or_identifier(),
                ch if Self::is_operator_char(ch) => self.parse_operator(),
                ch if Self::is_punctuation_char(ch) => self.parse_punctuation(),
                ch if Self::is_whitespace(ch) => self.move_next(),
                ch if Self::is_string_start(ch) => self.parse_string()?,
                _ => {
                    let span = self.pos..self.pos + 1;
                    return Err(LexError::unexpected_character(
                        c,
                        span,
                        self.source_id.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn make_snippet(&self, line: usize, column: usize) -> String {
        let lines: Vec<&str> = self.input.lines().collect();
        if let Some(src_line) = lines.get(line - 1) {
            let mut s = src_line.to_string();
            s.push('\n');
            s.push_str(&" ".repeat(column.saturating_sub(1)));
            s.push('^');
            s
        } else {
            "".to_string()
        }
    }

    pub fn get_precedence(token: &Token) -> Option<OpInfo> {
        match token.kind() {
            TokenKind::Multiply | TokenKind::Divide => Some(OpInfo {
                prec: 5,
                assoc: Assoc::Left,
            }),
            TokenKind::Plus | TokenKind::Minus => Some(OpInfo {
                prec: 4,
                assoc: Assoc::Left,
            }),
            TokenKind::Greater
            | TokenKind::Less
            | TokenKind::GreaterEqual
            | TokenKind::LessEqual => Some(OpInfo {
                prec: 3,
                assoc: Assoc::Left,
            }),
            TokenKind::EqualEqual | TokenKind::BangEqual => Some(OpInfo {
                prec: 2,
                assoc: Assoc::Left,
            }),
            TokenKind::And => Some(OpInfo {
                prec: 1,
                assoc: Assoc::Left,
            }),
            TokenKind::Or => Some(OpInfo {
                prec: 0,
                assoc: Assoc::Left,
            }),
            _ => None,
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }
}
