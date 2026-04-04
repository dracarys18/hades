use super::Builtin;
use crate::codegen::{
    LLVMContext,
    error::{CodegenError, CodegenResult},
};
use inkwell::AddressSpace;
use inkwell::intrinsics::Intrinsic;
use inkwell::values::{AnyValueEnum, BasicMetadataValueEnum, FunctionValue};

pub struct Sin;

impl Builtin for Sin {
    fn declare<'ctx>(context: &mut LLVMContext<'ctx>) -> FunctionValue<'ctx> {
        let sin_intrinsic = Intrinsic::find("llvm.sin").expect("llvm.sin intrinsic not found");
        let f64_type = context.context().f64_type();

        sin_intrinsic
            .get_declaration(context.module(), &[f64_type.into()])
            .expect("Failed to get llvm.sin declaration")
    }

    fn call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> CodegenResult<AnyValueEnum<'ctx>> {
        if args.len() != 1 {
            return Err(CodegenError::LLVMBuild {
                message: "sin() requires exactly one argument".to_string(),
            });
        }

        let builder = context.builder();

        let sin_fn = context
            .module()
            .get_function("llvm.sin.f64")
            .expect("llvm.sin.f64 function not found");

        let call_result =
            builder
                .build_call(sin_fn, args, "sintmp")
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to build sin call".to_string(),
                })?;

        Ok(call_result
            .try_as_basic_value()
            .basic()
            .expect("Expected a basic value")
            .into())
    }
}

pub struct Cos;

impl Builtin for Cos {
    fn declare<'ctx>(context: &mut LLVMContext<'ctx>) -> FunctionValue<'ctx> {
        let cos_intrinsic = Intrinsic::find("llvm.cos").expect("llvm.sin intrinsic not found");
        let f64_type = context.context().f64_type();

        cos_intrinsic
            .get_declaration(context.module(), &[f64_type.into()])
            .expect("Failed to get llvm.sin declaration")
    }

    fn call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> CodegenResult<AnyValueEnum<'ctx>> {
        if args.len() != 1 {
            return Err(CodegenError::LLVMBuild {
                message: "cos() requires exactly one argument".to_string(),
            });
        }

        let builder = context.builder();

        let cos_fn = context
            .module()
            .get_function("llvm.cos.f64")
            .expect("llvm.cos.f64 function not found");

        let call_result =
            builder
                .build_call(cos_fn, args, "sintmp")
                .map_err(|_| CodegenError::LLVMBuild {
                    message: "Failed to build sin call".to_string(),
                })?;

        Ok(call_result
            .try_as_basic_value()
            .basic()
            .expect("Expected a basic value")
            .into())
    }
}
