use super::stmt::Stmt;
use crate::codegen::CodeGen;
use inkwell::context::Context;

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
    pub fn compile_to_llvm(self, output_path: impl AsRef<std::path::Path>) {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "main_module");
        match codegen.compile(self) {
            Ok(_) => {
                println!("Code generation successful!");
                match codegen.write_ir_to_file(output_path) {
                    Ok(_) => println!("LLVM IR written to output.ll"),
                    Err(e) => eprintln!("Failed to write LLVM IR to file: {e}"),
                }
            }
            Err(e) => eprintln!("Code generation failed: {e}"),
        }
    }
}
