use crate::ast::Types;
use crate::ast::{AsExpression, WalkAst};
use hades_error::SemanticError;
use crate::typed_ast::TypedAsExpression;

const CONVERT_MAP: phf::Map<&'static str, &'static [Types]> = phf::phf_map! {
    "int" => &[Types::Float],
    "float" => &[Types::Int],
    "char" => &[Types::Int, Types::Float],
};

impl WalkAst for AsExpression {
    type Output = TypedAsExpression;
    fn walk(
        &self,
        ctx: &mut crate::ast::walk::CompilerContext,
        span: hades_error::Span,
    ) -> Result<Self::Output, hades_error::SemanticError> {
        let expr = self.expr.walk(ctx, span.clone())?;
        let source_type = expr.get_type();

        CONVERT_MAP
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
