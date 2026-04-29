use hades_ast::{
    TypedArrayIndex, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr, TypedExpr,
    TypedFieldAccess, TypedValue, Types,
};

use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};
use crate::mir::builder::MirBuilder;
use crate::mir::place::{ConstVal, Operand, Place, PlaceElem};
use crate::mir::stmt::{MirStmt, Rvalue};

impl ToMir for TypedExpr {
    type Output = Rvalue;

    fn to_mir(&self, builder: &mut MirBuilder, block: BasicBlock) -> BlockAnd<Rvalue> {
        match self {
            TypedExpr::Value(v) => block.and(lower_value(v)),

            TypedExpr::Ident { ident, .. } => {
                let idx = builder.lookup_local(ident);
                block.and(Rvalue::Use(Operand::Copy(Place::local(idx))))
            }

            TypedExpr::Binary(TypedBinaryExpr { left, op, right, .. }) => {
                let mut block = block;

                let lhs_rvalue = unpack!(block = left.to_mir(builder, block));
                let (block2, lhs) = emit_temp(builder, block, lhs_rvalue, &left.get_type());
                block = block2;

                let rhs_rvalue = unpack!(block = right.to_mir(builder, block));
                let (block3, rhs) = emit_temp(builder, block, rhs_rvalue, &right.get_type());
                block = block3;

                block.and(Rvalue::BinaryOp(op.clone(), lhs, rhs))
            }

            TypedExpr::Unary { op, expr, .. } => {
                let mut block = block;
                let rvalue = unpack!(block = expr.to_mir(builder, block));
                let (block2, operand) = emit_temp(builder, block, rvalue, &expr.get_type());
                block2.and(Rvalue::UnaryOp(op.clone(), operand))
            }

            TypedExpr::As(a) => {
                let mut block = block;
                let rvalue = unpack!(block = a.expr.to_mir(builder, block));
                let (block2, operand) = emit_temp(builder, block, rvalue, &a.expr.get_type());
                block2.and(Rvalue::Cast(operand, a.target_type.clone()))
            }

            TypedExpr::Assign(TypedAssignExpr { target, value, .. }) => {
                let mut block = block;
                let rvalue = unpack!(block = value.to_mir(builder, block));
                let (block2, dest) = lower_assign_target(builder, block, target);
                block = block2;
                builder.push_stmt(block, MirStmt::Assign(dest.clone(), rvalue));
                block.and(Rvalue::Use(Operand::Copy(dest)))
            }

            TypedExpr::FieldAccess(fa) => {
                let mut block = block;
                let (block2, place) = lower_field_access(builder, block, fa);
                block2.and(Rvalue::Use(Operand::Copy(place)))
            }

            TypedExpr::ArrayIndex(ai) => {
                let mut block = block;
                let (block2, place) = lower_array_index(builder, block, ai);
                block2.and(Rvalue::Use(Operand::Copy(place)))
            }

            TypedExpr::Null(typ) => {
                block.and(Rvalue::Use(Operand::Constant(ConstVal::Null(typ.clone()))))
            }

            TypedExpr::StructInit { .. } | TypedExpr::Call { .. } => {
                todo!("StructInit / Call lowering")
            }
        }
    }
}

fn lower_field_access(
    builder: &mut MirBuilder,
    block: BasicBlock,
    fa: &TypedFieldAccess,
) -> (BasicBlock, Place) {
    let mut block = block;
    let base_rvalue = unpack!(block = fa.expr.to_mir(builder, block));
    let (block2, base_op) = emit_temp(builder, block, base_rvalue, &fa.struct_type);
    let base_idx = match base_op {
        Operand::Copy(ref p) => p.local,
        Operand::Constant(_) => unreachable!("struct base cannot be a constant"),
    };
    let place = Place {
        local: base_idx,
        projection: vec![PlaceElem::Field(fa.field.clone())],
    };
    (block2, place)
}

fn lower_array_index(
    builder: &mut MirBuilder,
    block: BasicBlock,
    ai: &TypedArrayIndex,
) -> (BasicBlock, Place) {
    let mut block = block;

    let base_rvalue = unpack!(block = ai.expr.to_mir(builder, block));
    let (block2, base_op) = emit_temp(builder, block, base_rvalue, &ai.expr.get_type());
    block = block2;
    let base_idx = match base_op {
        Operand::Copy(ref p) => p.local,
        Operand::Constant(_) => unreachable!("array base cannot be a constant"),
    };

    let idx_rvalue = unpack!(block = ai.index.to_mir(builder, block));
    let (block3, idx_op) = emit_temp(builder, block, idx_rvalue, &ai.index.get_type());

    let place = Place {
        local: base_idx,
        projection: vec![PlaceElem::Index(idx_op)],
    };
    (block3, place)
}

fn lower_assign_target(
    builder: &mut MirBuilder,
    block: BasicBlock,
    target: &TypedAssignTarget,
) -> (BasicBlock, Place) {
    match target {
        TypedAssignTarget::Ident(ident) => {
            let idx = builder.lookup_local(ident);
            (block, Place::local(idx))
        }
        TypedAssignTarget::FieldAccess(fa) => lower_field_access(builder, block, fa),
        TypedAssignTarget::ArrayIndex(ai) => lower_array_index(builder, block, ai),
        TypedAssignTarget::Deref(expr) => {
            let mut block = block;
            let rvalue = unpack!(block = expr.to_mir(builder, block));
            let (block2, op) = emit_temp(builder, block, rvalue, &expr.get_type());
            let base_idx = match op {
                Operand::Copy(ref p) => p.local,
                Operand::Constant(_) => unreachable!("deref target cannot be a constant"),
            };
            let place = Place {
                local: base_idx,
                projection: vec![PlaceElem::Deref],
            };
            (block2, place)
        }
    }
}

fn lower_value(v: &TypedValue) -> Rvalue {
    let c = match v {
        TypedValue::Number(n) => ConstVal::Int(*n),
        TypedValue::Float(f) => ConstVal::Float(*f),
        TypedValue::Boolean(b) => ConstVal::Bool(*b),
        TypedValue::String(s) => ConstVal::Str(s.clone()),
        TypedValue::Char(c) => ConstVal::Char(*c),
        TypedValue::Array(_) => todo!("array literal lowering"),
    };
    Rvalue::Use(Operand::Constant(c))
}

fn emit_temp(
    builder: &mut MirBuilder,
    block: BasicBlock,
    rvalue: Rvalue,
    typ: &Types,
) -> (BasicBlock, Operand) {
    let tmp_name = hades_tokens::Ident::new(
        format!("_tmp{}", builder.local_count()),
        hades_error::Span::default(),
    );
    let idx = builder.build_local(tmp_name, typ.clone());
    builder.push_stmt(block, MirStmt::Assign(Place::local(idx), rvalue));
    (block, Operand::Copy(Place::local(idx)))
}
