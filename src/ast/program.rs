use super::stmt::Stmt;
use crate::{codegen::CodeGen, consts::BUILD_PATH};
use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Program(Vec<Stmt>);

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self(stmts)
    }
}

impl std::ops::Deref for Program {
    type Target = Vec<Stmt>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> IntoIterator for &'a Program {
    type Item = &'a Stmt;
    type IntoIter = std::slice::Iter<'a, Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Program {
    type Item = Stmt;
    type IntoIter = std::vec::IntoIter<Stmt>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Program {
    pub fn compile_program(self, output_path: impl AsRef<std::path::Path>) {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "main_module");

        codegen
            .compile(self)
            .map_err(|err| {
                eprintln!("Error during code generation: {err}");
                err
            })
            .expect("Code generation failed");

        codegen
            .write_exec(output_path)
            .map_err(|err| {
                eprintln!("Error writing executable: {err}");
                err
            })
            .expect("Failed to write executable");

        codegen.cleanup();
    }
}
