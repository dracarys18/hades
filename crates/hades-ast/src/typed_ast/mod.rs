mod ast;
mod builtins;
mod expr;
mod function;
mod ident;
mod meta;
mod struc;
mod value;

pub use ast::*;
pub use expr::{
    TypedArrayIndex, TypedAsExpression, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr,
    TypedExpr, TypedFieldAccess,
};
pub use function::{FuncKind, FunctionSignature, Functions, Params, TypedReceiver};
pub use meta::{CompilerContext, ModulePath};
pub use struc::{Field, Structs};
pub use value::{TypedArrayLiteral, TypedValue};

#[derive(Debug, Clone, PartialEq)]
pub struct TypedProgram(pub Vec<ast::TypedStmt>);

impl TypedProgram {
    pub fn new(stmts: Vec<TypedStmt>) -> Self {
        Self(stmts)
    }
}

impl std::ops::Deref for TypedProgram {
    type Target = Vec<TypedStmt>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> IntoIterator for &'a TypedProgram {
    type Item = &'a TypedStmt;
    type IntoIter = std::slice::Iter<'a, TypedStmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for TypedProgram {
    type Item = TypedStmt;
    type IntoIter = std::vec::IntoIter<TypedStmt>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
