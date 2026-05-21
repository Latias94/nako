use crate::redaction::sanitize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHttpResponse {
    pub request_id: String,
    pub status_code: i32,
    pub headers: Vec<crate::CoreHttpHeader>,
    pub body_utf8: String,
}

impl CoreHttpResponse {
    #[must_use]
    pub fn new(
        request_id: impl Into<String>,
        status_code: i32,
        headers: Vec<crate::CoreHttpHeader>,
        body_utf8: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            status_code,
            headers,
            body_utf8: body_utf8.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePublicError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreRuntimeFailureKind {
    MissingAccessToken,
    UnsupportedApiVersion,
    InvalidResponse,
    HttpError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreRuntimeFailure {
    pub kind: CoreRuntimeFailureKind,
    pub status_code: Option<i32>,
    pub observed_api_version: Option<String>,
    pub public_error: Option<CorePublicError>,
    pub request: Option<crate::CoreSafeRequestPreview>,
}

#[must_use]
pub fn interpret_core_response(
    response: &CoreHttpResponse,
    request: Option<&crate::CoreSafeRequestPreview>,
    secrets: &[&str],
) -> Result<(), CoreRuntimeFailure> {
    if let Some(failure) = http_failure(response, request, secrets) {
        return Err(failure);
    }
    if let Some(failure) = version_failure(response, request) {
        return Err(failure);
    }
    Ok(())
}

pub(crate) fn http_failure(
    response: &CoreHttpResponse,
    request: Option<&crate::CoreSafeRequestPreview>,
    secrets: &[&str],
) -> Option<CoreRuntimeFailure> {
    if (200..=299).contains(&response.status_code) {
        return None;
    }
    Some(CoreRuntimeFailure {
        kind: CoreRuntimeFailureKind::HttpError,
        status_code: Some(response.status_code),
        observed_api_version: header_value(response, taru_client_protocol::API_VERSION_HEADER)
            .map(str::to_owned),
        public_error: public_error(response, secrets),
        request: request.cloned(),
    })
}

pub(crate) fn version_failure(
    response: &CoreHttpResponse,
    request: Option<&crate::CoreSafeRequestPreview>,
) -> Option<CoreRuntimeFailure> {
    let observed = header_value(response, taru_client_protocol::API_VERSION_HEADER)?;
    if observed == taru_client_protocol::CLIENT_PROTOCOL_VERSION {
        return None;
    }
    Some(CoreRuntimeFailure {
        kind: CoreRuntimeFailureKind::UnsupportedApiVersion,
        status_code: Some(response.status_code),
        observed_api_version: Some(observed.to_owned()),
        public_error: None,
        request: request.cloned(),
    })
}

fn public_error(response: &CoreHttpResponse, secrets: &[&str]) -> Option<CorePublicError> {
    let error =
        serde_json::from_str::<taru_client_protocol::ErrorResponse>(&response.body_utf8).ok()?;
    Some(CorePublicError {
        code: sanitize(&error.code, secrets),
        message: sanitize(&error.message, secrets),
    })
}

pub(crate) fn header_value<'a>(response: &'a CoreHttpResponse, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}
