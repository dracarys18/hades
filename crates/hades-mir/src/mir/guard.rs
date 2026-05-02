use std::collections::HashMap;

use hades_error::Span;
use hades_tokens::Ident;

use crate::BasicBlock;
use crate::mir::block::BasicBlockData;
use crate::mir::local::Local;
use crate::mir::stmt::Statement;
use crate::mir::terminator::Terminator;

#[derive(Debug, Clone)]
pub struct LoopContext {
    pub continue_block: BasicBlock,
    pub break_block: BasicBlock,
}

#[derive(Debug, Clone)]
pub struct DeferEntry {
    pub stmts: Vec<Statement>,
    pub span: Span,
}

pub struct Guard {
    pub locals: Vec<Local>,
    pub local_map: HashMap<Ident, usize>,
    pub basic_blocks: Vec<BasicBlockData>,
    pub current: BasicBlock,
    pub loop_stack: Vec<LoopContext>,
    pub defer_stack: Vec<DeferEntry>,
}

impl Default for Guard {
    fn default() -> Self {
        Self::new()
    }
}

impl Guard {
    pub fn new() -> Self {
        Self {
            locals: Vec::new(),
            local_map: HashMap::new(),
            basic_blocks: vec![],
            current: BasicBlock(0),
            loop_stack: vec![],
            defer_stack: vec![],
        }
    }

    pub fn add_local(&mut self, local: Local) -> usize {
        let idx = self.locals.len();
        self.local_map.insert(local.name().clone(), idx);
        self.locals.push(local);
        idx
    }

    pub fn start_new_block(&mut self) -> BasicBlock {
        let idx = self.basic_blocks.len();
        self.basic_blocks.push(BasicBlockData::new());
        BasicBlock(idx)
    }

    pub fn switch_to(&mut self, block: BasicBlock) {
        self.current = block;
    }

    pub fn current_block(&self) -> BasicBlock {
        self.current
    }

    pub fn is_terminated(&self) -> bool {
        self.basic_blocks[self.current.0].is_terminated()
    }

    pub fn is_block_terminated(&self, block: BasicBlock) -> bool {
        self.basic_blocks[block.0].is_terminated()
    }

    pub fn emit(&mut self, stmt: Statement) {
        self.basic_blocks[self.current.0].emit(stmt);
    }

    pub fn push_stmt(&mut self, block: BasicBlock, stmt: Statement) {
        self.basic_blocks[block.0].emit(stmt);
    }

    pub fn terminate(&mut self, t: Terminator) {
        let block = self.current;
        self.basic_blocks[block.0].terminate(t);
        self.record_edges(block);
    }

    pub fn terminate_block(&mut self, block: BasicBlock, t: Terminator) {
        self.basic_blocks[block.0].terminate(t);
        self.record_edges(block);
    }

    fn record_edges(&mut self, from: BasicBlock) {
        let succs: Vec<BasicBlock> = self.basic_blocks[from.0]
            .terminator
            .as_ref()
            .map(|t| t.successors())
            .unwrap_or_default();
        self.basic_blocks[from.0].successors = succs.clone();
        for succ in succs {
            self.basic_blocks[succ.0].predecessors.push(from);
        }
    }

    pub fn lookup_local(&self, name: &Ident) -> usize {
        *self.local_map.get(name).expect("undeclared local")
    }

    pub fn push_loop(&mut self, continue_block: BasicBlock, break_block: BasicBlock) {
        self.loop_stack.push(LoopContext {
            continue_block,
            break_block,
        });
    }

    pub fn pop_loop(&mut self) {
        self.loop_stack.pop().expect("pop_loop: stack is empty");
    }

    pub fn current_loop(&self) -> Option<&LoopContext> {
        self.loop_stack.last()
    }

    pub fn push_defer(&mut self, stmts: Vec<Statement>, span: Span) {
        self.defer_stack.push(DeferEntry { stmts, span });
    }

    pub fn pop_defer(&mut self) {
        self.defer_stack.pop().expect("pop_defer: stack is empty");
    }

    pub fn deferred_stmts(&self) -> Vec<Statement> {
        self.defer_stack
            .iter()
            .rev()
            .flat_map(|d| d.stmts.iter().cloned())
            .collect()
    }

    pub fn drain_scratch_block(&mut self, id: BasicBlock) -> Vec<Statement> {
        std::mem::take(&mut self.basic_blocks[id.0].stmts)
    }
}
