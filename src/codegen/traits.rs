use crate::codegen::builtin::BuiltinRegistar;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenResult, CodegenValue};
use inkwell::values::FunctionValue;

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
        program: &crate::typed_ast::TypedProgram,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        self.visit_builtin_function(context)?;

        for stmt in program.iter() {
            self.visit_stmt(stmt, context)?;
        }
        Ok(())
    }

    pub fn visit_builtin_function<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        BuiltinRegistar::declare_all(context)?;
        Ok(())
    }

    pub fn visit_stmt<'ctx>(
        &self,
        stmt: &crate::typed_ast::TypedStmt,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        stmt.visit(context)
    }

    pub fn visit_expr<'ctx>(
        &self,
        expr: &crate::typed_ast::TypedExprAst,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>> {
        expr.expr().visit(context)
    }

    pub fn visit_block<'ctx>(
        &self,
        block: &crate::typed_ast::TypedBlock,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        context.enter_scope();
        let result = block.visit(context);
        context.exit_scope();
        result
    }

    pub fn visit_function<'ctx>(
        &self,
        function: &crate::typed_ast::TypedFuncDef,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        function.visit(context)
    }
}

impl Default for CodegenVisitor {
    fn default() -> Self {
        Self::new()
    }
}
