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
    Binary(BinaryExpr),
    Unary {
        op: Op,
        expr: Box<Expr>,
    },
    Assign(AssignExpr),
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
pub struct AssignExpr {
    pub name: Ident,
    pub op: Op,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: Op,
    pub right: Box<Expr>,
}
