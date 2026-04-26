use hades_ast::Types;

use super::place::Place;

/// A compile-time constant value.
#[derive(Debug, Clone, PartialEq)]
pub enum MirConst {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    /// Null pointer of the given pointer type.
    Null(Types),
}

impl MirConst {
    pub fn ty(&self) -> Types {
        match self {
            MirConst::Int(_) => Types::Int,
            MirConst::Float(_) => Types::Float,
            MirConst::Bool(_) => Types::Bool,
            MirConst::String(_) => Types::String,
            MirConst::Char(_) => Types::Char,
            MirConst::Null(ty) => ty.clone(),
        }
    }
}

/// An operand consumed by an `Rvalue` or terminator.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Read the value at a place (all scalar/struct loads in Hades).
    Copy(Place),
    /// Take a pointer to a place: `&x` — produces `Types::Pointer(inner)`.
    Ref(Place),
    /// An inline compile-time constant.
    Const(MirConst),
}

impl Operand {
    pub fn ty(&self, locals: &[super::local::LocalDecl]) -> Types {
        match self {
            Operand::Copy(p) => locals[p.local.index()].ty.clone(),
            Operand::Ref(p) => Types::Pointer(Box::new(locals[p.local.index()].ty.clone())),
            Operand::Const(c) => c.ty(),
        }
    }
}
