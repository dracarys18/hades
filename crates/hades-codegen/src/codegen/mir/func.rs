use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::{ReceiverKind, Types};
use hades_mir::mir::func::MirFunction;
use hades_mir::visit::Visit;
use inkwell::basic_block::BasicBlock as LlvmBlock;
use inkwell::values::PointerValue;

impl<'ctx> Visit<LLVMContext<'ctx>> for MirFunction {
    type Output = ();

    fn visit(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output> {
        let function = context.module().get_function(self.name.inner()).ok_or(
            CodegenError::FunctionNotFound {
                name: self.name.inner().to_string(),
            },
        )?;

        context.set_current_function(function);

        let entry = context.create_basic_block("entry");
        context.position_at_end(entry);

        let mut locals: Vec<(PointerValue<'ctx>, Types)> =
            Vec::with_capacity(self.guard.locals.len());
        for local in &self.guard.locals {
            if local.typ == Types::Void {
                locals.push((
                    context.type_converter().ptr_type().const_null(),
                    local.typ.clone(),
                ));
                continue;
            }
            let llvm_type = context
                .type_converter()
                .to_llvm_type(&local.typ, context.module())?;
            let alloca = context.create_alloca(local.name.inner(), llvm_type)?;
            locals.push((alloca, local.typ.clone()));
        }

        for i in 0..self.arg_count {
            let local_idx = i + 1;
            let param_val = function
                .get_nth_param(i as u32)
                .expect("parameter count mismatch");
            let local = &self.guard.locals[local_idx];
            param_val.set_name(local.name.inner());

            let is_self =
                local_idx == 1 && self.receiver.is_some() && local.name.inner() == hades_tokens::SELF;
            if is_self && self.receiver.as_ref().unwrap().kind == ReceiverKind::Value {
                locals[local_idx].0 = param_val.into_pointer_value();
                continue;
            }

            let ty = locals[local_idx].1.clone();
            context.create_store(locals[local_idx].0, param_val, &ty)?;
        }

        context.set_mir_locals(locals);

        let mut blocks: Vec<LlvmBlock<'ctx>> = Vec::with_capacity(self.guard.basic_blocks.len());
        for i in 0..self.guard.basic_blocks.len() {
            blocks.push(context.create_basic_block(&format!("bb{i}")));
        }

        if let Some(first) = blocks.first() {
            context.build_unconditional_branch(*first)?;
        }

        context.set_mir_blocks(blocks);

        for (i, bb) in self.guard.basic_blocks.iter().enumerate() {
            let llvm_block = context.mir_block(i);
            context.position_at_end(llvm_block);

            for stmt in &bb.stmts {
                stmt.visit(context)?;
            }

            match &bb.terminator {
                Some(term) => term.visit(context)?,
                None => {
                    context
                        .builder()
                        .build_unreachable()
                        .map(|_| ())
                        .map_err(CodegenError::from)?;
                }
            }
        }

        context.clear_current_function();
        Ok(())
    }
}
