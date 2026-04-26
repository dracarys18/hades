use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use hades_ast::Types;
use hades_mir::mir::operand::MirConst;
use inkwell::module::Linkage;

impl Visit for MirConst {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            MirConst::Int(val) => generate_int_value(*val, context),
            MirConst::Float(val) => generate_float_value(*val, context),
            MirConst::Bool(val) => generate_bool_value(*val, context),
            MirConst::String(val) => generate_string_value(val, context),
            MirConst::Char(val) => generate_char_value(*val as u8, context),
            MirConst::Null(ty) => {
                let ptr = context
                    .context()
                    .ptr_type(inkwell::AddressSpace::default())
                    .const_null();
                Ok(CodegenValue::new(ptr.into(), ty.clone()))
            }
        }
    }
}

pub(crate) fn generate_int_value<'ctx>(
    val: i64,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().i64_type().const_int(val as u64, false);
    Ok(CodegenValue::new(llvm_val.into(), Types::Int))
}

pub(crate) fn generate_float_value<'ctx>(
    val: f64,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().f64_type().const_float(val);
    Ok(CodegenValue::new(llvm_val.into(), Types::Float))
}

pub(crate) fn generate_bool_value<'ctx>(
    val: bool,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().bool_type().const_int(val as u64, false);
    Ok(CodegenValue::new(llvm_val.into(), Types::Bool))
}

pub(crate) fn generate_string_value<'ctx>(
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
    global.set_linkage(Linkage::Private);
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

pub(crate) fn generate_char_value<'ctx>(
    val: u8,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<CodegenValue<'ctx>> {
    let llvm_val = context.context().i8_type().const_int(val as u64, false);
    Ok(CodegenValue::new(llvm_val.into(), Types::Char))
}
