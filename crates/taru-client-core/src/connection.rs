use crate::response::{header_value, http_failure, version_failure};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConnectionProbeInput {
    pub base_url: String,
    pub access_token: String,
}

impl CoreConnectionProbeInput {
    #[must_use]
    pub fn new(base_url: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            access_token: access_token.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConnectionProbeSuccess {
    pub api_version: String,
    pub health_request: crate::CoreSafeRequestPreview,
    pub auth_probe_request: crate::CoreSafeRequestPreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreConnectionProbeOutcomeKind {
    NextRequest,
    Success,
    Failure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConnectionProbeOutcome {
    pub kind: CoreConnectionProbeOutcomeKind,
    pub next_request: Option<crate::CoreHttpRequest>,
    pub success: Option<CoreConnectionProbeSuccess>,
    pub failure: Option<crate::CoreRuntimeFailure>,
}

impl CoreConnectionProbeOutcome {
    #[must_use]
    pub fn next_request(request: crate::CoreHttpRequest) -> Self {
        Self {
            kind: CoreConnectionProbeOutcomeKind::NextRequest,
            next_request: Some(request),
            success: None,
            failure: None,
        }
    }

    #[must_use]
    pub fn success(success: CoreConnectionProbeSuccess) -> Self {
        Self {
            kind: CoreConnectionProbeOutcomeKind::Success,
            next_request: None,
            success: Some(success),
            failure: None,
        }
    }

    #[must_use]
    pub fn failure(failure: crate::CoreRuntimeFailure) -> Self {
        Self {
            kind: CoreConnectionProbeOutcomeKind::Failure,
            next_request: None,
            success: None,
            failure: Some(failure),
        }
    }
}

#[must_use]
pub fn start_connection_probe(input: &CoreConnectionProbeInput) -> CoreConnectionProbeOutcome {
    if input.access_token.trim().is_empty() {
        return CoreConnectionProbeOutcome::failure(crate::CoreRuntimeFailure {
            kind: crate::CoreRuntimeFailureKind::MissingAccessToken,
            status_code: None,
            observed_api_version: None,
            public_error: None,
            request: None,
        });
    }
    CoreConnectionProbeOutcome::next_request(health_request(input))
}

#[must_use]
pub fn advance_connection_probe(
    input: &CoreConnectionProbeInput,
    response: &crate::CoreHttpResponse,
) -> CoreConnectionProbeOutcome {
    match response.request_id.as_str() {
        crate::CONNECTION_HEALTH_REQUEST_ID => interpret_health_response(input, response),
        crate::CONNECTION_AUTH_PROBE_REQUEST_ID => interpret_auth_probe_response(input, response),
        _ => CoreConnectionProbeOutcome::failure(crate::CoreRuntimeFailure {
            kind: crate::CoreRuntimeFailureKind::InvalidResponse,
            status_code: Some(response.status_code),
            observed_api_version: header_value(response, taru_client_protocol::API_VERSION_HEADER)
                .map(str::to_owned),
            public_error: None,
            request: None,
        }),
    }
}

fn interpret_health_response(
    input: &CoreConnectionProbeInput,
    response: &crate::CoreHttpResponse,
) -> CoreConnectionProbeOutcome {
    let request = health_request(input).safe_preview;
    if let Some(failure) = http_failure(response, Some(&request), &[input.access_token.as_str()]) {
        return CoreConnectionProbeOutcome::failure(failure);
    }
    if let Some(failure) = version_failure(response, Some(&request)) {
        return CoreConnectionProbeOutcome::failure(failure);
    }

    let health =
        match serde_json::from_str::<taru_client_protocol::HealthResponse>(&response.body_utf8) {
            Ok(health) if !health.status.trim().is_empty() && !health.version.trim().is_empty() => {
                health
            }
            _ => {
                return CoreConnectionProbeOutcome::failure(crate::CoreRuntimeFailure {
                    kind: crate::CoreRuntimeFailureKind::InvalidResponse,
                    status_code: Some(response.status_code),
                    observed_api_version: header_value(
                        response,
                        taru_client_protocol::API_VERSION_HEADER,
                    )
                    .map(str::to_owned),
                    public_error: None,
                    request: Some(request),
                });
            }
        };

    if health.version != taru_client_protocol::CLIENT_PROTOCOL_VERSION {
        return CoreConnectionProbeOutcome::failure(crate::CoreRuntimeFailure {
            kind: crate::CoreRuntimeFailureKind::UnsupportedApiVersion,
            status_code: Some(response.status_code),
            observed_api_version: Some(health.version),
            public_error: None,
            request: Some(request),
        });
    }

    CoreConnectionProbeOutcome::next_request(auth_probe_request(input))
}

fn interpret_auth_probe_response(
    input: &CoreConnectionProbeInput,
    response: &crate::CoreHttpResponse,
) -> CoreConnectionProbeOutcome {
    let request = auth_probe_request(input).safe_preview;
    if let Some(failure) = http_failure(response, Some(&request), &[input.access_token.as_str()]) {
        return CoreConnectionProbeOutcome::failure(failure);
    }
    if let Some(failure) = version_failure(response, Some(&request)) {
        return CoreConnectionProbeOutcome::failure(failure);
    }

    CoreConnectionProbeOutcome::success(CoreConnectionProbeSuccess {
        api_version: taru_client_protocol::CLIENT_PROTOCOL_VERSION.to_owned(),
        health_request: health_request(input).safe_preview,
        auth_probe_request: request,
    })
}

fn health_request(input: &CoreConnectionProbeInput) -> crate::CoreHttpRequest {
    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(
            crate::CONNECTION_HEALTH_REQUEST_ID,
            &input.base_url,
            "GET",
            "/health",
        )
        .access_token(None),
    )
}

fn auth_probe_request(input: &CoreConnectionProbeInput) -> crate::CoreHttpRequest {
    crate::build_core_request(
        &crate::CoreHttpRequestSpec::new(
            crate::CONNECTION_AUTH_PROBE_REQUEST_ID,
            &input.base_url,
            "GET",
            "/libraries",
        )
        .query(vec![
            crate::CoreQueryParam::new("limit", "1"),
            crate::CoreQueryParam::new("offset", "0"),
        ])
        .access_token(Some(input.access_token.clone())),
    )
}
