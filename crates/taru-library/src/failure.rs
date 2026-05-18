use std::time::{SystemTime, UNIX_EPOCH};

use taru_core::{IngestionFailureClass, TaruError};

pub(crate) fn ingestion_failure_class(err: &TaruError) -> IngestionFailureClass {
    match err {
        TaruError::Storage { .. } => IngestionFailureClass::Storage,
        TaruError::Provider { provider, .. } if provider == "ffprobe" => {
            IngestionFailureClass::Probe
        }
        TaruError::Provider { .. } => IngestionFailureClass::Unknown,
        TaruError::Database { .. } => IngestionFailureClass::Database,
        TaruError::InvalidInput { .. } => IngestionFailureClass::InvalidInput,
        TaruError::Unsupported(_) => IngestionFailureClass::Unsupported,
        TaruError::NotFound { .. }
        | TaruError::Conflict { .. }
        | TaruError::Unauthorized { .. }
        | TaruError::Forbidden { .. } => IngestionFailureClass::Unknown,
    }
}

pub(crate) fn ingestion_failure_is_retryable(err: &TaruError) -> bool {
    matches!(
        err,
        TaruError::Storage { .. } | TaruError::Provider { .. } | TaruError::Database { .. }
    )
}

pub(crate) fn ingestion_failure_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}
