mod len;

use super::context::LLVMContext;
use super::error::{CodegenResult, CodegenValue};
use crate::typed_ast::TypedExpr;
use indexmap::{IndexMap, indexmap};
pub use len::Len;
use once_cell::sync::Lazy;

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
    pub fn is_builtin_function(name: &str) -> bool {
        COMPILE_TIME_HANDLERS.contains_key(name)
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
