use crate::ast::{Expr, Types};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Array(ArrayLiteral),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteral {
    pub elem: Vec<Expr>,
    pub size: usize,
    pub declared_type: Option<Types>,
    pub fill: Option<Box<Expr>>,
}

impl ArrayLiteral {
    pub fn new(elem: Vec<Expr>) -> Self {
        let size = elem.len();
        Self {
            elem,
            size,
            declared_type: None,
            fill: None,
        }
    }

    pub fn new_fill(fill: Expr, count: usize) -> Self {
        Self {
            elem: std::iter::repeat(fill.clone()).take(count).collect(),
            size: count,
            declared_type: None,
            fill: Some(Box::new(fill)),
        }
    }
}
