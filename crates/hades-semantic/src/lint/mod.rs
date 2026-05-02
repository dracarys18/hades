pub mod array_bounds;
pub mod null_deref;

use std::sync::{Arc, Mutex};

use hades_error::{Error, ErrorSeverity};
use hades_mir::mir::func::MirFunction;
use hades_mir::mir::module::MirModule;
use rayon::prelude::*;

use crate::evaluator::graph::EvaluationGraph;

#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub lint_name: &'static str,
    pub error: Error,
}

impl LintDiagnostic {
    pub fn error(lint_name: &'static str, error: Error) -> Self {
        Self { lint_name, error }
    }

    pub fn is_error(&self) -> bool {
        self.error.severity == ErrorSeverity::Error
    }
}

pub trait Lint: Send + Sync {
    fn name(&self) -> &'static str;
    fn check_function(&self, func: &MirFunction) -> Vec<LintDiagnostic>;
}

pub struct LintRunner {
    graph: EvaluationGraph<MirFunction>,
    sink: Arc<Mutex<Vec<LintDiagnostic>>>,
}

impl Default for LintRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl LintRunner {
    pub fn new() -> Self {
        Self {
            graph: EvaluationGraph::new(),
            sink: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register<L: Lint + 'static>(&mut self, lint: L) {
        let sink = Arc::clone(&self.sink);
        self.graph.eval(move |func: &MirFunction| {
            let diags = lint.check_function(func);
            sink.lock().unwrap().extend(diags);
            Ok(())
        });
    }

    pub fn run(&self, module: &MirModule) -> Vec<LintDiagnostic> {
        module.functions.par_iter().for_each(|func| {
            let _ = self.graph.execute(func);
        });

        let mut guard = self.sink.lock().unwrap();
        guard.drain(..).collect()
    }
}
