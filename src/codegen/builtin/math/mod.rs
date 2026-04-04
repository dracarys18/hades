pub mod exp;
pub mod power;
pub mod rounding;
pub mod trig;

pub use exp::{Exp, Exp2, Log, Log2, Log10, Sqrt};
pub use power::{Pow, Powi};
pub use rounding::{Ceil, Floor, Round, Trunc};
pub use trig::{Cos, Sin};

macro_rules! intrinsic {
    ($struct_name:ident, $llvm_name:literal, [$($arg_type:ident),+]) => {
        intrinsic!(
            @build
            $struct_name, $llvm_name,
            [$($arg_type),+],
            [$($arg_type),+]
        );
    };

    (@build $struct_name:ident, $llvm_name:literal, [$($arg_type:ident),+], [$($count_type:ident),+]) => {
        pub struct $struct_name;

        impl $crate::codegen::builtin::Builtin for $struct_name {
            fn declare<'ctx>(
                context: &mut $crate::codegen::LLVMContext<'ctx>,
            ) -> inkwell::values::FunctionValue<'ctx> {
                let intrinsic = inkwell::intrinsics::Intrinsic::find($llvm_name)
                    .unwrap_or_else(|| panic!("{} intrinsic not found", $llvm_name));
                let ctx = context.context();
                #[allow(unused_variables)]
                let type_slice: &[inkwell::types::BasicTypeEnum] = &[
                    $( intrinsic!(@type_expr $arg_type, ctx) ),+
                ];
                intrinsic
                    .get_declaration(context.module(), type_slice)
                    .unwrap_or_else(|| panic!("Failed to get {} declaration", $llvm_name))
            }

            fn call<'ctx>(
                context: &mut $crate::codegen::LLVMContext<'ctx>,
                args: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
            ) -> $crate::codegen::error::CodegenResult<inkwell::values::AnyValueEnum<'ctx>> {
                const ARG_COUNT: usize = 0usize $( + intrinsic!(@count $count_type) )+;
                if args.len() != ARG_COUNT {
                    return Err($crate::codegen::error::CodegenError::LLVMBuild {
                        message: format!(
                            "{}() requires exactly {} argument(s)",
                            $llvm_name, ARG_COUNT
                        ),
                    });
                }
                let fn_name = concat!($llvm_name $( , intrinsic!(@suffix $arg_type) )+);
                let func = context
                    .module()
                    .get_function(fn_name)
                    .unwrap_or_else(|| panic!("{} not found", fn_name));
                let result = context
                    .builder()
                    .build_call(func, args, "tmp")
                    .map_err(|_| $crate::codegen::error::CodegenError::LLVMBuild {
                        message: format!("Failed to build {} call", $llvm_name),
                    })?;
                Ok(result
                    .try_as_basic_value()
                    .basic()
                    .expect("Expected basic value")
                    .into())
            }
        }
    };

    (@type_expr f64, $ctx:expr) => { $ctx.f64_type().into() };
    (@type_expr f32, $ctx:expr) => { $ctx.f32_type().into() };
    (@type_expr i32, $ctx:expr) => { $ctx.i32_type().into() };

    (@suffix f64) => { ".f64" };
    (@suffix f32) => { ".f32" };
    (@suffix i32) => { ".i32" };

    (@count $any:ident) => { 1usize };
}

pub(super) use intrinsic;
