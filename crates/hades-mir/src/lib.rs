pub(crate) mod builder;
pub mod mir;

use hades_ast::TypedStmt;
use hades_module::{ModuleSignatures, TypedModule};

use builder::MirBuilder;
use mir::{func::MirFunction, module::MirModule};

/// The single lowering trait.
/// `Output` is what the lowering of `Self` produces.
/// All impls take `&self` (borrow, not consume).
pub trait ToMir {
    type Output;
    fn to_mir(&self, builder: &mut MirBuilder) -> Self::Output;
}

/// Lower a `TypedModule` into a `MirModule`.
///
/// `import_sigs` are the signatures of all directly-imported modules; they are
/// embedded into the returned `MirModule` so it is self-contained for codegen.
pub fn lower(module: TypedModule, import_sigs: Vec<ModuleSignatures>) -> MirModule {
    // Collect all function definitions from the top-level program.
    // Methods (struct Func fields) are also collected here.
    let mut functions: Vec<MirFunction> = vec![];

    // Dummy builder — only used as the ToMir API surface; each function
    // creates its own internal builder in TypedFuncDef::to_mir.
    let mut dummy_builder = MirBuilder::new(
        hades_ast::Types::Void,
        hades_error::Span::default(),
    );

    for stmt in module.program.iter() {
        match stmt {
            TypedStmt::FuncDef(func_def) => {
                if let Some(mir_fn) = func_def.to_mir(&mut dummy_builder) {
                    functions.push(mir_fn);
                }
            }
            TypedStmt::StructDef(struct_def) => {
                // Lower methods from struct definitions.
                for (_field_name, field_kind) in &struct_def.fields {
                    if let hades_ast::TypedFieldKind::Func(method) = field_kind {
                        if let Some(mir_fn) = method.to_mir(&mut dummy_builder) {
                            functions.push(mir_fn);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    MirModule {
        path: module.path,
        program: module.program,
        functions,
        ctx: module.ctx,
        imports: module.imports,
        signatures: module.signatures,
        import_sigs,
    }
}
