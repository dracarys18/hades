use super::builtins::BUILTIN_FUNCTIONS;
use crate::ast::Types;
use crate::consts::MAX_FUNCTION_PARAMS;
use crate::tokens::Ident;
use crate::typed_ast::SemanticError;

use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Params {
    Variadic,
    Fixed(IndexMap<Ident, Types>),
}

impl Params {
    pub fn type_match(&self, num: usize, other_type: &Types) -> bool {
        match self {
            Params::Variadic => true,
            Params::Fixed(map) => {
                let expected_type = map.values().nth(num).expect("Parameter not found");

                let cond = match expected_type {
                    Types::Generic(typs) => typs.contains(&other_type),
                    _ => other_type == expected_type,
                };
                cond
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub params: Params,
    pub return_type: Types,
}

impl FunctionSignature {
    pub fn new(params: IndexMap<Ident, Types>, return_type: Types) -> Self {
        Self {
            params: Params::Fixed(params),
            return_type,
        }
    }

    pub fn new_variadic(return_type: Types) -> Self {
        Self {
            params: Params::Variadic,
            return_type,
        }
    }

    pub fn param_count(&self) -> usize {
        match &self.params {
            Params::Variadic => MAX_FUNCTION_PARAMS,
            Params::Fixed(map) => map.len(),
        }
    }

    pub fn params(&self) -> Params {
        self.params.clone()
    }

    pub fn to_fixed_params(&self) -> IndexMap<Ident, Types> {
        match &self.params {
            Params::Variadic => panic!("Variadic functions are not supported yet"),
            Params::Fixed(map) => map.clone(),
        }
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
        let built_ins = BUILTIN_FUNCTIONS.clone();
        Self { inner: built_ins }
    }

    pub fn insert(&mut self, name: Ident, sig: FunctionSignature) -> Result<(), SemanticError> {
        if self.inner.contains_key(&name) {
            return Err(SemanticError::redefined_function(
                name.clone(),
                name.span().clone(),
            ));
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
            .ok_or_else(|| SemanticError::undefined_function(name.clone(), name.span().clone()))
    }
}
