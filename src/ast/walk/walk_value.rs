use crate::ast::Value;
use crate::ast::WalkAst;
use crate::error::SemanticError;
use crate::typed_ast::{CompilerContext, TypedValue};

impl WalkAst for Value {
    type Output = TypedValue;
    fn walk(&self, _ctx: &mut CompilerContext) -> Result<Self::Output, SemanticError> {
        Ok(match self {
            Self::Float(f) => TypedValue::Float(*f),
            Self::Number(n) => TypedValue::Number(*n),
            Self::String(s) => TypedValue::String(s.clone()),
            Self::Boolean(b) => TypedValue::Boolean(*b),
        })
    }
}
