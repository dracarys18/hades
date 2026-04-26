use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use hades_ast::Types;
use hades_tokens::Op;
use inkwell::values::BasicValueEnum;

/// Arithmetic negation / logical not helpers (no Ref/Deref — those are handled
/// directly in expr/mod.rs as Operand::Ref and PlaceElem::Deref).
pub(crate) fn dispatch_unary_op<'ctx>(
    op: &Op,
    operand_val: BasicValueEnum<'ctx>,
    operand_ty: &Types,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let builder = context.builder();
    match (operand_ty, op) {
        (Types::Int, Op::Minus | Op::Sub) => {
            let zero = context.context().i64_type().const_zero();
            builder
                .build_int_sub(zero, operand_val.into_int_value(), "neg")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LLVMBuild { message: format!("Int neg failed: {:?}", e) })
        }
        (Types::Float, Op::Minus | Op::Sub) => builder
            .build_float_neg(operand_val.into_float_value(), "fneg")
            .map(|v| v.into())
            .map_err(|e| CodegenError::LLVMBuild { message: format!("Float neg failed: {:?}", e) }),
        (Types::Bool, Op::Not) => {
            let true_val = context.context().bool_type().const_all_ones();
            builder
                .build_xor(operand_val.into_int_value(), true_val, "not")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LLVMBuild { message: format!("Bool not failed: {:?}", e) })
        }
        (Types::Int, Op::BitNot) => {
            let all_ones = operand_val.into_int_value().get_type().const_all_ones();
            builder
                .build_xor(operand_val.into_int_value(), all_ones, "bitnot")
                .map(|v| v.into())
                .map_err(|e| CodegenError::LLVMBuild { message: format!("Int bitnot failed: {:?}", e) })
        }
        _ => Err(CodegenError::LLVMBuild {
            message: format!("Unsupported unary operation: {:?} {:?}", op, operand_ty),
        }),
    }
}

/// Deref: load the value a pointer points to.
pub(crate) fn deref_pointer<'ctx>(
    ptr_val: BasicValueEnum<'ctx>,
    pointee_type: &Types,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_type = context
        .type_converter()
        .to_llvm_type(pointee_type, context.module())?;
    let loaded = context.load(ptr_val.into_pointer_value(), llvm_type, "deref")?;
    Ok(CodegenValue::new(loaded, pointee_type.clone()))
}
