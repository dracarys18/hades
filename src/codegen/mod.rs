use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;

use crate::ast::{Expr, Program, Stmt, Types};
use crate::tokens::Ident;

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
        let i8_ptr_type = self.context.i8_type();
        let printf_type = self.context.i32_type().fn_type(&[i8_ptr_type.into()], true);
        self.module.add_function("printf", printf_type, None);
    }

    pub fn compile(&mut self, program: Program) -> Result<(), String> {
        // Second pass: compile all statements
        for stmt in program {
            self.compile_stmt(stmt)?;
        }

        Ok(())
    }

    fn compile_stmt(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::FuncDef {
                name,
                params,
                return_type,
                body,
                ..
            } => self.compile_function(name, params, return_type, body),
            Stmt::Expr { expr, .. } => {
                self.compile_expr(expr)?;
                Ok(())
            }
            Stmt::Return { expr, .. } => self.compile_return(expr),
            _ => Err(format!("Statement not yet implemented: {stmt:?}")),
        }
    }

    fn compile_function(
        &mut self,
        name: Ident,
        params: Vec<(Ident, Types)>,
        return_type: Types,
        body: Program,
    ) -> Result<(), String> {
        let param_types: Vec<BasicMetadataTypeEnum> = params
            .iter()
            .map(|(_, ty)| self.type_to_llvm(ty).into())
            .collect();

        let fn_type = match self.type_to_llvm(&return_type) {
            BasicTypeEnum::IntType(int_type) => int_type.fn_type(&param_types, false),
            _ => self.context.void_type().fn_type(&param_types, false),
        };

        let function = self.module.add_function(name.inner(), fn_type, None);
        self.current_function = Some(function);

        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        self.variables.clear();

        for (i, (param_name, param_type)) in params.iter().enumerate() {
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

        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        if return_type == Types::Void {
            self.builder
                .build_return(None)
                .map_err(|e| format!("Failed to build return: {e}"))?;
        }

        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expr::Number(n) => Ok(self.context.i64_type().const_int(n as u64, true).into()),
            Expr::Float(f) => Ok(self.context.f64_type().const_float(f).into()),
            Expr::String(s) => {
                let global_string = self
                    .builder
                    .build_global_string_ptr(&s, "str")
                    .map_err(|e| format!("Failed to create string: {e}"))?;
                Ok(global_string.as_pointer_value().into())
            }
            Expr::Call { func, args } => self.compile_call(func, args),
            _ => Err(format!("Expression not yet implemented: {expr:?}")),
        }
    }

    fn compile_call(
        &mut self,
        func: Ident,
        args: Vec<Expr>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if func.inner() == "print" {
            return self.compile_print_call(args);
        }

        Err(format!("Function call not yet implemented: {func:?}"))
    }

    fn compile_print_call(&mut self, args: Vec<Expr>) -> Result<BasicValueEnum<'ctx>, String> {
        let printf = self
            .module
            .get_function("printf")
            .ok_or_else(|| "printf function not found".to_string())?;

        for arg in args {
            let format_string = match &arg {
                Expr::String(_) => "%s\n\0",
                Expr::Number(_) => "%lld\n\0",
                Expr::Float(_) => "%f\n\0",
                Expr::Boolean(_) => "%d\n\0",
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

    fn compile_return(&mut self, expr: Option<Expr>) -> Result<(), String> {
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
            Types::String => self.context.i8_type().into(),
            _ => unreachable!("These are not implemented yet"),
        }
    }

    pub fn write_ir_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        self.module
            .print_to_file(path)
            .map_err(|e| format!("Failed to write IR to file: {e}"))
    }
}
