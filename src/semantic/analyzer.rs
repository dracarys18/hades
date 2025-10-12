use std::marker::PhantomData;

use crate::{
    ast::Program,
    error::SemanticError,
    evaluator::graph::EvaluationGraph,
    typed_ast::{TypedAstMeta, TypedProgram},
};

pub struct Unprepared;
pub struct Prepared;

pub struct Analyzer<T> {
    data: TypedAstMeta,
    _m: PhantomData<T>,
}

impl<T> Analyzer<T> {
    pub fn new() -> Analyzer<Unprepared> {
        Analyzer {
            data: TypedAstMeta::new(),
            _m: PhantomData,
        }
    }

    pub fn ast(&self) -> &TypedProgram {
        self.data.ast()
    }
}

impl Analyzer<Unprepared> {
    pub fn prepare(self, program: &Program) -> Result<Analyzer<Prepared>, SemanticError> {
        self.data.prepare(program).map(|data| Analyzer {
            data,
            _m: PhantomData,
        })
    }
}

impl Analyzer<Prepared> {
    pub fn analyze(&self) -> Result<(), String> {
        let mut evaluator = EvaluationGraph::new();

        //TODO: Add more analysis steps here
        evaluator
            .eval(|program: &TypedProgram| {
                println!("Analyzing program with {} statements", program.len());
                Ok(())
            })
            .execute(self.data.ast())
    }
}
