use serde::{Deserialize, Serialize};

use crate::{LibraryId, NakoError, Result, StorageFailureClass};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendHealthStatus {
    Healthy,
    Recovering,
    Unavailable,
}

impl StorageBackendHealthStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Recovering => "recovering",
            Self::Unavailable => "unavailable",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "healthy" => Ok(Self::Healthy),
            "recovering" => Ok(Self::Recovering),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown storage backend health status stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageCircuitBreakerState {
    Closed,
    HalfOpen,
    Open,
}

impl StorageCircuitBreakerState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::HalfOpen => "half_open",
            Self::Open => "open",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "closed" => Ok(Self::Closed),
            "half_open" => Ok(Self::HalfOpen),
            "open" => Ok(Self::Open),
            _ => Err(NakoError::Database {
                message: format!(
                    "unknown storage circuit breaker state stored in database: {value}"
                ),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendHealthRecord {
    pub backend_key: String,
    pub library_id: Option<LibraryId>,
    pub scheme: String,
    pub status: StorageBackendHealthStatus,
    pub circuit_breaker_state: StorageCircuitBreakerState,
    pub consecutive_failures: u32,
    pub last_success_at_ms: Option<i64>,
    pub last_failure_at_ms: Option<i64>,
    pub last_failure_class: Option<StorageFailureClass>,
    pub last_failure_safe_message: Option<String>,
    pub circuit_opened_at_ms: Option<i64>,
    pub backoff_until_ms: Option<i64>,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendHealthListFilter {
    pub library_id: Option<LibraryId>,
    pub scheme: Option<String>,
    pub status: Option<StorageBackendHealthStatus>,
    pub circuit_breaker_state: Option<StorageCircuitBreakerState>,
}
