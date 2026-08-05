use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::traits::Visit;
use hades_ast::Types;
use hades_mir::mir::place::{Place, PlaceElem};
use inkwell::values::PointerValue;

impl Visit for Place {
    type Output<'ctx> = (PointerValue<'ctx>, Types);

    fn visit<'ctx>(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output<'ctx>> {
        let mut ptr = context.mir_local_ptr(self.local);
        let mut ty = context.mir_local_type(self.local);

        for elem in &self.projection {
            match elem {
                PlaceElem::Deref => {
                    let inner = match &ty {
                        Types::Pointer(inner) => (**inner).clone(),
                        other => {
                            return Err(CodegenError::TypeMismatch {
                                expected: "pointer".to_string(),
                                actual: other.to_string(),
                            });
                        }
                    };
                    let ptr_type = context.type_converter().ptr_type().into();
                    ptr = context.load(ptr, ptr_type, "deref")?.into_pointer_value();
                    ty = inner;
                }
                PlaceElem::Field {
                    index,
                    ty: field_ty,
                    ..
                } => {
                    ptr = context.struct_field_ptr(ptr, &ty, *index)?;
                    ty = field_ty.clone();
                }
                PlaceElem::Index(index_local) => {
                    let index_ptr = context.mir_local_ptr(*index_local);
                    let i64_type = context.context().i64_type();
                    let index_val = context
                        .load(index_ptr, i64_type.into(), "index")?
                        .into_int_value();
                    ptr = context.array_elem_ptr(ptr, &ty, index_val)?;
                    ty = ty.get_array_elem_type();
                }
            }
        }

        Ok((ptr, ty))
    }
}

pub(super) fn place_type(context: &LLVMContext<'_>, place: &Place) -> CodegenResult<Types> {
    let mut ty = context.mir_local_type(place.local);
    for elem in &place.projection {
        ty = match elem {
            PlaceElem::Deref => match ty {
                Types::Pointer(inner) => *inner,
                other => {
                    return Err(CodegenError::TypeMismatch {
                        expected: "pointer".to_string(),
                        actual: other.to_string(),
                    });
                }
            },
            PlaceElem::Field { ty: field_ty, .. } => field_ty.clone(),
            PlaceElem::Index(_) => ty.get_array_elem_type(),
        };
    }
    Ok(ty)
}
