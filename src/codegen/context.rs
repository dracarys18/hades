use crate::ast::Types;
use crate::codegen::error::{CodegenError, CodegenResult};
use crate::codegen::symbols::{CodegenSymbols, LLVMVariable};
use crate::codegen::types::TypeConverter;
use crate::tokens::Ident;
use crate::typed_ast::CompilerContext;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};

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
}

impl<'ctx> LLVMContext<'ctx> {
    pub fn new(
        compiler_context: &'ctx CompilerContext,
        context: &'ctx Context,
        module_name: &str,
    ) -> Self {
        let module = context.create_module(module_name);
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

    pub fn declare_function(&mut self, name: String, func: FunctionValue<'ctx>) {
        self.codegen_symbols.declare_function(name, func)
    }

    pub fn get_variable(&self, name: &Ident) -> CodegenResult<LLVMVariable<'ctx>> {
        self.codegen_symbols.get_variable(name)
    }

    pub fn get_function(&self, name: &str) -> CodegenResult<FunctionValue<'ctx>> {
        self.codegen_symbols.get_function(name)
    }

    pub fn lookup_variable(&self, name: &Ident) -> Option<LLVMVariable<'ctx>> {
        self.codegen_symbols.lookup_variable(name)
    }

    pub fn lookup_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.codegen_symbols.lookup_function(name)
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
        let alloca = self
            .builder
            .build_alloca(ty, name)
            .map_err(|_| CodegenError::LLVMBuild {
                message: format!("Failed to create alloca for {}", name),
            })?;
        Ok(alloca)
    }

    pub fn create_load(
        &self,
        ptr: PointerValue<'ctx>,
        name: &str,
    ) -> CodegenResult<BasicValueEnum<'ctx>> {
        let ptr_type: BasicTypeEnum = ptr.get_type().into();
        self.builder
            .build_load(ptr_type, ptr, name)
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
}
