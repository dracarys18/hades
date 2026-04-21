use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::typed_ast::{TypedArrayLiteral, TypedExpr};
use inkwell::module::Linkage;
use inkwell::types::{ArrayType, BasicTypeEnum, StructType};
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};

impl Visit for TypedArrayLiteral {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let _symbols = context.symbols();
        let array_type = context
            .type_converter()
            .to_llvm_type(&self.elem_typ, context.module())?;
        let elem_type = self.elem_typ.get_array_elem_type();
        let llvm_elem_type = context
            .type_converter()
            .to_llvm_type(&elem_type, context.module())?;
        let llvm_array_type = array_type.into_array_type();

        let elems = resolve_elements(context, &self.elements, &elem_type, llvm_elem_type)?;

        let ptr = match llvm_elem_type {
            BasicTypeEnum::StructType(t) => {
                build_struct_array(context, &elems, llvm_array_type, t)?
            }
            _ => build_primitive_array(context, &elems, llvm_array_type, llvm_elem_type)?,
        };

        Ok(CodegenValue::new(ptr.into(), self.elem_typ.clone()))
    }
}

fn resolve_elements<'ctx>(
    context: &mut LLVMContext<'ctx>,
    elements: &[TypedExpr],
    elem_type: &Types,
    llvm_elem_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<Vec<BasicValueEnum<'ctx>>> {
    let mut out = Vec::with_capacity(elements.len());
    for element in elements {
        let val = element.visit(context)?.value()?;
        let resolved = match (elem_type, val) {
            (Types::Struct(_), v) => {
                let tmp = context.create_alloca("struct_elem", llvm_elem_type)?;
                context
                    .builder()
                    .build_store(tmp, v)
                    .map_err(|e| CodegenError::LLVMBuild {
                        message: format!("Failed to store struct elem: {e}"),
                    })?;
                tmp.into()
            }
            (Types::Pointer(_), v) => v,
            (Types::String, v) => v,
            (_, BasicValueEnum::PointerValue(ptr)) => context.load(ptr, llvm_elem_type, "elem")?,
            (_, v) => v,
        };
        out.push(resolved);
    }
    Ok(out)
}

fn build_struct_array<'ctx>(
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

fn build_primitive_array<'ctx>(
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
