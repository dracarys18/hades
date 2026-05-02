use hades_ast::Types;
use hades_error::Span;
use hades_tokens::Name;

use crate::BasicBlock;
use super::operand::Operand;
use super::place::Place;

#[derive(Debug, Clone)]
pub struct SwitchTargets {
    pub values: Vec<u128>,
    pub blocks: Vec<BasicBlock>,
    pub otherwise: BasicBlock,
}

impl SwitchTargets {
    pub fn bool_branch(then_block: BasicBlock, else_block: BasicBlock) -> Self {
        Self {
            values: vec![1],
            blocks: vec![then_block],
            otherwise: else_block,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CallTarget {
    Function(Name),
    Method {
        receiver: Operand,
        method: Name,
    },
    Qualified {
        ty: Types,
        method: Name,
    },
}

#[derive(Debug, Clone)]
pub enum TerminatorKind {
    Goto(BasicBlock),
    SwitchInt {
        discriminant: Operand,
        targets: SwitchTargets,
    },
    Return,
    Unreachable,
    Call {
        target: CallTarget,
        args: Vec<Operand>,
        dest: Place,
        successor: BasicBlock,
    },
}

#[derive(Debug, Clone)]
pub struct Terminator {
    pub kind: TerminatorKind,
    pub span: Span,
}

impl Terminator {
    pub fn new(kind: TerminatorKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn successors(&self) -> Vec<BasicBlock> {
        match &self.kind {
            TerminatorKind::Goto(b) => vec![*b],
            TerminatorKind::SwitchInt { targets, .. } => {
                let mut succs = targets.blocks.clone();
                succs.push(targets.otherwise);
                succs.dedup();
                succs
            }
            TerminatorKind::Return | TerminatorKind::Unreachable => vec![],
            TerminatorKind::Call { successor, .. } => vec![*successor],
        }
    }
}

pub const RETURN_LOCAL: usize = 0;
