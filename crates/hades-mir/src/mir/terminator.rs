use crate::mir::place::{Operand, Place};

pub(crate) enum Terminator {
    Return,
    Call {
        func: Operand,
        args: Vec<Operand>,
        dest: Place,
    },
}
