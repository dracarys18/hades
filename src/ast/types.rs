use crate::tokens::Ident;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Types {
    Int,
    Float,
    Bool,
    String,
    Void,
    Generic(Vec<Types>),
    Struct(Ident),
}

impl std::fmt::Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Types::Int => write!(f, "int"),
            Types::Float => write!(f, "float"),
            Types::Bool => write!(f, "bool"),
            Types::String => write!(f, "string"),
            Types::Void => write!(f, "void"),
            Types::Generic(t) => write!(f, "{t:?}"),
            Types::Struct(name) => write!(f, "struct {name}"),
        }
    }
}

impl Types {
    pub fn from_str(type_str: &Ident) -> Self {
        match type_str.inner() {
            "int" => Types::Int,
            "float" => Types::Float,
            "bool" => Types::Bool,
            "string" => Types::String,
            "void" => Types::Void,
            _ => Types::Struct(type_str.to_owned()),
        }
    }
}
