use super::{stmt::Statement, terminator::Terminator};

/// Index into `Cfg::blocks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl BlockId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// A single basic block in a CFG.
/// All statements execute unconditionally; the terminator decides control flow.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub stmts: Vec<Statement>,
    /// `None` only while the block is being built; always `Some` in a finished `Cfg`.
    pub terminator: Option<Terminator>,
}

impl BasicBlock {
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            stmts: vec![],
            terminator: None,
        }
    }

    pub fn is_terminated(&self) -> bool {
        self.terminator.is_some()
    }

    pub fn terminate(&mut self, t: Terminator) {
        debug_assert!(
            self.terminator.is_none(),
            "block {} already terminated",
            self.id
        );
        self.terminator = Some(t);
    }
}
