use hades_ast::{CompilerContext, FunctionSignature, ModulePath, Structs, TypedProgram};
use hades_tokens::Name;
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub struct ModuleSignatures {
    pub path: ModulePath,
    pub functions: IndexMap<Name, FunctionSignature>,
    pub structs: Structs,
}

impl ModuleSignatures {
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

#[derive(Debug, Clone)]
pub struct TypedModule {
    pub path: ModulePath,
    pub program: TypedProgram,
    pub signatures: ModuleSignatures,
    pub ctx: CompilerContext,
    pub imports: Vec<ModulePath>,
}

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
