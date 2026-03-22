use crate::ast::Types;
use crate::tokens::{Ident, Op};
use crate::typed_ast::expr::{TypedArrayIndex, TypedExpr, TypedFieldAccess};

#[derive(Debug, Clone, PartialEq)]
pub enum TypedAssignTarget {
    FieldAccess(TypedFieldAccess),
    Ident(Ident),
    ArrayIndex(TypedArrayIndex),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedAssignExpr {
    pub target: TypedAssignTarget,
    pub op: Op,
    pub value: Box<TypedExpr>,
    pub typ: Types,
}
