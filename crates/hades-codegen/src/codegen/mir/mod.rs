mod func;
mod operand;
mod place;
mod rvalue;
mod statement;
mod terminator;

use crate::codegen::context::LLVMContext;
use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::{FuncKind, TypedFieldKind, TypedFuncDef, TypedStmt, TypedStructDef};
use hades_mir::mir::module::MirModule;
use hades_mir::visit::Visit;
use hades_semantic::{ModuleSignatures, TypedModule};
use inkwell::intrinsics::Intrinsic;
use inkwell::module::Linkage;
use inkwell::types::BasicTypeEnum;
use inkwell::{
    OptimizationLevel,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};

impl<'ctx> Visit<LLVMContext<'ctx>> for MirModule {
    type Output = ();

    fn visit(&self, context: &mut LLVMContext<'ctx>) -> CodegenResult<Self::Output> {
        for func in &self.functions {
            func.visit(context)?;
        }
        Ok(())
    }
}

fn build<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
) -> CodegenResult<LLVMContext<'ctx>> {
    let llvm_module = llvm_ctx.create_module(&typed_module.path.to_string());
    let mut context = LLVMContext::new(&typed_module.ctx, llvm_ctx, llvm_module);

    context.declare_imports(import_sigs)?;
    declare_module(&mut context, typed_module)?;

    let mir = hades_mir::lower(typed_module.clone());
    mir.visit(&mut context)?;

    Ok(context)
}

pub fn emit_ir<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
) -> CodegenResult<String> {
    let context = build(typed_module, import_sigs, llvm_ctx)?;
    Ok(context.module().print_to_string().to_string())
}

pub fn compile<'ctx>(
    typed_module: &'ctx TypedModule,
    import_sigs: &[&ModuleSignatures],
    llvm_ctx: &'ctx inkwell::context::Context,
    output_path: &std::path::Path,
) -> CodegenResult<()> {
    Target::initialize_all(&InitializationConfig::default());

    let context = build(typed_module, import_sigs, llvm_ctx)?;

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| CodegenError::LLVMBuild {
        message: format!("Failed to get target from triple: {e}"),
    })?;
    let target_machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| CodegenError::LLVMBuild {
            message: "Failed to create target machine".to_string(),
        })?;

    target_machine
        .write_to_file(context.module(), FileType::Object, output_path)
        .map_err(|e| CodegenError::LLVMBuild {
            message: format!("Failed to write object file: {e}"),
        })
}

fn declare_module<'ctx>(
    context: &mut LLVMContext<'ctx>,
    typed_module: &TypedModule,
) -> CodegenResult<()> {
    let structs: Vec<&TypedStructDef> = typed_module
        .program
        .iter()
        .filter_map(|stmt| match stmt {
            TypedStmt::StructDef(def) => Some(def),
            _ => None,
        })
        .collect();

    for def in &structs {
        context.context().opaque_struct_type(def.name.inner());
    }

    for def in &structs {
        declare_struct(context, def)?;
    }

    for stmt in typed_module.program.iter() {
        if let TypedStmt::FuncDef(def) = stmt {
            declare_function(context, def)?;
        }
    }

    Ok(())
}

fn declare_struct<'ctx>(context: &mut LLVMContext<'ctx>, def: &TypedStructDef) -> CodegenResult<()> {
    let struct_type = context
        .context()
        .get_struct_type(def.name.inner())
        .expect("opaque struct type created before bodies");

    let mut field_types: Vec<BasicTypeEnum> = Vec::new();
    for (_, field) in def.fields.iter() {
        if let TypedFieldKind::Var(_) = field {
            let typ = field.get_type();
            let llvm_type = context
                .type_converter()
                .to_llvm_type(&typ, context.module())?;
            field_types.push(llvm_type);
        }
    }
    struct_type.set_body(&field_types, false);

    for (_, field) in def.fields.iter() {
        if let TypedFieldKind::Func(method) = field {
            declare_function(context, method)?;
        }
    }

    Ok(())
}

fn declare_function<'ctx>(context: &mut LLVMContext<'ctx>, def: &TypedFuncDef) -> CodegenResult<()> {
    let signature = &def.signature;
    let param_types = context
        .type_converter()
        .params_to_llvm_types(signature, context.module())?;

    match &signature.kind {
        FuncKind::Extern { variadic } => {
            if context.module().get_function(def.name.link_name()).is_some() {
                return Ok(());
            }
            let fn_type = context.build_fn_type(&signature.return_type, &param_types, *variadic)?;
            context
                .module()
                .add_function(def.name.link_name(), fn_type, Some(Linkage::External));
        }
        FuncKind::Intrinsic(llvm_name) => {
            let type_slice: Vec<BasicTypeEnum> = param_types
                .iter()
                .map(|t| BasicTypeEnum::try_from(*t).expect("param type is not a basic type"))
                .collect();
            let intrinsic = Intrinsic::find(llvm_name).ok_or_else(|| CodegenError::LLVMBuild {
                message: format!("LLVM intrinsic '{llvm_name}' not found"),
            })?;
            intrinsic
                .get_declaration(context.module(), &type_slice)
                .ok_or_else(|| CodegenError::LLVMBuild {
                    message: format!("Failed to get declaration for '{llvm_name}'"),
                })?;
        }
        FuncKind::Normal => {
            if context.module().get_function(def.name.inner()).is_some() {
                return Ok(());
            }
            let fn_type = context.build_fn_type(&signature.return_type, &param_types, false)?;
            context
                .module()
                .add_function(def.name.inner(), fn_type, None);
        }
    }

    Ok(())
}
