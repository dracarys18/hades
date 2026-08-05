use crate::codegen::context::LLVMContext;
use crate::codegen::error::CodegenResult;
use hades_mir::mir::operand::Operand;
use hades_mir::mir::place::Place;
use hades_mir::mir::rvalue::Rvalue;
use hades_mir::mir::stmt::{Statement, StatementKind};
use hades_mir::visit::Visit;

impl<'ctx> Visit<LLVMContext<'ctx>> for Statement {
    type Output = ();

    fn visit(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output> {
        match &self.kind {
            StatementKind::Nop => Ok(()),
            StatementKind::Assign(place, rvalue) => match rvalue.as_ref() {
                Rvalue::Aggregate(_, operands) => store_aggregate(context, place, operands),
                Rvalue::Repeat(operand, count) => store_repeat(context, place, operand, *count),
                other => {
                    let (value, _) = other.visit(context)?;
                    let (ptr, ty) = place.visit(context)?;
                    context.create_store(ptr, value, &ty)
                }
            },
        }
    }
}

fn store_aggregate<'ctx>(
    context: &mut LLVMContext<'ctx>,
    place: &Place,
    operands: &[Operand],
) -> CodegenResult<()> {
    let (base_ptr, agg_ty) = place.visit(context)?;
    for (i, operand) in operands.iter().enumerate() {
        let (value, value_ty) = operand.visit(context)?;
        let index = context.context().i32_type().const_int(i as u64, false);
        let slot = context.array_elem_ptr(base_ptr, &agg_ty, index)?;
        context.create_store(slot, value, &value_ty)?;
    }
    Ok(())
}

fn store_repeat<'ctx>(
    context: &mut LLVMContext<'ctx>,
    place: &Place,
    operand: &Operand,
    count: usize,
) -> CodegenResult<()> {
    let (base_ptr, agg_ty) = place.visit(context)?;
    let (value, value_ty) = operand.visit(context)?;
    for i in 0..count {
        let index = context.context().i32_type().const_int(i as u64, false);
        let slot = context.array_elem_ptr(base_ptr, &agg_ty, index)?;
        context.create_store(slot, value, &value_ty)?;
    }
    Ok(())
}
