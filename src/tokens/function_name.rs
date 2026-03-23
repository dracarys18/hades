use crate::error::Span;
use crate::tokens::Ident;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone)]
pub struct FunctionName {
    name: String,
    span: Span,
}

impl FunctionName {
    pub fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }

    pub fn inner(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn mangle(&self, other: &Ident) -> FunctionName {
        FunctionName {
            name: format!("{}__{}", self.name, other.inner()),
            span: self.span.clone(),
        }
    }

    pub fn full_name(&self, qualifier: &str) -> FunctionName {
        FunctionName {
            name: format!("{}__{}", qualifier, self.name),
            span: self.span.clone(),
        }
    }

    pub fn to_ident(&self) -> Ident {
        Ident::new(self.name.clone(), self.span.clone())
    }
}

impl Hash for FunctionName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for FunctionName {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for FunctionName {}

impl std::fmt::Display for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Debug for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionName({})", self.name)
    }
}
