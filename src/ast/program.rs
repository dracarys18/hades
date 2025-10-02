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
