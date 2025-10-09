use rayon::prelude::*;

use crate::ast::Program;
use std::sync::Arc;

pub type EvaluatorFn<T> = dyn for<'a> Fn(&'a T) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub struct GraphNode {
    evaluators: Vec<Arc<EvaluatorFn<Program>>>,
}

impl GraphNode {
    pub fn new() -> Self {
        Self {
            evaluators: Vec::new(),
        }
    }

    pub fn eval<F>(mut self, evaluator: F) -> Self
    where
        F: for<'a> Fn(&'a Program) -> Result<(), String> + Send + Sync + 'static,
    {
        self.evaluators.push(Arc::new(evaluator));
        self
    }

    pub fn execute(&self, program: &Program) -> Result<(), String> {
        for evaluator in &self.evaluators {
            evaluator(program)?;
        }
        Ok(())
    }
}

pub struct EvaluationGraph {
    nodes: Vec<GraphNode>,
}

impl EvaluationGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn eval<T>(&mut self, evaluatable: T) -> &mut Self
    where
        T: Into<EvaluationStep>,
    {
        let step = evaluatable.into();
        match step {
            EvaluationStep::Node(node) => {
                self.nodes.push(node);
            }
            EvaluationStep::Function(evaluator) => {
                let mut node = GraphNode::new();
                node.evaluators.push(evaluator);

                self.nodes.push(node);
            }
        }
        self
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), String> {
        self.nodes
            .par_iter()
            .map(|node| node.execute(program))
            .collect::<Result<Vec<()>, String>>()?;

        Ok(())
    }
}

pub enum EvaluationStep {
    Node(GraphNode),
    Function(Arc<EvaluatorFn<Program>>),
}

impl From<GraphNode> for EvaluationStep {
    fn from(node: GraphNode) -> Self {
        EvaluationStep::Node(node)
    }
}

impl<F> From<F> for EvaluationStep
where
    F: for<'a> Fn(&'a Program) -> Result<(), String> + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        EvaluationStep::Function(Arc::new(f))
    }
}

pub fn node() -> GraphNode {
    GraphNode::new()
}
