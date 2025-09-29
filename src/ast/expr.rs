use indexmap::IndexMap;

use crate::tokens::{Ident, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Float(f64),
    String(String),
    Ident(Ident),
    Boolean(bool),
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
        op: Option<Op>,
        value: Box<Expr>,
    },
    Call {
        func: Ident,
        args: Vec<Expr>,
    },
}
