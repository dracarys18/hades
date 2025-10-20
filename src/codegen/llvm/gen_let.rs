use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::{ExprCodegen, StmtCodegen};
use crate::typed_ast::TypedLet;

impl StmtCodegen for TypedLet {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        let var_name = self.name.clone();

        let init_value = self.value.expr().generate_expr(context)?;

        let symbols = context.symbols();
        let var_type = context.type_converter().to_llvm_type(&self.typ, symbols)?;

        let alloca = context.create_alloca(var_name.inner(), var_type)?;

        context.create_store(alloca, init_value.value)?;

        context.declare_variable(var_name, alloca, self.typ.clone())?;

        Ok(())
    }
}

impl StmtCodegen for crate::typed_ast::TypedExprAst {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        self.expr().generate_expr(context)?;
        Ok(())
    }
}
