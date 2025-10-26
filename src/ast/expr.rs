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
    Binary {
        left: Box<Expr>,
        op: Op,
        right: Box<Expr>,
    },
    Unary {
        op: Op,
        expr: Box<Expr>,
    },
    Assign {
        name: Ident,
        op: Op,
        value: Box<Expr>,
    },
    Call {
        func: Ident,
        args: Vec<Expr>,
    },
}
