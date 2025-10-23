use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;

impl Visit for crate::typed_ast::TypedLet {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let var_name = self.name.clone();

        let init_value = self.value.expr().visit(context)?;

        let symbols = context.symbols();
        let var_type = context.type_converter().to_llvm_type(&self.typ, symbols)?;

        let alloca = context.create_alloca(var_name.inner(), var_type)?;

        context.create_store(alloca, init_value.value)?;

        context.declare_variable(var_name, alloca, self.typ.clone())?;

        Ok(())
    }
}

impl Visit for crate::typed_ast::TypedExprAst {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        self.expr().visit(context)?;
        Ok(())
    }
}
