mod ast;
mod builtins;
mod expr;
mod function;
mod ident;
mod meta;
mod struc;
mod value;

pub use ast::*;
pub use expr::{TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr, TypedExpr, TypedFieldAccess};
pub use function::{FunctionSignature, Params};
pub use meta::CompilerContext;
pub use value::TypedValue;

use crate::{
    ast::{Program, WalkAst},
    error::SemanticError,
};

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

#[derive(Debug, Clone, PartialEq)]
pub struct TypedAstMeta {
    ast: TypedProgram,
    ctx: CompilerContext,
}

impl TypedAstMeta {
    pub fn new() -> Self {
        let ast = TypedProgram::new(vec![]);
        let ctx = CompilerContext::new();

        Self { ast, ctx }
    }

    pub fn prepare(self, program: &Program) -> Result<Self, SemanticError> {
        let mut ctx = CompilerContext::new();
        let ast = program.walk(&mut ctx);
        match ast {
            Err(e) => Err(e),
            Ok(ast) => Ok(Self { ast, ctx }),
        }
    }

    pub fn ast(&self) -> &TypedProgram {
        &self.ast
    }
    pub fn ctx(&self) -> &CompilerContext {
        &self.ctx
    }
}
