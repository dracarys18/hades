use crate::error::Span;
use crate::tokens::Ident;
use std::fmt::Debug;
use std::hash::Hash;

/// A function name identifier that knows how to mangle itself into a method name.
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

    pub fn inner(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Produce the mangled name for a method: `self__method`.
    ///
    /// Example: `Point.mangle(get_x)` → `FunctionName("Point__get_x")`.
    pub fn mangle(&self, method: &FunctionName) -> FunctionName {
        FunctionName {
            name: format!("{}__{}", self.name, method.name),
            span: method.span.clone(),
        }
    }

    /// Convert to an `Ident` for use in contexts that require one (e.g. map keys).
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
