use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PddError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Ambiguity analysis error: {0}")]
    AmbiguityError(String),

    #[error("Decision resolution error: {0}")]
    ResolutionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, PddError>;

