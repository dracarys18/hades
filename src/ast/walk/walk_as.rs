use crate::ast::Types;
use crate::ast::{AsExpression, WalkAst, walk::CompilerContext};
use crate::error::{SemanticError, Span};
use crate::typed_ast::TypedAsExpression;
use std::collections::HashMap;

const ConvertMap: phf::Map<&'static str, &'static [Types]> = phf::phf_map! {
    "int" => &[Types::Float],
    "float" => &[Types::Int],
    "char" => &[Types::Int, Types::Float],
};

impl WalkAst for AsExpression {
    type Output = TypedAsExpression;
    fn walk(
        &self,
        ctx: &mut crate::ast::walk::CompilerContext,
        span: crate::error::Span,
    ) -> Result<Self::Output, crate::error::SemanticError> {
        let expr = self.expr.walk(ctx, span.clone())?;
        let source_type = expr.get_type();

        ConvertMap
            .get(&source_type.to_string())
            .ok_or_else(|| {
                SemanticError::invalid_type_cast(
                    source_type.to_string(),
                    self.target_type.to_string(),
                    span.clone(),
                )
            })
            .and_then(|valid_targets| {
                if valid_targets.contains(&self.target_type) {
                    Ok(())
                } else {
                    Err(SemanticError::invalid_type_cast(
                        source_type.to_string(),
                        self.target_type.to_string(),
                        span.clone(),
                    ))
                }
            })?;

        Ok(TypedAsExpression {
            expr: Box::new(expr),
            target_type: self.target_type.clone(),
        })
    }
}
