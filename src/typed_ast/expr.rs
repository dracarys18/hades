use crate::ast::Types;
use indexmap::IndexMap;

use super::value::TypedValue;
use crate::tokens::{Ident, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Value(TypedValue),
    Ident {
        ident: Ident,
        typ: Types,
    },
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
        op: Op,
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
            TypedExpr::Value(val) => val.get_type(),
            TypedExpr::Ident { typ, .. } => typ.clone(),
            TypedExpr::StructInit { types, .. } => types.clone(),
            TypedExpr::Binary { typ, .. } => typ.clone(),
            TypedExpr::Unary { typ, .. } => typ.clone(),
            TypedExpr::Assign { typ, .. } => typ.clone(),
            TypedExpr::Call { typ, .. } => typ.clone(),
        }
    }
}
