use crate::ast::{FuncDef, Types, WalkAst};
use crate::error::SemanticError;
use crate::tokens::ParamKind;
use crate::typed_ast::{CompilerContext, FunctionSignature, TypedFuncDef};
use indexmap::IndexMap;

impl FuncDef {
    pub fn register(&self, ctx: &mut CompilerContext) -> Result<(), SemanticError> {
        let effective_name = match &self.parent_struct {
            Some(s) => self.name.mangle(s),
            None => self.name.clone(),
        };
        let params_map = self
            .params
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<IndexMap<_, _>>();
        let receiver = self
            .parent_struct
            .as_ref()
            .map(|s| Types::Struct(s.clone()));
        let sig = FunctionSignature::new(params_map, self.return_type.clone(), receiver);
        ctx.register_function(effective_name, sig)
    }
}

impl WalkAst for FuncDef {
    type Output = TypedFuncDef;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let effective_name = match &self.parent_struct {
            Some(s) => self.name.mangle(s),
            None => self.name.clone(),
        };

        let sig = if self.parent_struct.is_some() {
            let s = ctx.get_function_signature(&effective_name)?.clone();
            ctx.set_current_function(effective_name.clone(), self.return_type.clone());
            s
        } else {
            let params_map = self
                .params
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<IndexMap<_, _>>();
            let sig = FunctionSignature::new(params_map, self.return_type.clone(), None);
            ctx.enter_function(effective_name.clone(), sig.clone())?;
            sig
        };

        for (param, declared_type) in &self.params {
            let resolved_type = match param {
                ParamKind::Self_(_) => match &self.parent_struct {
                    Some(s) => Types::Struct(s.clone()),
                    None => return Err(SemanticError::self_outside_method(param.span().clone())),
                },
                ParamKind::Ident(_) => declared_type.clone(),
            };
            ctx.insert_variable(param.name(), resolved_type);
        }

        let typed_body = self.body.walk(ctx, self.span.clone())?;
        ctx.exit_function();

        Ok(TypedFuncDef {
            name: effective_name,
            signature: sig,
            body: typed_body,
            span: self.span.clone(),
        })
    }
}
