use crate::ast::Expr;
use crate::tokens::Ident;
use crate::tokens::Op;

use super::{ArrayIndexExpr, FieldAccessExpr};

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Ident(Ident),
    FieldAccess(FieldAccessExpr),
    ArrayIndex(ArrayIndexExpr),
    /// Write-through deref: `*ptr = value`
    Deref(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr {
    pub target: AssignTarget,
    pub op: Op,
    pub value: Box<Expr>,
}
