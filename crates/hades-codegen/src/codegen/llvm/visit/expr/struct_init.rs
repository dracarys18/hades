use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::Types;
use inkwell::values::{BasicValueEnum, PointerValue};

/// Build a struct on the stack from a list of (field_value, field_type) pairs
/// in declaration order.
pub(crate) fn build_alloca_struct<'ctx>(
    context: &mut LLVMContext<'ctx>,
    struct_type: inkwell::types::StructType<'ctx>,
    values: &[(BasicValueEnum<'ctx>, Types)],
) -> CodegenResult<PointerValue<'ctx>> {
    let struct_ptr = context.create_alloca("struct_alloca", struct_type.into())?;
    let i32_type = context.context().i32_type();
    let zero = i32_type.const_zero();
    for (i, (field_val, field_ast_type)) in values.iter().enumerate() {
        let idx = i32_type.const_int(i as u64, false);
        let field_ptr = unsafe {
            context
                .builder()
                .build_in_bounds_gep(struct_type, struct_ptr, &[zero, idx], "field_ptr")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to GEP struct field: {e}"),
                })?
        };
        context.create_store(field_ptr, *field_val, field_ast_type)?;
    }
    Ok(struct_ptr)
}

/// Build a const struct value from already-evaluated field values.
pub(crate) fn build_const_struct<'ctx>(
    struct_type: inkwell::types::StructType<'ctx>,
    field_vals: &[BasicValueEnum<'ctx>],
) -> inkwell::values::StructValue<'ctx> {
    struct_type.const_named_struct(field_vals)
}
