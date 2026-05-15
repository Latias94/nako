use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use taru_api::ErrorResponse;
use taru_core::TaruError;
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
            code: code_for_error(&self.0).to_owned(),
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
        TaruError::Storage { message, .. } if is_staging_budget_exhausted(message) => {
            StatusCode::INSUFFICIENT_STORAGE
        }
        TaruError::Storage { message, .. } if is_storage_timeout(message) => {
            StatusCode::GATEWAY_TIMEOUT
        }
        TaruError::Storage { message, .. } if is_storage_rate_limited(message) => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        TaruError::InvalidInput { .. } | TaruError::Unsupported(_) => StatusCode::BAD_REQUEST,
        TaruError::NotFound { .. } => StatusCode::NOT_FOUND,
        TaruError::Conflict { .. } => StatusCode::CONFLICT,
        TaruError::Provider { .. } | TaruError::Storage { .. } => StatusCode::BAD_GATEWAY,
        TaruError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn code_for_error(error: &TaruError) -> &'static str {
    match error {
        TaruError::InvalidInput { .. } => "invalid_input",
        TaruError::NotFound { .. } => "not_found",
        TaruError::Conflict { .. } => "conflict",
        TaruError::Unsupported(_) => "unsupported",
        TaruError::Provider { provider, .. } if is_ffmpeg_provider(provider) => "ffmpeg_error",
        TaruError::Provider { .. } => "provider_error",
        TaruError::Storage { message, .. } if is_staging_budget_exhausted(message) => {
            "staging_budget_exhausted"
        }
        TaruError::Storage { message, .. } if is_staging_validation_mismatch(message) => {
            "staging_validation_mismatch"
        }
        TaruError::Storage { message, .. } if is_storage_timeout(message) => "storage_timeout",
        TaruError::Storage { message, .. } if is_storage_unauthorized(message) => {
            "storage_unauthorized"
        }
        TaruError::Storage { message, .. } if is_storage_rate_limited(message) => {
            "storage_rate_limited"
        }
        TaruError::Storage { .. } => "storage_error",
        TaruError::Database { .. } => "database_error",
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
        TaruError::Storage { message, .. } if is_staging_budget_exhausted(message) => {
            "staging disk budget exhausted".to_owned()
        }
        TaruError::Storage { message, .. } if is_staging_validation_mismatch(message) => {
            "staged input validation failed".to_owned()
        }
        TaruError::Storage { message, .. } if is_storage_timeout(message) => {
            "storage backend timed out".to_owned()
        }
        TaruError::Storage { message, .. } if is_storage_unauthorized(message) => {
            "storage backend rejected credentials".to_owned()
        }
        TaruError::Storage { message, .. } if is_storage_rate_limited(message) => {
            "storage backend rate limited the request".to_owned()
        }
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

fn is_staging_budget_exhausted(message: &str) -> bool {
    message.contains("staging disk budget exhausted")
}

fn is_staging_validation_mismatch(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("staged") && message.contains("did not match")
        || message.contains("staging validation")
}

fn is_storage_timeout(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("timed out")
        || message.contains("timeout")
        || message.contains("request timeout")
        || message.contains("408")
}

fn is_storage_unauthorized(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("unauthorized")
        || message.contains("forbidden")
        || message.contains("401")
        || message.contains("403")
}

fn is_storage_rate_limited(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("too many requests")
        || message.contains("rate limit")
        || message.contains("429")
}
