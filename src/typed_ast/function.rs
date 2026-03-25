use super::builtins::BUILTIN_FUNCTIONS;
use crate::ast::Types;
use crate::consts::MAX_FUNCTION_PARAMS;
use crate::error::SemanticError;
use crate::tokens::{FunctionName, Ident, ParamKind};
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Params {
    Variadic,
    Fixed(IndexMap<ParamKind, Types>),
}

impl Params {
    pub fn type_at(&self, num: usize) -> Option<&Types> {
        match self {
            Params::Variadic => None,
            Params::Fixed(map) => map
                .iter()
                .filter(|(k, _)| !matches!(k, ParamKind::Self_(_)))
                .map(|(_, v)| v)
                .nth(num),
        }
    }

    pub fn type_match(&self, num: usize, other_type: &Types) -> bool {
        match self {
            Params::Variadic => true,
            Params::Fixed(map) => {
                let expected_type = map
                    .iter()
                    .filter(|(k, _)| !matches!(k, ParamKind::Self_(_)))
                    .map(|(_, v)| v)
                    .nth(num)
                    .expect("Parameter not found");

                match expected_type {
                    Types::Generic(typs) => typs.iter().any(|t| match (t, other_type) {
                        (Types::Array(_), Types::Array(_)) => {
                            t.get_array_elem_type() == other_type.get_array_elem_type()
                        }
                        _ => t == other_type,
                    }),
                    _ => other_type == expected_type,
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub receiver: Option<Types>,
    pub params: Params,
    pub return_type: Types,
}

impl FunctionSignature {
    pub fn new(
        params: IndexMap<ParamKind, Types>,
        return_type: Types,
        receiver: Option<Types>,
    ) -> Self {
        Self {
            params: Params::Fixed(params),
            return_type,
            receiver,
        }
    }

    pub fn new_variadic(return_type: Types) -> Self {
        Self {
            params: Params::Variadic,
            receiver: None,
            return_type,
        }
    }

    pub fn param_count(&self) -> usize {
        match &self.params {
            Params::Variadic => MAX_FUNCTION_PARAMS,
            Params::Fixed(map) => map
                .keys()
                .filter(|p| !matches!(p, ParamKind::Self_(_)))
                .count(),
        }
    }

    pub fn check_arg_count(&self, provided: usize) -> bool {
        match &self.params {
            Params::Variadic => provided <= self.param_count(),
            Params::Fixed(_) => provided == self.param_count(),
        }
    }

    pub fn params(&self) -> Params {
        self.params.clone()
    }

    pub fn to_fixed_params(&self) -> IndexMap<ParamKind, Types> {
        match &self.params {
            Params::Variadic => panic!("Variadic functions are not supported yet"),
            Params::Fixed(map) => map.clone(),
        }
    }

    pub fn return_type(&self) -> &Types {
        &self.return_type
    }

    pub fn receiver(&self) -> Option<Types> {
        self.receiver.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Functions {
    inner: IndexMap<FunctionName, FunctionSignature>,
}

impl Functions {
    pub fn new() -> Self {
        let built_ins = BUILTIN_FUNCTIONS
            .iter()
            .map(|(k, v)| {
                let fn_name = FunctionName::new(k.inner().to_string(), k.span().clone());
                (fn_name, v.clone())
            })
            .collect();
        Self { inner: built_ins }
    }

    pub fn insert(
        &mut self,
        name: FunctionName,
        sig: FunctionSignature,
    ) -> Result<(), SemanticError> {
        if self.inner.contains_key(&name) {
            let ident = name.to_ident();
            return Err(SemanticError::redefined_function(
                ident.clone(),
                ident.span().clone(),
            ));
        }
        self.inner.insert(name, sig);
        Ok(())
    }

    pub fn get_unchecked(&self, name: &FunctionName) -> &FunctionSignature {
        self.inner.get(name).expect("Function not found")
    }

    pub fn get(&self, name: &FunctionName) -> Result<&FunctionSignature, SemanticError> {
        self.inner.get(name).ok_or_else(|| {
            let ident = name.to_ident();
            SemanticError::undefined_function(ident.clone(), ident.span().clone())
        })
    }

    pub fn into_user_defined(self) -> IndexMap<FunctionName, FunctionSignature> {
        let builtin_names: std::collections::HashSet<String> = BUILTIN_FUNCTIONS
            .keys()
            .map(|k| k.inner().to_string())
            .collect();
        self.inner
            .into_iter()
            .filter(|(name, _)| !builtin_names.contains(name.inner()))
            .collect()
    }
}
