use std::marker::PhantomData;

use indexmap::IndexMap;

use crate::{
    ast::WalkAst,
    error::{SemanticError, Span},
    evaluator::graph::EvaluationGraph,
    module::{Module, ModulePath},
    typed_ast::{CompilerContext, ModuleSignatures, TypedModule, TypedProgram},
};

pub struct Unprepared;
pub struct Prepared;

pub struct Analyzer<T> {
    modules: Vec<TypedModule>,
    _m: PhantomData<T>,
}

impl<T> Analyzer<T> {
    pub fn new() -> Analyzer<Unprepared> {
        Analyzer {
            modules: Vec::new(),
            _m: PhantomData,
        }
    }
}

impl Analyzer<Unprepared> {
    pub fn prepare(self, modules: Vec<Module>) -> Result<Analyzer<Prepared>, SemanticError> {
        let mut sig_cache: IndexMap<ModulePath, ModuleSignatures> = IndexMap::new();
        let mut typed_modules = Vec::with_capacity(modules.len());

        for module in modules {
            let mut ctx = CompilerContext::new();
            ctx.set_module_name(module.path.name().to_string());

            for dep_path in &module.imports {
                if let Some(sigs) = sig_cache.get(dep_path) {
                    for (name, sig) in &sigs.functions {
                        ctx.register_function(name.clone(), sig.clone())?;
                    }
                    for (name, fields) in sigs.structs.iter() {
                        ctx.insert_struct(name.clone(), fields.clone());
                    }
                }
            }

            let program = module.ast.walk(&mut ctx, Span::default())?;
            let imports = module.imports.clone();
            let path = module.path.clone();
            let ctx_clone = ctx.clone();
            let signatures = ctx.into_signatures(path.clone());

            sig_cache.insert(path.clone(), signatures.clone());

            typed_modules.push(TypedModule {
                path,
                program,
                signatures,
                ctx: ctx_clone,
                imports,
            });
        }

        Ok(Analyzer {
            modules: typed_modules,
            _m: PhantomData,
        })
    }
}

impl Analyzer<Prepared> {
    pub fn modules(&self) -> &[TypedModule] {
        &self.modules
    }

    pub fn analyze(&self) -> Result<(), String> {
        let mut evaluator = EvaluationGraph::new();
        evaluator.eval(|_program: &TypedProgram| Ok(()));

        for typed_module in &self.modules {
            evaluator.execute(&typed_module.program)?;
        }
        Ok(())
    }
}
