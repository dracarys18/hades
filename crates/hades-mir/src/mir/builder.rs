use hades_ast::{CompilerContext, Types};
use hades_tokens::Ident;

use crate::BasicBlock;
use crate::mir::guard::Guard;
use crate::mir::local::Local;
use crate::mir::stmt::MirStmt;
use crate::mir::terminator::Terminator;

pub struct MirBuilder<'ctx> {
    pub ctx: &'ctx CompilerContext,
    current_guard: Option<Guard>,
}

impl<'ctx> MirBuilder<'ctx> {
    pub fn new(ctx: &'ctx CompilerContext) -> Self {
        Self { ctx, current_guard: None }
    }

    pub fn enter_guard(&mut self) {
        self.current_guard = Some(Guard::new());
    }

    pub fn exit_guard(&mut self) -> Guard {
        self.current_guard.take().expect("no active guard")
    }

    pub fn build_local(&mut self, name: Ident, typ: Types) -> usize {
        let local = Local::new(name, typ);
        self.current_guard.as_mut().expect("no active guard").add_local(local)
    }

    pub fn start_block(&mut self) -> BasicBlock {
        self.current_guard.as_mut().expect("no active guard").start_new_block()
    }

    pub fn push_stmt(&mut self, block: BasicBlock, stmt: MirStmt) {
        self.current_guard.as_mut().expect("no active guard").push_stmt(block, stmt);
    }

    pub fn terminate(&mut self, block: BasicBlock, t: Terminator) {
        self.current_guard.as_mut().expect("no active guard").terminate(block, t);
    }

    pub fn lookup_local(&self, name: &Ident) -> usize {
        self.current_guard.as_ref().expect("no active guard").lookup_local(name)
    }

    pub fn local_count(&self) -> usize {
        self.current_guard.as_ref().expect("no active guard").locals.len()
    }

    pub fn build(&mut self) {}
}
