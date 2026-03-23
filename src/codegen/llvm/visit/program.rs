use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::{CodegenVisitor, Visit};
use crate::typed_ast::{ModuleSignatures, TypedModule, TypedProgram};
use inkwell::{
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};

impl Visit for TypedProgram {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let visitor = CodegenVisitor::new();
        visitor.visit_program(self, context)
    }
}

fn build<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
) -> CodegenResult<LLVMContext<'ctx>> {
    let llvm_module = llvm_ctx.create_module(&typed_module.path.to_string());
    let mut context = LLVMContext::new(&typed_module.ctx, llvm_ctx, llvm_module);
    context.declare_imports(import_sigs)?;
    typed_module.program.visit(&mut context)?;
    Ok(context)
}

pub fn emit_ir<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
) -> CodegenResult<String> {
    let context = build(typed_module, import_sigs, llvm_ctx)?;
    Ok(context.module().print_to_string().to_string())
}

pub fn compile<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
    output_path: &std::path::Path,
) -> CodegenResult<()> {
    Target::initialize_all(&InitializationConfig::default());

    let context = build(typed_module, import_sigs, llvm_ctx)?;

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
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| CodegenError::LLVMBuild {
            message: "Failed to create target machine".to_string(),
        })?;

    target_machine
        .write_to_file(context.module(), FileType::Object, output_path)
        .map_err(|e| CodegenError::LLVMBuild {
            message: format!("Failed to write object file: {e}"),
        })
}
