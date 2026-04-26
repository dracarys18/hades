use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::Visit;
use hades_ast::TypedStmt;
use hades_mir::mir::module::MirModule;
use inkwell::{
    OptimizationLevel,
    targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
    },
};
use std::path::Path;

impl Visit for MirModule {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        // 1. Declare imports.
        let import_sig_refs: Vec<&hades_semantic::ModuleSignatures> =
            self.import_sigs.iter().collect();
        context.declare_imports(&import_sig_refs)?;

        // 2. Declare struct types and their methods from the typed program.
        for stmt in self.program.iter() {
            if let TypedStmt::StructDef(struct_def) = stmt {
                struct_def.visit(context)?;
            }
        }

        // 3. Visit each MIR function (free functions + methods).
        for mir_fn in &self.functions {
            mir_fn.visit(context)?;
        }

        Ok(())
    }
}

fn build<'ctx>(
    mir_module: &'ctx MirModule,
    context: &'ctx inkwell::context::Context,
) -> CodegenResult<LLVMContext<'ctx>> {
    let module = context.create_module(mir_module.path.name());
    let mut llvm_ctx = LLVMContext::new(&mir_module.ctx, context, module);
    mir_module.visit(&mut llvm_ctx)?;
    Ok(llvm_ctx)
}

pub fn emit_ir<'ctx>(
    mir_module: &'ctx MirModule,
    context: &'ctx inkwell::context::Context,
) -> CodegenResult<String> {
    let llvm_ctx = build(mir_module, context)?;
    Ok(llvm_ctx.module().print_to_string().to_string())
}

pub fn compile<'ctx>(
    mir_module: &'ctx MirModule,
    context: &'ctx inkwell::context::Context,
    output_path: &Path,
) -> CodegenResult<()> {
    let llvm_ctx = build(mir_module, context)?;

    Target::initialize_all(&InitializationConfig::default());

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| CodegenError::LLVMBuild {
        message: format!("Failed to get target from triple: {}", e),
    })?;
    let cpu = TargetMachine::get_host_cpu_name();
    let features = TargetMachine::get_host_cpu_features();
    let machine = target
        .create_target_machine(
            &triple,
            cpu.to_str().unwrap_or(""),
            features.to_str().unwrap_or(""),
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| CodegenError::LLVMBuild {
            message: "Failed to create target machine".to_string(),
        })?;

    machine
        .write_to_file(llvm_ctx.module(), FileType::Object, output_path)
        .map_err(|e| CodegenError::LLVMBuild {
            message: format!("Failed to write object file: {}", e),
        })?;

    Ok(())
}
