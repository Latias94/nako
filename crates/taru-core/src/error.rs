use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageErrorKind {
    Unknown,
    Io,
    Network,
    Timeout,
    Unauthorized,
    RateLimited,
    HttpStatus,
    StagingBudgetExhausted,
    StagingValidationMismatch,
    ResourceBudgetClosed,
    SecurityViolation,
    Backup,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TaruError {
    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("not found: {entity} {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("forbidden: {message}")]
    Forbidden { message: String },

    #[error("operation is not supported: {0}")]
    Unsupported(&'static str),

    #[error("external provider error from {provider}: {message}")]
    Provider { provider: String, message: String },

    #[error("storage error at {uri}: {message}")]
    Storage {
        uri: String,
        kind: StorageErrorKind,
        message: String,
    },

    #[error("database error: {message}")]
    Database { message: String },
}

impl TaruError {
    pub fn storage(
        uri: impl Into<String>,
        kind: StorageErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self::Storage {
            uri: uri.into(),
            kind,
            message: message.into(),
        }
    }

    pub fn storage_unknown(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::Unknown, message)
    }

    pub fn storage_io(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::Io, message)
    }

    pub fn storage_network(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::Network, message)
    }

    pub fn storage_timeout(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::Timeout, message)
    }

    pub fn storage_unauthorized(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::Unauthorized, message)
    }

    pub fn storage_rate_limited(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::RateLimited, message)
    }

    pub fn storage_http_status(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::HttpStatus, message)
    }

    pub fn storage_staging_budget_exhausted(
        uri: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::storage(uri, StorageErrorKind::StagingBudgetExhausted, message)
    }

    pub fn storage_staging_validation_mismatch(
        uri: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::storage(uri, StorageErrorKind::StagingValidationMismatch, message)
    }

    pub fn storage_resource_budget_closed(
        uri: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::storage(uri, StorageErrorKind::ResourceBudgetClosed, message)
    }

    pub fn storage_security_violation(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::SecurityViolation, message)
    }

    pub fn storage_backup(uri: impl Into<String>, message: impl Into<String>) -> Self {
        Self::storage(uri, StorageErrorKind::Backup, message)
    }
}

pub type Result<T> = std::result::Result<T, TaruError>;
