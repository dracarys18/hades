use crate::ast::Types;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::typed_ast::{TypedArrayLiteral, TypedValue};
use inkwell::values::BasicValueEnum;

impl Visit for TypedValue {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            TypedValue::Number(val) => generate_int_value(*val, context),
            TypedValue::Float(val) => generate_float_value(*val, context),
            TypedValue::Boolean(val) => generate_bool_value(*val, context),
            TypedValue::String(val) => generate_string_value(val, context),
            TypedValue::Array(val) => val.visit(context),
        }
    }
}

impl Visit for TypedArrayLiteral {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let symbols = context.symbols();
        let array_type = context
            .type_converter()
            .to_llvm_type(&self.elem_typ, symbols)?;
        let elem_type = self.elem_typ.get_array_elem_type();
        let llvm_elem_type = context.type_converter().to_llvm_type(&elem_type, symbols)?;

        let array_ptr = context.builder().build_alloca(array_type, "array")?;

        for (i, element) in self.elements.iter().enumerate() {
            let elem_value = element.visit(context)?;
            let actual_value = match (&elem_type, elem_value.value) {
                (Types::String, val) => val,
                (_, BasicValueEnum::PointerValue(ptr)) => {
                    context.builder().build_load(llvm_elem_type, ptr, "elem")?
                }
                (_, val) => val,
            };

            let zero = context.context().i32_type().const_zero();
            let index = context.context().i32_type().const_int(i as u64, false);
            let elem_ptr = unsafe {
                context.builder().build_in_bounds_gep(
                    array_type,
                    array_ptr,
                    &[zero, index],
                    "elem_ptr",
                )?
            };
            context.builder().build_store(elem_ptr, actual_value)?;
        }

        Ok(CodegenValue::new(array_ptr.into(), self.elem_typ.clone()))
    }
}

fn generate_bool_value<'ctx>(
    val: bool,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().bool_type().const_int(val as u64, false);
    Ok(CodegenValue::new(llvm_val.into(), Types::Bool))
}

fn generate_string_value<'ctx>(
    val: &str,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let bytes = val.as_bytes();

    let const_str = context.context().const_string(bytes, true);

    let global = context
        .module()
        .add_global(const_str.get_type(), None, "str");
    global.set_initializer(&const_str);
    global.set_constant(true);

    let zero = context.context().i32_type().const_zero();
    let ptr = unsafe {
        context
            .builder()
            .build_in_bounds_gep(
                const_str.get_type().get_element_type(),
                global.as_pointer_value(),
                &[zero, zero],
                "str_ptr",
            )
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to create string pointer".to_string(),
            })?
    };

    Ok(CodegenValue::new(ptr.into(), Types::String))
}

fn generate_int_value<'ctx>(
    val: i64,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().i64_type().const_int(val as u64, false);
    Ok(CodegenValue::new(llvm_val.into(), Types::Int))
}

fn generate_float_value<'ctx>(
    val: f64,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().f64_type().const_float(val);
    Ok(CodegenValue::new(llvm_val.into(), Types::Float))
}
