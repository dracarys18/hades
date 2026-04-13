mod assignment;

pub use assignment::*;

use indexmap::IndexMap;

use super::value::Value;
use crate::ast::Types;
use crate::tokens::{Ident, Name, Op};

/// A `null` literal with the pointer type expected by the surrounding context.
/// `expected` is `None` when `null` appears with no type context (an error).
#[derive(Debug, Clone, PartialEq)]
pub struct NullExpr {
    pub expected: Option<Types>,
}

impl NullExpr {
    pub fn new(expected: Option<Types>) -> Self {
        Self { expected }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub func: Name,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCall {
    pub receiver: Box<Expr>,
    pub func: Name,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedCall {
    pub path: Vec<Ident>,
    pub func: Name,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallKind {
    Function(FunctionCall),
    Method(MethodCall),
    Qualified(QualifiedCall),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsExpression {
    pub expr: Box<Expr>,
    pub target_type: Types,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInitExpr {
    pub path: Vec<Ident>,
    pub fields: IndexMap<Ident, Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    Ident(Ident),
    StructInit(StructInitExpr),
    ArrayIndex(ArrayIndexExpr),
    Binary(BinaryExpr),
    Unary { op: Op, expr: Box<Expr> },
    Assign(AssignExpr),
    As(AsExpression),
    FieldAccess(FieldAccessExpr),
    Call(CallKind),
    Null,
}

impl Expr {
    pub fn unwrap_binary(&self) -> BinaryExpr {
        if let Expr::Binary(bin_expr) = self {
            bin_expr.clone()
        } else {
            panic!("Expected a BinaryExpr")
        }
    }

    pub fn unwrap_assign(&self) -> AssignExpr {
        if let Expr::Assign(assign_expr) = self {
            assign_expr.clone()
        } else {
            panic!("Expected an AssignExpr")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccessExpr {
    pub expr: Box<Expr>,
    pub field: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: Op,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayIndexExpr {
    pub expr: Box<Expr>,
    pub index: Box<Expr>,
}
