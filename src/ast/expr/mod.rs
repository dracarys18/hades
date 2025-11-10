mod assignment;

pub use assignment::*;

use indexmap::IndexMap;

use super::value::Value;
use crate::tokens::{Ident, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    Ident(Ident),
    StructInit {
        name: Ident,
        fields: IndexMap<Ident, Expr>,
    },
    ArrayIndex(ArrayIndexExpr),
    Binary(BinaryExpr),
    Unary {
        op: Op,
        expr: Box<Expr>,
    },
    Assign(AssignExpr),
    FieldAccess(FieldAccessExpr),
    Call {
        func: Ident,
        args: Vec<Expr>,
    },
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
