use hades_ast::{
    TypedArrayIndex, TypedArrayLiteral, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr,
    TypedExpr, TypedFieldAccess, TypedValue, Types,
};
use hades_error::Span;
use hades_tokens::Op;

use crate::mir::builder::MirBuilder;
use crate::mir::operand::{MirConst, Operand};
use crate::mir::place::Place;
use crate::mir::rvalue::{AggregateKind, Rvalue};
use crate::mir::stmt::Statement;
use crate::mir::terminator::{CallTarget, Terminator, TerminatorKind};
use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};

impl ToMir for TypedExpr {
    type Output = Rvalue;

    fn to_mir(&self, builder: &mut MirBuilder, mut block: BasicBlock) -> BlockAnd<Rvalue> {
        let span = Span::default();
        match self {
            TypedExpr::Value(TypedValue::Array(arr)) => {
                lower_array_literal(arr, builder, block, span)
            }
            TypedExpr::Value(v) => block.and(lower_value(v)),

            TypedExpr::Null(typ) => {
                block.and(Rvalue::Use(Operand::Const(MirConst::Null(typ.clone()))))
            }

            TypedExpr::Ident { ident, .. } => {
                let idx = builder.lookup_local(ident);
                block.and(Rvalue::Use(Operand::Copy(Place::local(idx))))
            }

            TypedExpr::Binary(TypedBinaryExpr {
                left, op, right, ..
            }) => {
                let lhs_rvalue = unpack!(block = left.to_mir(builder, block));
                let (block2, lhs) =
                    emit_temp(builder, block, lhs_rvalue, &left.get_type(), span.clone());
                block = block2;

                let rhs_rvalue = unpack!(block = right.to_mir(builder, block));
                let (block3, rhs) =
                    emit_temp(builder, block, rhs_rvalue, &right.get_type(), span.clone());
                block = block3;

                block.and(Rvalue::BinaryOp(op.clone(), lhs, rhs))
            }

            TypedExpr::Unary { op, expr, .. } => {
                let rvalue = unpack!(block = expr.to_mir(builder, block));
                if *op == Op::Ref {
                    let (block2, operand) =
                        emit_temp(builder, block, rvalue, &expr.get_type(), span.clone());
                    let place = match operand {
                        Operand::Copy(ref p) => p.clone(),
                        Operand::Ref(ref p) => p.clone(),
                        Operand::Const(_) => {
                            let tmp_name = hades_tokens::Ident::new(
                                format!("_tmp{}", builder.local_count()),
                                span.clone(),
                            );
                            let idx = builder.build_local(tmp_name, expr.get_type());
                            builder.push_stmt(
                                block2,
                                Statement::assign(
                                    Place::local(idx),
                                    Rvalue::Use(operand),
                                    span.clone(),
                                ),
                            );
                            Place::local(idx)
                        }
                    };
                    block2.and(Rvalue::Use(Operand::Ref(place)))
                } else {
                    let (block2, operand) =
                        emit_temp(builder, block, rvalue, &expr.get_type(), span);
                    block2.and(Rvalue::UnaryOp(op.clone(), operand))
                }
            }

            TypedExpr::As(a) => {
                let rvalue = unpack!(block = a.expr.to_mir(builder, block));
                let (block2, operand) = emit_temp(builder, block, rvalue, &a.expr.get_type(), span);
                block2.and(Rvalue::Cast(operand, a.target_type.clone()))
            }

            TypedExpr::Assign(TypedAssignExpr {
                target, value, op, ..
            }) => {
                let val_rvalue = unpack!(block = value.to_mir(builder, block));
                let rvalue = match op {
                    Op::PlusEqual => {
                        let (block2, val_op) =
                            emit_temp(builder, block, val_rvalue, &value.get_type(), span.clone());
                        block = block2;
                        let (block3, target_place) = lower_assign_target(builder, block, target);
                        block = block3;
                        let target_op = Operand::Copy(target_place.clone());
                        let (block4, dest) = (block, target_place);
                        block = block4;
                        builder.push_stmt(
                            block,
                            Statement::assign(
                                dest,
                                Rvalue::BinaryOp(Op::Plus, target_op, val_op),
                                span,
                            ),
                        );
                        return block.and(Rvalue::Use(Operand::Const(MirConst::Int(0))));
                    }
                    Op::MinusEqual => {
                        let (block2, val_op) =
                            emit_temp(builder, block, val_rvalue, &value.get_type(), span.clone());
                        block = block2;
                        let (block3, target_place) = lower_assign_target(builder, block, target);
                        block = block3;
                        let target_op = Operand::Copy(target_place.clone());
                        builder.push_stmt(
                            block,
                            Statement::assign(
                                target_place,
                                Rvalue::BinaryOp(Op::Minus, target_op, val_op),
                                span,
                            ),
                        );
                        return block.and(Rvalue::Use(Operand::Const(MirConst::Int(0))));
                    }
                    _ => val_rvalue,
                };
                let (block2, dest) = lower_assign_target(builder, block, target);
                block = block2;
                builder.push_stmt(block, Statement::assign(dest.clone(), rvalue, span));
                block.and(Rvalue::Use(Operand::Copy(dest)))
            }

            TypedExpr::FieldAccess(fa) => {
                let (block2, place) = lower_field_access(builder, block, fa, span);
                block2.and(Rvalue::Use(Operand::Copy(place)))
            }

            TypedExpr::ArrayIndex(ai) => {
                let (block2, place) = lower_array_index(builder, block, ai, span);
                block2.and(Rvalue::Use(Operand::Copy(place)))
            }

            TypedExpr::StructInit { name, fields, .. } => {
                let mut operands: Vec<Operand> = Vec::new();
                for (_, field_expr) in fields {
                    let rvalue = unpack!(block = field_expr.to_mir(builder, block));
                    let (block2, op) =
                        emit_temp(builder, block, rvalue, &field_expr.get_type(), span.clone());
                    block = block2;
                    operands.push(op);
                }
                block.and(Rvalue::Aggregate(
                    AggregateKind::Struct(name.clone()),
                    operands,
                ))
            }

            TypedExpr::Call {
                func,
                args,
                receiver,
                typ,
            } => {
                let call_target = if let Some(recv) = receiver {
                    let recv_rvalue = unpack!(block = recv.to_mir(builder, block));
                    let (block2, recv_op) =
                        emit_temp(builder, block, recv_rvalue, &recv.get_type(), span.clone());
                    block = block2;
                    CallTarget::Method {
                        receiver: recv_op,
                        method: func.clone(),
                    }
                } else {
                    CallTarget::Function(func.clone())
                };

                let mut operands: Vec<Operand> = Vec::new();
                for arg in args {
                    let rvalue = unpack!(block = arg.to_mir(builder, block));
                    let (block2, op) =
                        emit_temp(builder, block, rvalue, &arg.get_type(), span.clone());
                    block = block2;
                    operands.push(op);
                }

                let dest_name = hades_tokens::Ident::new(
                    format!("_tmp{}", builder.local_count()),
                    span.clone(),
                );
                let dest_idx = builder.build_local(dest_name, typ.clone());
                let dest = Place::local(dest_idx);

                let successor = builder.start_block();
                builder.switch_to(block);
                builder.terminate(Terminator::new(
                    TerminatorKind::Call {
                        target: call_target,
                        args: operands,
                        dest: dest.clone(),
                        successor,
                    },
                    span,
                ));

                successor.and(Rvalue::Use(Operand::Copy(dest)))
            }
        }
    }
}

