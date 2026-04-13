use crate::ast::{FuncBody, FuncDef, ReceiverKind, Types, WalkAst};
use crate::error::SemanticError;
use crate::tokens::{Ident, Name, ParamKind};
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
        let sig = match &self.body {
            FuncBody::Extern { variadic } => {
                FunctionSignature::new_extern(params_map, self.return_type.clone(), *variadic)
            }
            FuncBody::Intrinsic(llvm_name) => FunctionSignature::new_intrinsic(
                params_map,
                self.return_type.clone(),
                llvm_name.clone(),
            ),
            FuncBody::Block(_) => {
                FunctionSignature::new(params_map, self.return_type.clone(), receiver)
            }
        };
        ctx.register_function(name, sig)
    }

    fn full_name(&self, _ctx: &CompilerContext) -> Name {
        match &self.receiver {
            Some(r) => {
                let bare_struct = Ident::new(
                    r.struct_name.link_name().to_string(),
                    self.name.span().clone(),
                );
                let mangled = self.name.mangle(&bare_struct);
                mangled.full_name_optional(r.struct_name.module())
            }
            None => self.name.clone(),
        }
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

        match &self.body {
            FuncBody::Extern { .. } => {
                if !ctx.is_stdlib() {
                    return Err(SemanticError::extern_outside_stdlib(
                        self.name.link_name().to_string(),
                        self.span.clone(),
                    ));
                }
                return Ok(TypedFuncDef {
                    name,
                    signature: sig,
                    body: None,
                    span: self.span.clone(),
                });
            }
            FuncBody::Intrinsic(_) => {
                if !ctx.is_stdlib() {
                    return Err(SemanticError::intrinsic_outside_stdlib(
                        self.name.link_name().to_string(),
                        self.span.clone(),
                    ));
                }
                return Ok(TypedFuncDef {
                    name,
                    signature: sig,
                    body: None,
                    span: self.span.clone(),
                });
            }
            FuncBody::Block(block) => {
                ctx.set_current_function(name.clone(), self.return_type.clone());

                for (param, declared_type) in &self.params {
                    let resolved_type = match param {
                        ParamKind::Self_(_) => {
                            sig.receiver()
                                .ok_or_else(|| {
                                    SemanticError::self_outside_method(param.span().clone())
                                })?
                                .typ
                        }
                        ParamKind::Ident(_) => declared_type.clone(),
                    };
                    ctx.insert_variable(param.name(), resolved_type);
                }

                let typed_body = block.walk(ctx, self.span.clone())?;
                ctx.exit_function();

                Ok(TypedFuncDef {
                    name,
                    signature: sig,
                    body: Some(typed_body),
                    span: self.span.clone(),
                })
            }
        }
    }
}
