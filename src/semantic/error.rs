use crate::{
    error::{Error, Span},
    tokens::Ident,
};

#[derive(Debug)]
pub struct SemanticError {
    message: String,
    span: Span,
}

impl SemanticError {
    pub fn type_mismatch(expected: String, found: String, span: Span) -> Self {
        Self {
            message: format!("Type mismatch: expected {}, found {}", expected, found),
            span,
        }
    }

    pub fn undefined_variable(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Undefined variable: {}", name.inner()),
            span,
        }
    }

    pub fn undefined_function(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Undefined function: {}", name.inner()),
            span,
        }
    }

    pub fn undefined_struct(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Undefined struct: {}", name.inner()),
            span,
        }
    }

    pub fn not_a_struct(name: Ident, span: Span) -> Self {
        Self {
            message: format!("{} is not a struct", name.inner()),
            span,
        }
    }

    pub fn unknown_field(struct_name: Ident, field_name: Ident, span: Span) -> Self {
        Self {
            message: format!(
                "Unknown field '{}' in struct '{}'",
                field_name.inner(),
                struct_name.inner()
            ),
            span,
        }
    }

    pub fn argument_count_mismatch(
        expected: usize,
        found: usize,
        function: Ident,
        span: Span,
    ) -> Self {
        Self {
            message: format!(
                "Function {} expects {} arguments, found {}",
                function.inner(),
                expected,
                found
            ),
            span,
        }
    }

    pub fn return_type_mismatch(expected: String, found: String, span: Span) -> Self {
        Self {
            message: format!(
                "Return type mismatch: expected {}, found {}",
                expected, found
            ),
            span,
        }
    }

    pub fn invalid_binary_operation(left: String, op: String, right: String, span: Span) -> Self {
        Self {
            message: format!("Invalid binary operation: {} {} {}", left, op, right),
            span,
        }
    }

    pub fn invalid_unary_operation(op: String, operand: String, span: Span) -> Self {
        Self {
            message: format!("Invalid unary operation: {} {}", op, operand),
            span,
        }
    }

    pub fn redefined_variable(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Variable {} is already defined", name.inner()),
            span,
        }
    }

    pub fn redefined_function(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Function {} is already defined", name.inner()),
            span,
        }
    }

    pub fn redefined_struct(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Struct {} is already defined", name.inner()),
            span,
        }
    }

    pub fn invalid_type(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Invalid type: {}", name.inner()),
            span,
        }
    }

    pub fn invalid_module_name(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Invalid module name: {}", name.inner()),
            span,
        }
    }

    pub fn invalid_import(name: Ident, span: Span) -> Self {
        Self {
            message: format!("Invalid import: {}", name.inner()),
            span,
        }
    }

    pub fn into_error(self) -> Error {
        Error::new_with_span(self.message, self.span)
    }
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SemanticError {}
