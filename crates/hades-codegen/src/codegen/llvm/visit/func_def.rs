use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::Visit;
use hades_ast::{FuncKind, Types};
use hades_mir::mir::func::MirFunction;
use hades_mir::mir::block::BlockId;
use inkwell::values::FunctionValue;

impl Visit for MirFunction {
    type Output<'ctx> = FunctionValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let signature = &self.signature;

        let param_types = context
            .type_converter()
            .params_to_llvm_types(signature, context.module())?;

        match &signature.kind {
            FuncKind::Extern { variadic } => {
                let fn_type =
                    context.build_fn_type(&signature.return_type, &param_types, *variadic)?;
                let function = context.module().add_function(
                    self.name.link_name(),
                    fn_type,
                    Some(inkwell::module::Linkage::External),
                );
                return Ok(function);
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
                return Ok(function);
            }
            FuncKind::Normal => {}
        }

        // ── Normal function ──────────────────────────────────────────────────

        let fn_type = context.build_fn_type(&signature.return_type, &param_types, false)?;
        let function = context
            .module()
            .add_function(self.name.inner(), fn_type, None);

        context.set_current_function(function);

        let entry_block = context.create_basic_block("entry");
        context.position_at_end(entry_block);

        // 1. Allocate one alloca per MIR local (including _0 return slot).
        let mut alloca_map = Vec::with_capacity(self.locals.len());
        for (i, local_decl) in self.locals.iter().enumerate() {
            if local_decl.ty == Types::Void {
                // Void return slot — push a null placeholder pointer.
                alloca_map.push(context.context().ptr_type(inkwell::AddressSpace::default()).const_null());
                continue;
            }
            // Array-typed locals hold a pointer to the array data (global or heap)
            // rather than a stack copy, so we avoid large stack allocations.
            let llvm_type = if matches!(local_decl.ty, Types::Array(_)) {
                context.context().ptr_type(inkwell::AddressSpace::default()).into()
            } else {
                context
                    .type_converter()
                    .to_llvm_type(&local_decl.ty, context.module())?
            };
            let name = format!("_{}", i);
            let alloca = context.create_alloca(&name, llvm_type)?;
            alloca_map.push(alloca);
        }

        // 2. Store each parameter LLVM value into its alloca (_1.._N).
        // Iterate ALL params (including self) to match MIR local indices.
        let all_params: Vec<_> = signature.params().into_iter().collect();
        for (i, (param_kind, _ty)) in all_params.iter().enumerate() {
            use hades_tokens::ParamKind;
            let param_val = function.get_nth_param(i as u32).unwrap();
            let local_idx = i + 1; // _0 is return slot
            match param_kind {
                ParamKind::Self_(_) => {
                    let local_ty = &self.locals[local_idx].ty;
                    if matches!(local_ty, Types::Pointer(_)) {
                        // self: &Self — the param is already the pointer; store it directly.
                        context
                            .builder()
                            .build_store(alloca_map[local_idx], param_val)
                            .map_err(|_| CodegenError::LLVMBuild {
                                message: "Failed to store self pointer param".to_string(),
                            })?;
                    } else {
                        // self: Self (value receiver) — load the struct value from the pointer
                        // and store it into the struct-typed alloca.
                        let local_llvm_ty = context
                            .type_converter()
                            .to_llvm_type(local_ty, context.module())?;
                        let struct_val = context
                            .load(param_val.into_pointer_value(), local_llvm_ty, "self_val")?;
                        context
                            .builder()
                            .build_store(alloca_map[local_idx], struct_val)
                            .map_err(|_| CodegenError::LLVMBuild {
                                message: "Failed to store self param".to_string(),
                            })?;
                    }
                }
                ParamKind::Ident(_) => {
                    context
                        .builder()
                        .build_store(alloca_map[local_idx], param_val)
                        .map_err(|_| CodegenError::LLVMBuild {
                            message: format!("Failed to store param _{}", local_idx),
                        })?;
                }
            }
        }

        // 3. Pre-create one LLVM BasicBlock per MIR BlockId.
        let mut llvm_blocks = Vec::with_capacity(self.cfg.blocks.len());
        for (i, _block) in self.cfg.blocks.iter().enumerate() {
            let bb = context.create_basic_block(&format!("bb{}", i));
            llvm_blocks.push(bb);
        }

        // Store alloca_map, llvm_blocks, and locals in the function context.
        {
            let fc = context.current_fn_mut_unchecked();
            fc.alloca_map = alloca_map;
            fc.llvm_blocks = llvm_blocks;
            fc.locals = self.locals.clone();
        }

        // Branch from entry into the MIR entry block.
        let entry_mir_block = self.cfg.entry;
        let entry_llvm_block = context.current_function_unchecked().llvm_blocks[entry_mir_block.index()];
        context.build_unconditional_branch(entry_llvm_block)?;

        // 4. Walk each MIR block.
        for mir_block in &self.cfg.blocks {
            let llvm_bb = context.current_function_unchecked().llvm_blocks[mir_block.id.index()];
            context.position_at_end(llvm_bb);

            // Emit statements.
            for stmt in &mir_block.stmts {
                stmt.visit(context)?;
            }

            // Emit terminator.
            if let Some(term) = &mir_block.terminator {
                term.visit(context)?;
            }
        }

        // Patch: if the return type is Void and _0 is Void, make sure every
        // block that falls through has a return.
        if self.return_type == Types::Void {
            for mir_block in &self.cfg.blocks {
                let llvm_bb = context.current_function_unchecked().llvm_blocks[mir_block.id.index()];
                if llvm_bb.get_terminator().is_none() {
                    context.position_at_end(llvm_bb);
                    context.build_return(None)?;
                }
            }
        }

        context.clear_current_function();
        Ok(function)
    }
}
