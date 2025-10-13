use crate::ast::Types;
use crate::semantic::scope::{Scope, SymbolNode};

use crate::tokens::Ident;

#[derive(Debug, Clone, PartialEq)]
pub struct IdentMap {
    inner: Scope,
}

impl IdentMap {
    pub fn empty() -> Self {
        Self {
            inner: Scope::global(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.inner.enter_scope();
    }
    pub fn exit_scope(&mut self) {
        self.inner.exit_scope();
    }

    pub fn current_scope(&self) -> &SymbolNode {
        self.inner.current_scope()
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut SymbolNode> {
        self.inner.current_scope_mut()
    }

    pub fn insert(&mut self, name: Ident, typ: Types) -> Result<(), String> {
        if let Some(current_scope) = self.inner.current_scope_mut() {
            if current_scope.contains(&name) {
                return Err(format!("Variable '{name}' already declared in this scope",));
            }
            current_scope.insert(name, typ);
            Ok(())
        } else {
            Err("No scope available".to_string())
        }
    }

    pub fn lookup<'a>(&'a self, name: &'a Ident) -> Option<&'a Types> {
        self.inner.lookup_scope(name)
    }
}
