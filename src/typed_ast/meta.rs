use crate::error::SemanticError;
use crate::module::ModulePath;
use crate::typed_ast::{
    function::{FunctionSignature, Functions},
    ident::IdentMap,
    signatures::ModuleSignatures,
    struc::{Field, Structs},
    TypedFieldKind,
};

use crate::ast::Types;
use crate::error::Span;
use crate::tokens::{Ident, Name, Op};
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CompilerContext {
    idents: IdentMap,
    functions: Functions,
    structs: Structs,
    current_function: Option<(Name, Types)>,
    module_path: Option<ModulePath>,
}

impl CompilerContext {
    pub fn new() -> Self {
        Self {
            idents: IdentMap::empty(),
            functions: Functions::new(),
            structs: Structs::new(),
            current_function: None,
            module_path: None,
        }
    }

    pub fn set_module_path(&mut self, path: ModulePath) {
        self.module_path = Some(path);
    }

    pub fn module_name(&self) -> Option<&str> {
        self.module_path.as_ref().map(|p| p.name())
    }

    pub fn is_stdlib(&self) -> bool {
        matches!(self.module_path, Some(ModulePath::Std(_)))
    }

    pub fn structs(&self) -> &Structs {
        &self.structs
    }

    pub fn register_function(
        &mut self,
        name: Name,
        sig: FunctionSignature,
    ) -> Result<(), SemanticError> {
        self.functions.insert(name, sig)
    }

    pub fn set_current_function(&mut self, name: Name, return_type: Types) {
        self.current_function = Some((name, return_type));
    }

    pub fn enter_scope(&mut self) {
        self.idents.enter_scope();
    }

    pub fn exit_scope(&mut self) {
        self.idents.exit_scope();
    }

    pub fn exit_function(&mut self) {
        self.current_function = None;
    }

    pub fn insert_variable(&mut self, name: Ident, typ: Types) {
        self.idents.insert(name, typ);
    }

    pub fn get_variable_type(&self, name: &Ident, span: Span) -> Result<Types, SemanticError> {
        self.idents
            .lookup(name)
            .ok_or_else(|| SemanticError::undefined_variable(name.clone(), span))
            .cloned()
    }

    pub fn insert_struct(&mut self, name: Name, fields: IndexMap<Ident, TypedFieldKind>) {
        self.structs.insert(name.clone(), fields);
    }

    pub fn get_struct_type(&self, name: &Name, span: Span) -> Result<Field, SemanticError> {
        if let Some(fields) = self.structs.fields(name) {
            Ok(fields.clone())
        } else {
            Err(SemanticError::undefined_struct(name.to_ident(), span))
        }
    }

    pub fn get_function_signature(&self, name: &Name) -> Result<&FunctionSignature, SemanticError> {
        self.functions.get(name)
    }

    pub fn check_return_type(&self, return_type: Types, span: Span) -> Result<(), SemanticError> {
        if let Some((_, expected_return_type)) = &self.current_function {
            if *expected_return_type != return_type {
                return Err(SemanticError::return_type_mismatch(
                    expected_return_type.clone().to_string(),
                    return_type.to_string(),
                    span,
                ));
            }
        }
        Ok(())
    }

