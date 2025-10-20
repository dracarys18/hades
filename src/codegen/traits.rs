use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenResult, CodegenValue};
use inkwell::types::BasicTypeEnum;
use inkwell::values::FunctionValue;

pub trait ExprCodegen {
    fn generate_expr<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>>;
}

pub trait StmtCodegen {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()>;
}

pub trait TypeCodegen {
    fn generate_type<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<BasicTypeEnum<'ctx>>;
}

pub trait FunctionCodegen {
    fn generate_function<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<FunctionValue<'ctx>>;
}

pub trait BlockCodegen {
    fn generate_block<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()>;
}

pub trait ValueCodegen {
    fn generate_value<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>>;
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
        for stmt in program.iter() {
            self.visit_stmt(stmt, context)?;
        }
        Ok(())
    }

    pub fn visit_stmt<'ctx>(
        &self,
        stmt: &crate::typed_ast::TypedStmt,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        stmt.generate_stmt(context)
    }

    pub fn visit_expr<'ctx>(
        &self,
        expr: &crate::typed_ast::TypedExprAst,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>> {
        expr.expr().generate_expr(context)
    }

    pub fn visit_block<'ctx>(
        &self,
        block: &crate::typed_ast::TypedBlock,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<()> {
        context.enter_scope();
        let result = block.generate_block(context);
        context.exit_scope();
        result
    }

    pub fn visit_function<'ctx>(
        &self,
        function: &crate::typed_ast::TypedFuncDef,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<FunctionValue<'ctx>> {
        function.generate_function(context)
    }
}

impl Default for CodegenVisitor {
    fn default() -> Self {
        Self::new()
    }
}
