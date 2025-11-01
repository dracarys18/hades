use crate::ast::{ArrayType, Types};
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::typed_ast::{TypedArrayLiteral, TypedValue};
use inkwell::types::{BasicType, BasicTypeEnum};
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
        let value = self
            .elements
            .iter()
            .map(|v| v.visit(context))
            .collect::<Result<Vec<_>, _>>()?;

        let symbols = context.symbols();
        let type_converter = context.type_converter();
        let var_type =
            type_converter.to_llvm_type(&self.elem_typ.get_array_elem_type(), symbols)?;
        let llvm_array_type = type_converter.to_llvm_type(&self.elem_typ, symbols)?;

        let values: Vec<_> = value.iter().map(|v| v.value).collect();

        let array = match &self.elem_typ {
            Types::Array(ArrayType::IntArray(_)) => context.context().i64_type().const_array(
                &values
                    .iter()
                    .map(|v| v.into_int_value())
                    .collect::<Vec<_>>(),
            ),

            Types::Array(ArrayType::FloatArray(_)) => context.context().f64_type().const_array(
                &values
                    .iter()
                    .map(|v| v.into_float_value())
                    .collect::<Vec<_>>(),
            ),
            typ => unimplemented!("Array for type {} is not implemented yet", typ),
        };

        let global = context.module().add_global(var_type, None, "array");
        global.set_initializer(&array);
        global.set_constant(true);

        let size = context
            .context()
            .i32_type()
            .const_int(self.elements.len() as u64, false);
        let array_ptr = context
            .builder()
            .build_array_alloca(llvm_array_type, size, "arr_ptr")
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to create array alloca".to_string(),
            })?;

        context.builder().build_memcpy(
            array_ptr,
            8,
            global.as_pointer_value(),
            8,
            llvm_array_type.size_of().unwrap(),
        )?;
        Ok(CodegenValue::new(array.into(), self.elem_typ.clone()))
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
