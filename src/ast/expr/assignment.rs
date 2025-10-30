use crate::ast::Expr;
use crate::tokens::Ident;
use crate::tokens::Op;

use super::FieldAccessExpr;

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Ident(Ident),
    FieldAccess(FieldAccessExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr {
    pub target: AssignTarget,
    pub op: Op,
    pub value: Box<Expr>,
}
