use hades_error::Span;

use super::{place::Place, rvalue::Rvalue};

/// The kind of a MIR statement.
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// `place = rvalue` — the only real statement in MIR.
    Assign(Place, Rvalue),

    /// A no-op produced when a statement has no observable effect.
    /// Used as a placeholder so block statement indices remain stable.
    Nop,
}

/// A single MIR statement with its source span.
#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

impl Statement {
    pub fn assign(place: Place, rvalue: Rvalue, span: Span) -> Self {
        Self {
            kind: StatementKind::Assign(place, rvalue),
            span,
        }
    }

    pub fn nop(span: Span) -> Self {
        Self {
            kind: StatementKind::Nop,
            span,
        }
    }
}
