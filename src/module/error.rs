use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Module not found: {0}")]
    NotFound(String),

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in module {module}: {error}")]
    ParseError { module: String, error: String },
}
