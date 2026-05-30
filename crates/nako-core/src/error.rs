use serde::{Deserialize, Serialize};
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageFailureClass {
    Timeout,
    Unavailable,
    Permission,
    RateLimited,
    StaleCache,
    PartialRead,
    Budget,
    Security,
    Unknown,
}

impl StorageFailureClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Unavailable => "unavailable",
            Self::Permission => "permission",
            Self::RateLimited => "rate_limited",
            Self::StaleCache => "stale_cache",
            Self::PartialRead => "partial_read",
            Self::Budget => "budget",
            Self::Security => "security",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub const fn safe_message(self) -> &'static str {
        match self {
            Self::Timeout => "storage timeout",
            Self::Unavailable => "storage backend unavailable",
            Self::Permission => "storage permission failure",
            Self::RateLimited => "storage rate limited",
            Self::StaleCache => "storage stale cache fallback",
            Self::PartialRead => "storage partial read",
            Self::Budget => "storage budget exhausted",
            Self::Security => "storage security failure",
            Self::Unknown => "storage failure",
        }
    }

    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::Timeout
                | Self::Unavailable
                | Self::RateLimited
                | Self::StaleCache
                | Self::PartialRead
                | Self::Budget
        )
    }
}

impl StorageErrorKind {
    #[must_use]
    pub const fn failure_class(self) -> StorageFailureClass {
        match self {
            Self::Timeout => StorageFailureClass::Timeout,
            Self::Unauthorized => StorageFailureClass::Permission,
            Self::RateLimited => StorageFailureClass::RateLimited,
            Self::Network | Self::HttpStatus | Self::Io => StorageFailureClass::Unavailable,
            Self::StagingValidationMismatch => StorageFailureClass::PartialRead,
            Self::StagingBudgetExhausted | Self::ResourceBudgetClosed => {
                StorageFailureClass::Budget
            }
            Self::SecurityViolation | Self::Backup => StorageFailureClass::Security,
            Self::Unknown => StorageFailureClass::Unknown,
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum NakoError {
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

impl NakoError {
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

    #[must_use]
    pub const fn storage_failure_class(&self) -> Option<StorageFailureClass> {
        match self {
            Self::Storage { kind, .. } => Some(kind.failure_class()),
            _ => None,
        }
    }

    #[must_use]
    pub fn safe_storage_message(&self) -> Option<String> {
        self.storage_failure_class()
            .map(|class| class.safe_message().to_owned())
    }
}

pub type Result<T> = std::result::Result<T, NakoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_errors_expose_safe_failure_classification() {
        let timeout = NakoError::storage_timeout(
            "webdav:///Movies/Private/Demo.mkv",
            "request timed out for webdav:///Movies/Private/Demo.mkv",
        );
        let permission = NakoError::storage_unauthorized(
            "webdav:///Movies/Private/Demo.mkv",
            "forbidden for user media",
        );
        let rate_limited = NakoError::storage_rate_limited(
            "webdav:///Movies/Private/Demo.mkv",
            "429 for webdav:///Movies/Private/Demo.mkv",
        );
        let partial = NakoError::storage_staging_validation_mismatch(
            "webdav:///Movies/Private/Demo.mkv",
            "staged file was shorter than expected",
        );
        let unknown = NakoError::storage_unknown(
            "webdav:///Movies/Private/Demo.mkv",
            "adapter did not classify failure",
        );

        assert_eq!(
            timeout.storage_failure_class(),
            Some(StorageFailureClass::Timeout)
        );
        assert_eq!(
            permission.storage_failure_class(),
            Some(StorageFailureClass::Permission)
        );
        assert_eq!(
            rate_limited.storage_failure_class(),
            Some(StorageFailureClass::RateLimited)
        );
        assert_eq!(
            partial.storage_failure_class(),
            Some(StorageFailureClass::PartialRead)
        );
        assert_eq!(
            unknown.storage_failure_class(),
            Some(StorageFailureClass::Unknown)
        );

        assert!(timeout.storage_failure_class().unwrap().is_retryable());
        assert!(!permission.storage_failure_class().unwrap().is_retryable());
        assert!(rate_limited.storage_failure_class().unwrap().is_retryable());
        assert!(partial.storage_failure_class().unwrap().is_retryable());
        assert!(!unknown.storage_failure_class().unwrap().is_retryable());

        assert_eq!(
            timeout.safe_storage_message(),
            Some("storage timeout".to_owned())
        );
        assert_eq!(
            permission.safe_storage_message(),
            Some("storage permission failure".to_owned())
        );
        assert_eq!(
            rate_limited.safe_storage_message(),
            Some("storage rate limited".to_owned())
        );
        assert_eq!(
            partial.safe_storage_message(),
            Some("storage partial read".to_owned())
        );
        assert_eq!(
            unknown.safe_storage_message(),
            Some("storage failure".to_owned())
        );
    }
}
