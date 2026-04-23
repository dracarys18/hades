use super::{CodegenValue, CompileTimeBuiltin};
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::TypedExpr;
use hades_ast::Types;

pub struct Len;

impl CompileTimeBuiltin for Len {
    fn call<'ctx>(
        args: &[TypedExpr],
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>> {
        let arg = args.first().ok_or(CodegenError::LLVMBuild {
            message: "len requires exactly one array argument".to_string(),
        })?;

        let size = arg.get_type().get_array_size();
        let val = context
            .context()
            .i64_type()
            .const_int(size as u64, false)
            .into();

        Ok(CodegenValue::new(val, Types::Int))
    }
}
