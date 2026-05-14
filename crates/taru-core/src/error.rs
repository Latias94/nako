use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TaruError {
    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("not found: {entity} {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("operation is not supported: {0}")]
    Unsupported(&'static str),

    #[error("external provider error from {provider}: {message}")]
    Provider { provider: String, message: String },

    #[error("storage error at {uri}: {message}")]
    Storage { uri: String, message: String },

    #[error("database error: {message}")]
    Database { message: String },
}

pub type Result<T> = std::result::Result<T, TaruError>;
