use hades_ast::Types;
use hades_tokens::{Name, Op};

use super::operand::Operand;

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateKind {
    Struct(Name),
    Array(Types),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp(Op, Operand, Operand),
    UnaryOp(Op, Operand),
    Cast(Operand, Types),
    Aggregate(AggregateKind, Vec<Operand>),
    Repeat(Operand, usize),
}
