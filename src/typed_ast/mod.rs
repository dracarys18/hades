mod ast;
mod expr;
mod function;
mod ident;
mod meta;

pub use ast::{TypedBlock, TypedStmt};
pub use expr::TypedExpr;
pub use meta::TypeContext;

use crate::{
    ast::{Program, ToTyped},
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
    ctx: TypeContext,
}

impl TypedAstMeta {
    pub fn new() -> Self {
        let ast = TypedProgram::new(vec![]);
        let ctx = TypeContext::new();

        Self { ast, ctx }
    }

    pub fn prepare(self, program: &Program) -> Result<Self, SemanticError> {
        let mut ctx = TypeContext::new();
        let ast = program.to_typed(&mut ctx);
        match ast {
            Err(e) => {
                print!("{ctx:?}");
                Err(e)
            }
            Ok(ast) => Ok(Self { ast, ctx }),
        }
    }

    pub fn ast(&self) -> &TypedProgram {
        &self.ast
    }
    pub fn ctx(&self) -> &TypeContext {
        &self.ctx
    }
}
