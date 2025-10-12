use crate::ast::Types;
use crate::tokens::Ident;
use crate::typed_ast::SemanticError;

use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub params: Vec<Types>,
    pub return_type: Types,
}

impl FunctionSignature {
    pub fn new(params: Vec<Types>, return_type: Types) -> Self {
        Self {
            params,
            return_type,
        }
    }

    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    pub fn params(&self) -> &Vec<Types> {
        &self.params
    }

    pub fn return_type(&self) -> &Types {
        &self.return_type
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Functions {
    inner: IndexMap<Ident, FunctionSignature>,
}

impl Functions {
    pub fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, name: Ident, sig: FunctionSignature) -> Result<(), SemanticError> {
        if self.inner.contains_key(&name) {
            return Err(SemanticError::RedefinedFunction {
                name: name.clone(),
                span: name.span().clone(),
            });
        }
        self.inner.insert(name, sig);
        Ok(())
    }

    pub fn get_unchecked(&self, name: &Ident) -> &FunctionSignature {
        self.inner.get(name).expect("Function not found")
    }

    pub fn get(&self, name: &Ident) -> Result<&FunctionSignature, SemanticError> {
        self.inner
            .get(name)
            .ok_or(SemanticError::UndefinedFunction {
                // TODO: Dont clone
                name: name.clone(),
                span: name.span().clone(),
            })
    }
}
