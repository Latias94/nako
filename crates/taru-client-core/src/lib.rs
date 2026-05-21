pub const CONNECTION_HEALTH_REQUEST_ID: &str = "connection.health";
pub const CONNECTION_AUTH_PROBE_REQUEST_ID: &str = "connection.auth_probe";
pub const PLAYBACK_DECISION_REQUEST_ID: &str = "playback.decision";
pub const PLAYBACK_DIRECT_STREAM_REQUEST_ID: &str = "playback.direct_stream";
pub const PLAYBACK_DIRECT_STREAM_HEAD_REQUEST_ID: &str = "playback.direct_stream_head";
pub const PLAYBACK_REMUX_STREAM_REQUEST_ID: &str = "playback.remux_stream";
pub const PLAYBACK_REMUX_SESSION_PROBE_REQUEST_ID: &str = "playback.remux_session_probe";
pub const PLAYBACK_HLS_PLAYLIST_REQUEST_ID: &str = "playback.hls_playlist";
pub const PLAYBACK_HLS_SEGMENT_REQUEST_ID: &str = "playback.hls_segment";

const REDACTED: &str = "<redacted>";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConnectionProbeInput {
    pub base_url: String,
    pub access_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackCapabilities {
    pub direct_play: Option<bool>,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
}

impl CorePlaybackCapabilities {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            direct_play: None,
            containers: Vec::new(),
            video_codecs: Vec::new(),
            audio_codecs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackDecisionRequestInput {
    pub base_url: String,
    pub access_token: String,
    pub source_id: String,
    pub capabilities: CorePlaybackCapabilities,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorePlaybackMode {
    DirectPlay,
    Remux,
    Transcode,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreOutputContainer {
    Hls,
    Mp4,
    Mkv,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackDecisionSummary {
    pub source_id: String,
    pub mode: CorePlaybackMode,
    pub transcode_output_container: Option<CoreOutputContainer>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackTargetInput {
    pub base_url: String,
    pub decision: CorePlaybackDecisionSummary,
    pub capabilities: CorePlaybackCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreDirectPlaybackTargetInput {
    pub base_url: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreRemuxPlaybackTargetInput {
    pub base_url: String,
    pub source_id: String,
    pub capabilities: CorePlaybackCapabilities,
    pub output_container: Option<CoreOutputContainer>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreHlsPlaylistTargetInput {
    pub base_url: String,
    pub source_id: String,
    pub capabilities: CorePlaybackCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackTarget {
    pub request: CoreHttpRequest,
    pub session_probe_request: Option<CoreHttpRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePlaybackSegmentInput {
    pub base_url: String,
    pub session_id: String,
    pub segment_name: String,
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

#[must_use]
pub fn build_playback_decision_request(
    input: &CorePlaybackDecisionRequestInput,
) -> CoreHttpRequest {
    build_core_request(
        &CoreHttpRequestSpec::new(
            PLAYBACK_DECISION_REQUEST_ID,
            &input.base_url,
            "GET",
            &format!(
                "/sources/{}/playback/decision",
                encode_path_segment(&input.source_id)
            ),
        )
        .query(playback_capability_query(&input.capabilities))
        .access_token(Some(input.access_token.clone())),
    )
}

#[must_use]
pub fn build_recommended_playback_target(
    input: &CorePlaybackTargetInput,
) -> Option<CorePlaybackTarget> {
    match input.decision.mode {
        CorePlaybackMode::DirectPlay => Some(build_direct_playback_target(
            &CoreDirectPlaybackTargetInput {
                base_url: input.base_url.clone(),
                source_id: input.decision.source_id.clone(),
            },
        )),
        CorePlaybackMode::Remux => {
            Some(build_remux_playback_target(&CoreRemuxPlaybackTargetInput {
                base_url: input.base_url.clone(),
                source_id: input.decision.source_id.clone(),
                capabilities: input.capabilities.clone(),
                output_container: remux_output_container(&input.decision),
            }))
        }
        CorePlaybackMode::Transcode => {
            Some(build_hls_playlist_target(&CoreHlsPlaylistTargetInput {
                base_url: input.base_url.clone(),
                source_id: input.decision.source_id.clone(),
                capabilities: input.capabilities.clone(),
            }))
        }
        CorePlaybackMode::Unknown => None,
    }
}

#[must_use]
pub fn build_direct_playback_target(input: &CoreDirectPlaybackTargetInput) -> CorePlaybackTarget {
    CorePlaybackTarget {
        request: streaming_request(
            PLAYBACK_DIRECT_STREAM_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream",
            Vec::new(),
            None,
        ),
        session_probe_request: None,
    }
}

#[must_use]
pub fn build_head_direct_playback_target(
    input: &CoreDirectPlaybackTargetInput,
) -> CorePlaybackTarget {
    CorePlaybackTarget {
        request: streaming_request(
            PLAYBACK_DIRECT_STREAM_HEAD_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream",
            Vec::new(),
            Some("HEAD"),
        ),
        session_probe_request: None,
    }
}

#[must_use]
pub fn build_remux_playback_target(input: &CoreRemuxPlaybackTargetInput) -> CorePlaybackTarget {
    let query = remux_query(&input.capabilities, input.output_container);
    CorePlaybackTarget {
        request: streaming_request(
            PLAYBACK_REMUX_STREAM_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/remux",
            query.clone(),
            None,
        ),
        session_probe_request: Some(streaming_request(
            PLAYBACK_REMUX_SESSION_PROBE_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/remux",
            query,
            Some("HEAD"),
        )),
    }
}

#[must_use]
pub fn build_hls_playlist_target(input: &CoreHlsPlaylistTargetInput) -> CorePlaybackTarget {
    let query = playback_capability_query(&input.capabilities);
    CorePlaybackTarget {
        request: streaming_request(
            PLAYBACK_HLS_PLAYLIST_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/hls/playlist.m3u8",
            query.clone(),
            None,
        ),
        session_probe_request: Some(streaming_request(
            PLAYBACK_HLS_PLAYLIST_REQUEST_ID,
            &input.base_url,
            &input.source_id,
            "/stream/hls/playlist.m3u8",
            query,
            None,
        )),
    }
}

#[must_use]
pub fn build_hls_segment_request(input: &CorePlaybackSegmentInput) -> CoreHttpRequest {
    build_core_request(&CoreHttpRequestSpec::new(
        PLAYBACK_HLS_SEGMENT_REQUEST_ID,
        &input.base_url,
        "GET",
        &format!(
            "/playback/sessions/{}/hls/segments/{}",
            encode_path_segment(&input.session_id),
            encode_path_segment(&input.segment_name)
        ),
    ))
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

fn streaming_request(
    request_id: &str,
    base_url: &str,
    source_id: &str,
    suffix: &str,
    query: Vec<CoreQueryParam>,
    method: Option<&str>,
) -> CoreHttpRequest {
    build_core_request(
        &CoreHttpRequestSpec::new(
            request_id,
            base_url,
            method.unwrap_or("GET"),
            &format!("/sources/{}{}", encode_path_segment(source_id), suffix),
        )
        .query(query),
    )
}

fn playback_capability_query(capabilities: &CorePlaybackCapabilities) -> Vec<CoreQueryParam> {
    let mut query = Vec::new();
    if let Some(direct_play) = capabilities.direct_play {
        query.push(CoreQueryParam::new(
            "direct_play",
            if direct_play { "true" } else { "false" },
        ));
    }
    if !capabilities.containers.is_empty() {
        query.push(CoreQueryParam::new(
            "container",
            capabilities.containers.join(","),
        ));
    }
    if !capabilities.video_codecs.is_empty() {
        query.push(CoreQueryParam::new(
            "video_codec",
            capabilities.video_codecs.join(","),
        ));
    }
    if !capabilities.audio_codecs.is_empty() {
        query.push(CoreQueryParam::new(
            "audio_codec",
            capabilities.audio_codecs.join(","),
        ));
    }
    query
}

fn remux_query(
    capabilities: &CorePlaybackCapabilities,
    output_container: Option<CoreOutputContainer>,
) -> Vec<CoreQueryParam> {
    let mut query = playback_capability_query(capabilities);
    if let Some(value) = output_container.and_then(output_container_wire_value) {
        query.push(CoreQueryParam::new("output_container", value));
    }
    query
}

fn remux_output_container(decision: &CorePlaybackDecisionSummary) -> Option<CoreOutputContainer> {
    match decision.transcode_output_container {
        Some(CoreOutputContainer::Mkv) => Some(CoreOutputContainer::Mkv),
        Some(CoreOutputContainer::Mp4) | None => Some(CoreOutputContainer::Mp4),
        Some(CoreOutputContainer::Hls | CoreOutputContainer::Unknown) => None,
    }
}

fn output_container_wire_value(value: CoreOutputContainer) -> Option<&'static str> {
    match value {
        CoreOutputContainer::Hls | CoreOutputContainer::Unknown => None,
        CoreOutputContainer::Mp4 => Some("mp4"),
        CoreOutputContainer::Mkv => Some("mkv"),
    }
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
    fn playback_decision_request_uses_core_route_query_auth_and_redaction() {
        let request = build_playback_decision_request(&CorePlaybackDecisionRequestInput {
            base_url: "https://taru.example/api".to_owned(),
            access_token: "secret-token".to_owned(),
            source_id: "source 1".to_owned(),
            capabilities: CorePlaybackCapabilities {
                direct_play: Some(true),
                containers: vec!["mp4".to_owned(), "webm".to_owned()],
                video_codecs: vec!["h264".to_owned()],
                audio_codecs: vec!["aac".to_owned(), "opus".to_owned()],
            },
        });

        assert_eq!(request.request_id, PLAYBACK_DECISION_REQUEST_ID);
        assert_eq!(
            request.url,
            "https://taru.example/api/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm&video_codec=h264&audio_codec=aac%2Copus"
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
    fn playback_targets_follow_mode_without_auth_or_media3_policy() {
        let input = CorePlaybackTargetInput {
            base_url: "https://taru.example/api".to_owned(),
            decision: CorePlaybackDecisionSummary {
                source_id: "source 1".to_owned(),
                mode: CorePlaybackMode::Remux,
                transcode_output_container: Some(CoreOutputContainer::Mkv),
            },
            capabilities: CorePlaybackCapabilities {
                direct_play: Some(false),
                containers: vec!["mp4".to_owned(), "mkv".to_owned()],
                video_codecs: vec!["h264".to_owned()],
                audio_codecs: vec!["aac".to_owned()],
            },
        };

        let target = build_recommended_playback_target(&input).unwrap();

        assert_eq!(target.request.request_id, PLAYBACK_REMUX_STREAM_REQUEST_ID);
        assert_eq!(
            target.request.url,
            "https://taru.example/api/sources/source%201/stream/remux?direct_play=false&container=mp4%2Cmkv&video_codec=h264&audio_codec=aac&output_container=mkv"
        );
        assert!(target.request.headers.is_empty());
        let preflight = target.session_probe_request.unwrap();
        assert_eq!(
            preflight.request_id,
            PLAYBACK_REMUX_SESSION_PROBE_REQUEST_ID
        );
        assert_eq!(preflight.method, "HEAD");
        assert_eq!(preflight.url, target.request.url);

        let explicit_hls_remux = build_remux_playback_target(&CoreRemuxPlaybackTargetInput {
            base_url: "https://taru.example/api".to_owned(),
            source_id: "source 1".to_owned(),
            capabilities: CorePlaybackCapabilities::empty(),
            output_container: Some(CoreOutputContainer::Hls),
        });
        assert_eq!(
            explicit_hls_remux.request.url,
            "https://taru.example/api/sources/source%201/stream/remux"
        );

        let explicit_direct = build_direct_playback_target(&CoreDirectPlaybackTargetInput {
            base_url: "https://taru.example/api".to_owned(),
            source_id: "source 1".to_owned(),
        });
        assert_eq!(
            explicit_direct.request.url,
            "https://taru.example/api/sources/source%201/stream"
        );

        let unknown = build_recommended_playback_target(&CorePlaybackTargetInput {
            decision: CorePlaybackDecisionSummary {
                mode: CorePlaybackMode::Unknown,
                ..input.decision
            },
            ..input
        });
        assert_eq!(unknown, None);
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
