use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use crate::codegen::llvm::visit::expr::{codegen_place_ptr, call::build_call};
use hades_ast::Types;
use hades_mir::mir::stmt::{Statement, StatementKind};
use hades_mir::mir::terminator::{CallTarget, Terminator, TerminatorKind};
use inkwell::values::BasicMetadataValueEnum;

impl Visit for Statement {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match &self.kind {
            StatementKind::Nop => Ok(()),
            StatementKind::Assign(place, rvalue) => {
                let val = rvalue.visit(context)?;
                let (dest_ptr, dest_ty) = codegen_place_ptr(place, context)?;
                match val {
                    CodegenValue::Void => Ok(()),
                    _ => context.create_store(dest_ptr, val.value()?, &dest_ty),
                }
            }
        }
    }
}

impl Visit for Terminator {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match &self.kind {
            TerminatorKind::Goto(block_id) => {
                let target = context.current_function_unchecked().llvm_blocks[block_id.index()];
                context.build_unconditional_branch(target)
            }

            TerminatorKind::SwitchInt { discriminant, targets } => {
                let cond_val = discriminant.visit(context)?.value()?;
                // For boolean SwitchInt (values = [1], blocks = [then], otherwise = else).
                let then_block = context.current_function_unchecked().llvm_blocks[targets.blocks[0].index()];
                let else_block = context.current_function_unchecked().llvm_blocks[targets.otherwise.index()];
                context.build_conditional_branch(cond_val, then_block, else_block)
            }

            TerminatorKind::Return => {
                let return_ty = context.current_function_unchecked().locals[0].ty.clone();
                if return_ty == Types::Void {
                    context.build_return(None)
                } else {
                    let ret_alloca = context.current_function_unchecked().alloca_map[0];
                    let llvm_ty = context
                        .type_converter()
                        .to_llvm_type(&return_ty, context.module())?;
                    let val = context.load(ret_alloca, llvm_ty, "ret_val")?;
                    context.build_return(Some(val))
                }
            }

            TerminatorKind::Unreachable => {
                context
                    .builder()
                    .build_unreachable()
                    .map_err(|_| CodegenError::LLVMBuild {
                        message: "Failed to build unreachable".to_string(),
                    })?;
                Ok(())
            }

            TerminatorKind::Call { target, args, destination, successor } => {
                // Evaluate all argument operands.
                let mut arg_values: Vec<BasicMetadataValueEnum> = Vec::new();
                for arg in args {
                    let cv = arg.visit(context)?;
                    arg_values.push(cv.value()?.into());
                }

                let result = match target {
                    CallTarget::Function(name) if name.inner() == "len" => {
                        // `len` is a compile-time builtin: returns the array size as an i64 const.
                        let arg_op = args.first().ok_or(CodegenError::LLVMBuild {
                            message: "len requires one argument".to_string(),
                        })?;
                        let locals = &context.current_function_unchecked().locals;
                        let arr_ty = arg_op.ty(locals);
                        let size = arr_ty.get_array_size();
                        let val = context.context().i64_type().const_int(size as u64, false).into();
                        CodegenValue::new(val, hades_ast::Types::Int)
                    }
                    CallTarget::Function(name) => {
                        build_call(name.inner(), &arg_values, context)?
                    }
                    CallTarget::Method { receiver, method } => {
                        // Prepend the receiver pointer as the first argument.
                        let recv_ptr = {
                            use hades_mir::mir::operand::Operand;
                            match receiver {
                                Operand::Copy(place) | Operand::Ref(place) => {
                                    let (ptr, recv_ty) = codegen_place_ptr(place, context)?;
                                    // Deref if the receiver is a pointer type.
                                    if let Types::Pointer(_) = recv_ty {
                                        let llvm_ptr = context.context().ptr_type(inkwell::AddressSpace::default()).into();
                                        context.load(ptr, llvm_ptr, "recv_deref")?.into_pointer_value()
                                    } else {
                                        ptr
                                    }
                                }
                                Operand::Const(_) => {
                                    return Err(CodegenError::LLVMBuild {
                                        message: "Method receiver cannot be a const operand".to_string(),
                                    });
                                }
                            }
                        };
                        let mut method_args: Vec<BasicMetadataValueEnum> =
                            vec![recv_ptr.into()];
                        method_args.extend_from_slice(&arg_values);
                        build_call(method.inner(), &method_args, context)?
                    }
                    CallTarget::Qualified { ty: _, method } => {
                        build_call(method.inner(), &arg_values, context)?
                    }
                };

                // Store result into destination place (if non-void).
                if !matches!(result, CodegenValue::Void) {
                    let (dest_ptr, dest_ty) = codegen_place_ptr(destination, context)?;
                    context.create_store(dest_ptr, result.value()?, &dest_ty)?;
                }

                // Unconditional branch to successor.
                let succ_block = context.current_function_unchecked().llvm_blocks[successor.index()];
                context.build_unconditional_branch(succ_block)
            }
        }
    }
}
