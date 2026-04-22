use hades_ast::{CompilerContext, ModulePath, TypedProgram};

use crate::signatures::ModuleSignatures;

#[derive(Debug, Clone)]
pub struct TypedModule {
    pub path: ModulePath,
    pub program: TypedProgram,
    pub signatures: ModuleSignatures,
    pub ctx: CompilerContext,
    pub imports: Vec<ModulePath>,
}

/// Assemble a [`TypedModule`] from its components.
pub fn make_typed_module(
    ctx: CompilerContext,
    path: ModulePath,
    program: TypedProgram,
    imports: Vec<ModulePath>,
) -> TypedModule {
    let signatures = ModuleSignatures::from_context(ctx.clone(), path.clone());
    TypedModule {
        path,
        program,
        signatures,
        ctx,
        imports,
    }
}
