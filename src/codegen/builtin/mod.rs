mod printf;

use super::context::LLVMContext;
use super::error::CodegenResult;
use indexmap::{indexmap, IndexMap};
use inkwell::values::{AnyValueEnum, BasicMetadataValueEnum, FunctionValue};
use once_cell::sync::Lazy;
pub use printf::Printf;

pub trait Builtin {
    fn declare<'ctx>(context: &mut LLVMContext<'ctx>) -> FunctionValue<'ctx>;
    fn call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
    ) -> CodegenResult<AnyValueEnum<'ctx>>;
}

pub type BuiltinHandler = for<'ctx> fn(
    &mut LLVMContext<'ctx>,
    &[BasicMetadataValueEnum<'ctx>],
) -> CodegenResult<AnyValueEnum<'ctx>>;

pub static BUILTIN_HANDLERS: Lazy<IndexMap<String, BuiltinHandler>> = Lazy::new(|| {
    indexmap! {
        String::from("printf") => Printf::call as BuiltinHandler,
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
    ) -> CodegenResult<AnyValueEnum<'ctx>> {
        let handler = BUILTIN_HANDLERS
            .get(name)
            .expect("Builtin function not found");

        handler(context, args)
    }

    pub fn is_builtin_function(name: &str) -> bool {
        BUILTIN_HANDLERS.contains_key(name)
    }
}
