use super::block::{BasicBlock, BlockId};

/// A control-flow graph for a single function.
///
/// `successors[i]` and `predecessors[i]` are eagerly built by `finish()` and
/// are always valid after construction — no lazy computation needed for analysis passes.
#[derive(Debug, Clone)]
pub struct Cfg {
    pub entry: BlockId,
    pub blocks: Vec<BasicBlock>,
    /// `successors[i]` — blocks reachable from `blocks[i]` via its terminator.
    pub successors: Vec<Vec<BlockId>>,
    /// `predecessors[i]` — blocks that have `blocks[i]` as a successor.
    pub predecessors: Vec<Vec<BlockId>>,
}

impl Cfg {
    /// Build the CFG from a list of finished basic blocks.
    /// Panics if any block has no terminator.
    pub fn finish(entry: BlockId, blocks: Vec<BasicBlock>) -> Self {
        let n = blocks.len();
        let mut successors: Vec<Vec<BlockId>> = vec![vec![]; n];
        let mut predecessors: Vec<Vec<BlockId>> = vec![vec![]; n];

        for block in &blocks {
            let term = block
                .terminator
                .as_ref()
                .unwrap_or_else(|| panic!("block {} has no terminator", block.id));
            let succs = term.successors();
            for &succ in &succs {
                predecessors[succ.index()].push(block.id);
            }
            successors[block.id.index()] = succs;
        }

        Self {
            entry,
            blocks,
            successors,
            predecessors,
        }
    }

    pub fn block(&self, id: BlockId) -> &BasicBlock {
        &self.blocks[id.index()]
    }

    pub fn block_mut(&mut self, id: BlockId) -> &mut BasicBlock {
        &mut self.blocks[id.index()]
    }
}
