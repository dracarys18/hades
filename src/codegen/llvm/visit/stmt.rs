use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::Visit;
use crate::typed_ast::{
    FuncKind, TypedBlock, TypedBreak, TypedContinue, TypedFieldKind, TypedFor, TypedFuncDef,
    TypedIf, TypedReturn, TypedStmt, TypedStructDef, TypedWhile,
};

impl Visit for TypedStmt {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Self::Let(let_stmt) => let_stmt.visit(context),
            Self::TypedExpr(expr) => expr.visit(context),
            Self::If(if_stmt) => if_stmt.visit(context),
            Self::While(while_stmt) => while_stmt.visit(context),
            Self::For(for_stmt) => for_stmt.visit(context),
            Self::Block(block) => block.visit(context),
            Self::Return(return_stmt) => return_stmt.visit(context),
            Self::Continue(continue_stmt) => continue_stmt.visit(context),
            Self::FuncDef(func_def) => {
                func_def.visit(context)?;
                Ok(())
            }
            Self::StructDef(struct_def) => struct_def.visit(context),
            Self::Break(break_stmt) => break_stmt.visit(context),
            Self::ModuleDecl(_) => Ok(()),
            Self::Import(_) => Ok(()),
        }
    }
}

impl Visit for TypedBlock {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        for stmt in &self.stmts.0 {
            stmt.visit(context)?;
            if context.is_block_terminated() {
                break;
            }
        }
        Ok(())
    }
}

impl Visit for TypedIf {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let cond_val = self.cond.expr().visit(context)?;
        let cond_int = cond_val.value()?.into_int_value();

        let then_block = context.create_basic_block("if.then");
        let else_block = context.create_basic_block("if.else");
        let merge_block = context.create_basic_block("if.merge");

        let final_else_block = if self.else_branch.is_some() {
            else_block
        } else {
            merge_block
        };

        context.build_conditional_branch(cond_int.into(), then_block, final_else_block)?;

        context.position_at_end(then_block);
        self.then_branch.visit(context)?;
        if !context.is_block_terminated() {
            context.build_unconditional_branch(merge_block)?;
        }

        if let Some(else_branch) = &self.else_branch {
            context.position_at_end(else_block);
            else_branch.visit(context)?;
            if !context.is_block_terminated() {
                context.build_unconditional_branch(merge_block)?;
            }
        }

        context.position_at_end(merge_block);
        Ok(())
    }
}

impl Visit for TypedWhile {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let loop_header = context.create_basic_block("while.header");
        let loop_body = context.create_basic_block("while.body");
        let loop_exit = context.create_basic_block("while.exit");

        context.build_unconditional_branch(loop_header)?;
        context.position_at_end(loop_header);

        let cond_val = self.cond.visit(context)?;
        let cond_int = cond_val.value()?.into_int_value();

        context.build_conditional_branch(cond_int.into(), loop_body, loop_exit)?;

        context.position_at_end(loop_body);
        context.push_loop(loop_header, loop_exit);
        self.body.visit(context)?;
        context.pop_loop();

        if !context.is_block_terminated() {
            context.build_unconditional_branch(loop_header)?;
        }

        context.position_at_end(loop_exit);
        Ok(())
    }
}

impl Visit for TypedFor {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        self.init.visit(context)?;

        let loop_header = context.create_basic_block("for.header");
        let loop_body = context.create_basic_block("for.body");
        let loop_update = context.create_basic_block("for.update");
        let loop_exit = context.create_basic_block("for.exit");

        context.build_unconditional_branch(loop_header)?;
        context.position_at_end(loop_header);

        let cond_val = self.cond.visit(context)?;
        let cond_int = cond_val.value()?.into_int_value();

        context.build_conditional_branch(cond_int.into(), loop_body, loop_exit)?;

        context.position_at_end(loop_body);
        context.push_loop(loop_update, loop_exit);
        self.body.visit(context)?;
        context.pop_loop();

        if !context.is_block_terminated() {
            context.build_unconditional_branch(loop_update)?;
        }

        context.position_at_end(loop_update);
        self.update.visit(context)?;
        context.build_unconditional_branch(loop_header)?;

        context.position_at_end(loop_exit);
        Ok(())
    }
}

impl Visit for TypedReturn {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match &self.expr {
            Some(expr) => {
                let return_val = expr.expr().visit(context)?;
                context.build_return(Some(return_val.value()?))?;
            }
            None => {
                context.build_return(None)?;
            }
        }
        Ok(())
    }
}

impl Visit for TypedContinue {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let loop_ctx = context
            .current_loop()
            .ok_or_else(|| CodegenError::LLVMBuild {
                message: "Continue statement outside of loop".to_string(),
            })?;

        let continue_block = loop_ctx.continue_block;
        context.build_unconditional_branch(continue_block)?;
        Ok(())
    }
}

impl Visit for TypedBreak {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let loop_ctx = context
            .current_loop()
            .ok_or_else(|| CodegenError::LLVMBuild {
                message: "Break statement outside of loop".to_string(),
            })?;

        let break_block = loop_ctx.break_block;
        context.build_unconditional_branch(break_block)?;
        Ok(())
    }
}

