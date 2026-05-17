use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use taru_api::{ClientErrorCode, ErrorResponse};
use taru_core::{StorageErrorKind, TaruError};
use tracing::{error, warn};

pub(super) type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Debug)]
pub(super) struct ApiError(pub(super) TaruError);

impl From<TaruError> for ApiError {
    fn from(value: TaruError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = status_for_error(&self.0);
        let body = ErrorResponse {
            code: code_for_error(&self.0).into(),
            message: public_message(&self.0),
        };

        if status.is_server_error() {
            error!(error = %self.0, status = %status, "request failed");
        } else {
            warn!(error = %self.0, status = %status, "request rejected");
        }

        (status, Json(body)).into_response()
    }
}

fn status_for_error(error: &TaruError) -> StatusCode {
    match error {
        TaruError::Storage {
            kind: StorageErrorKind::StagingBudgetExhausted,
            ..
        } => StatusCode::INSUFFICIENT_STORAGE,
        TaruError::Storage {
            kind: StorageErrorKind::Timeout,
            ..
        } => StatusCode::GATEWAY_TIMEOUT,
        TaruError::Storage {
            kind: StorageErrorKind::RateLimited,
            ..
        } => StatusCode::SERVICE_UNAVAILABLE,
        TaruError::InvalidInput { .. } | TaruError::Unsupported(_) => StatusCode::BAD_REQUEST,
        TaruError::NotFound { .. } => StatusCode::NOT_FOUND,
        TaruError::Conflict { .. } => StatusCode::CONFLICT,
        TaruError::Provider { .. } | TaruError::Storage { .. } => StatusCode::BAD_GATEWAY,
        TaruError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn code_for_error(error: &TaruError) -> ClientErrorCode {
    match error {
        TaruError::InvalidInput { .. } => ClientErrorCode::InvalidInput,
        TaruError::NotFound { .. } => ClientErrorCode::NotFound,
        TaruError::Conflict { .. } => ClientErrorCode::Conflict,
        TaruError::Unsupported(_) => ClientErrorCode::Unsupported,
        TaruError::Provider { provider, .. } if is_ffmpeg_provider(provider) => {
            ClientErrorCode::FfmpegError
        }
        TaruError::Provider { .. } => ClientErrorCode::ProviderError,
        TaruError::Storage {
            kind: StorageErrorKind::StagingBudgetExhausted,
            ..
        } => ClientErrorCode::StagingBudgetExhausted,
        TaruError::Storage {
            kind: StorageErrorKind::StagingValidationMismatch,
            ..
        } => ClientErrorCode::StagingValidationMismatch,
        TaruError::Storage {
            kind: StorageErrorKind::Timeout,
            ..
        } => ClientErrorCode::StorageTimeout,
        TaruError::Storage {
            kind: StorageErrorKind::Unauthorized,
            ..
        } => ClientErrorCode::StorageUnauthorized,
        TaruError::Storage {
            kind: StorageErrorKind::RateLimited,
            ..
        } => ClientErrorCode::StorageRateLimited,
        TaruError::Storage { .. } => ClientErrorCode::StorageError,
        TaruError::Database { .. } => ClientErrorCode::DatabaseError,
    }
}

fn public_message(error: &TaruError) -> String {
    match error {
        TaruError::Database { .. } => "database operation failed".to_owned(),
        TaruError::Provider { provider, .. } if is_ffmpeg_provider(provider) => {
            "ffmpeg operation failed".to_owned()
        }
        TaruError::Provider { provider, .. } => {
            format!("external provider operation failed: {provider}")
        }
        TaruError::Storage {
            kind: StorageErrorKind::StagingBudgetExhausted,
            ..
        } => "staging disk budget exhausted".to_owned(),
        TaruError::Storage {
            kind: StorageErrorKind::StagingValidationMismatch,
            ..
        } => "staged input validation failed".to_owned(),
        TaruError::Storage {
            kind: StorageErrorKind::Timeout,
            ..
        } => "storage backend timed out".to_owned(),
        TaruError::Storage {
            kind: StorageErrorKind::Unauthorized,
            ..
        } => "storage backend rejected credentials".to_owned(),
        TaruError::Storage {
            kind: StorageErrorKind::RateLimited,
            ..
        } => "storage backend rate limited the request".to_owned(),
        TaruError::Storage { .. } => "storage operation failed".to_owned(),
        TaruError::InvalidInput { .. }
        | TaruError::NotFound { .. }
        | TaruError::Conflict { .. }
        | TaruError::Unsupported(_) => error.to_string(),
    }
}

fn is_ffmpeg_provider(provider: &str) -> bool {
    provider == "ffmpeg" || provider == "ffmpeg_remux" || provider == "ffmpeg_hls"
}
