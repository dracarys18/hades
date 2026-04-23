use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use hades_ast::TypedAsExpression;
use hades_ast::Types;

impl Visit for TypedAsExpression {
    type Output<'ctx> = CodegenValue<'ctx>;
    fn visit<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> crate::codegen::error::CodegenResult<crate::codegen::error::CodegenValue<'ctx>> {
        let target_type = {
            let _symbols = context.symbols();
            let converter = context.type_converter();
            converter.to_llvm_type(&self.target_type, context.module())?
        };

        let value = self.expr.visit(context)?;
        let casted_value = match (self.expr.get_type(), &self.target_type) {
            (Types::Int, Types::Float) => cast_int_to_float(context, value, target_type)?,
            (Types::Char, Types::Int) => cast_char_to_int(context, value, target_type)?,
            (Types::Char, Types::Float) => cast_char_to_float(context, value, target_type)?,
            (Types::Float, Types::Int) => cast_float_to_int(context, value, target_type)?,
            (_, _) => {
                unreachable!(
                    "This should have been caught by the type checker. Invalid cast from {:?} to {:?}",
                    self.expr.get_type(),
                    self.target_type
                );
            }
        };
        Ok(CodegenValue::new(casted_value, self.target_type.clone()))
    }
}

fn cast_int_to_float<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let casted_value = context.builder().build_signed_int_to_float(
        value.value()?.into_int_value(),
        target_type.into_float_type(),
        "int_to_float_cast",
    )?;

    Ok(casted_value.into())
}

fn cast_char_to_int<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let casted_value = context.builder().build_int_cast(
        value.value()?.into_int_value(),
        target_type.into_int_type(),
        "char_to_int_cast",
    )?;

    Ok(casted_value.into())
}

fn cast_float_to_int<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let casted_value = context.builder().build_float_to_signed_int(
        value.value()?.into_float_value(),
        target_type.into_int_type(),
        "float_to_int_cast",
    )?;

    Ok(casted_value.into())
}

fn cast_char_to_float<'ctx>(
    context: &mut LLVMContext<'ctx>,
    value: CodegenValue<'ctx>,
    target_type: BasicTypeEnum<'ctx>,
) -> CodegenResult<BasicValueEnum<'ctx>> {
    let casted_value = context.builder().build_unsigned_int_to_float(
        value.value()?.into_int_value(),
        target_type.into_float_type(),
        "char_to_float_cast",
    )?;

    Ok(casted_value.into())
}
