use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::{CodegenVisitor, Visit};
use crate::typed_ast::{ModuleSignatures, TypedModule, TypedProgram};
use inkwell::module::Linkage;
use inkwell::types::BasicType;
use inkwell::{
    targets::{CodeModel, FileType, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};

impl Visit for TypedProgram {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let visitor = CodegenVisitor::new();
        visitor.visit_program(self, context)
    }
}

pub fn declare_imports<'ctx>(
    sigs: &[&ModuleSignatures],
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<()> {
    for sig in sigs {
        for (name, fn_sig) in &sig.functions {
            if context.module().get_function(name.inner()).is_some() {
                continue;
            }
            let symbols = context.symbols();
            let param_types = context
                .type_converter()
                .params_to_llvm_types(fn_sig, symbols)?;
            let fn_type = if fn_sig.return_type == crate::ast::Types::Void {
                context
                    .type_converter()
                    .void_type()
                    .fn_type(&param_types, false)
            } else {
                let symbols = context.symbols();
                let ret = context
                    .type_converter()
                    .to_llvm_type(&fn_sig.return_type, symbols)?;
                ret.fn_type(&param_types, false)
            };
            context
                .module()
                .add_function(name.inner(), fn_type, Some(Linkage::External));
        }
    }
    Ok(())
}

pub fn codegen_module<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
) -> CodegenResult<LLVMContext<'ctx>> {
    let llvm_module = llvm_ctx.create_module(&typed_module.path.to_string());
    let mut context = LLVMContext::new(&typed_module.ctx, llvm_ctx, llvm_module);
    declare_imports(import_sigs, &mut context)?;
    typed_module.program.visit(&mut context)?;
    Ok(context)
}

pub fn emit_module_ir<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
) -> CodegenResult<String> {
    let context = codegen_module(typed_module, import_sigs, llvm_ctx)?;
    Ok(context.module().print_to_string().to_string())
}

pub fn compile_module_to_object<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
    output_path: &std::path::Path,
) -> CodegenResult<()> {
    let context = codegen_module(typed_module, import_sigs, llvm_ctx)?;

    Target::initialize_all(&inkwell::targets::InitializationConfig::default());

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| CodegenError::LLVMBuild {
        message: format!("Failed to get target from triple: {e}"),
    })?;

    let target_machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| CodegenError::LLVMBuild {
            message: "Failed to create target machine".to_string(),
        })?;

    target_machine
        .write_to_file(context.module(), FileType::Object, output_path)
        .map_err(|e| CodegenError::LLVMBuild {
            message: format!("Failed to write object file: {e}"),
        })?;

    Ok(())
}
