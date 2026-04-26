use hades_ast::Types;
use hades_tokens::Op;

use super::operand::Operand;

/// The kind of aggregate being constructed.
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateKind {
    /// Struct literal: `MyStruct { field: val, ... }`
    Struct(String),
    /// Array literal: `[a, b, c]`
    Array(Types),
}

/// The right-hand side of a MIR assignment.
#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    /// `Use(op)` — copy a value or take a reference (`Operand::Copy` or `Operand::Ref`).
    Use(Operand),

    /// `BinaryOp(op, lhs, rhs)` — arithmetic / comparison / logical.
    BinaryOp(Op, Operand, Operand),

    /// `UnaryOp(op, operand)` — negation, logical not, pointer deref-load.
    UnaryOp(Op, Operand),

    /// `Cast(operand, target_type)` — `expr as T`.
    Cast(Operand, Types),

    /// `Aggregate(kind, fields)` — struct / array construction.
    Aggregate(AggregateKind, Vec<Operand>),
}
