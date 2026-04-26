use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult, CodegenValue};
use crate::codegen::traits::Visit;
use hades_ast::Types;
use hades_mir::mir::operand::{Operand};
use hades_mir::mir::place::{Place, PlaceElem};
use hades_mir::mir::rvalue::{AggregateKind, Rvalue};
use inkwell::AddressSpace;
use inkwell::types::BasicType;
use inkwell::values::{BasicValueEnum, PointerValue};

pub mod asexpr;
pub mod binary;
pub mod call;
pub mod struct_init;
pub mod unary;

use super::array_literal::build_array;
use asexpr::cast_value;
use binary::dispatch_binary_op;
use struct_init::build_alloca_struct;
use unary::{deref_pointer, dispatch_unary_op};

// ── Place → PointerValue ─────────────────────────────────────────────────────

/// Resolve a MIR `Place` to the LLVM pointer where the value lives.
/// Handles field projections (GEP), index projections (GEP), and deref.
pub(crate) fn codegen_place_ptr<'ctx>(
    place: &Place,
    context: &mut LLVMContext<'ctx>,
) -> CodegenResult<(PointerValue<'ctx>, Types)> {
    let fc = context.current_function_unchecked();
    let alloca = fc.alloca_map[place.local.index()];

    // We need the type of the base local to navigate projections.
    // We'll thread `cur_ptr` and `cur_ty` through the projection list.
    // The base type must be loaded from the function's locals — we grab it
    // by loading from the FunctionContext which we already got above.
    // But we need `locals` — those live on the MIR function, not context.
    // Strategy: stash a reference to locals separately.
    // Actually LLVMContext doesn't own locals; we need to pass them in.
    // Since this function is called from Visit impls that already have context,
    // we'll require callers to also pass the locals slice.
    // But we want codegen_place_ptr to only take `context`. So instead we rely
    // on the fact that the alloca already encodes the right LLVM type — we can
    // get the element type from the alloca's pointee type where needed.
    //
    // For now return the raw alloca; projections are applied below.
    let mut cur_ptr = alloca;

    let locals = context.current_function_unchecked().locals.clone();

    // Track current Hades type as we walk through projections.
    let mut cur_ty = locals[place.local.index()].ty.clone();

    for proj in &place.projection {
        match proj {
            PlaceElem::Deref => {
                // Load the pointer stored at cur_ptr.
                let llvm_ptr_type = context.context().ptr_type(AddressSpace::default()).into();
                cur_ptr = context
                    .load(cur_ptr, llvm_ptr_type, "deref_ptr")?
                    .into_pointer_value();
                // Advance the tracked type through the pointer.
                if let Types::Pointer(inner) = cur_ty {
                    cur_ty = *inner;
                }
            }
            PlaceElem::Field { name, index: _, ty } => {
                // cur_ty is the struct type (after any derefs above).
                let struct_name = cur_ty.unwrap_struct_name();
                let field_index = context
                    .symbols()
                    .structs()
                    .field_index(struct_name, name);
                let struct_llvm_type = context
                    .module()
                    .get_struct_type(struct_name.inner())
                    .ok_or_else(|| CodegenError::LLVMBuild {
                        message: format!("Struct type '{}' not found in LLVM module", struct_name.inner()),
                    })?;
                let zero = context.context().i32_type().const_zero();
                let field_idx = context.context().i32_type().const_int(field_index as u64, false);
                cur_ptr = unsafe {
                    context
                        .builder()
                        .build_in_bounds_gep(struct_llvm_type, cur_ptr, &[zero, field_idx], "field_ptr")
                        .map_err(|e| CodegenError::LLVMBuild {
                            message: format!("GEP field failed: {e}"),
                        })?
                };
                cur_ty = ty.clone();
            }
            PlaceElem::Index(index_local) => {
                // Get the index value from the index local's alloca.
                let index_alloca = context.current_function_unchecked().alloca_map[index_local.index()];
                let index_val = context
                    .load(index_alloca, context.context().i64_type().into(), "idx")?
                    .into_int_value();
                // Array locals hold a pointer alloca — load the array pointer first.
                let ptr_ty = context.context().ptr_type(inkwell::AddressSpace::default()).into();
                let array_ptr = context.load(cur_ptr, ptr_ty, "arr_ptr")?.into_pointer_value();
                let array_llvm = context
                    .type_converter()
                    .to_llvm_type(&cur_ty, context.module())?;
                let elem_ty = cur_ty.get_array_elem_type();
                cur_ty = elem_ty;
                let zero = context.context().i64_type().const_zero();
                cur_ptr = unsafe {
                    context
                        .builder()
                        .build_in_bounds_gep(array_llvm, array_ptr, &[zero, index_val], "elem_ptr")
                        .map_err(|e| CodegenError::LLVMBuild {
                            message: format!("GEP index failed: {e}"),
                        })?
                };
            }
        }
    }

    // Return the final pointer + the tracked type at that location.
    Ok((cur_ptr, cur_ty))
}

/// Determine the Hades type that a Place points to after all projections.
fn resolve_place_type(place: &Place, locals: &[hades_mir::mir::local::LocalDecl]) -> Types {
    let mut ty = locals[place.local.index()].ty.clone();
    for proj in &place.projection {
        match proj {
            PlaceElem::Deref => {
                if let Types::Pointer(inner) = ty {
                    ty = *inner;
                }
            }
            PlaceElem::Field { ty: field_ty, .. } => {
                ty = field_ty.clone();
            }
            PlaceElem::Index(_) => {
                ty = ty.get_array_elem_type();
            }
        }
    }
    ty
}

// ── Operand ──────────────────────────────────────────────────────────────────

