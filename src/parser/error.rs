use crate::error::Error;
use std::ops::Range;

#[derive(Debug)]
pub struct ParseError(Box<Error>);

#[derive(Debug)]
pub struct FinalParseError(Vec<ParseError>);

impl FinalParseError {
    pub fn new(errors: Vec<ParseError>) -> Self {
        Self(errors)
    }
    pub fn into_errors(self) -> Vec<ParseError> {
        self.0
    }
}

impl<'a> ParseError {
    pub fn unexpected_token(
        found: Option<crate::tokens::Token>,
        expected: &str,
        span: Range<usize>,
        source_id: String,
    ) -> Self {
        let message = match found {
            Some(token) => format!("Expected {expected}, but found {token:?}"),
            None => format!("Expected {expected}, but reached end of input"),
        };

        Self(Box::new(
            Error::new_with_span(message, span, source_id)
                .with_help(format!("Try adding a {expected}")),
        ))
    }

    pub fn unexpected_eof(expected: &str, span: Range<usize>, source_id: String) -> Self {
        Self(Box::new(
            Error::new_with_span(
                format!("Unexpected end of file, expected {expected}"),
                span,
                source_id,
            )
            .with_help("Try completing the expression or statement".to_string()),
        ))
    }

    pub fn invalid_assignment_target(span: Range<usize>, source_id: String) -> Self {
        Self(Box::new(
            Error::new_with_span("Invalid assignment target".to_string(), span, source_id)
                .with_help("Only variables can be assigned to".to_string())
                .with_note("Assignment targets must be identifiers".to_string()),
        ))
    }

    pub fn missing_semicolon(span: Range<usize>, source_id: String) -> Self {
        Self(Box::new(
            Error::new_with_span("Missing semicolon".to_string(), span, source_id)
                .with_help("Try adding a ';' at the end of the statement".to_string()),
        ))
    }
}

impl std::ops::Deref for ParseError {
    type Target = Error;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
pub type FinalParseResult<T> = Result<T, FinalParseError>;
