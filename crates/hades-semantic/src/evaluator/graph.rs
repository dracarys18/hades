use rayon::prelude::*;
use std::sync::Arc;

pub type EvaluatorFn<T> = dyn Fn(&T) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub struct GraphNode<T> {
    evaluators: Vec<Arc<EvaluatorFn<T>>>,
}

impl<T: Sync + Send> Default for GraphNode<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sync + Send> GraphNode<T> {
    pub fn default_boxed() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub fn new() -> Self {
        Self {
            evaluators: Vec::new(),
        }
    }

    pub fn eval<F>(mut self, evaluator: F) -> Self
    where
        F: Fn(&T) -> Result<(), String> + Send + Sync + 'static,
    {
        self.evaluators.push(Arc::new(evaluator));
        self
    }

    pub fn execute(&self, input: &T) -> Result<(), String> {
        for evaluator in &self.evaluators {
            evaluator(input)?;
        }
        Ok(())
    }
}

pub struct EvaluationGraph<T> {
    nodes: Vec<GraphNode<T>>,
}

impl<T: Sync + Send> Default for EvaluationGraph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Sync + Send> EvaluationGraph<T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn eval<S>(&mut self, evaluatable: S) -> &mut Self
    where
        S: Into<EvaluationStep<T>>,
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

    pub fn execute(&self, input: &T) -> Result<(), String> {
        self.nodes
            .par_iter()
            .map(|node| node.execute(input))
            .collect::<Result<Vec<()>, String>>()?;
        Ok(())
    }
}

pub enum EvaluationStep<T> {
    Node(GraphNode<T>),
    Function(Arc<EvaluatorFn<T>>),
}

impl<T: Sync + Send + 'static> From<GraphNode<T>> for EvaluationStep<T> {
    fn from(node: GraphNode<T>) -> Self {
        EvaluationStep::Node(node)
    }
}

impl<T: Sync + Send + 'static, F> From<F> for EvaluationStep<T>
where
    F: Fn(&T) -> Result<(), String> + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        EvaluationStep::Function(Arc::new(f))
    }
}

pub fn node<T: Sync + Send>() -> GraphNode<T> {
    GraphNode::new()
}
