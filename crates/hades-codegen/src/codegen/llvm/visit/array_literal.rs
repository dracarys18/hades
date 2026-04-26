use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::Types;
use inkwell::module::Linkage;
use inkwell::types::{ArrayType, BasicTypeEnum, StructType};
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};

pub(crate) fn build_array<'ctx>(
    context: &mut LLVMContext<'ctx>,
    elems: Vec<BasicValueEnum<'ctx>>,
    elem_ty: &Types,
    llvm_array_type: ArrayType<'ctx>,
    llvm_elem_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<PointerValue<'ctx>> {
    match llvm_elem_type {
        BasicTypeEnum::StructType(t) => build_struct_array(context, &elems, llvm_array_type, t),
        _ => build_primitive_array(context, &elems, llvm_array_type, llvm_elem_type),
    }
}

pub(crate) fn build_struct_array<'ctx>(
    context: &mut LLVMContext<'ctx>,
    elems: &[BasicValueEnum<'ctx>],
    llvm_array_type: ArrayType<'ctx>,
    t: StructType<'ctx>,
) -> CodegenResult<PointerValue<'ctx>> {
    let array_alloca = context.create_alloca("arr_tmp", llvm_array_type.into())?;
    let i32_type = context.context().i32_type();
    let zero = i32_type.const_zero();
    let size_bytes = t.size_of().ok_or(CodegenError::LLVMBuild {
        message: "Could not compute struct size".to_string(),
    })?;
    let align = t
        .size_of()
        .and_then(|s| s.get_zero_extended_constant())
        .map(|s| (s as u32).min(8).next_power_of_two())
        .unwrap_or(8);
    for (i, elem) in elems.iter().enumerate() {
        let src_ptr = elem.into_pointer_value();
        let idx = i32_type.const_int(i as u64, false);
        let slot_ptr = unsafe {
            context
                .builder()
                .build_in_bounds_gep(llvm_array_type, array_alloca, &[zero, idx], "slot")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to GEP array slot: {e}"),
                })?
        };
        context
            .builder()
            .build_memcpy(slot_ptr, align, src_ptr, align, size_bytes)
            .map_err(CodegenError::from)?;
    }
    Ok(array_alloca)
}

pub(crate) fn build_primitive_array<'ctx>(
    context: &mut LLVMContext<'ctx>,
    elems: &[BasicValueEnum<'ctx>],
    llvm_array_type: ArrayType<'ctx>,
    llvm_elem_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<PointerValue<'ctx>> {
    if elems.iter().all(|v| v.is_const()) {
        let const_array = match llvm_elem_type {
            BasicTypeEnum::IntType(t) => {
                let vals: Vec<_> = elems.iter().map(|v| v.into_int_value()).collect();
                t.const_array(&vals).as_basic_value_enum()
            }
            BasicTypeEnum::FloatType(t) => {
                let vals: Vec<_> = elems.iter().map(|v| v.into_float_value()).collect();
                t.const_array(&vals).as_basic_value_enum()
            }
            BasicTypeEnum::PointerType(t) => {
                let vals: Vec<_> = elems.iter().map(|v| v.into_pointer_value()).collect();
                t.const_array(&vals).as_basic_value_enum()
            }
            _ => {
                return Err(CodegenError::LLVMBuild {
                    message: format!(
                        "Unsupported element type for array literal: {llvm_elem_type:?}"
                    ),
                });
            }
        };
        let global = context.module().add_global(llvm_array_type, None, "arr");
        global.set_initializer(&const_array);
        global.set_constant(true);
        global.set_linkage(Linkage::Private);
        return Ok(global.as_pointer_value());
    }

    let array_alloca = context.create_alloca("arr_tmp", llvm_array_type.into())?;
    let i64_type = context.context().i64_type();
    let zero = i64_type.const_zero();
    for (i, elem) in elems.iter().enumerate() {
        let idx = i64_type.const_int(i as u64, false);
        let slot_ptr = unsafe {
            context
                .builder()
                .build_in_bounds_gep(llvm_array_type, array_alloca, &[zero, idx], "slot")
                .map_err(|e| CodegenError::LLVMBuild {
                    message: format!("Failed to GEP array slot: {e}"),
                })?
        };
        context
            .builder()
            .build_store(slot_ptr, *elem)
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to store array elem: {e}"),
            })?;
    }
    Ok(array_alloca)
}
