use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::Visit;
use hades_ast::{FuncKind, TypedFuncDef, TypedReturn};

impl Visit for TypedFuncDef {
    type Output<'ctx> = inkwell::values::FunctionValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let signature = self.signature.clone();

        let param_types = context
            .type_converter()
            .params_to_llvm_types(&signature, context.module())?;

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
                        hades_tokens::ParamKind::Self_(_) => {
                            let typed_receiver = signature
                                .receiver()
                                .expect("Self_ param but no receiver on signature");
                            match typed_receiver.kind {
                                hades_ast::ReceiverKind::Pointer => {
                                    let llvm_type = context
                                        .type_converter()
                                        .to_llvm_type(&typed_receiver.typ, context.module())?;
                                    let alloca = context.create_alloca(name.inner(), llvm_type)?;
                                    context.create_store(alloca, param_val, &typed_receiver.typ)?;
                                    context.declare_variable(name, alloca, typed_receiver.typ)?;
                                }
                                hades_ast::ReceiverKind::Value => {
                                    context.declare_variable(
                                        name,
                                        param_val.into_pointer_value(),
                                        typed_receiver.typ,
                                    )?;
                                }
                            }
                        }
                        hades_tokens::ParamKind::Ident(_) => {
                            let typ = declared_type.clone();
                            let llvm_type = context
                                .type_converter()
                                .to_llvm_type(&typ, context.module())?;
                            let alloca = context.create_alloca(name.inner(), llvm_type)?;
                            context.create_store(alloca, param_val, &typ)?;
                            context.declare_variable(name, alloca, typ)?;
                        }
                    }
                }

                let body = self.body.as_ref().expect("Normal function has no body");
                body.visit(context)?;

                if !context.is_block_terminated()
                    && self.signature.return_type == hades_ast::Types::Void
                {
                    TypedReturn::void(self.span.clone()).visit(context)?;
                }

                context.clear_current_function();
                Ok(function)
            }
        }
    }
}
