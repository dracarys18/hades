use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;
use std::process::Command;

use crate::consts::BUILD_PATH;

use crate::ast::Types;
use crate::tokens::Ident;
use crate::typed_ast::TypedExpr;
use crate::typed_ast::TypedFuncDef;
use crate::typed_ast::TypedProgram;
use crate::typed_ast::TypedStmt;

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    variables: HashMap<String, PointerValue<'ctx>>,
    current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        let codegen = CodeGen {
            context,
            module,
            builder,
            variables: HashMap::new(),
            current_function: None,
        };

        // Declare printf function for print support
        codegen.declare_printf();

        codegen
    }

    fn declare_printf(&self) {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let printf_type = self.context.i32_type().fn_type(&[ptr_type.into()], true);
        self.module.add_function("printf", printf_type, None);
    }

    pub fn compile(&mut self, program: &TypedProgram) -> Result<(), String> {
        // Second pass: compile all statements
        for stmt in program {
            self.compile_stmt(stmt)?;
        }

        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &TypedStmt) -> Result<(), String> {
        match stmt {
            TypedStmt::FuncDef(func) => self.compile_function(func),
            TypedStmt::TypedExpr(expr) => {
                let expr = &expr.expr;
                self.compile_expr(expr)?;
                Ok(())
            }
            TypedStmt::Return(expr) => {
                let expr = expr.expr.clone().map(|i| i.expr);
                self.compile_return(&expr)
            }
            _ => Err(format!("Statement not yet implemented: {stmt:?}")),
        }
    }

    fn compile_function(&mut self, func: &TypedFuncDef) -> Result<(), String> {
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty).into())
            .collect();

        let fn_type = match self.type_to_llvm(&func.return_type) {
            BasicTypeEnum::IntType(int_type) => int_type.fn_type(&param_types, false),
            _ => self.context.void_type().fn_type(&param_types, false),
        };

        let function = self.module.add_function(func.name.inner(), fn_type, None);
        self.current_function = Some(function);

        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        self.variables.clear();

        for (i, (param_name, param_type)) in func.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(self.type_to_llvm(param_type), param_name.inner())
                .map_err(|e| format!("Failed to create alloca: {e}"))?;

            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| format!("Failed to store parameter: {e}"))?;

            self.variables
                .insert(param_name.inner().to_string(), alloca);
        }

        for stmt in &func.body.stmts {
            self.compile_stmt(stmt)?;
        }

        if func.return_type.eq(&Types::Void) {
            self.builder
                .build_return(None)
                .map_err(|e| format!("Failed to build return: {e}"))?;
        }

        Ok(())
    }

    fn compile_expr(&mut self, expr: &TypedExpr) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            TypedExpr::Number(n) => Ok(self.context.i64_type().const_int(*n as u64, true).into()),
            TypedExpr::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            TypedExpr::String(s) => {
                let global_string = self
                    .builder
                    .build_global_string_ptr(&s, "str")
                    .map_err(|e| format!("Failed to create string: {e}"))?;
                Ok(global_string.as_pointer_value().into())
            }
            TypedExpr::Call { func, args, .. } => self.compile_call(func, args),
            _ => Err(format!("Expression not yet implemented: {expr:?}")),
        }
    }

    fn compile_call(
        &mut self,
        func: &Ident,
        args: &[TypedExpr],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if func.inner() == "print" {
            return self.compile_print_call(args);
        }

        Err(format!("Function call not yet implemented: {func:?}"))
    }

    fn compile_print_call(&mut self, args: &[TypedExpr]) -> Result<BasicValueEnum<'ctx>, String> {
        let printf = self
            .module
            .get_function("printf")
            .ok_or_else(|| "printf function not found".to_string())?;

        for arg in args {
            let format_string = match &arg {
                TypedExpr::String(_) => "%s\n\0",
                TypedExpr::Number(_) => "%lld\n\0",
                TypedExpr::Float(_) => "%f\n\0",
                TypedExpr::Boolean(_) => "%d\n\0",
                _ => unreachable!("Unsupported type for print"),
            };

            let format_str = self
                .builder
                .build_global_string_ptr(format_string, "fmt")
                .map_err(|e| format!("Failed to create format string: {e}"))?;

            let val = self.compile_expr(arg)?;

            self.builder
                .build_call(
                    printf,
                    &[format_str.as_pointer_value().into(), val.into()],
                    "printf_call",
                )
                .map_err(|e| format!("Failed to build printf call: {e}"))?;
        }

        Ok(self.context.i32_type().const_zero().into())
    }

    fn compile_return(&mut self, expr: &Option<TypedExpr>) -> Result<(), String> {
        if let Some(expr) = expr {
            let val = self.compile_expr(expr)?;
            self.builder
                .build_return(Some(&val))
                .map_err(|e| format!("Failed to build return: {e}"))?;
        } else {
            self.builder
                .build_return(None)
                .map_err(|e| format!("Failed to build return: {e}"))?;
        }
        Ok(())
    }

    fn type_to_llvm(&self, ty: &Types) -> BasicTypeEnum<'ctx> {
        match ty {
            Types::Int => self.context.i64_type().into(),
            Types::Float => self.context.f64_type().into(),
            Types::Bool => self.context.bool_type().into(),
            Types::String => self.context.ptr_type(AddressSpace::default()).into(),
            _ => unreachable!("These are not implemented yet"),
        }
    }

    pub fn write_ir_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        self.module
            .print_to_file(path)
            .map_err(|e| format!("Failed to write IR to file: {e}"))
    }

    pub fn write_exec(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| format!("Failed to get target from triple: {e}"))?;
        let obj_path = format!("{BUILD_PATH}/temp.o");
        let object_path = std::path::Path::new(obj_path.as_str());

        let target_machine_options = inkwell::targets::TargetMachineOptions::default()
            .set_level(OptimizationLevel::Aggressive)
            .set_code_model(CodeModel::Default)
            .set_reloc_mode(RelocMode::Default);

        let tm = target
            .create_target_machine_from_options(&target_triple, target_machine_options)
            .ok_or("Failed to create TargetMachine")?;

        tm.write_to_file(&self.module, FileType::Object, object_path)
            .expect("Failed to write object file");

        let status = Command::new("clang")
            .arg(obj_path)
            .arg("-o")
            .arg(path.as_ref())
            .status()
            .map_err(|e| format!("Failed to invoke clang: {e}"))?;

        if !status.success() {
            return Err(format!("Clang failed with exit code: {status:?}"));
        }

        Ok(())
    }

    pub fn cleanup(&self) {
        let _ = std::fs::remove_file(format!("{BUILD_PATH}/temp.ll"));
        let _ = std::fs::remove_file(format!("{BUILD_PATH}/temp.o"));
    }
}
