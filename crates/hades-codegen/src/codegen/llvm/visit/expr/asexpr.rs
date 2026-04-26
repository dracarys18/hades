use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use hades_ast::Types;

/// Cast operand value from `from_ty` to `to_ty`, producing a new `BasicValueEnum`.
pub(crate) fn cast_value<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    from_ty: &Types,
    to_ty: &Types,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let target_llvm = context
        .type_converter()
        .to_llvm_type(to_ty, context.module())?;

    match (from_ty, to_ty) {
        (Types::Int, Types::Float) => cast_int_to_float(context, value, target_llvm),
        (Types::Char, Types::Int) => cast_char_to_int(context, value, target_llvm),
        (Types::Char, Types::Float) => cast_char_to_float(context, value, target_llvm),
        (Types::Float, Types::Int) => cast_float_to_int(context, value, target_llvm),
        _ => Err(CodegenError::LLVMBuild {
            message: format!("Unsupported cast from {:?} to {:?}", from_ty, to_ty),
        }),
    }
}

fn cast_int_to_float<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    context
        .builder()
        .build_signed_int_to_float(
            value.value()?.into_int_value(),
            target_type.into_float_type(),
            "int_to_float_cast",
        )
        .map(|v| v.into())
        .map_err(|e| CodegenError::LLVMBuild { message: format!("int_to_float failed: {:?}", e) })
}

fn cast_char_to_int<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    context
        .builder()
        .build_int_cast(
            value.value()?.into_int_value(),
            target_type.into_int_type(),
            "char_to_int_cast",
        )
        .map(|v| v.into())
        .map_err(|e| CodegenError::LLVMBuild { message: format!("char_to_int failed: {:?}", e) })
}

fn cast_float_to_int<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    context
        .builder()
        .build_float_to_signed_int(
            value.value()?.into_float_value(),
            target_type.into_int_type(),
            "float_to_int_cast",
        )
        .map(|v| v.into())
        .map_err(|e| CodegenError::LLVMBuild { message: format!("float_to_int failed: {:?}", e) })
}

fn cast_char_to_float<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    context
        .builder()
        .build_unsigned_int_to_float(
            value.value()?.into_int_value(),
            target_type.into_float_type(),
            "char_to_float_cast",
        )
        .map(|v| v.into())
        .map_err(|e| CodegenError::LLVMBuild { message: format!("char_to_float failed: {:?}", e) })
}
