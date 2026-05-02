use std::fmt;

use hades_tokens::Op;

use crate::BasicBlock;
use crate::mir::block::BasicBlockData;
use crate::mir::func::MirFunction;
use crate::mir::module::MirModule;
use crate::mir::operand::{MirConst, Operand};
use crate::mir::place::{Place, PlaceElem};
use crate::mir::rvalue::{AggregateKind, Rvalue};
use crate::mir::stmt::StatementKind;
use crate::mir::terminator::{CallTarget, TerminatorKind};

fn op_str(op: &Op) -> &'static str {
    match op {
        Op::Plus | Op::Add => "+",
        Op::Minus | Op::Sub => "-",
        Op::Multiply | Op::Mul => "*",
        Op::Divide | Op::Div => "/",
        Op::Mod => "%",
        Op::PlusEqual => "+=",
        Op::MinusEqual => "-=",
        Op::EqualEqual | Op::Eq => "==",
        Op::BangEqual | Op::Ne => "!=",
        Op::Greater | Op::Gt => ">",
        Op::GreaterEqual | Op::Ge => ">=",
        Op::Less | Op::Lt => "<",
        Op::LessEqual | Op::Le => "<=",
        Op::And | Op::BoleanAnd => "&&",
        Op::Or | Op::BooleanOr => "||",
        Op::Not => "!",
        Op::BitAnd => "&",
        Op::BitOr => "|",
        Op::BitXor => "^",
        Op::BitNot => "~",
        Op::Shl => "<<",
        Op::Shr => ">>",
        Op::Assign => "=",
        Op::Ref => "&",
        Op::Deref => "*",
    }
}

impl fmt::Display for MirConst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirConst::Int(n) => write!(f, "{n}"),
            MirConst::Float(v) => write!(f, "{v}"),
            MirConst::Bool(b) => write!(f, "{b}"),
            MirConst::Str(s) => write!(f, "\"{s}\""),
            MirConst::Char(c) => write!(f, "'{c}'"),
            MirConst::Null(_) => write!(f, "null"),
        }
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.projection.is_empty() {
            return write!(f, "_{}", self.local);
        }
        write!(f, "(")?;
        let mut s = format!("_{}", self.local);
        for elem in &self.projection {
            match elem {
                PlaceElem::Deref => s = format!("*{s}"),
                PlaceElem::Field { name, .. } => s = format!("{s}.{name}"),
                PlaceElem::Index(idx) => s = format!("{s}[_{idx}]"),
            }
        }
        write!(f, "{s})")
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Copy(p) => write!(f, "copy {p}"),
            Operand::Ref(p) => write!(f, "&{p}"),
            Operand::Const(c) => write!(f, "const {c}"),
        }
    }
}

impl fmt::Display for Rvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rvalue::Use(op) => write!(f, "{op}"),
            Rvalue::BinaryOp(op, lhs, rhs) => write!(f, "{lhs} {} {rhs}", op_str(op)),
            Rvalue::UnaryOp(op, val) => write!(f, "{}{val}", op_str(op)),
            Rvalue::Cast(op, ty) => write!(f, "{op} as {ty}"),
            Rvalue::Aggregate(kind, operands) => match kind {
                AggregateKind::Struct(name) => {
                    write!(f, "{name} {{ ")?;
                    for (i, op) in operands.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{op}")?;
                    }
                    write!(f, " }}")
                }
                AggregateKind::Array(_) => {
                    write!(f, "[")?;
                    for (i, op) in operands.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{op}")?;
                    }
                    write!(f, "]")
                }
            },
        }
    }
}

impl fmt::Display for CallTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallTarget::Function(name) => write!(f, "{name}"),
            CallTarget::Method { receiver, method } => write!(f, "{receiver}.{method}"),
            CallTarget::Qualified { ty, method } => write!(f, "{ty}::{method}"),
        }
    }
}

fn fmt_block(
    data: &BasicBlockData,
    id: BasicBlock,
    locals_len: usize,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(f, "    bb{}: {{", id.0)?;

    for stmt in &data.stmts {
        match &stmt.kind {
            StatementKind::Assign(place, rvalue) => {
                writeln!(f, "        {place} = {rvalue};")?;
            }
            StatementKind::Nop => {
                writeln!(f, "        nop;")?;
            }
        }
    }

    if let Some(term) = &data.terminator {
        match &term.kind {
            TerminatorKind::Goto(bb) => writeln!(f, "        goto -> bb{};", bb.0)?,
            TerminatorKind::Return => writeln!(f, "        return;")?,
            TerminatorKind::Unreachable => writeln!(f, "        unreachable;")?,
            TerminatorKind::SwitchInt { discriminant, targets } => {
                writeln!(f, "        switchInt({discriminant}) {{")?;
                for (val, bb) in targets.values.iter().zip(targets.blocks.iter()) {
                    writeln!(f, "            {val} => bb{},", bb.0)?;
                }
                writeln!(f, "            otherwise => bb{},", targets.otherwise.0)?;
                writeln!(f, "        }};")?;
            }
            TerminatorKind::Call { target, args, dest, successor } => {
                write!(f, "        {dest} = {target}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                writeln!(f, ") -> bb{};", successor.0)?;
            }
        }
    }

    let _ = locals_len;
    writeln!(f, "    }}")
}

impl fmt::Display for MirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let locals = &self.guard.locals;
        let return_ty = &self.return_type;

        write!(f, "fn {}(", self.name)?;
        for i in 1..=self.arg_count {
            if i > 1 {
                write!(f, ", ")?;
            }
            let ty = locals.get(i).map(|l| l.typ.to_string()).unwrap_or_else(|| "?".to_string());
            write!(f, "_{i}: {ty}")?;
        }
        writeln!(f, ") -> {return_ty} {{")?;

        for (i, local) in locals.iter().enumerate() {
            writeln!(f, "    let _{i}: {};", local.typ)?;
        }

        if !locals.is_empty() && !self.guard.basic_blocks.is_empty() {
            writeln!(f)?;
        }

        for (i, block) in self.guard.basic_blocks.iter().enumerate() {
            fmt_block(block, BasicBlock(i), locals.len(), f)?;
        }

        write!(f, "}}")
    }
}

impl fmt::Display for MirModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, func) in self.functions.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            writeln!(f, "{func}")?;
        }
        Ok(())
    }
}
