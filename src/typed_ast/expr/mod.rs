mod assign;

use crate::{
    ast::Types,
    error::{SemanticError, Span},
};
pub use assign::*;
use indexmap::IndexMap;

use super::value::TypedValue;
use crate::tokens::{Ident, Name, Op};

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Value(TypedValue),
    Ident {
        ident: Ident,
        typ: Types,
    },
    StructInit {
        name: Name,
        fields: IndexMap<Ident, TypedExpr>,
        types: Types,
    },
    Binary(TypedBinaryExpr),
    Unary {
        op: Op,
        expr: Box<TypedExpr>,
        typ: Types,
    },
    FieldAccess(TypedFieldAccess),
    ArrayIndex(TypedArrayIndex),
    Assign(TypedAssignExpr),
    As(TypedAsExpression),
    Call {
        func: Name,
        args: Vec<TypedExpr>,
        receiver: Option<Box<TypedExpr>>,
        typ: Types,
    },
    /// Null pointer literal with the concrete pointer type inferred from context.
    Null(Types),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedFieldAccess {
    pub expr: Box<TypedExpr>,
    pub field: Ident,
    pub struct_type: Types,
    pub field_type: Types,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedBinaryExpr {
    pub left: Box<TypedExpr>,
    pub op: Op,
    pub right: Box<TypedExpr>,
    pub typ: Types,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedArrayIndex {
    pub expr: Box<TypedExpr>,
    pub index: Box<TypedExpr>,
    pub typ: Types,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedAsExpression {
    pub expr: Box<TypedExpr>,
    pub target_type: Types,
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
            TypedExpr::FieldAccess(TypedFieldAccess { field_type, .. }) => field_type.clone(),
            TypedExpr::ArrayIndex(TypedArrayIndex { typ, .. }) => typ.get_array_elem_type(),
            TypedExpr::As(TypedAsExpression { target_type, .. }) => target_type.clone(),
            TypedExpr::Null(typ) => typ.clone(),
        }
    }

    pub fn get_deref_type(&self, span: Span) -> Result<Types, SemanticError> {
        let typ = self.get_type();
        if let Types::Pointer(inner) = typ {
            Ok(*inner)
        } else {
            Err(SemanticError::invalid_dereference(typ.to_string(), span))
        }
    }
}
