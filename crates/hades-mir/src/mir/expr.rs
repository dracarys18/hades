use hades_ast::{
    TypedArrayIndex, TypedArrayLiteral, TypedAssignExpr, TypedAssignTarget, TypedBinaryExpr,
    TypedExpr, TypedFieldAccess, TypedValue, Types,
};
use hades_error::Span;
use hades_tokens::Op;

use crate::mir::builder::MirBuilder;
use crate::mir::operand::{MirConst, Operand};
use crate::mir::place::{Place, PlaceElem};
use crate::mir::rvalue::{AggregateKind, Rvalue};
use crate::mir::stmt::Statement;
use crate::mir::terminator::{CallTarget, Terminator, TerminatorKind};
use crate::{BasicBlock, BlockAnd, BlockAndExt, ToMir, unpack};

impl ToMir for TypedExpr {
    type Output = Rvalue;

    fn to_mir(&self, builder: &mut MirBuilder<'_>, mut block: BasicBlock) -> BlockAnd<Rvalue> {
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
                    builder.as_operand(block, lhs_rvalue, &left.get_type(), span.clone());
                block = block2;

                let rhs_rvalue = unpack!(block = right.to_mir(builder, block));
                let (block3, rhs) =
                    builder.as_operand(block, rhs_rvalue, &right.get_type(), span.clone());
                block = block3;

                block.and(Rvalue::BinaryOp(op.clone(), lhs, rhs))
            }

            TypedExpr::Unary { op, expr, .. } => {
                let rvalue = unpack!(block = expr.to_mir(builder, block));
                if *op == Op::Ref {
                    let (block2, operand) =
                        builder.as_operand(block, rvalue, &expr.get_type(), span.clone());
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
                        builder.as_operand(block, rvalue, &expr.get_type(), span);
                    block2.and(Rvalue::UnaryOp(op.clone(), operand))
                }
            }

            TypedExpr::As(a) => {
                let rvalue = unpack!(block = a.expr.to_mir(builder, block));
                let (block2, operand) = builder.as_operand(block, rvalue, &a.expr.get_type(), span);
                block2.and(Rvalue::Cast(operand, a.target_type.clone()))
            }

