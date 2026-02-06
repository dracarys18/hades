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
    InvalidType {
        name: Ident,
        span: Span,
    },
    InvalidModuleName {
        name: Ident,
        span: Span,
    },
    InvalidImport {
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
            SemanticError::InvalidType { name, .. } => {
                write!(f, "Invalid type: {}", name.inner())
            }
            SemanticError::InvalidModuleName { name, .. } => {
                write!(f, "Invalid module name: {}", name.inner())
            }
            SemanticError::InvalidImport { name, .. } => {
                write!(f, "Invalid import: {}", name.inner())
            }
        }
    }
}

impl std::error::Error for SemanticError {}

impl SemanticError {
    pub fn eprint(&self, source: &str, filename: &str) {
        let err: crate::error::Error = self.clone().into_error(filename.to_string());
        err.eprint(source);
    }

    pub fn into_error(self, filename: String) -> crate::error::Error {
        let (message, span) = match &self {
            SemanticError::TypeMismatch {
                expected,
                found,
                span,
            } => (
                format!("Type mismatch: expected {}, found {}", expected, found),
                span.clone(),
            ),
            SemanticError::UndefinedVariable { name, span } => (
                format!("Undefined variable: {}", name.inner()),
                span.clone(),
            ),
            SemanticError::UndefinedFunction { name, span } => (
                format!("Undefined function: {}", name.inner()),
                span.clone(),
            ),
            SemanticError::UndefinedStruct { name, span } => {
                (format!("Undefined struct: {}", name.inner()), span.clone())
            }
            SemanticError::NotAStruct { name } => {
                (format!("{} is not a struct", name.inner()), Span::new(0, 0))
            }
            SemanticError::UnknownField {
                struct_name,
                field_name,
            } => (
                format!(
                    "Unknown field {} in struct {}",
                    field_name.inner(),
                    struct_name.inner()
                ),
                Span::new(0, 0),
            ),
            SemanticError::ArgumentCountMismatch {
                expected,
                found,
                function,
            } => (
                format!(
                    "Function {} expects {} arguments, found {}",
                    function.inner(),
                    expected,
                    found
                ),
                Span::new(0, 0),
            ),
            SemanticError::ReturnTypeMismatch {
                expected,
                found,
                span,
            } => (
                format!(
                    "Return type mismatch: expected {}, found {}",
                    expected, found
                ),
                span.clone(),
            ),
            SemanticError::InvalidBinaryOperation {
                left,
                op,
                right,
                span,
            } => (
                format!("Invalid binary operation: {} {} {}", left, op, right),
                span.clone(),
            ),
            SemanticError::InvalidUnaryOperation { op, operand, span } => (
                format!("Invalid unary operation: {} {}", op, operand),
                span.clone(),
            ),
            SemanticError::RedefinedVariable { name, span } => (
                format!("Variable {} is already defined", name.inner()),
                span.clone(),
            ),
            SemanticError::RedefinedFunction { name, span } => (
                format!("Function {} is already defined", name.inner()),
                span.clone(),
            ),
            SemanticError::RedefinedStruct { name, span } => (
                format!("Struct {} is already defined", name.inner()),
                span.clone(),
            ),
            SemanticError::InvalidType { name, span } => {
                (format!("Invalid type: {}", name.inner()), span.clone())
            }
            SemanticError::InvalidModuleName { name, span } => (
                format!("Invalid module name: {}", name.inner()),
                span.clone(),
            ),
            SemanticError::InvalidImport { name, span } => {
                (format!("Invalid import: {}", name.inner()), span.clone())
            }
        };

        crate::error::Error {
            message,
            span,
            context: filename,
            severity: crate::error::ErrorSeverity::Error,
            help: None,
            note: None,
        }
    }
}
