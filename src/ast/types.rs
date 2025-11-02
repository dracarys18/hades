use crate::tokens::Ident;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayType {
    IntArray(usize),
    FloatArray(usize),
    StringArray(usize),
    BoolArray(usize),
    StructArray(usize, Ident),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Types {
    Int,
    Float,
    Bool,
    String,
    Void,
    Generic(Vec<Types>),
    Array(ArrayType),
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
            Types::Array(arr_type) => match arr_type {
                ArrayType::IntArray(size) => write!(f, "int[{size}]"),
                ArrayType::FloatArray(size) => write!(f, "float[{size}]"),
                ArrayType::StringArray(size) => write!(f, "string[{size}]"),
                ArrayType::BoolArray(size) => write!(f, "bool[{size}]"),
                ArrayType::StructArray(size, name) => write!(f, "struct {name}[{size}]"),
            },
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
            "[]int" => Types::Array(ArrayType::IntArray(0)),
            "[]float" => Types::Array(ArrayType::FloatArray(0)),
            "[]string" => Types::Array(ArrayType::FloatArray(0)),
            _ => Types::Struct(type_str.to_owned()),
        }
    }

    pub fn array_type(&self, size: usize) -> ArrayType {
        match self {
            Self::Int => ArrayType::IntArray(size),
            Self::Float => ArrayType::FloatArray(size),
            Self::String => ArrayType::StringArray(size),
            Self::Bool => ArrayType::BoolArray(size),
            Self::Struct(name) => ArrayType::StructArray(size, name.to_owned()),
            _ => unimplemented!("Array type for {:?} is not implemented yet", self),
        }
    }

    pub fn get_array_size(&self) -> usize {
        if let Types::Array(arr_type) = self {
            match arr_type {
                ArrayType::IntArray(size) => *size,
                ArrayType::FloatArray(size) => *size,
                ArrayType::StringArray(size) => *size,
                ArrayType::BoolArray(size) => *size,
                ArrayType::StructArray(size, _) => *size,
            }
        } else {
            panic!("Expected an Array type")
        }
    }

    pub fn get_array_elem_type(&self) -> Types {
        if let Types::Array(arr_type) = self {
            match arr_type {
                ArrayType::IntArray(_) => Types::Int,
                ArrayType::FloatArray(_) => Types::Float,
                ArrayType::StringArray(_) => Types::String,
                ArrayType::BoolArray(_) => Types::Bool,
                ArrayType::StructArray(_, name) => Types::Struct(name.to_owned()),
            }
        } else {
            panic!("Expected an Array type")
        }
    }

    pub fn unwrap_struct_name(&self) -> &Ident {
        if let Types::Struct(name) = self {
            name
        } else {
            panic!("Expected a Struct type")
        }
    }
}
