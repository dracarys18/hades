use crate::{error::Span, tokens::Ident};

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },
    UndefinedVariable {
        name: Ident,
        span: Span,
    },
    UndefinedFunction {
        name: Ident,
        span: Span,
    },
    UndefinedStruct {
        name: Ident,
        span: Span,
    },
    NotAStruct {
        name: Ident,
    },
    UnknownField {
        struct_name: Ident,
        field_name: Ident,
    },
    ArgumentCountMismatch {
        expected: usize,
        found: usize,
        function: Ident,
    },
    ReturnTypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },
    InvalidBinaryOperation {
        left: String,
        op: String,
        right: String,
        span: Span,
    },
    InvalidUnaryOperation {
        op: String,
        operand: String,
        span: Span,
    },
    RedefinedVariable {
        name: Ident,
        span: Span,
    },
    RedefinedFunction {
        name: Ident,
        span: Span,
    },
    RedefinedStruct {
        name: Ident,
        span: Span,
    },
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticError::TypeMismatch {
                expected, found, ..
            } => {
                write!(f, "Type mismatch: expected {expected:?}, found {found:?}",)
            }
            SemanticError::UndefinedVariable { name, .. } => {
                write!(f, "Undefined variable: {}", name.inner())
            }
            SemanticError::UndefinedFunction { name, .. } => {
                write!(f, "Undefined function: {}", name.inner())
            }
            SemanticError::UndefinedStruct { name, .. } => {
                write!(f, "Undefined struct: {}", name.inner())
            }
            SemanticError::NotAStruct { name } => {
                write!(f, "{} is not a struct", name.inner())
            }
            SemanticError::UnknownField {
                struct_name,
                field_name,
            } => {
                write!(
                    f,
                    "Unknown field {} in struct {}",
                    field_name.inner(),
                    struct_name.inner()
                )
            }
            SemanticError::ArgumentCountMismatch {
                expected,
                found,
                function,
            } => {
                write!(
                    f,
                    "Function {} expects {} arguments, found {}",
                    function.inner(),
                    expected,
                    found
                )
            }
            SemanticError::ReturnTypeMismatch {
                expected, found, ..
            } => {
                write!(
                    f,
                    "Return type mismatch: expected {:?}, found {:?}",
                    expected, found
                )
            }
            SemanticError::InvalidBinaryOperation {
                left, op, right, ..
            } => {
                write!(f, "Invalid binary operation: {:?} {} {:?}", left, op, right)
            }
            SemanticError::InvalidUnaryOperation { op, operand, .. } => {
                write!(f, "Invalid unary operation: {} {:?}", op, operand)
            }
            SemanticError::RedefinedVariable { name, .. } => {
                write!(f, "Variable {} is already defined", name.inner())
            }
            SemanticError::RedefinedFunction { name, .. } => {
                write!(f, "Function {} is already defined", name.inner())
            }
            SemanticError::RedefinedStruct { name, .. } => {
                write!(f, "Struct {} is already defined", name.inner())
            }
        }
    }
}

impl std::error::Error for SemanticError {}