            TypedExpr::Assign(TypedAssignExpr {
                target, value, op, ..
            }) => {
                let val_rvalue = unpack!(block = value.to_mir(builder, block));
                let rvalue = match op {
                    Op::PlusEqual => {
                        let (block2, val_op) =
                            builder.as_operand(block, val_rvalue, &value.get_type(), span.clone());
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
                            builder.as_operand(block, val_rvalue, &value.get_type(), span.clone());
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
                let (block2, place) = lower_field_place(builder, block, fa);
                block2.and(Rvalue::Use(Operand::Copy(place)))
            }

            TypedExpr::ArrayIndex(ai) => {
                let (block2, place) = lower_index_place(builder, block, ai);
                block2.and(Rvalue::Use(Operand::Copy(place)))
            }

            TypedExpr::StructInit { name, fields, .. } => {
                let mut operands: Vec<Operand> = Vec::new();
                for (_, field_expr) in fields {
                    let rvalue = unpack!(block = field_expr.to_mir(builder, block));
                    let (block2, op) =
                        builder.as_operand(block, rvalue, &field_expr.get_type(), span.clone());
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
                        builder.as_operand(block, recv_rvalue, &recv.get_type(), span.clone());
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
                        builder.as_operand(block, rvalue, &arg.get_type(), span.clone());
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

fn lower_place(
    builder: &mut MirBuilder<'_>,
    mut block: BasicBlock,
    expr: &TypedExpr,
) -> (BasicBlock, Place) {
    match expr {
        TypedExpr::Ident { ident, .. } => (block, Place::local(builder.lookup_local(ident))),
        TypedExpr::Unary {
            op: Op::Deref,
            expr: inner,
            ..
        } => {
            let (block2, mut place) = lower_place(builder, block, inner);
            place.projection.push(PlaceElem::Deref);
            (block2, place)
        }
        TypedExpr::FieldAccess(fa) => lower_field_place(builder, block, fa),
        TypedExpr::ArrayIndex(ai) => lower_index_place(builder, block, ai),
        _ => {
            let span = Span::default();
            let rvalue = unpack!(block = expr.to_mir(builder, block));
            let (block2, op) = builder.as_operand(block, rvalue, &expr.get_type(), span.clone());
            let local = operand_to_local(builder, block2, op, &expr.get_type(), span);
            (block2, Place::local(local))
        }
    }
}

fn lower_field_place(
    builder: &mut MirBuilder<'_>,
    block: BasicBlock,
    fa: &TypedFieldAccess,
) -> (BasicBlock, Place) {
    let (block2, mut place) = lower_place(builder, block, &fa.expr);
    if matches!(fa.expr.get_type(), Types::Pointer(_)) {
        place.projection.push(PlaceElem::Deref);
    }
    let struct_name = fa.struct_type.unwrap_struct_name();
    let field_index = builder.symbols().structs().field_index(struct_name, &fa.field);
    place.projection.push(PlaceElem::Field {
        name: fa.field.clone(),
        index: field_index,
        ty: fa.field_type.clone(),
    });
    (block2, place)
}

fn lower_index_place(
    builder: &mut MirBuilder<'_>,
    mut block: BasicBlock,
    ai: &TypedArrayIndex,
) -> (BasicBlock, Place) {
    let span = Span::default();
    let (block2, mut place) = lower_place(builder, block, &ai.expr);
    block = block2;
    if matches!(ai.expr.get_type(), Types::Pointer(_)) {
        place.projection.push(PlaceElem::Deref);
    }
    let idx_rvalue = unpack!(block = ai.index.to_mir(builder, block));
    let (block3, idx_op) = builder.as_operand(block, idx_rvalue, &ai.index.get_type(), span.clone());
    let idx_local = operand_to_local(builder, block3, idx_op, &ai.index.get_type(), span);
    place.projection.push(PlaceElem::Index(idx_local));
    (block3, place)
}

fn operand_to_local(
    builder: &mut MirBuilder<'_>,
    block: BasicBlock,
    op: Operand,
    ty: &Types,
    span: Span,
) -> usize {
    if let Operand::Copy(p) = &op
        && p.projection.is_empty()
    {
        return p.local;
    }
    let tmp = hades_tokens::Ident::new(format!("_tmp{}", builder.local_count()), span.clone());
    let idx = builder.build_local(tmp, ty.clone());
    builder.push_stmt(block, Statement::assign(Place::local(idx), Rvalue::Use(op), span));
    idx
}

fn lower_assign_target(
    builder: &mut MirBuilder<'_>,
    block: BasicBlock,
    target: &TypedAssignTarget,
) -> (BasicBlock, Place) {
    match target {
        TypedAssignTarget::Ident(ident) => (block, Place::local(builder.lookup_local(ident))),
        TypedAssignTarget::FieldAccess(fa) => lower_field_place(builder, block, fa),
        TypedAssignTarget::ArrayIndex(ai) => lower_index_place(builder, block, ai),
        TypedAssignTarget::Deref(expr) => {
            let (block2, mut place) = lower_place(builder, block, expr);
            place.projection.push(PlaceElem::Deref);
            (block2, place)
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
    builder: &mut MirBuilder<'_>,
    mut block: BasicBlock,
    span: Span,
) -> BlockAnd<Rvalue> {
    let elem_ty = arr.elem_typ.get_array_elem_type();
    if let Some(fill) = &arr.fill {
        let fill_rvalue = unpack!(block = fill.to_mir(builder, block));
        let (b, fill_op) = builder.as_operand(block, fill_rvalue, &elem_ty, span.clone());
        b.and(Rvalue::Repeat(fill_op, arr.size))
    } else {
        let mut operands = Vec::with_capacity(arr.elements.len());
        for elem in &arr.elements {
            let rvalue = unpack!(block = elem.to_mir(builder, block));
            let (b, op) = builder.as_operand(block, rvalue, &elem_ty, span.clone());
            block = b;
            operands.push(op);
        }
        block.and(Rvalue::Aggregate(AggregateKind::Array(elem_ty), operands))
    }
}
