pub const CONNECTION_HEALTH_REQUEST_ID: &str = "connection.health";
pub const CONNECTION_AUTH_PROBE_REQUEST_ID: &str = "connection.auth_probe";

const REDACTED: &str = "<redacted>";

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
pub struct CoreHttpHeader {
    pub name: String,
    pub value: String,
}

impl CoreHttpHeader {
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreQueryParam {
    pub name: String,
    pub value: String,
}

impl CoreQueryParam {
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHttpRequestSpec {
    pub request_id: String,
    pub base_url: String,
    pub method: String,
    pub path: String,
    pub query: Vec<CoreQueryParam>,
    pub headers: Vec<CoreHttpHeader>,
    pub access_token: Option<String>,
    pub body_utf8: Option<String>,
}

impl CoreHttpRequestSpec {
    #[must_use]
    pub fn new(
        request_id: impl Into<String>,
        base_url: impl Into<String>,
        method: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            base_url: base_url.into(),
            method: method.into(),
            path: path.into(),
            query: Vec::new(),
            headers: Vec::new(),
            access_token: None,
            body_utf8: None,
        }
    }

    #[must_use]
    pub fn query(mut self, query: Vec<CoreQueryParam>) -> Self {
        self.query = query;
        self
    }

    #[must_use]
    pub fn headers(mut self, headers: Vec<CoreHttpHeader>) -> Self {
        self.headers = headers;
        self
    }

    #[must_use]
    pub fn access_token(mut self, access_token: Option<String>) -> Self {
        self.access_token = access_token;
        self
    }

