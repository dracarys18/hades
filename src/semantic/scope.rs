use crate::ast::Types;
use crate::tokens::Ident;
use indexmap::IndexMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolNode {
    symbols: indexmap::IndexMap<Ident, Types>,
    parent: Option<Rc<SymbolNode>>,
}

impl SymbolNode {
    pub fn new(parent: Option<Rc<SymbolNode>>) -> Rc<Self> {
        Rc::new(Self {
            symbols: IndexMap::new(),
            parent,
        })
    }

    pub fn global() -> Rc<Self> {
        Self::new(None)
    }

    pub fn lookup_scope<'a>(&'a self, name: &Ident) -> Option<&'a Types> {
        if self.symbols.contains_key(name) {
            self.symbols.get(name)
        } else if let Some(parent) = &self.parent {
            parent.lookup_scope(name)
        } else {
            None
        }
    }

    pub fn insert(&mut self, name: Ident, typ: Types) {
        self.symbols.insert(name, typ);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    current: Rc<SymbolNode>,
}

impl Scope {
    pub fn global() -> Self {
        Self {
            current: SymbolNode::global(),
        }
    }

    pub fn enter_scope(&mut self) {
        let scope = SymbolNode::new(Some(self.current.clone()));
        self.current = scope;
    }

    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.current.parent.clone() {
            self.current = parent;
        }
    }

    pub fn current_scope(&self) -> &SymbolNode {
        self.current.as_ref()
    }

    pub fn current_scope_mut(&mut self) -> Option<&mut SymbolNode> {
        Rc::get_mut(&mut self.current)
    }

    pub fn on_scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&SymbolNode) -> R,
    {
        f(self.current_scope())
    }

    pub fn lookup_scope<'a>(&'a self, name: &Ident) -> Option<&'a Types> {
        self.current.as_ref().lookup_scope(name)
    }
}
