use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::{CodegenVisitor, Visit};
use crate::semantic::analyzer::{Analyzer, Prepared};
use crate::typed_ast::TypedProgram;
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

impl Analyzer<Prepared> {
    pub fn generate_code<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        self.ast().visit(context)
    }

    pub fn compile_to_llvm<'ctx>(
        &'ctx self,
        llvm_context: &'ctx inkwell::context::Context,
        module_name: &str,
    ) -> CodegenResult<LLVMContext<'ctx>> {
        let mut context = LLVMContext::new(self.ctx(), llvm_context, module_name);
        self.generate_code(&mut context)?;
        Ok(context)
    }

    pub fn generate_ir<'ctx>(
        &self,
        llvm_context: &'ctx inkwell::context::Context,
        module_name: &str,
    ) -> CodegenResult<String> {
        let context = self.compile_to_llvm(llvm_context, module_name)?;
        Ok(context.module().print_to_string().to_string())
    }

    pub fn verify_module(
        &self,
        llvm_context: &inkwell::context::Context,
        module_name: &str,
    ) -> CodegenResult<bool> {
        let context = self.compile_to_llvm(llvm_context, module_name)?;
        Ok(context.module().verify().is_ok())
    }

    pub fn write_to_file(
        &self,
        llvm_context: &inkwell::context::Context,
        module_name: &str,
        file_path: &std::path::Path,
    ) -> CodegenResult<()> {
        let ir = self.generate_ir(llvm_context, module_name)?;
        std::fs::write(file_path, ir).map_err(|e| {
            crate::codegen::error::CodegenError::LLVMBuild {
                message: format!("Failed to write IR to file: {e}"),
            }
        })?;
        Ok(())
    }

    pub fn compile_to_object(
        &self,
        llvm_context: &inkwell::context::Context,
        module_name: &str,
        output_path: &std::path::Path,
    ) -> CodegenResult<()> {
        let context = self.compile_to_llvm(llvm_context, module_name)?;

        Target::initialize_all(&inkwell::targets::InitializationConfig::default());

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| {
            crate::codegen::error::CodegenError::LLVMBuild {
                message: format!("Failed to get target from triple: {e}"),
            }
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

    pub fn compile(
        &self,
        llvm_context: &inkwell::context::Context,
        module_name: &str,
        output_path: &std::path::Path,
    ) -> CodegenResult<()> {
        let object_path = output_path.with_extension("o");
        self.compile_to_object(llvm_context, module_name, &object_path)?;

        let status = std::process::Command::new("clang")
            .arg(object_path.to_str().unwrap())
            .arg("-o")
            .arg(output_path.to_str().unwrap())
            .arg("-lc")
            .status()
            .map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to invoke gcc: {e}"),
            })?;

        if !status.success() {
            return Err(CodegenError::LLVMBuild {
                message: "Clang failed to create executable".to_string(),
            });
        }
        Ok(())
    }

    pub fn cleanup(&self, output_path: &std::path::Path) -> CodegenResult<()> {
        let object_path = output_path.with_extension("o");
        if object_path.exists() {
            std::fs::remove_file(&object_path).map_err(|e| CodegenError::LLVMBuild {
                message: format!("Failed to remove object file: {e}"),
            })?;
        }
        Ok(())
    }
}
