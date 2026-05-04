use std::marker::PhantomData;

use indexmap::IndexMap;

use crate::evaluator::graph::EvaluationGraph;
use crate::lint::array_bounds::ArrayBoundsLint;
use crate::lint::null_deref::NullDerefLint;
use crate::lint::{LintDiagnostic, LintRunner};
use hades_ast::{CompilerContext, ModulePath as AstModulePath, WalkAst};
use hades_error::{SemanticError, Span};
use hades_module::{Module, ModulePath, ModuleSignatures, TypedModule};

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

impl Default for Analyzer<Unprepared> {
    fn default() -> Self {
        Analyzer::<Unprepared>::new()
    }
}

fn to_ast_path(path: &ModulePath) -> AstModulePath {
    match path {
        ModulePath::Std(name) => AstModulePath::Std(name.clone()),
        ModulePath::Local(name) => AstModulePath::Local(name.clone()),
    }
}

impl Analyzer<Unprepared> {
    pub fn prepare(self, modules: Vec<Module>) -> Result<Analyzer<Prepared>, SemanticError> {
        let mut sig_cache: IndexMap<ModulePath, ModuleSignatures> = IndexMap::new();
        let mut typed_modules = Vec::with_capacity(modules.len());

        for module in modules {
            let mut ctx = CompilerContext::new();
            ctx.set_module_path(to_ast_path(&module.path));

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
            let ast_path = to_ast_path(&path);

            let signatures = ModuleSignatures::from_context(ctx.clone(), ast_path);
            sig_cache.insert(path.clone(), signatures.clone());

            typed_modules.push(TypedModule {
                path: to_ast_path(&path),
                program,
                signatures,
                ctx,
                imports: imports.iter().map(to_ast_path).collect(),
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

    pub fn analyze(&self) -> Result<Vec<LintDiagnostic>, String> {
        let mut ast_graph = EvaluationGraph::new();
        ast_graph.eval(|_program: &hades_ast::TypedProgram| Ok(()));

        for typed_module in &self.modules {
            ast_graph.execute(&typed_module.program)?;
        }

        let mut runner = LintRunner::new();
        runner.register(ArrayBoundsLint);
        runner.register(NullDerefLint);

        let mut all_diags = Vec::new();
        for typed_module in &self.modules {
            let mir = hades_mir::lower(typed_module.clone());
            let diags = runner.run(&mir);
            all_diags.extend(diags);
        }

        Ok(all_diags)
    }

    pub fn emit_mir(&self) -> Result<(), String> {
        for typed_module in &self.modules {
            let mir = hades_mir::lower(typed_module.clone());
            println!("; === module: {} ===", typed_module.path);
            println!("{}", mir);
        }
        Ok(())
    }
}
