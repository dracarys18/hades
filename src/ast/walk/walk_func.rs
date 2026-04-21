use crate::ast::{FuncBody, FuncDef, ReceiverKind, Types, WalkAst};
use crate::consts::ENTRY_POINT;
use crate::error::SemanticError;
use crate::tokens::{Ident, Name, ParamKind};
use crate::typed_ast::{
    CompilerContext, FunctionSignature, TypedBlock, TypedFuncDef, TypedReceiver, TypedStmt,
};
use indexmap::IndexMap;

impl FuncDef {
    pub fn register(&self, ctx: &mut CompilerContext) -> Result<(), SemanticError> {
        let name = self.full_name(ctx);
        let params_map = self
            .params
            .iter()
            .map(|(k, v)| (k.clone(), v.qualify(ctx.module_name())))
            .collect::<IndexMap<_, _>>();
        let receiver = self.receiver.as_ref().map(|r| {
            let qualified_struct = r.struct_name.full_name_optional(ctx.module_name());
            TypedReceiver {
                struct_name: qualified_struct.clone(),
                kind: r.kind.clone(),
                typ: match r.kind {
                    ReceiverKind::Value => Types::Struct(qualified_struct),
                    ReceiverKind::Pointer => {
                        Types::Pointer(Box::new(Types::Struct(qualified_struct)))
                    }
                },
            }
        });
        let qualified_return = self.return_type.qualify(ctx.module_name());
        let sig = match &self.body {
            FuncBody::Extern { variadic } => {
                FunctionSignature::new_extern(params_map, qualified_return, *variadic)
            }
            FuncBody::Intrinsic(llvm_name) => {
                FunctionSignature::new_intrinsic(params_map, qualified_return, llvm_name.clone())
            }
            FuncBody::Block(_) => FunctionSignature::new(params_map, qualified_return, receiver),
        };
        ctx.register_function(name, sig)
    }

    fn full_name(&self, ctx: &CompilerContext) -> Name {
        match &self.receiver {
            Some(r) => {
                let bare_struct = Ident::new(
                    r.struct_name.link_name().to_string(),
                    self.name.span().clone(),
                );
                let mangled = self.name.mangle(&bare_struct);
                mangled.full_name_optional(ctx.module_name())
            }
            None => {
                if self.name.inner() == ENTRY_POINT {
                    self.name.clone()
                } else {
                    self.name.full_name_optional(ctx.module_name())
                }
            }
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
                Ok(TypedFuncDef {
                    name,
                    signature: sig,
                    body: None,
                    span: self.span.clone(),
                })
            }
            FuncBody::Intrinsic(_) => {
                if !ctx.is_stdlib() {
                    return Err(SemanticError::intrinsic_outside_stdlib(
                        self.name.link_name().to_string(),
                        self.span.clone(),
                    ));
                }
                Ok(TypedFuncDef {
                    name,
                    signature: sig,
                    body: None,
                    span: self.span.clone(),
                })
            }
            FuncBody::Block(block) => {
                ctx.set_current_function(name.clone(), self.return_type.qualify(ctx.module_name()));

                for (param, declared_type) in &self.params {
                    let resolved_type = match param {
                        ParamKind::Self_(_) => {
                            sig.receiver()
                                .ok_or_else(|| {
                                    SemanticError::self_outside_method(param.span().clone())
                                })?
                                .typ
                        }
                        ParamKind::Ident(_) => declared_type.qualify(ctx.module_name()),
                    };
                    ctx.insert_variable(param.name(), resolved_type);
                }

                let typed_body = block.walk(ctx, self.span.clone())?;

                if !self.return_type.eq(&Types::Void) {
                    check_return_path(&typed_body)?;
                }

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

fn check_return_path(body: &TypedBlock) -> Result<(), SemanticError> {
    for stmt in &body.stmts {
        match stmt {
            TypedStmt::Return(_) => return Ok(()),
            TypedStmt::If(if_stmt) => {
                if check_return_path(&if_stmt.then_branch).is_ok()
                    && let Some(else_branch) = &if_stmt.else_branch
                        && check_return_path(else_branch).is_ok() {
                            return Ok(());
                        }
            }
            TypedStmt::Block(block) => {
                if check_return_path(block).is_ok() {
                    return Ok(());
                }
            }
            _ => {}
        }
    }
    Err(SemanticError::missing_return(body.span.clone()))
}
