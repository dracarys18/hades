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
    Binary(TypedBinaryExpr),
    Unary {
        op: Op,
        expr: Box<TypedExpr>,
        typ: Types,
    },
    Assign(TypedAssignExpr),
    Call {
        func: Ident,
        args: Vec<TypedExpr>,
        typ: Types,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedAssignExpr {
    pub name: Ident,
    pub op: Op,
    pub value: Box<TypedExpr>,
    pub typ: Types,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedBinaryExpr {
    pub left: Box<TypedExpr>,
    pub op: Op,
    pub right: Box<TypedExpr>,
    pub typ: Types,
}

impl TypedExpr {
    pub fn get_type(&self) -> Types {
        match self {
            TypedExpr::Value(val) => val.get_type(),
            TypedExpr::Ident { typ, .. } => typ.clone(),
            TypedExpr::StructInit { types, .. } => types.clone(),
            TypedExpr::Binary(TypedBinaryExpr { typ, .. }) => typ.clone(),
            TypedExpr::Unary { typ, .. } => typ.clone(),
            TypedExpr::Assign(TypedAssignExpr { typ, .. }) => typ.clone(),
            TypedExpr::Call { typ, .. } => typ.clone(),
        }
    }
}
