use indexmap::IndexMap;

use crate::tokens::Ident;

pub struct TypeId(pub u64);

pub struct TypeTable {
    inner: IndexMap<TypeId, Types>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Types {
    Int,
    Float,
    Bool,
    String,
    Void,
    Custom(String),
    Struct {
        name: Ident,
        fields: IndexMap<Ident, Types>,
    },
}

impl Types {
    pub fn from_str(type_str: &Ident) -> Self {
        let type_str = type_str.inner();
        match type_str {
            "int" => Types::Int,
            "float" => Types::Float,
            "bool" => Types::Bool,
            "string" => Types::String,
            "void" => Types::Void,
            other => Types::Custom(other.to_string()),
        }
    }
}
