use crate::ast::{FuncDef, ReceiverKind, Types, WalkAst};
use crate::consts::ENTRY_POINT;
use crate::error::SemanticError;
use crate::tokens::{FunctionName, ParamKind};
use crate::typed_ast::{CompilerContext, FunctionSignature, TypedFuncDef, TypedReceiver};
use indexmap::IndexMap;

impl FuncDef {
    pub fn register(&self, ctx: &mut CompilerContext) -> Result<(), SemanticError> {
        let name = self.full_name(ctx);
        let params_map = self
            .params
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<IndexMap<_, _>>();
        let receiver = self.receiver.as_ref().map(|r| TypedReceiver {
            struct_name: r.struct_name.clone(),
            kind: r.kind.clone(),
            typ: match r.kind {
                ReceiverKind::Value => Types::Struct(r.struct_name.clone()),
                ReceiverKind::Pointer => {
                    Types::Pointer(Box::new(Types::Struct(r.struct_name.clone())))
                }
            },
        });
        let sig = FunctionSignature::new(params_map, self.return_type.clone(), receiver);
        ctx.register_function(name, sig)
    }

    fn full_name(&self, ctx: &CompilerContext) -> FunctionName {
        let base = match &self.receiver {
            Some(r) => self.name.mangle(&r.struct_name),
            None => self.name.clone(),
        };
        if base.inner() == ENTRY_POINT {
            return base;
        }
        ctx.module_name().map(|m| base.full_name(m)).unwrap_or(base)
    }
}

impl WalkAst for FuncDef {
    type Output = TypedFuncDef;
    fn walk(
        &self,
        ctx: &mut CompilerContext,
        _span: crate::error::Span,
    ) -> Result<Self::Output, SemanticError> {
        let name = self.full_name(ctx);

        if self.receiver.is_none() {
            self.register(ctx)?;
        }

        let sig = ctx.get_function_signature(&name)?.clone();
        ctx.set_current_function(name.clone(), self.return_type.clone());

        for (param, declared_type) in &self.params {
            let resolved_type = match param {
                ParamKind::Self_(_) => {
                    sig.receiver()
                        .ok_or_else(|| SemanticError::self_outside_method(param.span().clone()))?
                        .typ
                }
                ParamKind::Ident(_) => declared_type.clone(),
            };
            ctx.insert_variable(param.name(), resolved_type);
        }

        let typed_body = self.body.walk(ctx, self.span.clone())?;
        ctx.exit_function();

        Ok(TypedFuncDef {
            name,
            signature: sig,
            body: typed_body,
            span: self.span.clone(),
        })
    }
}
