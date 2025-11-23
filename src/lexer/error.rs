use crate::{error::Error, lexer::simd::bytes::Byte};
use std::ops::Range;

#[derive(Debug)]
pub struct LexError(Box<Error>);

impl LexError {
    pub fn unable_to_lex(message: String, span: Range<usize>, source_id: String) -> Self {
        Self(Box::new(Error::new_with_span(message, span, source_id)))
    }

    pub fn invalid_number(
        number_str: &str,
        error: &str,
        span: Range<usize>,
        source_id: String,
    ) -> Self {
        Self(Box::new(
            Error::new_with_span(
                format!("Invalid number '{number_str}': {error}"),
                span,
                source_id,
            )
            .with_help("Check the number format".to_string()),
        ))
    }

    pub fn unterminated_string(span: Range<usize>, source_id: String) -> Self {
        Self(Box::new(
            Error::new_with_span("Unterminated string literal".to_string(), span, source_id)
                .with_help("Add a closing quote '\"' to complete the string".to_string()),
        ))
    }

    pub fn unexpected_character(ch: Byte, span: Range<usize>, source_id: String) -> Self {
        Self(Box::new(
            Error::new_with_span(format!("Unexpected character '{ch}'"), span, source_id)
                .with_help("Remove or replace the invalid character".to_string()),
        ))
    }

    pub fn invalid_escape_sequence(sequence: &str, span: Range<usize>, source_id: String) -> Self {
        Self(Box::new(
            Error::new_with_span(
                format!("Invalid escape sequence '{sequence}'"),
                span,
                source_id,
            )
            .with_help("Use valid escape sequences like \\n, \\t, \\\\, or \\\"".to_string()),
        ))
    }
}

impl std::ops::Deref for LexError {
    type Target = Error;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LexError: {}", self.0.message)
    }
}

impl std::error::Error for LexError {}

pub type LexResult<T> = Result<T, LexError>;
