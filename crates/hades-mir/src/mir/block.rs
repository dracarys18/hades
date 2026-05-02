use crate::BasicBlock;
use super::stmt::Statement;
use super::terminator::Terminator;

#[derive(Debug, Clone)]
pub struct BasicBlockData {
    pub stmts: Vec<Statement>,
    pub terminator: Option<Terminator>,
    pub successors: Vec<BasicBlock>,
    pub predecessors: Vec<BasicBlock>,
}

impl BasicBlockData {
    pub fn new() -> Self {
        Self { stmts: vec![], terminator: None, successors: vec![], predecessors: vec![] }
    }

    pub fn emit(&mut self, stmt: Statement) {
        debug_assert!(self.terminator.is_none(), "emitting into already-terminated block");
        self.stmts.push(stmt);
    }

    pub fn terminate(&mut self, t: Terminator) {
        debug_assert!(self.terminator.is_none(), "block already terminated");
        self.terminator = Some(t);
    }

    pub fn is_terminated(&self) -> bool {
        self.terminator.is_some()
    }
}
