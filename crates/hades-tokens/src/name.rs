use crate::Ident;
use hades_error::Span;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone)]
pub struct Name {
    module: Option<String>,
    name: String,
    key: String,
    span: Span,
}

impl Name {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            key: name.clone(),
            module: None,
            name,
            span,
        }
    }

    pub fn with_module(module: String, name: String, span: Span) -> Self {
        Self::build(Some(module), name, span)
    }

    pub fn from_key(key: &str, span: Span) -> Self {
        match key.rsplit_once("__") {
            Some((module, name)) => Self::build(Some(module.to_string()), name.to_string(), span),
            None => Self::new(key.to_string(), span),
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

    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn mangle(&self, other: &Ident) -> Name {
        Name::build(
            self.module.clone(),
            format!("{}__{}", self.name, other.inner()),
            self.span.clone(),
        )
    }

    pub fn mangle_optional(&self, other: Option<&Ident>) -> Name {
        if let Some(other) = other {
            self.mangle(other)
        } else {
            self.clone()
        }
    }

    pub fn full_name(&self, qualifier: &str) -> Name {
        Name::build(
            Some(qualifier.to_string()),
            self.name.clone(),
            self.span.clone(),
        )
    }

    pub fn full_name_optional(&self, qualifier: Option<&str>) -> Name {
        match qualifier {
            Some(m) => self.full_name(m),
            None => self.clone(),
        }
    }

    pub fn to_ident(&self) -> Ident {
        Ident::new(self.key.clone(), self.span.clone())
    }
}

impl Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Name {}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionName({})", self.key)
    }
}