fn lower_field_access(
    builder: &mut MirBuilder,
    block: BasicBlock,
    fa: &TypedFieldAccess,
    span: Span,
) -> (BasicBlock, Place) {
    let mut block = block;
    let base_rvalue = unpack!(block = fa.expr.to_mir(builder, block));
    let (block2, base_op) = emit_temp(builder, block, base_rvalue, &fa.struct_type, span.clone());
    let base_idx = match base_op {
        Operand::Copy(ref p) | Operand::Ref(ref p) => p.local,
        Operand::Const(_) => unreachable!("struct base cannot be a constant"),
    };
    let place = Place::with_field(base_idx, fa.field.clone(), 0, fa.field_type.clone());
    (block2, place)
}

fn lower_array_index(
    builder: &mut MirBuilder,
    block: BasicBlock,
    ai: &TypedArrayIndex,
    span: Span,
) -> (BasicBlock, Place) {
    let mut block = block;

    let base_rvalue = unpack!(block = ai.expr.to_mir(builder, block));
    let (block2, base_op) = emit_temp(
        builder,
        block,
        base_rvalue,
        &ai.expr.get_type(),
        span.clone(),
    );
    block = block2;
    let base_idx = match base_op {
        Operand::Copy(ref p) | Operand::Ref(ref p) => p.local,
        Operand::Const(_) => unreachable!("array base cannot be a constant"),
    };

    let idx_rvalue = unpack!(block = ai.index.to_mir(builder, block));
    let (block3, idx_op) = emit_temp(builder, block, idx_rvalue, &ai.index.get_type(), span);
    let idx_local = match idx_op {
        Operand::Copy(ref p) | Operand::Ref(ref p) => p.local,
        Operand::Const(_) => unreachable!("array index cannot be a constant"),
    };

    let place = Place::with_index(base_idx, idx_local);
    (block3, place)
}

