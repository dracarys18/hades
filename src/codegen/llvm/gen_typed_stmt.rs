use inkwell::types::{AnyType, AnyTypeEnum, FunctionType};

use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::{BlockCodegen, ExprCodegen, FunctionCodegen, StmtCodegen};
use crate::typed_ast::{
    TypedBlock, TypedContinue, TypedFor, TypedFuncDef, TypedIf, TypedReturn, TypedStmt, TypedWhile,
};

impl StmtCodegen for TypedStmt {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        match self {
            Self::Let(let_stmt) => let_stmt.generate_stmt(context),
            Self::TypedExpr(expr) => expr.generate_stmt(context),
            Self::If(if_stmt) => if_stmt.generate_stmt(context),
            Self::While(while_stmt) => while_stmt.generate_stmt(context),
            Self::For(for_stmt) => for_stmt.generate_stmt(context),
            Self::Block(block) => block.generate_block(context),
            Self::Return(return_stmt) => return_stmt.generate_stmt(context),
            Self::Continue(continue_stmt) => continue_stmt.generate_stmt(context),
            Self::FuncDef(func_def) => {
                func_def.generate_function(context)?;
                Ok(())
            }
            Self::StructDef(_) => Ok(()),
        }
    }
}

impl BlockCodegen for TypedBlock {
    fn generate_block<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        for stmt in &self.stmts.0 {
            stmt.generate_stmt(context)?;
            if context.is_block_terminated() {
                break;
            }
        }
        Ok(())
    }
}

impl StmtCodegen for TypedIf {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        let cond_val = self.cond.expr().generate_expr(context)?;
        let cond_int = cond_val.value.into_int_value();

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
        self.then_branch.generate_block(context)?;
        if !context.is_block_terminated() {
            context.build_unconditional_branch(merge_block)?;
        }

        if let Some(else_branch) = &self.else_branch {
            context.position_at_end(else_block);
            else_branch.generate_block(context)?;
            if !context.is_block_terminated() {
                context.build_unconditional_branch(merge_block)?;
            }
        }

        context.position_at_end(merge_block);
        Ok(())
    }
}

impl StmtCodegen for TypedWhile {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        let loop_header = context.create_basic_block("while.header");
        let loop_body = context.create_basic_block("while.body");
        let loop_exit = context.create_basic_block("while.exit");

        context.build_unconditional_branch(loop_header)?;
        context.position_at_end(loop_header);

        let cond_val = self.cond.generate_expr(context)?;
        let cond_int = cond_val.value.into_int_value();

        context.build_conditional_branch(cond_int.into(), loop_body, loop_exit)?;

        context.position_at_end(loop_body);
        context.push_loop(loop_header, loop_exit);
        self.body.generate_block(context)?;
        context.pop_loop();

        if !context.is_block_terminated() {
            context.build_unconditional_branch(loop_header)?;
        }

        context.position_at_end(loop_exit);
        Ok(())
    }
}

impl StmtCodegen for TypedFor {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        self.init.expr().generate_expr(context)?;

        let loop_header = context.create_basic_block("for.header");
        let loop_body = context.create_basic_block("for.body");
        let loop_update = context.create_basic_block("for.update");
        let loop_exit = context.create_basic_block("for.exit");

        context.build_unconditional_branch(loop_header)?;
        context.position_at_end(loop_header);

        let cond_val = self.cond.expr().generate_expr(context)?;
        let cond_int = cond_val.value.into_int_value();

        context.build_conditional_branch(cond_int.into(), loop_body, loop_exit)?;

        context.position_at_end(loop_body);
        context.push_loop(loop_update, loop_exit);
        self.body.generate_block(context)?;
        context.pop_loop();

        if !context.is_block_terminated() {
            context.build_unconditional_branch(loop_update)?;
        }

        context.position_at_end(loop_update);
        self.update.expr().generate_expr(context)?;
        context.build_unconditional_branch(loop_header)?;

        context.position_at_end(loop_exit);
        Ok(())
    }
}

impl StmtCodegen for TypedReturn {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        match &self.expr {
            Some(expr) => {
                let return_val = expr.expr().generate_expr(context)?;
                context.build_return(Some(return_val.value))?;
            }
            None => {
                context.build_return(None)?;
            }
        }
        Ok(())
    }
}

impl StmtCodegen for TypedContinue {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
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

impl FunctionCodegen for TypedFuncDef {
    fn generate_function<'ctx>(
        &self,
        context: &mut LLVMContext<'ctx>,
    ) -> CodegenResult<inkwell::values::FunctionValue<'ctx>> {
        let mut param_types = Vec::new();
        let symbols = context.symbols();
        for (_, param_type) in &self.params {
            let llvm_type = context.type_converter().to_llvm_type(param_type, symbols)?;
            param_types.push(llvm_type.into());
        }

        let return_type: AnyTypeEnum = if self.return_type == crate::ast::Types::Void {
            context.type_converter().void_type().into()
        } else {
            context
                .type_converter()
                .to_llvm_type(&self.return_type, symbols)?
                .as_any_type_enum()
        };

        let fn_type: FunctionType = if self.return_type == crate::ast::Types::Void {
            context
                .type_converter()
                .void_type()
                .fn_type(&param_types, false)
        } else {
            context.type_converter().fn_type(&return_type)
        };

        let function = context
            .module()
            .add_function(self.name.inner(), fn_type, None);

        context.declare_function(self.name.inner().to_string(), function);
        context.set_current_function(function);

        let entry_block = context.create_basic_block("entry");
        context.position_at_end(entry_block);

        for (i, (param_name, param_type)) in self.params.iter().enumerate() {
            let param_val = function.get_nth_param(i as u32).unwrap();
            param_val.set_name(param_name.inner());

            let param_llvm_type = context.type_converter().to_llvm_type(param_type, symbols)?;

            let param_alloca = context.create_alloca(param_name.inner(), param_llvm_type)?;
            context.create_store(param_alloca, param_val)?;
            context.declare_variable(param_name.clone(), param_alloca, param_type.clone())?;
        }

        self.body.generate_block(context)?;

        if !context.is_block_terminated() {
            if self.return_type == crate::ast::Types::Void {
                context.build_return(None)?;
            } else {
                let default_val = match self.return_type {
                    crate::ast::Types::Int => context.context().i32_type().const_zero().into(),
                    crate::ast::Types::Float => context.context().f64_type().const_zero().into(),
                    crate::ast::Types::Bool => context.context().bool_type().const_zero().into(),
                    _ => {
                        return Err(CodegenError::LLVMBuild {
                            message: format!(
                                "Cannot generate default return value for type {:?}",
                                self.return_type
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