    pub fn infer_binary_type(
        &self,
        left: &Types,
        op: &Op,
        right: &Types,
        span: Span,
    ) -> Result<Types, SemanticError> {
        match op {
            Op::Add
            | Op::Sub
            | Op::Mul
            | Op::Div
            | Op::Mod
            | Op::Plus
            | Op::PlusEqual
            | Op::MinusEqual
            | Op::Minus
            | Op::Multiply
            | Op::Divide => match (left, right) {
                (Types::Int, Types::Int) => Ok(Types::Int),
                (Types::Float, Types::Float) => Ok(Types::Float),
                (Types::Int, Types::Float) | (Types::Float, Types::Int) => Ok(Types::Float),
                (Types::String, Types::String) if matches!(op, Op::Add | Op::Plus) => {
                    Ok(Types::String)
                }
                _ => Err(SemanticError::invalid_binary_operation(
                    left.to_string().to_string(),
                    format!("{op:?}"),
                    right.to_string().to_string(),
                    span,
                )),
            },
            Op::Eq
            | Op::Ne
            | Op::Lt
            | Op::Le
            | Op::Gt
            | Op::Ge
            | Op::EqualEqual
            | Op::BangEqual
            | Op::Less
            | Op::LessEqual
            | Op::Greater
            | Op::GreaterEqual => match (left, right) {
                (Types::Int, Types::Int)
                | (Types::Float, Types::Float)
                | (Types::String, Types::String)
                | (Types::Bool, Types::Bool) => Ok(Types::Bool),
                (Types::Int, Types::Float) | (Types::Float, Types::Int) => Ok(Types::Bool),
                (Types::Pointer(_), Types::Pointer(_))
                    if matches!(op, Op::Eq | Op::Ne | Op::EqualEqual | Op::BangEqual) =>
                {
                    Ok(Types::Bool)
                }
                _ => Err(SemanticError::invalid_binary_operation(
                    left.to_string().to_string(),
                    format!("{op:?}"),
                    right.to_string().to_string(),
                    span,
                )),
            },
            Op::Assign => {
                if left == right {
                    Ok(left.clone())
                } else {
                    Err(SemanticError::invalid_binary_operation(
                        left.to_string(),
                        format!("{op:?}"),
                        right.to_string(),
                        span,
                    ))
                }
            }
            Op::And | Op::Or | Op::BoleanAnd | Op::BooleanOr => match (left, right) {
                (Types::Bool, Types::Bool) => Ok(Types::Bool),
                _ => Err(SemanticError::invalid_binary_operation(
                    left.to_string(),
                    format!("{op:?}"),
                    right.to_string(),
                    span,
                )),
            },
            Op::BitAnd | Op::BitOr | Op::BitXor | Op::Shl | Op::Shr => match (left, right) {
                (Types::Int, Types::Int) => Ok(Types::Int),
                _ => Err(SemanticError::invalid_binary_operation(
                    left.to_string(),
                    format!("{op:?}"),
                    right.to_string(),
                    span,
                )),
            },
            _ => Err(SemanticError::invalid_binary_operation(
                left.to_string(),
                format!("{op:?}"),
                right.to_string(),
                span,
            )),
        }
    }

    pub fn infer_unary_type(
        &self,
        op: &Op,
        operand: &Types,
        span: Span,
    ) -> Result<Types, SemanticError> {
        match op {
            Op::Sub | Op::Minus => match operand {
                Types::Int => Ok(Types::Int),
                Types::Float => Ok(Types::Float),
                _ => Err(SemanticError::invalid_unary_operation(
                    format!("{op:?}"),
                    operand.to_string(),
                    span,
                )),
            },
            Op::Not => match operand {
                Types::Bool => Ok(Types::Bool),
                _ => Err(SemanticError::invalid_unary_operation(
                    format!("{op:?}"),
                    operand.to_string(),
                    span,
                )),
            },
            Op::BitNot => match operand {
                Types::Int => Ok(Types::Int),
                _ => Err(SemanticError::invalid_unary_operation(
                    format!("{op:?}"),
                    operand.to_string(),
                    span,
                )),
            },
            Op::Ref => Ok(Types::Pointer(Box::new(operand.clone()))),
            Op::Deref => match operand {
                Types::Pointer(inner) => Ok(*inner.clone()),
                _ => Err(SemanticError::invalid_unary_operation(
                    format!("{op:?}"),
                    operand.to_string(),
                    span,
                )),
            },
            _ => Err(SemanticError::invalid_unary_operation(
                format!("{op:?}"),
                operand.to_string(),
                span,
            )),
        }
    }

    pub fn into_signatures(self, path: ModulePath) -> ModuleSignatures {
        ModuleSignatures {
            path,
            functions: self.functions.into_user_defined(),
            structs: self.structs,
        }
    }
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self::new()
    }
}
