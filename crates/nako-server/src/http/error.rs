use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use nako_api::public_client::{ClientErrorCode, ErrorResponse};
use nako_core::{NakoError, StorageErrorKind};
use tracing::{error, warn};

pub(super) type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(Debug)]
pub(super) struct ApiError(pub(super) NakoError);

impl From<NakoError> for ApiError {
    fn from(value: NakoError) -> Self {
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

        let mut response = (status, Json(body)).into_response();
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }

        response
    }
}

fn status_for_error(error: &NakoError) -> StatusCode {
    match error {
        NakoError::Storage {
            kind: StorageErrorKind::StagingBudgetExhausted,
            ..
        } => StatusCode::INSUFFICIENT_STORAGE,
        NakoError::Storage {
            kind: StorageErrorKind::Timeout,
            ..
        } => StatusCode::GATEWAY_TIMEOUT,
        NakoError::Storage {
            kind: StorageErrorKind::RateLimited,
            ..
        } => StatusCode::SERVICE_UNAVAILABLE,
        NakoError::InvalidInput { .. } | NakoError::Unsupported(_) => StatusCode::BAD_REQUEST,
        NakoError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
        NakoError::Forbidden { .. } => StatusCode::FORBIDDEN,
        NakoError::NotFound { .. } => StatusCode::NOT_FOUND,
        NakoError::Conflict { .. } => StatusCode::CONFLICT,
        NakoError::Provider { .. } | NakoError::Storage { .. } => StatusCode::BAD_GATEWAY,
        NakoError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn code_for_error(error: &NakoError) -> ClientErrorCode {
    match error {
        NakoError::InvalidInput { .. } => ClientErrorCode::InvalidInput,
        NakoError::NotFound { .. } => ClientErrorCode::NotFound,
        NakoError::Conflict { .. } => ClientErrorCode::Conflict,
        NakoError::Unauthorized { .. } => ClientErrorCode::Unauthorized,
        NakoError::Forbidden { .. } => ClientErrorCode::Forbidden,
        NakoError::Unsupported(_) => ClientErrorCode::Unsupported,
        NakoError::Provider { provider, .. } if is_ffmpeg_provider(provider) => {
            ClientErrorCode::FfmpegError
        }
        NakoError::Provider { .. } => ClientErrorCode::ProviderError,
        NakoError::Storage {
            kind: StorageErrorKind::StagingBudgetExhausted,
            ..
        } => ClientErrorCode::StagingBudgetExhausted,
        NakoError::Storage {
            kind: StorageErrorKind::StagingValidationMismatch,
            ..
        } => ClientErrorCode::StagingValidationMismatch,
        NakoError::Storage {
            kind: StorageErrorKind::Timeout,
            ..
        } => ClientErrorCode::StorageTimeout,
        NakoError::Storage {
            kind: StorageErrorKind::Unauthorized,
            ..
        } => ClientErrorCode::StorageUnauthorized,
        NakoError::Storage {
            kind: StorageErrorKind::RateLimited,
            ..
        } => ClientErrorCode::StorageRateLimited,
        NakoError::Storage { .. } => ClientErrorCode::StorageError,
        NakoError::Database { .. } => ClientErrorCode::DatabaseError,
    }
}

fn public_message(error: &NakoError) -> String {
    match error {
        NakoError::Database { .. } => "database operation failed".to_owned(),
        NakoError::Provider { provider, .. } if is_ffmpeg_provider(provider) => {
            "ffmpeg operation failed".to_owned()
        }
        NakoError::Provider { provider, .. } => {
            format!("external provider operation failed: {provider}")
        }
        NakoError::Storage {
            kind: StorageErrorKind::StagingBudgetExhausted,
            ..
        } => "staging disk budget exhausted".to_owned(),
        NakoError::Storage {
            kind: StorageErrorKind::StagingValidationMismatch,
            ..
        } => "staged input validation failed".to_owned(),
        NakoError::Storage {
            kind: StorageErrorKind::Timeout,
            ..
        } => "storage backend timed out".to_owned(),
        NakoError::Storage {
            kind: StorageErrorKind::Unauthorized,
            ..
        } => "storage backend rejected credentials".to_owned(),
        NakoError::Storage {
            kind: StorageErrorKind::RateLimited,
            ..
        } => "storage backend rate limited the request".to_owned(),
        NakoError::Storage { .. } => "storage operation failed".to_owned(),
        NakoError::InvalidInput { .. }
        | NakoError::NotFound { .. }
        | NakoError::Conflict { .. }
        | NakoError::Unauthorized { .. }
        | NakoError::Forbidden { .. }
        | NakoError::Unsupported(_) => error.to_string(),
    }
}

fn is_ffmpeg_provider(provider: &str) -> bool {
    provider == "ffmpeg" || provider == "ffmpeg_remux" || provider == "ffmpeg_hls"
}
