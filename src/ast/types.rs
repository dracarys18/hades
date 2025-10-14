use indexmap::IndexMap;

use crate::tokens::Ident;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Types {
    Int,
    Float,
    Bool,
    String,
    Void,
    Custom(String),
    Any,
    Struct {
        name: Ident,
        fields: IndexMap<Ident, Types>,
    },
}

impl std::fmt::Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Types::Int => write!(f, "int"),
            Types::Float => write!(f, "float"),
            Types::Bool => write!(f, "bool"),
            Types::String => write!(f, "string"),
            Types::Void => write!(f, "void"),
            Types::Custom(name) => write!(f, "{}", name),
            Types::Any => write!(f, "any"),
            Types::Struct { name, .. } => write!(f, "struct {}", name),
        }
    }
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
