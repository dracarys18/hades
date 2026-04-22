use std::fmt;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ModulePath {
    Std(String),
    Local(String),
}

impl ModulePath {
    pub fn name(&self) -> &str {
        match self {
            ModulePath::Std(name) => name,
            ModulePath::Local(name) => name,
        }
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModulePath::Std(name) => write!(f, "std::{}", name),
            ModulePath::Local(name) => write!(f, "self::{}", name),
        }
    }
}
