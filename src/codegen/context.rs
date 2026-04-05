use crate::ast::Types;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::symbols::{CodegenSymbols, LLVMVariable};
use crate::codegen::types::TypeConverter;
use crate::tokens::Ident;
use crate::typed_ast::{CompilerContext, FuncKind, ModuleSignatures};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;

pub struct LoopContext<'ctx> {
    pub continue_block: BasicBlock<'ctx>,
    pub break_block: BasicBlock<'ctx>,
}

pub struct LLVMContext<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    symbols: &'ctx CompilerContext,
    codegen_symbols: CodegenSymbols<'ctx>,
    type_converter: TypeConverter<'ctx>,
    current_function: Option<FunctionValue<'ctx>>,
    loop_stack: Vec<LoopContext<'ctx>>,
    function_aliases: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> LLVMContext<'ctx> {
    pub fn new(
        compiler_context: &'ctx CompilerContext,
        context: &'ctx Context,
        module: Module<'ctx>,
    ) -> Self {
        let builder = context.create_builder();
        let type_converter = TypeConverter::new(context);
        let codegen_symbols = CodegenSymbols::new();

        Self {
            symbols: compiler_context,
            context,
            builder,
            module,
            codegen_symbols,
            type_converter,
            current_function: None,
            loop_stack: Vec::new(),
            function_aliases: HashMap::new(),
        }
    }

    pub fn symbols(&self) -> &'ctx CompilerContext {
        self.symbols
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    pub fn builder(&self) -> &Builder<'ctx> {
        &self.builder
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn type_converter(&mut self) -> &mut TypeConverter<'ctx> {
        &mut self.type_converter
    }

    pub fn declare_variable(
        &mut self,
        name: Ident,
        ptr: PointerValue<'ctx>,
        typ: Types,
    ) -> CodegenResult<()> {
        self.codegen_symbols.declare_variable(name, ptr, typ)
    }

    pub fn get_variable(&self, name: &Ident) -> CodegenResult<LLVMVariable<'ctx>> {
        self.codegen_symbols.get_variable(name)
    }

    pub fn get_function(&self, name: &str) -> CodegenResult<FunctionValue<'ctx>> {
        if let Some(&func) = self.function_aliases.get(name) {
            return Ok(func);
        }
        self.module()
            .get_function(name)
            .ok_or(CodegenError::FunctionNotFound {
                name: name.to_string(),
            })
    }

    pub fn register_function_alias(&mut self, name: String, func: FunctionValue<'ctx>) {
        self.function_aliases.insert(name, func);
    }

    pub fn lookup_variable(&self, name: &Ident) -> Option<LLVMVariable<'ctx>> {
        self.codegen_symbols.lookup_variable(name)
    }

    pub fn set_current_function(&mut self, func: FunctionValue<'ctx>) {
        self.current_function = Some(func);
    }

    pub fn current_function(&self) -> Option<FunctionValue<'ctx>> {
        self.current_function
    }

    pub fn clear_current_function(&mut self) {
        self.current_function = None;
    }

    pub fn push_loop(&mut self, continue_block: BasicBlock<'ctx>, break_block: BasicBlock<'ctx>) {
        self.loop_stack.push(LoopContext {
            continue_block,
            break_block,
        });
    }

    pub fn pop_loop(&mut self) -> Option<LoopContext<'ctx>> {
        self.loop_stack.pop()
    }

    pub fn current_loop(&self) -> Option<&LoopContext<'ctx>> {
        self.loop_stack.last()
    }

    pub fn create_alloca(
        &mut self,
        name: &str,
        ty: inkwell::types::BasicTypeEnum<'ctx>,
    ) -> CodegenResult<PointerValue<'ctx>> {
        let func = self
            .current_function()
            .ok_or(CodegenError::AllocaOutsideFunction)?;
        let entry_block = func
            .get_first_basic_block()
            .expect("function has no entry block");
        let bx = self.context.create_builder();
        match entry_block.get_first_instruction() {
            Some(instr) => bx.position_before(&instr),
            None => bx.position_at_end(entry_block),
        }
        bx.build_alloca(ty, name)
            .map_err(|_| CodegenError::LLVMBuild {
                message: format!("Failed to create alloca for {}", name),
            })
    }

    pub fn create_load(
        &self,
        ptr: PointerValue<'ctx>,
        element_type: BasicTypeEnum<'ctx>,
        name: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        self.builder
            .build_load(element_type, ptr, name)
            .map_err(|_| CodegenError::LLVMBuild {
                message: format!("Failed to load value from {}", name),
            })
    }

    pub fn create_store(
        &self,
        ptr: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> CodegenResult<()> {
        self.builder
            .build_store(ptr, value)
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to store value".to_string(),
            })?;
        Ok(())
    }

    pub fn create_basic_block(&self, name: &str) -> BasicBlock<'ctx> {
        self.context.append_basic_block(
            self.current_function()
                .expect("Cannot create basic block without current function"),
            name,
        )
    }

    pub fn position_at_end(&self, block: BasicBlock<'ctx>) {
        self.builder.position_at_end(block);
    }

    pub fn build_unconditional_branch(&self, dest: BasicBlock<'ctx>) -> CodegenResult<()> {
        self.builder
            .build_unconditional_branch(dest)
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to build unconditional branch".to_string(),
            })?;
        Ok(())
    }

    pub fn build_conditional_branch(
        &self,
        cond: BasicValueEnum<'ctx>,
        then_block: BasicBlock<'ctx>,
        else_block: BasicBlock<'ctx>,
    ) -> CodegenResult<()> {
        let cond_val = cond.into_int_value();
        self.builder
            .build_conditional_branch(cond_val, then_block, else_block)
            .map_err(|_| CodegenError::LLVMBuild {
                message: "Failed to build conditional branch".to_string(),
            })?;
        Ok(())
    }

    pub fn build_return(&self, value: Option<BasicValueEnum<'ctx>>) -> CodegenResult<()> {
        match value {
            Some(val) => {
                self.builder
                    .build_return(Some(&val))
                    .map_err(|_| CodegenError::LLVMBuild {
                        message: "Failed to build return with value".to_string(),
                    })?;
            }
            None => {
                self.builder
                    .build_return(None)
                    .map_err(|_| CodegenError::LLVMBuild {
                        message: "Failed to build void return".to_string(),
                    })?;
            }
        }
        Ok(())
    }

    pub fn get_current_block(&self) -> Option<BasicBlock<'ctx>> {
        self.builder.get_insert_block()
    }

    pub fn is_block_terminated(&self) -> bool {
        self.get_current_block()
            .map(|block| block.get_terminator().is_some())
            .unwrap_or(false)
    }

    pub fn enter_scope(&mut self) {
        self.codegen_symbols.enter_scope();
    }

    pub fn exit_scope(&mut self) {
        self.codegen_symbols.exit_scope();
    }

    pub fn build_fn_type(
        &mut self,
        return_type: &Types,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
        variadic: bool,
    ) -> CodegenResult<FunctionType<'ctx>> {
        if *return_type == Types::Void {
            Ok(self
                .type_converter()
                .void_type()
                .fn_type(param_types, variadic))
        } else {
            let symbols = self.symbols();
            let ret = self.type_converter().to_llvm_type(return_type, symbols)?;
            Ok(ret.fn_type(param_types, variadic))
        }
    }

    pub fn declare_imports(&mut self, sigs: &[&ModuleSignatures]) -> CodegenResult<()> {
        for sig in sigs {
            for (name, fn_sig) in &sig.functions {
                if self.get_function(name.inner()).is_ok() {
                    continue;
                }
                match &fn_sig.kind {
                    FuncKind::Intrinsic(llvm_name) => {
                        let llvm_name = llvm_name.clone();
                        let symbols = self.symbols();
                        let param_types = self
                            .type_converter()
                            .params_to_llvm_types(fn_sig, symbols)?;
                        let type_slice: Vec<BasicTypeEnum> = param_types
                            .iter()
                            .map(|t| {
                                BasicTypeEnum::try_from(*t).expect("param is not a basic type")
                            })
                            .collect();
                        let intrinsic = inkwell::intrinsics::Intrinsic::find(&llvm_name)
                            .ok_or_else(|| CodegenError::LLVMBuild {
                                message: format!("LLVM intrinsic '{}' not found", llvm_name),
                            })?;
                        let func = intrinsic
                            .get_declaration(self.module(), &type_slice)
                            .ok_or_else(|| CodegenError::LLVMBuild {
                                message: format!("Failed to get declaration for '{}'", llvm_name),
                            })?;
                        self.function_aliases.insert(name.inner().to_string(), func);
                    }
                    FuncKind::Extern { variadic } => {
                        let variadic = *variadic;
                        let symbols = self.symbols();
                        let param_types = self
                            .type_converter()
                            .params_to_llvm_types(fn_sig, symbols)?;
                        let fn_type = self.build_fn_type(
                            &fn_sig.return_type.clone(),
                            &param_types,
                            variadic,
                        )?;
                        let function = self.module().add_function(
                            name.link_name(),
                            fn_type,
                            Some(Linkage::External),
                        );
                        self.function_aliases
                            .insert(name.inner().to_string(), function);
                    }
                    FuncKind::Normal => {
                        let symbols = self.symbols();
                        let param_types = self
                            .type_converter()
                            .params_to_llvm_types(fn_sig, symbols)?;
                        let fn_type =
                            self.build_fn_type(&fn_sig.return_type.clone(), &param_types, false)?;
                        self.module()
                            .add_function(name.inner(), fn_type, Some(Linkage::External));
                    }
                }
            }
        }
        Ok(())
    }
}
