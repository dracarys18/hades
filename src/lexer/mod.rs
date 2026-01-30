mod error;
mod simd;

use crate::error::Span;
use crate::tok;
use crate::tokens::{Ident, Token, TokenKind};
pub use error::{LexError, LexResult};
use simd::bytes::{Byte, ByteSlice, Bytes};

use memchr::memchr3;
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
    "module"=> TokenKind::Module,
};

pub struct Lexer {
    input: Bytes,
    source_id: String,
    tokens: Vec<Token>,
    pos: usize,
}

impl Lexer {
    pub fn new(source: &[u8], source_id: String) -> Self {
        Self {
            input: Bytes::new(source),
            source_id,
            tokens: Vec::new(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<Byte> {
        self.input.into_iter().nth(self.pos)
    }

    fn peek_and_check(&self, expected: u8) -> bool {
        self.peek().map(|c| c.eq(&expected)).unwrap_or_default()
    }

    fn next(&mut self) -> Option<Byte> {
        if self.peek_and_check(b'\n') {
            self.push_token(tok!(TokenKind::Newline, self.pos, self.pos + 1));
        }
        self.pos += 1;
        self.peek()
    }

    fn move_next(&mut self) {
        self.pos += 1;
    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn consume_while<F>(&mut self, mut f: F) -> &ByteSlice
    where
        F: FnMut(Byte) -> bool,
    {
        let (new_pos, slice) = self.input.consume_while(self.pos, f);
        self.pos = new_pos;
        slice
    }

    fn parse_plus_equal(&mut self) {
        let start_pos = self.pos;

        if self.peek_and_check(b'+') {
            self.next();
            if self.peek_and_check(b'=') {
                self.next();
                self.push_token(tok!(TokenKind::PlusEqual, start_pos, self.pos));
            } else {
                self.push_token(tok!(TokenKind::Plus, start_pos, self.pos));
            }
        }
    }

    fn parse_minus_equal(&mut self) {
        let start_pos = self.pos;
        if self.peek_and_check(b'-') {
            self.next();
            if self.peek_and_check(b'=') {
                self.next();
                self.push_token(tok!(TokenKind::MinusEqual, start_pos, self.pos));
            } else {
                self.push_token(tok!(TokenKind::Minus, start_pos, self.pos));
            }
        }
    }

    fn parse_string(&mut self) -> LexResult<()> {
        if self.peek_and_check(b'"') {
            let start_pos = self.pos;
            self.move_next();

            let mut s = String::with_capacity(32);

            loop {
                match self.peek() {
                    Some(ch) if ch.eq(&b'"') => {
                        self.move_next();
                        break;
                    }
                    Some(ch) if ch.eq(&b'\\') => {
                        self.move_next();
                        match self.peek() {
                            Some(ch) if ch.eq(&b'n') => {
                                s.push('\n');
                                self.move_next();
                            }
                            Some(ch) if ch.eq(&b't') => {
                                s.push('\t');
                                self.move_next();
                            }
                            Some(ch) if ch.eq(&b'"') => {
                                s.push('"');
                                self.move_next();
                            }
                            Some(ch) if ch.eq(&b'\\') => {
                                s.push('\\');
                                self.move_next();
                            }
                            Some(ch) => {
                                s.push(ch.as_char());
                                self.move_next();
                            }
                            None => {
                                let span = start_pos..self.pos;
                                return Err(LexError::unterminated_string(
                                    span,
                                    self.source_id.clone(),
                                ));
                            }
                        }
                    }
                    Some(c) => {
                        s.push(c.as_char());
                        self.move_next();
                    }
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
            if c.is_alphanumeric() || c.eq(&b'_') {
                let ident = self.consume_while(|ch| ch.is_alphanumeric() || ch.eq(&b'_'));
                let ident_str = ident.to_string();
                if let Some(keyword_token) = KEYWORDS.get(&ident_str) {
                    self.push_token(tok!(keyword_token.clone(), start_pos, self.pos));
                } else {
                    let span = Span::new(start_pos, self.pos);
                    self.push_token(tok!(
                        TokenKind::Ident(Ident::new(ident_str, span)),
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
            if c.is_digit() || c.eq(&b'.') {
                self.next();
            } else {
                break;
            }
        }

        let num_slice = &self.input[start_pos..self.pos];
        let num_str = num_slice.to_string();
        let span = start_pos..self.pos;

        if num_slice.contains_byte(b'.') {
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
                ch if ch.eq(&b'+') => self.parse_plus_equal(),
                ch if ch.eq(&b'-') => self.parse_minus_equal(),
                ch if ch.eq(&b'*') => {
                    self.next();
                    self.push_token(tok!(TokenKind::Multiply, start_pos, self.pos));
                }
                ch if ch.eq(&b'/') => {
                    self.next();
                    self.push_token(tok!(TokenKind::Divide, start_pos, self.pos));
                }
                ch if ch.eq(&b'=') => {
                    self.next();
                    if self.peek_and_check(b'=') {
                        self.next();

                        self.push_token(tok!(TokenKind::EqualEqual, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Assign, start_pos, self.pos));
                    }
                }
                ch if ch.eq(&b'!') => {
                    self.move_next();
                    if self.peek_and_check(b'=') {
                        self.next();
                        self.push_token(tok!(TokenKind::BangEqual, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Bang, start_pos, self.pos));
                    }
                }
                ch if ch.eq(&b'|') => {
                    self.move_next();
                    if self.peek_and_check(b'|') {
                        self.next();
                        self.push_token(tok!(TokenKind::Or, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::BooleanOr, start_pos, self.pos));
                    }
                }
                ch if ch.eq(&b'&') => {
                    self.move_next();
                    if self.peek_and_check(b'&') {
                        self.next();
                        self.push_token(tok!(TokenKind::And, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::BoleanAnd, start_pos, self.pos));
                    }
                }
                ch if ch.eq(&b'>') => {
                    self.move_next();
                    if self.peek_and_check(b'=') {
                        self.next();
                        self.push_token(tok!(TokenKind::GreaterEqual, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Greater, start_pos, self.pos));
                    }
                }
                ch if ch.eq(&b'<') => {
                    self.move_next();
                    if self.peek_and_check(b'=') {
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
                c if c.eq(&b'(') => {
                    self.next();
                    self.push_token(tok!(TokenKind::LeftParen, start_pos, self.pos));
                }
                c if c.eq(&b')') => {
                    self.next();
                    self.push_token(tok!(TokenKind::RightParen, start_pos, self.pos));
                }
                c if c.eq(&b'{') => {
                    self.next();
                    self.push_token(tok!(TokenKind::LeftBrace, start_pos, self.pos));
                }
                c if c.eq(&b'}') => {
                    self.next();
                    self.push_token(tok!(TokenKind::RightBrace, start_pos, self.pos));
                }
                c if c.eq(&b'[') => {
                    self.next();
                    self.push_token(tok!(TokenKind::LeftBracket, start_pos, self.pos));
                }
                c if c.eq(&b']') => {
                    self.next();
                    self.push_token(tok!(TokenKind::RightBracket, start_pos, self.pos));
                }
                c if c.eq(&b',') => {
                    self.next();
                    self.push_token(tok!(TokenKind::Comma, start_pos, self.pos));
                }
                c if c.eq(&b'.') => {
                    self.next();
                    if self.peek_and_check(b'.') {
                        self.next();
                        self.push_token(tok!(TokenKind::Range, start_pos, self.pos));
                    } else {
                        self.push_token(tok!(TokenKind::Dot, start_pos, self.pos));
                    }
                }
                c if c.eq(&b';') => {
                    self.next();
                    self.push_token(tok!(TokenKind::Semicolon, start_pos, self.pos));
                }
                c if c.eq(&b':') => {
                    self.next();
                    self.push_token(tok!(TokenKind::Colon, start_pos, self.pos));
                }
                _ => {}
            }
        }
    }

    pub fn tokenize(&mut self) -> LexResult<()> {
        while let Some(c) = self.peek() {
            match c {
                ch if ch.is_digit() => self.parse_number()?,
                ch if (ch.is_alphabetic() || ch.eq(&b'_')) => self.parse_keyword_or_identifier(),
                ch if ch.is_operator() => self.parse_operator(),
                ch if ch.is_punctuation() => self.parse_punctuation(),
                ch if ch.is_whitespace() => self.move_next(),
                ch if ch.is_string_start() => self.parse_string()?,
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

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }
}
