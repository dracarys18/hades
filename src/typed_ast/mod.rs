mod ast;
mod builtins;
mod expr;
mod function;
mod ident;
mod meta;
pub mod signatures;
mod struc;
mod value;

pub use ast::*;
pub use expr::{
    TypedArrayIndex, TypedAsExpression, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr,
    TypedExpr, TypedFieldAccess,
};
pub use function::{FunctionSignature, Params, TypedReceiver};
pub use meta::CompilerContext;
pub use signatures::ModuleSignatures;
pub use value::{TypedArrayLiteral, TypedValue};

use crate::module::ModulePath;

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

pub struct TypedModule {
    pub path: ModulePath,
    pub program: TypedProgram,
    pub signatures: ModuleSignatures,
    pub ctx: CompilerContext,
    pub imports: Vec<ModulePath>,
}
