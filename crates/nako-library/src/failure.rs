use std::time::{SystemTime, UNIX_EPOCH};

use nako_core::{IngestionFailureClass, NakoError};

pub(crate) fn ingestion_failure_class(err: &NakoError) -> IngestionFailureClass {
    match err {
        NakoError::Storage { .. } => IngestionFailureClass::Storage,
        NakoError::Provider { provider, .. } if provider == "ffprobe" => {
            IngestionFailureClass::Probe
        }
        NakoError::Provider { .. } => IngestionFailureClass::Unknown,
        NakoError::Database { .. } => IngestionFailureClass::Database,
        NakoError::InvalidInput { .. } => IngestionFailureClass::InvalidInput,
        NakoError::Unsupported(_) => IngestionFailureClass::Unsupported,
        NakoError::NotFound { .. }
        | NakoError::Conflict { .. }
        | NakoError::Unauthorized { .. }
        | NakoError::Forbidden { .. } => IngestionFailureClass::Unknown,
    }
}

pub(crate) fn ingestion_failure_is_retryable(err: &NakoError) -> bool {
    match err {
        NakoError::Storage { .. } => err
            .storage_failure_class()
            .is_some_and(|class| class.is_retryable()),
        NakoError::Provider { .. } | NakoError::Database { .. } => true,
        _ => false,
    }
}

pub(crate) fn ingestion_failure_message(err: &NakoError) -> String {
    err.safe_storage_message()
        .unwrap_or_else(|| err.to_string())
}

pub(crate) fn ingestion_failure_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}