fn lower_assign_target(
    builder: &mut MirBuilder,
    block: BasicBlock,
    target: &TypedAssignTarget,
) -> (BasicBlock, Place) {
    let span = Span::default();
    match target {
        TypedAssignTarget::Ident(ident) => {
            let idx = builder.lookup_local(ident);
            (block, Place::local(idx))
        }
        TypedAssignTarget::FieldAccess(fa) => lower_field_access(builder, block, fa, span),
        TypedAssignTarget::ArrayIndex(ai) => lower_array_index(builder, block, ai, span),
        TypedAssignTarget::Deref(expr) => {
            let mut block = block;
            let rvalue = unpack!(block = expr.to_mir(builder, block));
            let (block2, op) = emit_temp(builder, block, rvalue, &expr.get_type(), span);
            let base_idx = match op {
                Operand::Copy(ref p) | Operand::Ref(ref p) => p.local,
                Operand::Const(_) => unreachable!("deref target cannot be a constant"),
            };
            (block2, Place::with_deref(base_idx))
        }
    }
}

fn lower_value(v: &TypedValue) -> Rvalue {
    let c = match v {
        TypedValue::Number(n) => MirConst::Int(*n),
        TypedValue::Float(f) => MirConst::Float(*f),
        TypedValue::Boolean(b) => MirConst::Bool(*b),
        TypedValue::String(s) => MirConst::Str(s.clone()),
        TypedValue::Char(c) => MirConst::Char(*c),
        TypedValue::Array(_) => unreachable!("array literals handled before lower_value"),
    };
    Rvalue::Use(Operand::Const(c))
}

fn lower_array_literal(
    arr: &TypedArrayLiteral,
    builder: &mut MirBuilder,
    mut block: BasicBlock,
    span: Span,
) -> BlockAnd<Rvalue> {
    let elem_ty = arr.elem_typ.clone();
    if let Some(fill) = &arr.fill {
        let fill_rvalue = unpack!(block = fill.to_mir(builder, block));
        let (b, fill_op) = emit_temp(builder, block, fill_rvalue, &elem_ty, span.clone());
        block = b;
        let operands = (0..arr.size).map(|_| fill_op.clone()).collect();
        block.and(Rvalue::Aggregate(AggregateKind::Array(elem_ty), operands))
    } else {
        let mut operands = Vec::with_capacity(arr.elements.len());
        for elem in &arr.elements {
            let rvalue = unpack!(block = elem.to_mir(builder, block));
            let (b, op) = emit_temp(builder, block, rvalue, &elem_ty, span.clone());
            block = b;
            operands.push(op);
        }
        block.and(Rvalue::Aggregate(AggregateKind::Array(elem_ty), operands))
    }
}

pub(crate) fn emit_temp(
    builder: &mut MirBuilder,
    block: BasicBlock,
    rvalue: Rvalue,
    typ: &Types,
    span: Span,
) -> (BasicBlock, Operand) {
    let tmp_name = hades_tokens::Ident::new(format!("_tmp{}", builder.local_count()), span.clone());
    let idx = builder.build_local(tmp_name, typ.clone());
    let dest = Place::local(idx);
    builder.push_stmt(block, Statement::assign(dest.clone(), rvalue, span));
    (block, Operand::Copy(dest))
}
