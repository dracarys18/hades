use crate::error::Span;
use crate::tokens::Ident;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone)]
pub struct FunctionName {
    module: Option<String>,
    name: String,
    key: String,
    span: Span,
}

impl FunctionName {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            key: name.clone(),
            module: None,
            name,
            span,
        }
    }

    fn build(module: Option<String>, name: String, span: Span) -> Self {
        let key = match &module {
            Some(m) => format!("{}__{}", m, name),
            None => name.clone(),
        };
        Self {
            module,
            name,
            key,
            span,
        }
    }

    pub fn inner(&self) -> &str {
        &self.key
    }

    pub fn link_name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn mangle(&self, other: &Ident) -> FunctionName {
        FunctionName::build(
            self.module.clone(),
            format!("{}__{}", self.name, other.inner()),
            self.span.clone(),
        )
    }

    pub fn full_name(&self, qualifier: &str) -> FunctionName {
        FunctionName::build(
            Some(qualifier.to_string()),
            self.name.clone(),
            self.span.clone(),
        )
    }

    pub fn to_ident(&self) -> Ident {
        Ident::new(self.key.clone(), self.span.clone())
    }
}

impl Hash for FunctionName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for FunctionName {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for FunctionName {}

impl std::fmt::Display for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl Debug for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionName({})", self.key)
    }
}