impl Visit for Operand {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Operand::Const(c) => c.visit(context),
            Operand::Copy(place) => {
                let (ptr, ty) = codegen_place_ptr(place, context)?;
                let llvm_ty = context.type_converter().to_llvm_type(&ty, context.module())?;
                let val = context.load(ptr, llvm_ty, "copy")?;
                Ok(CodegenValue::new(val, ty))
            }
            Operand::Ref(place) => {
                let (ptr, ty) = codegen_place_ptr(place, context)?;
                Ok(CodegenValue::new(
                    ptr.into(),
                    Types::Pointer(Box::new(ty)),
                ))
            }
        }
    }
}

// ── Rvalue ───────────────────────────────────────────────────────────────────

impl Visit for Rvalue {
    type Output<'ctx> = CodegenValue<'ctx>;

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        match self {
            Rvalue::Use(op) => op.visit(context),

            Rvalue::BinaryOp(op, lhs, rhs) => {
                let lv = lhs.visit(context)?;
                let rv = rhs.visit(context)?;
                let lty = lv.unwrap_concrete()?.type_info().clone();
                let rty = rv.unwrap_concrete()?.type_info().clone();
                // Infer result type via compiler context.
                let result_type = context
                    .symbols()
                    .infer_binary_type(&lty, op, &rty, hades_error::Span::default())
                    .map_err(|_| CodegenError::TypeMismatch {
                        expected: format!("{:?} {:?} {:?}", lty, op, rty),
                        actual: "incompatible types".to_string(),
                    })?;
                let raw = dispatch_binary_op(lv.value()?, op, rv.value()?, &lty, context)?;
                Ok(CodegenValue::new(raw, result_type))
            }

            Rvalue::UnaryOp(op, operand) => {
                let val = operand.visit(context)?;
                let ty = val.unwrap_concrete()?.type_info().clone();
                match op {
                    hades_tokens::Op::Deref => {
                        // UnaryOp Deref: load through the pointer.
                        if let Types::Pointer(inner) = &ty {
                            deref_pointer(val.value()?, inner, context)
                        } else {
                            Err(CodegenError::TypeMismatch {
                                expected: "pointer".to_string(),
                                actual: ty.to_string(),
                            })
                        }
                    }
                    _ => {
                        let result_type = context
                            .symbols()
                            .infer_unary_type(op, &ty, hades_error::Span::default())
                            .map_err(|_| CodegenError::TypeMismatch {
                                expected: format!("{:?} {:?}", op, ty),
                                actual: "incompatible type".to_string(),
                            })?;
                        let raw = dispatch_unary_op(op, val.value()?, &ty, context)?;
                        Ok(CodegenValue::new(raw, result_type))
                    }
                }
            }

            Rvalue::Cast(operand, target_ty) => {
                let val = operand.visit(context)?;
                let from_ty = val.unwrap_concrete()?.type_info().clone();
                let raw = cast_value(context, val, &from_ty, target_ty)?;
                Ok(CodegenValue::new(raw, target_ty.clone()))
            }

            Rvalue::Aggregate(AggregateKind::Struct(struct_name), fields) => {
                let struct_llvm = context
                    .module()
                    .get_struct_type(struct_name)
                    .ok_or_else(|| CodegenError::LLVMBuild {
                        message: format!("Struct '{}' not found in LLVM module", struct_name),
                    })?;

                // Evaluate each field operand.
                let mut values: Vec<(BasicValueEnum, Types)> = Vec::new();
                for field_op in fields {
                    let cv = field_op.visit(context)?;
                    let ty = cv.unwrap_concrete()?.type_info().clone();
                    values.push((cv.value()?, ty));
                }

                let ptr = build_alloca_struct(context, struct_llvm, &values)?;
                let struct_val = context.load(ptr, struct_llvm.into(), "struct_val")?;
                let struct_ty = Types::Struct(hades_tokens::Name::new(
                    struct_name.clone(),
                    Default::default(),
                ));
                Ok(CodegenValue::new(struct_val, struct_ty))
            }

            Rvalue::Aggregate(AggregateKind::Array(elem_ty), elems) => {
                let llvm_elem_type = context
                    .type_converter()
                    .to_llvm_type(elem_ty, context.module())?;
                let count = elems.len() as u32;
                let llvm_array_type = llvm_elem_type.array_type(count);

                // Evaluate each element operand, resolving structs to their
                // alloca pointer (same as old resolve_elements).
                let mut resolved: Vec<BasicValueEnum> = Vec::new();
                for elem_op in elems {
                    let cv = elem_op.visit(context)?;
                    let val = cv.value()?;
                    let resolved_val = match (elem_ty, val) {
                        (Types::Struct(_), v) => {
                            let tmp = context.create_alloca("struct_elem", llvm_elem_type)?;
                            context
                                .builder()
                                .build_store(tmp, v)
                                .map_err(|e| CodegenError::LLVMBuild {
                                    message: format!("Failed to store struct elem: {e}"),
                                })?;
                            tmp.into()
                        }
                        // String values are already pointers — pass through directly.
                        (Types::String, v) => v,
                        (_, BasicValueEnum::PointerValue(ptr)) => {
                            context.load(ptr, llvm_elem_type, "elem")?
                        }
                        (_, v) => v,
                    };
                    resolved.push(resolved_val);
                }

                let array_ty_wrapper = Types::Array(elem_ty.array_type(count as usize));
                let ptr = build_array(
                    context,
                    resolved,
                    elem_ty,
                    llvm_array_type,
                    llvm_elem_type,
                )?;
                Ok(CodegenValue::new(ptr.into(), array_ty_wrapper))
            }
        }
    }
}
