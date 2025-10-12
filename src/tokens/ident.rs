use crate::error::Span;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone)]
pub struct Ident {
    name: String,
    span: Span,
}

impl Ident {
    pub fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }

    pub fn len(&self) -> usize {
        self.name.len()
    }

    pub fn inner(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

impl Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Ident {}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ident({})", self.name)
    }
}
