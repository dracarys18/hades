use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::{CodegenVisitor, StmtCodegen};
use crate::semantic::analyzer::{Analyzer, Prepared};
use crate::typed_ast::TypedProgram;
use inkwell::{
    OptimizationLevel,
    targets::{CodeModel, FileType, RelocMode, Target, TargetMachine},
};

impl StmtCodegen for TypedProgram {
    fn generate_stmt<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        let visitor = CodegenVisitor::new();
        visitor.visit_program(self, context)
    }
}

impl Analyzer<Prepared> {
    pub fn generate_code<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<()> {
        self.ast().generate_stmt(context)
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

    pub fn verify_module<'ctx>(
        &self,
        llvm_context: &'ctx inkwell::context::Context,
        module_name: &str,
    ) -> CodegenResult<bool> {
        let context = self.compile_to_llvm(llvm_context, module_name)?;
        Ok(context.module().verify().is_ok())
    }

    pub fn write_to_file<'ctx>(
        &self,
        llvm_context: &'ctx inkwell::context::Context,
        module_name: &str,
        file_path: &std::path::Path,
    ) -> CodegenResult<()> {
        let ir = self.generate_ir(llvm_context, module_name)?;
        std::fs::write(file_path, ir).map_err(|e| {
            crate::codegen::error::CodegenError::LLVMBuild {
                message: format!("Failed to write IR to file: {}", e),
            }
        })?;
        Ok(())
    }

    pub fn compile_to_object<'ctx>(
        &self,
        llvm_context: &'ctx inkwell::context::Context,
        module_name: &str,
        output_path: &std::path::Path,
    ) -> CodegenResult<()> {
        let context = self.compile_to_llvm(llvm_context, module_name)?;

        Target::initialize_all(&inkwell::targets::InitializationConfig::default());

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).map_err(|e| {
            crate::codegen::error::CodegenError::LLVMBuild {
                message: format!("Failed to get target from triple: {}", e),
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
                message: format!("Failed to write object file: {}", e),
            })?;

        Ok(())
    }
}

pub struct ProgramGenerator<'ctx> {
    context: LLVMContext<'ctx>,
    visitor: CodegenVisitor,
}

impl<'ctx> ProgramGenerator<'ctx> {
    pub fn new(context: LLVMContext<'ctx>) -> Self {
        Self {
            context,
            visitor: CodegenVisitor::new(),
        }
    }

    pub fn generate(&mut self, program: &TypedProgram) -> CodegenResult<()> {
        self.visitor.visit_program(program, &mut self.context)
    }

    pub fn into_context(self) -> LLVMContext<'ctx> {
        self.context
    }

    pub fn context(&self) -> &LLVMContext<'ctx> {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut LLVMContext<'ctx> {
        &mut self.context
    }

    pub fn get_ir(&self) -> String {
        self.context.module().print_to_string().to_string()
    }

    pub fn verify(&self) -> bool {
        self.context.module().verify().is_ok()
    }

    pub fn dump_ir(&self) {
        self.context.module().print_to_stderr();
    }
}
