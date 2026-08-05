use crate::BuiltinRegistar;
use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use hades_ast::Types;
use hades_mir::mir::operand::Operand;
use hades_mir::mir::place::Place;
use hades_mir::mir::terminator::{CallTarget, SwitchTargets, Terminator, TerminatorKind};
use inkwell::values::{BasicMetadataValueEnum, PointerValue};

use super::operand::operand_type;

impl Visit for Terminator {
    type Output<'ctx> = ();

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match &self.kind {
            TerminatorKind::Goto(target) => {
                let block = context.mir_block(target.0);
                context.build_unconditional_branch(block)
            }
            TerminatorKind::Return => lower_return(context),
            TerminatorKind::Unreachable => context
                .builder()
                .build_unreachable()
                .map(|_| ())
                .map_err(CodegenError::from),
            TerminatorKind::SwitchInt {
                discriminant,
                targets,
            } => lower_switch(context, discriminant, targets),
            TerminatorKind::Call {
                target,
                args,
                dest,
                successor,
            } => lower_call(context, target, args, dest, successor.0),
        }
    }
}

fn lower_return(context: &mut LLVMContext<'_>) -> CodegenResult<()> {
    let return_ty = context.mir_local_type(hades_mir::RETURN_LOCAL);
    if return_ty == Types::Void {
        return context.build_return(None);
    }
    let return_ptr = context.mir_local_ptr(hades_mir::RETURN_LOCAL);
    let llvm_type = context
        .type_converter()
        .to_llvm_type(&return_ty, context.module())?;
    let value = context.load(return_ptr, llvm_type, "ret")?;
    context.build_return(Some(value))
}

fn lower_switch(
    context: &mut LLVMContext<'_>,
    discriminant: &Operand,
    targets: &SwitchTargets,
) -> CodegenResult<()> {
    let (value, _) = discriminant.visit(context)?;
    let discriminant = value.into_int_value();

    if targets.values.as_slice() == [1] && targets.blocks.len() == 1 {
        let then_block = context.mir_block(targets.blocks[0].0);
        let else_block = context.mir_block(targets.otherwise.0);
        return context.build_conditional_branch(discriminant.into(), then_block, else_block);
    }

    let else_block = context.mir_block(targets.otherwise.0);
    let cases: Vec<_> = targets
        .values
        .iter()
        .zip(targets.blocks.iter())
        .map(|(value, block)| {
            (
                discriminant.get_type().const_int(*value as u64, false),
                context.mir_block(block.0),
            )
        })
        .collect();

    context
        .builder()
        .build_switch(discriminant, else_block, &cases)
        .map(|_| ())
        .map_err(CodegenError::from)
}

fn lower_call<'ctx>(
    context: &mut LLVMContext<'ctx>,
    target: &CallTarget,
    args: &[Operand],
    dest: &Place,
    successor: usize,
) -> CodegenResult<()> {
    let result = match target {
        CallTarget::Function(name) => {
            let name = name.inner().to_string();
            if BuiltinRegistar::is_compile_time_function(&name) {
                lower_builtin(context, &name, args)?
            } else {
                let mut argv: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
                for arg in args {
                    let (value, _) = arg.visit(context)?;
                    argv.push(value.into());
                }
                context.build_call(&name, &argv)?
            }
        }
        CallTarget::Method { receiver, method } => {
            let self_ptr = method_self_ptr(context, receiver)?;
            let mut argv: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len() + 1);
            argv.push(self_ptr.into());
            for arg in args {
                let (value, _) = arg.visit(context)?;
                argv.push(value.into());
            }
            context.build_call(method.inner(), &argv)?
        }
        CallTarget::Qualified { ty, method } => {
            return Err(CodegenError::LLVMBuild {
                message: format!("Qualified call {ty:?}::{method} is not supported"),
            });
        }
    };

    if let CodegenValue::Concrete(_) = &result {
        let value = result.value()?;
        let (ptr, ty) = dest.visit(context)?;
        context.create_store(ptr, value, &ty)?;
    }

    let successor = context.mir_block(successor);
    context.build_unconditional_branch(successor)
}

fn method_self_ptr<'ctx>(
    context: &mut LLVMContext<'ctx>,
    receiver: &Operand,
) -> CodegenResult<PointerValue<'ctx>> {
    match receiver {
        Operand::Copy(place) | Operand::Ref(place) => {
            let (ptr, ty) = place.visit(context)?;
            context.deref_if_pointer(ptr, &ty)
        }
        Operand::Const(_) => Err(CodegenError::LLVMBuild {
            message: "method receiver cannot be a constant".to_string(),
        }),
    }
}

fn lower_builtin<'ctx>(
    context: &LLVMContext<'ctx>,
    name: &str,
    args: &[Operand],
) -> CodegenResult<CodegenValue<'ctx>> {
    match name {
        "len" => {
            let arg = args.first().ok_or_else(|| CodegenError::LLVMBuild {
                message: "len requires exactly one array argument".to_string(),
            })?;
            let size = operand_type(context, arg)?.get_array_size();
            let value = context.context().i64_type().const_int(size as u64, false);
            Ok(CodegenValue::new(value.into(), Types::Int))
        }
        _ => Err(CodegenError::LLVMBuild {
            message: format!("Unknown compile-time builtin: {name}"),
        }),
    }
}
