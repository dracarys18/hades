use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::tokens::Ident;
use crate::typed_ast::TypedExpr;

pub struct Assignment<'a> {
    pub name: &'a Ident,
    pub value: &'a TypedExpr,
}

impl<'a> Assignment<'a> {
    pub fn new(name: &'a Ident, value: &'a TypedExpr) -> Self {
        Self { name, value }
    }
}

impl<'a> Visit for Assignment<'a> {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let value_val = self.value.visit(context)?;
        let var_ptr = context.get_variable(self.name)?;
        context
            .builder()
            .build_store(var_ptr.value(), value_val.value)
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to build store for assignment: {e:?}"),
            })?;

        Ok(CodegenValue {
            value: value_val.value,
            type_info: value_val.type_info,
        })
    }
}
