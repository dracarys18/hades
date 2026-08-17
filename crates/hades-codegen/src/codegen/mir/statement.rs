use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::Types;
use hades_mir::mir::operand::Operand;
use hades_mir::mir::place::Place;
use hades_mir::mir::rvalue::Rvalue;
use hades_mir::mir::stmt::{Statement, StatementKind};
use hades_mir::visit::Visit;
use inkwell::IntPredicate;
use inkwell::module::Linkage;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};

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

    let mut values: Vec<(BasicValueEnum<'ctx>, Types)> = Vec::with_capacity(operands.len());
    for operand in operands {
        values.push(operand.visit(context)?);
    }

    if let Types::Array(_) = &agg_ty
        && values.iter().all(|(v, _)| v.is_const())
        && let Some(global_ptr) = build_const_array_global(context, &agg_ty, &values)?
    {
        context.create_store(base_ptr, global_ptr.into(), &agg_ty)?;
        return Ok(());
    }

    for (i, (value, value_ty)) in values.into_iter().enumerate() {
        let index = context.context().i32_type().const_int(i as u64, false);
        let slot = context.array_elem_ptr(base_ptr, &agg_ty, index)?;
        context.create_store(slot, value, &value_ty)?;
    }
    Ok(())
}

fn build_const_array_global<'ctx>(
    context: &LLVMContext<'ctx>,
    agg_ty: &Types,
    values: &[(BasicValueEnum<'ctx>, Types)],
) -> CodegenResult<Option<PointerValue<'ctx>>> {
    let elem_ty = context
        .type_converter()
        .to_llvm_type(&agg_ty.get_array_elem_type(), context.module())?;
    let array = match elem_ty {
        BasicTypeEnum::IntType(t) => {
            let elems: Vec<_> = values.iter().map(|(v, _)| v.into_int_value()).collect();
            t.const_array(&elems).as_basic_value_enum()
        }
        BasicTypeEnum::FloatType(t) => {
            let elems: Vec<_> = values.iter().map(|(v, _)| v.into_float_value()).collect();
            t.const_array(&elems).as_basic_value_enum()
        }
        BasicTypeEnum::PointerType(t) => {
            let elems: Vec<_> = values.iter().map(|(v, _)| v.into_pointer_value()).collect();
            t.const_array(&elems).as_basic_value_enum()
        }
        _ => return Ok(None),
    };

    let global = context
        .module()
        .add_global(array.get_type(), None, "const_arr");
    global.set_initializer(&array);
    global.set_constant(true);
    global.set_linkage(Linkage::Private);
    Ok(Some(global.as_pointer_value()))
}

fn store_repeat<'ctx>(
    context: &mut LLVMContext<'ctx>,
    place: &Place,
    operand: &Operand,
    count: usize,
) -> CodegenResult<()> {
    let (base_ptr, agg_ty) = place.visit(context)?;
    let (value, value_ty) = operand.visit(context)?;

    let i64_type = context.context().i64_type();
    let one = i64_type.const_int(1, false);
    let bound = i64_type.const_int(count as u64, false);

    let counter = context.create_alloca("repeat.i", i64_type.into())?;
    context
        .builder()
        .build_store(counter, i64_type.const_zero())
        .map_err(CodegenError::from)?;

    let header = context.create_basic_block("repeat.header");
    let body = context.create_basic_block("repeat.body");
    let exit = context.create_basic_block("repeat.exit");

    context.build_unconditional_branch(header)?;

    context.position_at_end(header);
    let index = context
        .load(counter, i64_type.into(), "repeat.i")?
        .into_int_value();
    let cond = context
        .builder()
        .build_int_compare(IntPredicate::ULT, index, bound, "repeat.cond")
        .map_err(CodegenError::from)?;
    context.build_conditional_branch(cond.into(), body, exit)?;

    context.position_at_end(body);
    let index = context
        .load(counter, i64_type.into(), "repeat.i")?
        .into_int_value();
    let slot = context.array_elem_ptr(base_ptr, &agg_ty, index)?;
    context.create_store(slot, value, &value_ty)?;
    let next = context
        .builder()
        .build_int_add(index, one, "repeat.next")
        .map_err(CodegenError::from)?;
    context
        .builder()
        .build_store(counter, next)
        .map_err(CodegenError::from)?;
    context.build_unconditional_branch(header)?;

    context.position_at_end(exit);
    Ok(())
}
