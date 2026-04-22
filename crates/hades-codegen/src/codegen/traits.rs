use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;

pub trait Visit {
    type Output<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>>;
}

pub struct CodegenVisitor;

impl CodegenVisitor {
    pub fn new() -> Self {
        Self
    }

    pub fn visit_program<'ctx>(
        &self,
        program: &hades_ast::TypedProgram,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        for stmt in program.iter() {
            self.visit_stmt(stmt, context)?;
        }
        Ok(())
    }

    pub fn visit_stmt<'ctx>(
        &self,
        stmt: &hades_ast::TypedStmt,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        stmt.visit(context)
    }

}

impl Default for CodegenVisitor {
    fn default() -> Self {
        Self::new()
    }
}
