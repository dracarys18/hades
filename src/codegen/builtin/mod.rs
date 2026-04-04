mod len;
mod math;
mod printf;

use super::context::LLVMContext;
use super::error::CodegenResult as CResult;
use super::error::{CodegenResult, CodegenValue};
use crate::typed_ast::TypedExpr;
use indexmap::{indexmap, IndexMap};
use inkwell::values::{AnyValueEnum, BasicMetadataValueEnum, FunctionValue};
pub use len::Len;
use math::{Ceil, Cos, Exp, Exp2, Floor, Log, Log10, Log2, Pow, Powi, Round, Sin, Sqrt, Trunc};
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
        String::from("printf")  => Printf::call as BuiltinHandler,
        String::from("sin")     => Sin::call   as BuiltinHandler,
        String::from("cos")     => Cos::call   as BuiltinHandler,
        String::from("sqrt")    => Sqrt::call  as BuiltinHandler,
        String::from("pow")     => Pow::call   as BuiltinHandler,
        String::from("powi")    => Powi::call  as BuiltinHandler,
        String::from("exp")     => Exp::call   as BuiltinHandler,
        String::from("exp2")    => Exp2::call  as BuiltinHandler,
        String::from("log")     => Log::call   as BuiltinHandler,
        String::from("log10")   => Log10::call as BuiltinHandler,
        String::from("log2")    => Log2::call  as BuiltinHandler,
        String::from("floor")   => Floor::call as BuiltinHandler,
        String::from("ceil")    => Ceil::call  as BuiltinHandler,
        String::from("trunc")   => Trunc::call as BuiltinHandler,
        String::from("round")   => Round::call as BuiltinHandler,
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
        Printf::declare(context);
        Sin::declare(context);
        Cos::declare(context);
        Sqrt::declare(context);
        Pow::declare(context);
        Powi::declare(context);
        Exp::declare(context);
        Exp2::declare(context);
        Log::declare(context);
        Log10::declare(context);
        Log2::declare(context);
        Floor::declare(context);
        Ceil::declare(context);
        Trunc::declare(context);
        Round::declare(context);
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
