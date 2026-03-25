mod len;
mod printf;

use super::context::LLVMContext;
use super::error::CodegenResult as CResult;
use super::error::{CodegenResult, CodegenValue};
use crate::typed_ast::TypedExpr;
use indexmap::{IndexMap, indexmap};
use inkwell::values::{AnyValueEnum, BasicMetadataValueEnum, FunctionValue};
pub use len::Len;
use once_cell::sync::Lazy;
pub use printf::Printf;

pub trait Builtin {
    fn declare<'ctx>(context: &mut LLVMContext<'ctx>) -> FunctionValue<'ctx>;
    fn call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> CResult<AnyValueEnum<'ctx>>;
}

pub type BuiltinHandler = for<'ctx> fn(
    &mut LLVMContext<'ctx>,
    &[BasicMetadataValueEnum<'ctx>],
) -> CResult<AnyValueEnum<'ctx>>;

pub static BUILTIN_HANDLERS: Lazy<IndexMap<String, BuiltinHandler>> = Lazy::new(|| {
    indexmap! {
        String::from("printf") => Printf::call as BuiltinHandler,
    }
});

pub trait CompileTimeBuiltin {
    fn call<'ctx>(
        args: &[TypedExpr],
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>>;
}

pub type CompileTimeHandler =
    for<'ctx> fn(&[TypedExpr], &mut LLVMContext<'ctx>) -> CodegenResult<CodegenValue<'ctx>>;

pub static COMPILE_TIME_HANDLERS: Lazy<IndexMap<String, CompileTimeHandler>> = Lazy::new(|| {
    indexmap! {
        String::from("len") => Len::call as CompileTimeHandler,
    }
});

pub struct BuiltinRegistar;

impl BuiltinRegistar {
    pub fn declare_all<'ctx>(context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        for (name, _) in BUILTIN_HANDLERS.iter() {
            match name.as_str() {
                "printf" => {
                    Printf::declare(context);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn handle<'ctx>(
        name: &str,
        context: &mut LLVMContext<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> CResult<AnyValueEnum<'ctx>> {
        let handler = BUILTIN_HANDLERS
            .get(name)
            .expect("Builtin function not found");

        handler(context, args)
    }

    pub fn is_builtin_function(name: &str) -> bool {
        BUILTIN_HANDLERS.contains_key(name) || COMPILE_TIME_HANDLERS.contains_key(name)
    }

    pub fn is_compile_time_function(name: &str) -> bool {
        COMPILE_TIME_HANDLERS.contains_key(name)
    }

    pub fn handle_compile_time<'ctx>(
        name: &str,
        args: &[TypedExpr],
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<CodegenValue<'ctx>> {
        let handler = COMPILE_TIME_HANDLERS
            .get(name)
            .expect("Compile-time builtin function not found");

        handler(args, context)
    }
}
