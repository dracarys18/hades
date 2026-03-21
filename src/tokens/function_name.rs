use crate::error::Span;
use crate::tokens::Ident;
use std::fmt::Debug;
use std::hash::Hash;

/// A function name that knows how to mangle itself into a method name.
///
/// Method name mangling: `Outer__method` is produced by `outer.mangle(method)`.
/// This is the canonical place for all name-mangling logic.
#[derive(Clone)]
pub struct FunctionName {
    name: String,
    span: Span,
}

impl FunctionName {
    pub fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }

    /// The raw name string.
    pub fn inner(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Produce the mangled name: `self__other`.
    ///
    /// Example: `get_x.mangle(Point)` → `FunctionName("get_x__Point")`.
    pub fn mangle(&self, other: &Ident) -> FunctionName {
        FunctionName {
            name: format!("{}__{}", self.name, other.inner()),
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