    #[must_use]
    pub fn body_utf8(mut self, body_utf8: Option<String>) -> Self {
        self.body_utf8 = body_utf8;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHttpRequest {
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
    pub body_utf8: Option<String>,
    pub safe_preview: CoreSafeRequestPreview,
}

#[must_use]
pub fn build_core_request(spec: &CoreHttpRequestSpec) -> CoreHttpRequest {
    let mut headers = spec.headers.clone();
    let access_token = spec.access_token.as_deref().map(str::trim);
    if let Some(token) = access_token.filter(|token| !token.is_empty()) {
        if !headers
            .iter()
            .any(|header| header.name.eq_ignore_ascii_case("authorization"))
        {
            headers.insert(
                0,
                CoreHttpHeader::new("Authorization", format!("Bearer {token}")),
            );
        }
    }

    let secrets = access_token.into_iter().collect::<Vec<_>>();
    request(
        &spec.request_id,
        &spec.method,
        &url_on(&spec.base_url, &path_with_query(&spec.path, &spec.query)),
        headers,
        spec.body_utf8.clone(),
        &secrets,
    )
}

#[must_use]
pub fn interpret_core_response(
    response: &CoreHttpResponse,
    request: Option<&CoreSafeRequestPreview>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHttpResponse {
    pub request_id: String,
    pub status_code: i32,
    pub headers: Vec<CoreHttpHeader>,
    pub body_utf8: String,
}

impl CoreHttpResponse {
    #[must_use]
    pub fn new(
        request_id: impl Into<String>,
        status_code: i32,
        headers: Vec<CoreHttpHeader>,
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
pub struct CoreSafeRequestPreview {
    pub method: String,
    pub url: String,
    pub headers: Vec<CoreHttpHeader>,
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
    pub request: Option<CoreSafeRequestPreview>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConnectionProbeSuccess {
    pub api_version: String,
    pub health_request: CoreSafeRequestPreview,
    pub auth_probe_request: CoreSafeRequestPreview,
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
    pub next_request: Option<CoreHttpRequest>,
    pub success: Option<CoreConnectionProbeSuccess>,
    pub failure: Option<CoreRuntimeFailure>,
}

impl CoreConnectionProbeOutcome {
    #[must_use]
    pub fn next_request(request: CoreHttpRequest) -> Self {
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
    pub fn failure(failure: CoreRuntimeFailure) -> Self {
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
        return CoreConnectionProbeOutcome::failure(CoreRuntimeFailure {
            kind: CoreRuntimeFailureKind::MissingAccessToken,
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
    response: &CoreHttpResponse,
) -> CoreConnectionProbeOutcome {
    match response.request_id.as_str() {
        CONNECTION_HEALTH_REQUEST_ID => interpret_health_response(input, response),
        CONNECTION_AUTH_PROBE_REQUEST_ID => interpret_auth_probe_response(input, response),
        _ => CoreConnectionProbeOutcome::failure(CoreRuntimeFailure {
            kind: CoreRuntimeFailureKind::InvalidResponse,
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
    response: &CoreHttpResponse,
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
                return CoreConnectionProbeOutcome::failure(CoreRuntimeFailure {
                    kind: CoreRuntimeFailureKind::InvalidResponse,
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
        return CoreConnectionProbeOutcome::failure(CoreRuntimeFailure {
            kind: CoreRuntimeFailureKind::UnsupportedApiVersion,
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
    response: &CoreHttpResponse,
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

fn health_request(input: &CoreConnectionProbeInput) -> CoreHttpRequest {
    build_core_request(
        &CoreHttpRequestSpec::new(
            CONNECTION_HEALTH_REQUEST_ID,
            &input.base_url,
            "GET",
            "/health",
        )
        .access_token(None),
    )
}

fn auth_probe_request(input: &CoreConnectionProbeInput) -> CoreHttpRequest {
    build_core_request(
        &CoreHttpRequestSpec::new(
            CONNECTION_AUTH_PROBE_REQUEST_ID,
            &input.base_url,
            "GET",
            "/libraries",
        )
        .query(vec![
            CoreQueryParam::new("limit", "1"),
            CoreQueryParam::new("offset", "0"),
        ])
        .access_token(Some(input.access_token.clone())),
    )
}

fn request(
    request_id: &str,
    method: &str,
    url: &str,
    headers: Vec<CoreHttpHeader>,
    body_utf8: Option<String>,
    secrets: &[&str],
) -> CoreHttpRequest {
    let safe_preview = CoreSafeRequestPreview {
        method: method.to_owned(),
        url: sanitize(url, secrets),
        headers: headers
            .iter()
            .map(|header| safe_header(header, secrets))
            .collect(),
    };
    CoreHttpRequest {
        request_id: request_id.to_owned(),
        method: method.to_owned(),
        url: url.to_owned(),
        headers,
        body_utf8,
        safe_preview,
    }
}

fn safe_header(header: &CoreHttpHeader, secrets: &[&str]) -> CoreHttpHeader {
    if header.name.eq_ignore_ascii_case("authorization") {
        return CoreHttpHeader::new(&header.name, format!("Bearer {REDACTED}"));
    }
    CoreHttpHeader::new(&header.name, sanitize(&header.value, secrets))
}

fn http_failure(
    response: &CoreHttpResponse,
    request: Option<&CoreSafeRequestPreview>,
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

fn version_failure(
    response: &CoreHttpResponse,
    request: Option<&CoreSafeRequestPreview>,
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

fn header_value<'a>(response: &'a CoreHttpResponse, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}

fn sanitize(input: &str, secrets: &[&str]) -> String {
    let mut sanitized = input.to_owned();
    for secret in secrets.iter().copied().filter(|secret| !secret.is_empty()) {
        sanitized = sanitized.replace(secret, REDACTED);
    }
    redact_bearer_tokens(&sanitized)
}

fn redact_bearer_tokens(input: &str) -> String {
    let mut output = Vec::new();
    let mut redact_next = false;
    for token in input.split_whitespace() {
        if redact_next {
            output.push(REDACTED);
            redact_next = false;
            continue;
        }
        output.push(token);
        if token.eq_ignore_ascii_case("bearer") {
            redact_next = true;
        }
    }
    if output.is_empty() {
        input.to_owned()
    } else {
        output.join(" ")
    }
}

#[must_use]
pub fn encode_path_segment(value: &str) -> String {
    percent_encode(value)
}

fn path_with_query(path: &str, query: &[CoreQueryParam]) -> String {
    if query.is_empty() {
        return path.to_owned();
    }
    let mut path_and_query = path.to_owned();
    path_and_query.push('?');
    for (index, param) in query.iter().enumerate() {
        if index > 0 {
            path_and_query.push('&');
        }
        path_and_query.push_str(&percent_encode(&param.name));
        path_and_query.push('=');
        path_and_query.push_str(&percent_encode(&param.value));
    }
    path_and_query
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{byte:02X}"));
            }
        }
    }
    encoded
}

#[must_use]
pub fn url_on(base_url: &str, path_and_query: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path_and_query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input() -> CoreConnectionProbeInput {
        CoreConnectionProbeInput::new("https://taru.example/api/", "secret-token")
    }

    fn api_header() -> CoreHttpHeader {
        CoreHttpHeader::new(
            taru_client_protocol::API_VERSION_HEADER,
            taru_client_protocol::CLIENT_PROTOCOL_VERSION,
        )
    }

    #[test]
    fn connection_probe_starts_with_unauthenticated_health_request() {
        let outcome = start_connection_probe(&input());

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::NextRequest);
        let request = outcome.next_request.unwrap();
        assert_eq!(request.request_id, CONNECTION_HEALTH_REQUEST_ID);
        assert_eq!(request.method, "GET");
        assert_eq!(request.url, "https://taru.example/api/health");
        assert!(request.headers.is_empty());
        assert_eq!(request.safe_preview.url, "https://taru.example/api/health");
    }

    #[test]
    fn generic_core_request_builds_encoded_url_auth_header_and_safe_preview() {
        let request = build_core_request(
            &CoreHttpRequestSpec::new(
                "playback.decision",
                "https://taru.example/api/",
                "GET",
                &format!(
                    "/sources/{}/playback/decision",
                    encode_path_segment("source 1")
                ),
            )
            .query(vec![
                CoreQueryParam::new("direct_play", "true"),
                CoreQueryParam::new("container", "mp4,webm"),
            ])
            .access_token(Some("secret-token".to_owned())),
        );

        assert_eq!(request.request_id, "playback.decision");
        assert_eq!(
            request.url,
            "https://taru.example/api/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm"
        );
        assert_eq!(
            request.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer secret-token")]
        );
        assert_eq!(
            request.safe_preview.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn generic_response_interpreter_preserves_public_error_and_version_failures() {
        let request = build_core_request(
            &CoreHttpRequestSpec::new("demo", "https://taru.example", "GET", "/demo")
                .access_token(Some("secret-token".to_owned())),
        )
        .safe_preview;
        let http = CoreHttpResponse::new(
            "demo",
            403,
            vec![api_header()],
            json!({"code": "forbidden", "message": "secret-token cannot access this source"})
                .to_string(),
        );
        let failure =
            interpret_core_response(&http, Some(&request), &["secret-token"]).unwrap_err();

        assert_eq!(failure.kind, CoreRuntimeFailureKind::HttpError);
        assert_eq!(failure.status_code, Some(403));
        assert_eq!(
            failure.public_error,
            Some(CorePublicError {
                code: "forbidden".to_owned(),
                message: "<redacted> cannot access this source".to_owned(),
            })
        );

        let version = CoreHttpResponse::new(
            "demo",
            200,
            vec![CoreHttpHeader::new(
                taru_client_protocol::API_VERSION_HEADER,
                "v2",
            )],
            "{}",
        );
        let failure = interpret_core_response(&version, Some(&request), &[]).unwrap_err();
        assert_eq!(failure.kind, CoreRuntimeFailureKind::UnsupportedApiVersion);
        assert_eq!(failure.observed_api_version.as_deref(), Some("v2"));
    }

    #[test]
    fn connection_probe_reports_missing_token_before_auth_probe() {
        let outcome =
            start_connection_probe(&CoreConnectionProbeInput::new("https://taru.example", "  "));

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        assert_eq!(
            outcome.failure.unwrap().kind,
            CoreRuntimeFailureKind::MissingAccessToken
        );
    }

    #[test]
    fn health_success_advances_to_redacted_authenticated_probe_request() {
        let response = CoreHttpResponse::new(
            CONNECTION_HEALTH_REQUEST_ID,
            200,
            vec![api_header()],
            json!({"status": "ok", "version": "v1"}).to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::NextRequest);
        let request = outcome.next_request.unwrap();
        assert_eq!(request.request_id, CONNECTION_AUTH_PROBE_REQUEST_ID);
        assert_eq!(
            request.url,
            "https://taru.example/api/libraries?limit=1&offset=0"
        );
        assert_eq!(
            request.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer secret-token")]
        );
        assert_eq!(
            request.safe_preview.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn health_body_version_mismatch_is_unsupported_api_version() {
        let response = CoreHttpResponse::new(
            CONNECTION_HEALTH_REQUEST_ID,
            200,
            vec![api_header()],
            json!({"status": "ok", "version": "v2"}).to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        let failure = outcome.failure.unwrap();
        assert_eq!(failure.kind, CoreRuntimeFailureKind::UnsupportedApiVersion);
        assert_eq!(failure.observed_api_version.as_deref(), Some("v2"));
    }

    #[test]
    fn auth_probe_http_error_preserves_public_error_and_redacts_token() {
        let response = CoreHttpResponse::new(
            CONNECTION_AUTH_PROBE_REQUEST_ID,
            401,
            vec![api_header()],
            json!({
                "code": "unauthorized",
                "message": "Bearer secret-token is expired"
            })
            .to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        let failure = outcome.failure.unwrap();
        assert_eq!(failure.kind, CoreRuntimeFailureKind::HttpError);
        assert_eq!(failure.status_code, Some(401));
        assert_eq!(
            failure.public_error,
            Some(CorePublicError {
                code: "unauthorized".to_owned(),
                message: "Bearer <redacted> is expired".to_owned(),
            })
        );
        assert_eq!(
            failure.request.unwrap().headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn auth_probe_success_returns_connection_success_with_safe_previews() {
        let response = CoreHttpResponse::new(
            CONNECTION_AUTH_PROBE_REQUEST_ID,
            200,
            vec![api_header()],
            json!({"libraries": [], "page": {"limit": 1, "offset": 0, "returned": 0}}).to_string(),
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Success);
        let success = outcome.success.unwrap();
        assert_eq!(success.api_version, "v1");
        assert_eq!(
            success.health_request.url,
            "https://taru.example/api/health"
        );
        assert_eq!(
            success.auth_probe_request.headers,
            vec![CoreHttpHeader::new("Authorization", "Bearer <redacted>")]
        );
    }

    #[test]
    fn invalid_health_json_is_invalid_response() {
        let response = CoreHttpResponse::new(
            CONNECTION_HEALTH_REQUEST_ID,
            200,
            vec![api_header()],
            "not-json",
        );

        let outcome = advance_connection_probe(&input(), &response);

        assert_eq!(outcome.kind, CoreConnectionProbeOutcomeKind::Failure);
        assert_eq!(
            outcome.failure.unwrap().kind,
            CoreRuntimeFailureKind::InvalidResponse
        );
    }
}
