use hades_ast::Types;
use crate::codegen::error::{CodegenError, CodegenResult};
use hades_ast::scope::Scope;
use hades_tokens::Ident;
use inkwell::values::PointerValue;

#[derive(Debug, Clone)]
pub struct LLVMVariable<'ctx> {
    value: PointerValue<'ctx>,
    typ: Types,
}

impl<'ctx> LLVMVariable<'ctx> {
    pub fn new(value: PointerValue<'ctx>, typ: Types) -> Self {
        Self { value, typ }
    }
    pub fn value(&self) -> PointerValue<'ctx> {
        self.value
    }
    pub fn typ(&self) -> &Types {
        &self.typ
    }
}

#[derive(Debug)]
pub struct CodegenSymbols<'ctx> {
    variables: Scope<LLVMVariable<'ctx>>,
}

impl<'ctx> CodegenSymbols<'ctx> {
    pub fn new() -> Self {
        Self {
            variables: Scope::global(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.variables.enter_scope();
    }

    pub fn exit_scope(&mut self) {
        self.variables.exit_scope();
    }

    pub fn declare_variable(
        &mut self,
        name: Ident,
        ptr: PointerValue<'ctx>,
        typ: Types,
    ) -> CodegenResult<()> {
        let variable = LLVMVariable { value: ptr, typ };
        self.variables.on_scope_mut(|node: &mut hades_ast::scope::SymbolNode<LLVMVariable<'ctx>>| {
            node.insert(name, variable);
        });
        Ok(())
    }

    pub fn lookup_variable(&self, name: &Ident) -> Option<LLVMVariable<'ctx>> {
        self.variables.lookup_scope(name).cloned()
    }

    pub fn get_variable(&self, name: &Ident) -> CodegenResult<LLVMVariable<'ctx>> {
        self.lookup_variable(name)
            .ok_or_else(|| CodegenError::UndefinedVariable {
                name: name.to_string(),
            })
    }
}
