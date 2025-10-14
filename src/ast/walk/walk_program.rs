use crate::ast::{Program, WalkAst};
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedProgram};

impl WalkAst for Program {
    type Output = TypedProgram;

    fn walk(&self, ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        let mut typed_stmts = Vec::new();
        for stmt in self.iter() {
            typed_stmts.push(stmt.walk(ctx)?);
        }
        Ok(crate::typed_ast::TypedProgram::new(typed_stmts))
    }
}
