use hades_ast::Types;
use hades_error::Span;
use hades_tokens::Ident;

use crate::BasicBlock;
use crate::mir::guard::{Guard, LoopContext};
use crate::mir::local::Local;
use crate::mir::stmt::Statement;
use crate::mir::terminator::Terminator;

pub struct MirBuilder {
    pub current_guard: Option<Guard>,
}

impl Default for MirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MirBuilder {
    pub fn new() -> Self {
        Self {
            current_guard: None,
        }
    }

    pub fn enter_guard(&mut self) {
        self.current_guard = Some(Guard::new());
    }

    pub fn exit_guard(&mut self) -> Guard {
        self.current_guard.take().expect("no active guard")
    }

    fn guard(&self) -> &Guard {
        self.current_guard.as_ref().expect("no active guard")
    }

    fn guard_mut(&mut self) -> &mut Guard {
        self.current_guard.as_mut().expect("no active guard")
    }

    pub fn build_local(&mut self, name: Ident, typ: Types) -> usize {
        let local = Local::new(name, typ);
        self.guard_mut().add_local(local)
    }

    pub fn start_block(&mut self) -> BasicBlock {
        self.guard_mut().start_new_block()
    }

    pub fn switch_to(&mut self, block: BasicBlock) {
        self.guard_mut().switch_to(block);
    }

    pub fn current_block(&self) -> BasicBlock {
        self.guard().current_block()
    }

    pub fn is_terminated(&self) -> bool {
        self.guard().is_terminated()
    }

    pub fn is_block_terminated(&self, block: BasicBlock) -> bool {
        self.guard().is_block_terminated(block)
    }

    pub fn emit(&mut self, stmt: Statement) {
        self.guard_mut().emit(stmt);
    }

    pub fn push_stmt(&mut self, block: BasicBlock, stmt: Statement) {
        self.guard_mut().push_stmt(block, stmt);
    }

    pub fn terminate(&mut self, t: Terminator) {
        self.guard_mut().terminate(t);
    }

    pub fn terminate_block(&mut self, block: BasicBlock, t: Terminator) {
        self.guard_mut().terminate_block(block, t);
    }

    pub fn lookup_local(&self, name: &Ident) -> usize {
        self.guard().lookup_local(name)
    }

    pub fn local_count(&self) -> usize {
        self.guard().locals.len()
    }

    pub fn push_loop(&mut self, continue_block: BasicBlock, break_block: BasicBlock) {
        self.guard_mut().push_loop(continue_block, break_block);
    }

    pub fn pop_loop(&mut self) {
        self.guard_mut().pop_loop();
    }

    pub fn current_loop(&self) -> Option<&LoopContext> {
        self.guard().current_loop()
    }

    pub fn push_defer(&mut self, stmts: Vec<Statement>, span: Span) {
        self.guard_mut().push_defer(stmts, span);
    }

    pub fn pop_defer(&mut self) {
        self.guard_mut().pop_defer();
    }

    pub fn deferred_stmts(&self) -> Vec<Statement> {
        self.guard().deferred_stmts()
    }

    pub fn drain_scratch_block(&mut self, id: BasicBlock) -> Vec<Statement> {
        self.guard_mut().drain_scratch_block(id)
    }
}
