use crate::ast::Types;

#[derive(Debug, Clone, PartialEq)]
pub enum TypedValue {
    Number(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl TypedValue {
    pub fn get_type(&self) -> Types {
        match self {
            Self::Number(_) => Types::Int,
            Self::Float(_) => Types::Float,
            Self::Boolean(_) => Types::Bool,
            Self::String(_) => Types::String,
        }
    }
}
