use crate::ast::Types;
use indexmap::IndexMap;

use crate::tokens::{Ident, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Number(i64),
    Float(f64),
    String(String),
    Ident {
        ident: Ident,
        typ: Types,
    },
    Boolean(bool),
    StructInit {
        name: Ident,
        fields: IndexMap<Ident, TypedExpr>,
        types: Types,
    },
    Binary {
        left: Box<TypedExpr>,
        op: Op,
        right: Box<TypedExpr>,
        typ: Types,
    },
    Unary {
        op: Op,
        expr: Box<TypedExpr>,
        typ: Types,
    },
    Assign {
        name: Ident,
        op: Option<Op>,
        value: Box<TypedExpr>,
        typ: Types,
    },
    Call {
        func: Ident,
        args: Vec<TypedExpr>,
        typ: Types,
    },
}

impl TypedExpr {
    pub fn get_type(&self) -> Types {
        match self {
            TypedExpr::Number(_) => Types::Int,
            TypedExpr::Float(_) => Types::Float,
            TypedExpr::String(_) => Types::String,
            TypedExpr::Boolean(_) => Types::Bool,
            TypedExpr::Ident { typ, .. } => typ.clone(),
            TypedExpr::StructInit { types, .. } => types.clone(),
            TypedExpr::Binary { typ, .. } => typ.clone(),
            TypedExpr::Unary { typ, .. } => typ.clone(),
            TypedExpr::Assign { typ, .. } => typ.clone(),
            TypedExpr::Call { typ, .. } => typ.clone(),
        }
    }
}
