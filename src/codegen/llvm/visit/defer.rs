use crate::codegen::{LLVMContext, error::CodegenResult, traits::Visit};
use crate::consts::GOOLAG_MESSAGE;
use crate::typed_ast::TypedDefer;

impl Visit for TypedDefer {
    type Output<'ctx> = ();
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let current_function = context.current_function().expect(GOOLAG_MESSAGE);
        let last_block = current_function
            .get_last_basic_block()
            .expect(GOOLAG_MESSAGE);

        self.stmt.visit(context)?;
        Ok(())
    }
}
