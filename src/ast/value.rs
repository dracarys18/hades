use crate::ast::{Expr, Types};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(ArrayLiteral),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteral {
    pub elem: Vec<Expr>,
    pub size: usize,
    pub declared_type: Option<Types>,
}

impl ArrayLiteral {
    pub fn new(elem: Vec<Expr>) -> Self {
        let size = elem.len();
        Self {
            elem,
            size,
            declared_type: None,
        }
    }
}
