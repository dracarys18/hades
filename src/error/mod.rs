mod span;

pub use crate::semantic::error::SemanticError;
pub use span::*;

use ariadne::{Cache, Label, Report, ReportKind};
use std::{ops::Range, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
    pub span: Span,
    pub severity: ErrorSeverity,
    pub help: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Error,
    Warning,
}

impl Error {
    pub fn new_with_span(message: String, span: Span) -> Self {
        Self {
            message,
            span,
            severity: ErrorSeverity::Error,
            help: None,
            note: None,
        }
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn eprint(&self, cache: impl Cache<PathBuf>) {
        let report = self.to_report();
        report.eprint(cache).expect("Failed to print error report");
    }

    pub fn to_report(&self) -> Report<'static, Span> {
        let report_kind = match self.severity {
            ErrorSeverity::Error => ReportKind::Error,
            ErrorSeverity::Warning => ReportKind::Warning,
        };

        let span = self.span.clone();

        let mut report = Report::build(report_kind, span.clone())
            .with_message(&self.message)
            .with_label(Label::new(span).with_message(&self.message).with_color(
                match self.severity {
                    ErrorSeverity::Error => ariadne::Color::Red,
                    ErrorSeverity::Warning => ariadne::Color::Yellow,
                },
            ));

        if let Some(help) = &self.help {
            report = report.with_help(help);
        }

        if let Some(note) = &self.note {
            report = report.with_note(note);
        }

        report.finish()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError at {:?}: {}", self.span, self.message)
    }
}

impl std::error::Error for Error {}
