use super::SemanticError;
use crate::typed_ast::{
    function::{FunctionSignature, Functions},
    ident::IdentMap,
    struc::{Field, Structs},
};

use crate::ast::Types;
use crate::error::Span;
use crate::tokens::{Ident, Op};
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CompilerContext {
    idents: IdentMap,
    functions: Functions,
    structs: Structs,
    current_function: Option<(Ident, Types)>,
}

impl CompilerContext {
    pub fn new() -> Self {
        Self {
            idents: IdentMap::empty(),
            functions: Functions::new(),
            structs: Structs::new(),
            current_function: None,
        }
    }

    pub fn structs(&self) -> &Structs {
        &self.structs
    }

    pub fn enter_function(
        &mut self,
        name: Ident,
        signature: FunctionSignature,
    ) -> Result<(), SemanticError> {
        self.current_function = Some((name.clone(), signature.return_type.clone()));
        self.functions.insert(name, signature)
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

    pub fn insert_struct(&mut self, name: Ident, fields: IndexMap<Ident, Types>) {
        self.structs.insert(name.clone(), fields);
    }

    pub fn get_struct_type(&self, name: &Ident, span: Span) -> Result<Field, SemanticError> {
        if let Some(fields) = self.structs.fields(name) {
            Ok(fields.clone())
        } else {
            Err(SemanticError::undefined_struct(name.clone(), span))
        }
    }

    pub fn get_function_signature(
        &self,
        name: &Ident,
    ) -> Result<&FunctionSignature, SemanticError> {
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
            _ => Err(SemanticError::invalid_unary_operation(
                format!("{op:?}"),
                operand.to_string(),
                span,
            )),
        }
    }
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self::new()
    }
}
