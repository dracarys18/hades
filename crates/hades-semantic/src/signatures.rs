use hades_ast::{CompilerContext, FunctionSignature, ModulePath, Structs};
use hades_tokens::Name;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct ModuleSignatures {
    pub path: ModulePath,
    pub functions: IndexMap<Name, FunctionSignature>,
    pub structs: Structs,
}

impl ModuleSignatures {
    /// Build a `ModuleSignatures` from a consumed `CompilerContext` and its path.
    pub fn from_context(ctx: CompilerContext, path: ModulePath) -> Self {
        let structs = ctx.structs().clone();
        let functions = ctx.into_functions().into_user_defined();
        Self {
            path,
            functions,
            structs,
        }
    }
}
