use hades_ast::{CompilerContext, ModulePath, TypedProgram};
use hades_module::ModuleSignatures;

use super::func::MirFunction;

/// The MIR representation of a compiled module.
/// Self-contained: carries everything codegen needs, including import signatures.
#[derive(Debug, Clone)]
pub struct MirModule {
    /// Path of this module (e.g. `Local("main")` or `Std("io")`).
    pub path: ModulePath,

    /// The original typed program, retained for struct definitions
    /// (struct defs are not in the CFG).
    pub program: TypedProgram,

    /// All lowered functions in this module (free functions + methods).
    pub functions: Vec<MirFunction>,

    /// The compiler context from semantic analysis
    /// (used by codegen for type lookups, struct field indices, etc.).
    pub ctx: CompilerContext,

    /// Paths of imported modules.
    pub imports: Vec<ModulePath>,

    /// This module's own exported signatures (functions + structs).
    pub signatures: ModuleSignatures,

    /// Signatures of all directly-imported modules.
    pub import_sigs: Vec<ModuleSignatures>,
}
