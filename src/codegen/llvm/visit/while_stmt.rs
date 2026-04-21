use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use crate::codegen::traits::Visit;
use crate::typed_ast::TypedWhile;

impl Visit for TypedWhile {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let loop_header = context.create_basic_block("while.header");
        let loop_body = context.create_basic_block("while.body");
        let loop_exit = context.create_basic_block("while.exit");

        context.build_unconditional_branch(loop_header)?;
        context.position_at_end(loop_header);

        let cond_val = self.cond.visit(context)?;
        let cond_int = cond_val.value()?.into_int_value();

        context.build_conditional_branch(cond_int.into(), loop_body, loop_exit)?;

        context.position_at_end(loop_body);
        context.push_loop(loop_header, loop_exit);
        self.body.visit(context)?;
        context.pop_loop();

        if !context.is_block_terminated() {
            context.build_unconditional_branch(loop_header)?;
        }

        context.position_at_end(loop_exit);
        Ok(())
    }
}
