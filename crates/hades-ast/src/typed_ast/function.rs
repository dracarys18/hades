use super::builtins::BUILTIN_FUNCTIONS;
use crate::ast::{ReceiverKind, Types};
use hades_error::SemanticError;
use hades_tokens::{Name, ParamKind};
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TypedReceiver {
    pub struct_name: Name,
    pub kind: ReceiverKind,
    pub typ: Types,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Params {
    Fixed(IndexMap<ParamKind, Types>),
    Variadic(IndexMap<ParamKind, Types>),
}

impl Params {
    fn map(&self) -> &IndexMap<ParamKind, Types> {
        match self {
            Params::Fixed(m) | Params::Variadic(m) => m,
        }
    }

    pub fn type_at(&self, num: usize) -> Option<&Types> {
        self.map()
            .iter()
            .filter(|(k, _)| !matches!(k, ParamKind::Self_(_)))
            .map(|(_, v)| v)
            .nth(num)
    }

    pub fn type_match(&self, num: usize, other_type: &Types) -> bool {
        let entry = self
            .map()
            .iter()
            .filter(|(k, _)| !matches!(k, ParamKind::Self_(_)))
            .map(|(_, v)| v)
            .nth(num);

        match entry {
            None => matches!(self, Params::Variadic(_)),
            Some(expected) => match expected {
                Types::Generic(typs) => typs.iter().any(|t| match (t, other_type) {
                    (Types::Array(_), Types::Array(_)) => {
                        t.get_array_elem_type() == other_type.get_array_elem_type()
                    }
                    _ => t == other_type,
                }),
                _ => other_type == expected,
            },
        }
    }

    pub fn named_count(&self) -> usize {
        self.map()
            .keys()
            .filter(|p| !matches!(p, ParamKind::Self_(_)))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncKind {
    Normal,
    Extern { variadic: bool },
    Intrinsic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub receiver: Option<TypedReceiver>,
    pub params: Params,
    pub return_type: Types,
    pub kind: FuncKind,
}

impl FunctionSignature {
    pub fn new(
        params: IndexMap<ParamKind, Types>,
        return_type: Types,
        receiver: Option<TypedReceiver>,
    ) -> Self {
        Self {
            params: Params::Fixed(params),
            return_type,
            receiver,
            kind: FuncKind::Normal,
        }
    }

    pub fn new_extern(
        params: IndexMap<ParamKind, Types>,
        return_type: Types,
        variadic: bool,
    ) -> Self {
        Self {
            params: if variadic {
                Params::Variadic(params)
            } else {
                Params::Fixed(params)
            },
            return_type,
            receiver: None,
            kind: FuncKind::Extern { variadic },
        }
    }

    pub fn new_intrinsic(
        params: IndexMap<ParamKind, Types>,
        return_type: Types,
        llvm_name: String,
    ) -> Self {
        Self {
            params: Params::Fixed(params),
            return_type,
            receiver: None,
            kind: FuncKind::Intrinsic(llvm_name),
        }
    }

    pub fn param_count(&self) -> usize {
        self.params.named_count()
    }

    pub fn check_arg_count(&self, provided: usize) -> bool {
        match &self.kind {
            FuncKind::Extern { variadic: true } => provided >= self.params.named_count(),
            _ => provided == self.params.named_count(),
        }
    }

    pub fn params(&self) -> Params {
        self.params.clone()
    }

    pub fn to_fixed_params(&self) -> IndexMap<ParamKind, Types> {
        self.params.map().clone()
    }

    pub fn return_type(&self) -> &Types {
        &self.return_type
    }

    pub fn receiver(&self) -> Option<TypedReceiver> {
        self.receiver.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Functions {
    inner: IndexMap<Name, FunctionSignature>,
}

impl Default for Functions {
    fn default() -> Self {
        Self::new()
    }
}

impl Functions {
    pub fn new() -> Self {
        let built_ins = BUILTIN_FUNCTIONS
            .iter()
            .map(|(k, v)| {
                let fn_name = Name::new(k.inner().to_string(), k.span().clone());
                (fn_name, v.clone())
            })
            .collect();
        Self { inner: built_ins }
    }

    pub fn insert(&mut self, name: Name, sig: FunctionSignature) -> Result<(), SemanticError> {
        if self.inner.contains_key(&name) {
            if matches!(sig.kind, FuncKind::Extern { .. } | FuncKind::Intrinsic(_)) {
                return Ok(());
            }
            return Err(SemanticError::redefined_function(
                name.inner().to_string(),
                name.span().clone(),
            ));
        }
        self.inner.insert(name, sig);
        Ok(())
    }

    pub fn get(&self, name: &Name) -> Result<&FunctionSignature, SemanticError> {
        self.inner.get(name).ok_or_else(|| {
            SemanticError::undefined_function(name.inner().to_string(), name.span().clone())
        })
    }

    pub fn into_user_defined(self) -> IndexMap<Name, FunctionSignature> {
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
