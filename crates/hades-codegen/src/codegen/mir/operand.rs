use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use hades_ast::Types;
use hades_mir::mir::operand::{MirConst, Operand};
use hades_mir::visit::Visit;
use inkwell::AddressSpace;
use inkwell::values::BasicValueEnum;

use super::place::place_type;

impl<'ctx> Visit<LLVMContext<'ctx>> for Operand {
    type Output = (BasicValueEnum<'ctx>, Types);

    fn visit(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output> {
        match self {
            Operand::Copy(place) => {
                let (ptr, ty) = place.visit(context)?;
                let llvm_type = context
                    .type_converter()
                    .to_llvm_type(&ty, context.module())?;
                let value = context.load(ptr, llvm_type, "copy")?;
                Ok((value, ty))
            }
            Operand::Ref(place) => {
                let (ptr, ty) = place.visit(context)?;
                Ok((ptr.into(), Types::Pointer(Box::new(ty))))
            }
            Operand::Const(constant) => eval_const(context, constant),
        }
    }
}

pub(super) fn operand_type(context: &LLVMContext<'_>, operand: &Operand) -> CodegenResult<Types> {
    match operand {
        Operand::Copy(place) => place_type(context, place),
        Operand::Ref(place) => Ok(Types::Pointer(Box::new(place_type(context, place)?))),
        Operand::Const(constant) => Ok(const_type(constant)),
    }
}

fn eval_const<'ctx>(
    context: &mut LLVMContext<'ctx>,
    constant: &MirConst,
) -> CodegenResult<(BasicValueEnum<'ctx>, Types)> {
    let value: BasicValueEnum<'ctx> = match constant {
        MirConst::Int(n) => context
            .context()
            .i64_type()
            .const_int(*n as u64, false)
            .into(),
        MirConst::Float(f) => context.context().f64_type().const_float(*f).into(),
        MirConst::Bool(b) => context
            .context()
            .bool_type()
            .const_int(*b as u64, false)
            .into(),
        MirConst::Char(c) => context
            .context()
            .i8_type()
            .const_int(*c as u64, false)
            .into(),
        MirConst::Null(_) => context
            .context()
            .ptr_type(AddressSpace::default())
            .const_null()
            .into(),
        MirConst::Str(s) => context.const_string_ptr(s)?.into(),
    };
    Ok((value, const_type(constant)))
}

fn const_type(constant: &MirConst) -> Types {
    match constant {
        MirConst::Int(_) => Types::Int,
        MirConst::Float(_) => Types::Float,
        MirConst::Bool(_) => Types::Bool,
        MirConst::Char(_) => Types::Char,
        MirConst::Str(_) => Types::String,
        MirConst::Null(ty) => ty.clone(),
    }
}
