#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreHttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreSafeRequestPreview {
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreHttpRequest {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
    pub body_utf8: Option<String>,
    pub safe_preview: CoreSafeRequestPreview,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreHttpResponse {
    pub request_id: String,
    pub status_code: i32,
    pub headers: Vec<CoreHttpHeader>,
    pub body_utf8: String,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CorePublicError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CoreRuntimeFailureKind {
    MissingAccessToken,
    UnsupportedApiVersion,
    InvalidResponse,
    HttpError,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreRuntimeFailure {
    pub kind: CoreRuntimeFailureKind,
    pub status_code: Option<i32>,
    pub observed_api_version: Option<String>,
    pub public_error: Option<CorePublicError>,
    pub request: Option<CoreSafeRequestPreview>,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreConnectionProbeSuccess {
    pub api_version: String,
    pub health_request: CoreSafeRequestPreview,
    pub auth_probe_request: CoreSafeRequestPreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, uniffi::Enum)]
pub enum CoreConnectionProbeOutcomeKind {
    NextRequest,
    Success,
    Failure,
}

#[derive(Clone, Debug, Eq, PartialEq, uniffi::Record)]
pub struct CoreConnectionProbeOutcome {
    pub kind: CoreConnectionProbeOutcomeKind,
    pub next_request: Option<CoreHttpRequest>,
    pub success: Option<CoreConnectionProbeSuccess>,
    pub failure: Option<CoreRuntimeFailure>,
}

#[uniffi::export]
pub fn start_connection_probe(
    base_url: String,
    access_token: String,
) -> CoreConnectionProbeOutcome {
    let input = taru_client_core::CoreConnectionProbeInput::new(base_url, access_token);
    taru_client_core::start_connection_probe(&input).into()
}

#[uniffi::export]
pub fn advance_connection_probe(
    base_url: String,
    access_token: String,
    response: CoreHttpResponse,
) -> CoreConnectionProbeOutcome {
    let input = taru_client_core::CoreConnectionProbeInput::new(base_url, access_token);
    taru_client_core::advance_connection_probe(&input, &response.into()).into()
}

impl From<taru_client_core::CoreHttpHeader> for CoreHttpHeader {
    fn from(value: taru_client_core::CoreHttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<CoreHttpHeader> for taru_client_core::CoreHttpHeader {
    fn from(value: CoreHttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<taru_client_core::CoreSafeRequestPreview> for CoreSafeRequestPreview {
    fn from(value: taru_client_core::CoreSafeRequestPreview) -> Self {
        Self {
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CoreSafeRequestPreview> for taru_client_core::CoreSafeRequestPreview {
    fn from(value: CoreSafeRequestPreview) -> Self {
        Self {
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<taru_client_core::CoreHttpRequest> for CoreHttpRequest {
    fn from(value: taru_client_core::CoreHttpRequest) -> Self {
        Self {
            request_id: value.request_id,
            method: value.method,
            url: value.url,
            headers: value.headers.into_iter().map(Into::into).collect(),
            body_utf8: value.body_utf8,
            safe_preview: value.safe_preview.into(),
        }
    }
}

impl From<CoreHttpResponse> for taru_client_core::CoreHttpResponse {
    fn from(value: CoreHttpResponse) -> Self {
        Self {
            request_id: value.request_id,
            status_code: value.status_code,
            headers: value.headers.into_iter().map(Into::into).collect(),
            body_utf8: value.body_utf8,
        }
    }
}

impl From<taru_client_core::CorePublicError> for CorePublicError {
    fn from(value: taru_client_core::CorePublicError) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}

impl From<taru_client_core::CoreRuntimeFailureKind> for CoreRuntimeFailureKind {
    fn from(value: taru_client_core::CoreRuntimeFailureKind) -> Self {
        match value {
            taru_client_core::CoreRuntimeFailureKind::MissingAccessToken => {
                Self::MissingAccessToken
            }
            taru_client_core::CoreRuntimeFailureKind::UnsupportedApiVersion => {
                Self::UnsupportedApiVersion
            }
            taru_client_core::CoreRuntimeFailureKind::InvalidResponse => Self::InvalidResponse,
            taru_client_core::CoreRuntimeFailureKind::HttpError => Self::HttpError,
        }
    }
}

impl From<taru_client_core::CoreRuntimeFailure> for CoreRuntimeFailure {
    fn from(value: taru_client_core::CoreRuntimeFailure) -> Self {
        Self {
            kind: value.kind.into(),
            status_code: value.status_code,
            observed_api_version: value.observed_api_version,
            public_error: value.public_error.map(Into::into),
            request: value.request.map(Into::into),
        }
    }
}

impl From<taru_client_core::CoreConnectionProbeSuccess> for CoreConnectionProbeSuccess {
    fn from(value: taru_client_core::CoreConnectionProbeSuccess) -> Self {
        Self {
            api_version: value.api_version,
            health_request: value.health_request.into(),
            auth_probe_request: value.auth_probe_request.into(),
        }
    }
}

impl From<taru_client_core::CoreConnectionProbeOutcomeKind> for CoreConnectionProbeOutcomeKind {
    fn from(value: taru_client_core::CoreConnectionProbeOutcomeKind) -> Self {
        match value {
            taru_client_core::CoreConnectionProbeOutcomeKind::NextRequest => Self::NextRequest,
            taru_client_core::CoreConnectionProbeOutcomeKind::Success => Self::Success,
            taru_client_core::CoreConnectionProbeOutcomeKind::Failure => Self::Failure,
        }
    }
}

impl From<taru_client_core::CoreConnectionProbeOutcome> for CoreConnectionProbeOutcome {
    fn from(value: taru_client_core::CoreConnectionProbeOutcome) -> Self {
        Self {
            kind: value.kind.into(),
            next_request: value.next_request.map(Into::into),
            success: value.success.map(Into::into),
            failure: value.failure.map(Into::into),
        }
    }
}

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniffi_surface_exposes_core_connection_start() {
        let outcome = start_connection_probe(
            "https://taru.example/api".to_owned(),
            "secret-token".to_owned(),
        );

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::NextRequest);
        let request = outcome.next_request.unwrap();
        assert_eq!(
            request.request_id,
            taru_client_core::CONNECTION_HEALTH_REQUEST_ID
        );
        assert_eq!(request.url, "https://taru.example/api/health");
    }
}
