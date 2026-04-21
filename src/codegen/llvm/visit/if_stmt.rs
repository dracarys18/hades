use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedIf;

impl Visit for TypedIf {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let cond_val = self.cond.expr().visit(context)?;
        let cond_int = cond_val.value()?.into_int_value();

        let then_block = context.create_basic_block("if.then");
        let else_block = context.create_basic_block("if.else");
        let merge_block = context.create_basic_block("if.merge");

        let final_else_block = if self.else_branch.is_some() {
            else_block
        } else {
            merge_block
        };

        context.build_conditional_branch(cond_int.into(), then_block, final_else_block)?;

        context.position_at_end(then_block);
        self.then_branch.visit(context)?;
        if !context.is_block_terminated() {
            context.build_unconditional_branch(merge_block)?;
        }

        if let Some(else_branch) = &self.else_branch {
            context.position_at_end(else_block);
            else_branch.visit(context)?;
            if !context.is_block_terminated() {
                context.build_unconditional_branch(merge_block)?;
            }
        }

        context.position_at_end(merge_block);
        Ok(())
    }
}
