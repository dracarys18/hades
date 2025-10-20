use crate::tokens::Ident;
use indexmap::IndexMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolNode<T> {
    symbols: indexmap::IndexMap<Ident, T>,
    parent: Option<Rc<SymbolNode<T>>>,
}

impl<T> SymbolNode<T> {
    pub fn new(parent: Option<Rc<SymbolNode<T>>>) -> Rc<Self> {
        Rc::new(Self {
            symbols: IndexMap::new(),
            parent,
        })
    }

    pub fn global() -> Rc<Self> {
        Self::new(None)
    }

    pub fn lookup_scope<'a>(&'a self, name: &Ident) -> Option<&'a T> {
        if self.symbols.contains_key(name) {
            self.symbols.get(name)
        } else if let Some(parent) = &self.parent {
            parent.lookup_scope(name)
        } else {
            None
        }
    }

    pub fn insert(&mut self, name: Ident, typ: T) {
        self.symbols.insert(name, typ);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<T: Clone> {
    current: Rc<SymbolNode<T>>,
}

impl<T: Clone> Scope<T> {
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

    pub fn current_scope(&self) -> &SymbolNode<T> {
        self.current.as_ref()
    }

    pub fn current_scope_mut(&mut self) -> &mut SymbolNode<T> {
        Rc::make_mut(&mut self.current)
    }

    pub fn on_scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&SymbolNode<T>) -> R,
    {
        f(self.current_scope())
    }

    pub fn on_scope_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut SymbolNode<T>) -> R,
    {
        f(self.current_scope_mut())
    }

    pub fn lookup_scope<'a>(&'a self, name: &Ident) -> Option<&'a T> {
        self.current.as_ref().lookup_scope(name)
    }
}