impl Visit for TypedFuncDef {
    type Output<'ctx> = inkwell::values::FunctionValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let symbols = context.symbols();
        let signature = self.signature.clone();

        let param_types = context
            .type_converter()
            .params_to_llvm_types(&signature, symbols)?;

        match &signature.kind {
            FuncKind::Extern { variadic } => {
                let fn_type =
                    context.build_fn_type(&signature.return_type, &param_types, *variadic)?;
                let function = context.module().add_function(
                    self.name.link_name(),
                    fn_type,
                    Some(inkwell::module::Linkage::External),
                );
                Ok(function)
            }
            FuncKind::Intrinsic(llvm_name) => {
                let type_slice: Vec<inkwell::types::BasicTypeEnum> = param_types
                    .iter()
                    .map(|t| {
                        inkwell::types::BasicTypeEnum::try_from(*t)
                            .expect("param type is not a basic type")
                    })
                    .collect();
                let intrinsic =
                    inkwell::intrinsics::Intrinsic::find(llvm_name).ok_or_else(|| {
                        CodegenError::LLVMBuild {
                            message: format!("LLVM intrinsic '{}' not found", llvm_name),
                        }
                    })?;
                let function = intrinsic
                    .get_declaration(context.module(), &type_slice)
                    .ok_or_else(|| CodegenError::LLVMBuild {
                        message: format!("Failed to get declaration for '{}'", llvm_name),
                    })?;
                Ok(function)
            }
            FuncKind::Normal => {
                let fn_type = context.build_fn_type(&signature.return_type, &param_types, false)?;

                let function = context
                    .module()
                    .add_function(self.name.inner(), fn_type, None);

                context.set_current_function(function);

                let entry_block = context.create_basic_block("entry");
                context.position_at_end(entry_block);

                let params = signature.to_fixed_params();
                for (i, (param, declared_type)) in params.iter().enumerate() {
                    let param_val = function.get_nth_param(i as u32).unwrap();
                    let name = param.name();
                    param_val.set_name(name.inner());

                    match param {
                        crate::tokens::ParamKind::Self_(_) => {
                            let typed_receiver = signature
                                .receiver()
                                .expect("Self_ param but no receiver on signature");
                            match typed_receiver.kind {
                                crate::ast::ReceiverKind::Pointer => {
                                    let symbols = context.symbols();
                                    let llvm_type = context
                                        .type_converter()
                                        .to_llvm_type(&typed_receiver.typ, symbols)?;
                                    let alloca = context.create_alloca(name.inner(), llvm_type)?;
                                    context.create_store(alloca, param_val, &typed_receiver.typ)?;
                                    context.declare_variable(name, alloca, typed_receiver.typ)?;
                                }
                                crate::ast::ReceiverKind::Value => {
                                    context.declare_variable(
                                        name,
                                        param_val.into_pointer_value(),
                                        typed_receiver.typ,
                                    )?;
                                }
                            }
                        }
                        crate::tokens::ParamKind::Ident(_) => {
                            let typ = declared_type.clone();
                            let symbols = context.symbols();
                            let llvm_type = context.type_converter().to_llvm_type(&typ, symbols)?;
                            let alloca = context.create_alloca(name.inner(), llvm_type)?;
                            context.create_store(alloca, param_val, &typ)?;
                            context.declare_variable(name, alloca, typ)?;
                        }
                    }
                }

                let body = self.body.as_ref().expect("Normal function has no body");
                body.visit(context)?;

                if !context.is_block_terminated() {
                    if self.signature.return_type == crate::ast::Types::Void {
                        context.build_return(None)?;
                    } else {
                        let default_val = match self.signature.return_type {
                            crate::ast::Types::Int => {
                                context.context().i64_type().const_zero().into()
                            }
                            crate::ast::Types::Float => {
                                context.context().f64_type().const_zero().into()
                            }
                            crate::ast::Types::Bool => {
                                context.context().bool_type().const_zero().into()
                            }
                            _ => {
                                return Err(CodegenError::LLVMBuild {
                                    message: format!(
                                        "Cannot generate default return value for type {:?}",
                                        self.signature.return_type
                                    ),
                                });
                            }
                        };
                        context.build_return(Some(default_val))?;
                    }
                }

                context.clear_current_function();
                Ok(function)
            }
        }
    }
}

impl Visit for TypedStructDef {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let opaque_struct = context.context().opaque_struct_type(self.name.inner());
        opaque_struct.set_body(
            &self
                .fields
                .iter()
                .filter_map(|(_, field)| match field {
                    TypedFieldKind::Var(typ) => {
                        let symbols = context.symbols();
                        Some(context.type_converter().to_llvm_type(typ, symbols).ok()?)
                    }
                    TypedFieldKind::Func(_) => None,
                })
                .collect::<Vec<_>>(),
            false,
        );

        self.fields
            .iter()
            .filter(|(_, field)| matches!(field, TypedFieldKind::Func(_)))
            .try_for_each(|(name, field)| {
                if let TypedFieldKind::Func(method) = field {
                    method.visit(context)?;
                }
                Ok(())
            })
    }
}
