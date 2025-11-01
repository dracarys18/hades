use crate::ast::Types;
use crate::typed_ast::TypedExpr;

#[derive(Debug, Clone, PartialEq)]
pub enum TypedValue {
    Number(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(TypedArrayLiteral),
}

impl TypedValue {
    pub fn get_type(&self) -> Types {
        match self {
            Self::Number(_) => Types::Int,
            Self::Float(_) => Types::Float,
            Self::Boolean(_) => Types::Bool,
            Self::String(_) => Types::String,
            Self::Array(arr) => arr.elem_typ.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedArrayLiteral {
    pub elements: Vec<TypedExpr>,
    pub size: usize,
    pub elem_typ: Types,
}
