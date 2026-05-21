use std::time::Duration;

use async_trait::async_trait;
use taru_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonHealthCheckRequest, AddonHealthCheckResponse,
    AddonManifest, AddonManifestError, AddonResource, AddonResourceRequest, AddonResourceResponse,
    AddonScope, ensure_scope_grant, validate_health_check_response, validate_manifest,
    validate_resource_response,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonHttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonHttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddonClientError {
    Protocol(AddonManifestError),
    HttpStatus { status: u16, retryable: bool },
    Http { message: String },
}

impl std::fmt::Display for AddonClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protocol(err) => write!(formatter, "{err}"),
            Self::HttpStatus { status, .. } => write!(formatter, "addon returned HTTP {status}"),
            Self::Http { message } => write!(formatter, "addon HTTP call failed: {message}"),
        }
    }
}

impl std::error::Error for AddonClientError {}

impl From<AddonManifestError> for AddonClientError {
    fn from(value: AddonManifestError) -> Self {
        Self::Protocol(value)
    }
}

pub type AddonClientResult<T> = std::result::Result<T, AddonClientError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonResourceCallOutcome {
    pub response: AddonResourceResponse,
    pub http_status: u16,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonResourceCallFailure {
    pub error: AddonClientError,
    pub attempts: u32,
}

#[async_trait]
pub trait AddonTransport: Send + Sync {
    async fn post(&self, request: AddonHttpRequest) -> AddonClientResult<AddonHttpResponse>;
}

#[derive(Clone, Debug)]
pub struct ReqwestAddonTransport {
    client: reqwest::Client,
}

impl Default for ReqwestAddonTransport {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl ReqwestAddonTransport {
    #[must_use]
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AddonTransport for ReqwestAddonTransport {
    async fn post(&self, request: AddonHttpRequest) -> AddonClientResult<AddonHttpResponse> {
        let mut builder = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .body(request.body);

        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }

        let response = builder.send().await.map_err(|err| AddonClientError::Http {
            message: err.to_string(),
        })?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|err| AddonClientError::Http {
                message: err.to_string(),
            })?;

        Ok(AddonHttpResponse { status, body })
    }
}

pub async fn call_addon_resource<T>(
    transport: &T,
    manifest: &AddonManifest,
    resource: AddonResource,
    granted_scopes: &[AddonScope],
    request_id: impl Into<String>,
    payload: serde_json::Value,
    bearer_token: Option<&str>,
) -> AddonClientResult<AddonResourceResponse>
where
    T: AddonTransport,
{
    call_addon_resource_with_outcome(
        transport,
        manifest,
        resource,
        granted_scopes,
        request_id,
        payload,
        bearer_token,
    )
    .await
    .map(|outcome| outcome.response)
    .map_err(|failure| failure.error)
}

pub async fn call_addon_resource_with_outcome<T>(
    transport: &T,
    manifest: &AddonManifest,
    resource: AddonResource,
    granted_scopes: &[AddonScope],
    request_id: impl Into<String>,
    payload: serde_json::Value,
    bearer_token: Option<&str>,
) -> Result<AddonResourceCallOutcome, AddonResourceCallFailure>
where
    T: AddonTransport,
{
    validate_manifest(manifest).map_err(resource_call_setup_failure)?;
    ensure_scope_grant(manifest, resource, granted_scopes).map_err(resource_call_setup_failure)?;
    let declaration = manifest
        .resources
        .iter()
        .find(|candidate| candidate.kind == resource)
        .ok_or(AddonManifestError::ResourceNotDeclared { resource })
        .map_err(resource_call_setup_failure)?;
    let request_id = request_id.into();
    let timeout_ms = declaration
        .timeout_ms
        .or(manifest.default_timeout_ms)
        .unwrap_or(10_000);
    let max_attempts = declaration
        .max_attempts
        .or(manifest.default_max_attempts)
        .unwrap_or(1);
    let envelope = AddonResourceRequest {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        addon_id: manifest.id.clone(),
        resource,
        request_id: request_id.clone(),
        payload,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|err| AddonManifestError::InvalidEnvelope {
            message: format!("failed to serialize addon request: {err}"),
        })
        .map_err(resource_call_setup_failure)?;
    let mut headers = vec![
        ("content-type".to_owned(), "application/json".to_owned()),
        (
            "x-taru-addon-protocol-version".to_owned(),
            ADDON_PROTOCOL_VERSION.to_owned(),
        ),
        ("x-taru-addon-id".to_owned(), manifest.id.clone()),
        (
            "x-taru-addon-resource".to_owned(),
            resource.as_str().to_owned(),
        ),
        ("x-taru-request-id".to_owned(), request_id.clone()),
    ];
    match manifest.auth {
        AddonAuth::None => {}
        AddonAuth::Bearer => {
            let token = bearer_token
                .ok_or(AddonManifestError::MissingAuthToken {
                    auth: AddonAuth::Bearer,
                })
                .map_err(resource_call_setup_failure)?;
            headers.push(("authorization".to_owned(), format!("Bearer {token}")));
        }
        AddonAuth::SharedSecret => {
            let token = bearer_token
                .ok_or(AddonManifestError::MissingAuthToken {
                    auth: AddonAuth::SharedSecret,
                })
                .map_err(resource_call_setup_failure)?;
            headers.push(("x-taru-addon-secret".to_owned(), token.to_owned()));
        }
    }

    let mut last_error = None;
    for attempt in 1..=max_attempts {
        let mut attempt_headers = headers.clone();
        attempt_headers.push(("x-taru-attempt".to_owned(), attempt.to_string()));
        let response = transport
            .post(AddonHttpRequest {
                url: resource_url(&manifest.base_url, &declaration.path),
                headers: attempt_headers,
                body: body.clone(),
                timeout_ms,
            })
            .await;

        let response = match response {
            Ok(response) => response,
            Err(err) if attempt < max_attempts && err.is_retryable() => {
                last_error = Some(AddonResourceCallFailure {
                    error: err,
                    attempts: attempt,
                });
                continue;
            }
            Err(err) => {
                return Err(AddonResourceCallFailure {
                    error: err,
                    attempts: attempt,
                });
            }
        };

        if !(200..300).contains(&response.status) {
            let failure = AddonResourceCallFailure {
                error: AddonClientError::HttpStatus {
                    status: response.status,
                    retryable: is_retryable_http_status(response.status),
                },
                attempts: attempt,
            };
            if attempt < max_attempts && failure.error.is_retryable() {
                last_error = Some(failure);
                continue;
            }
            return Err(failure);
        }

        let envelope = serde_json::from_str::<AddonResourceResponse>(&response.body)
            .map_err(|err| AddonManifestError::InvalidEnvelope {
                message: format!("failed to parse addon response: {err}"),
            })
            .map_err(|error| AddonResourceCallFailure {
                error: error.into(),
                attempts: attempt,
            })?;
        validate_resource_response(&envelope, manifest, resource, &request_id).map_err(
            |error| AddonResourceCallFailure {
                error: error.into(),
                attempts: attempt,
            },
        )?;

        return Ok(AddonResourceCallOutcome {
            response: envelope,
            http_status: response.status,
            attempts: attempt,
        });
    }

    Err(last_error.unwrap_or_else(|| AddonResourceCallFailure {
        error: AddonManifestError::InvalidMaxAttempts {
            value: max_attempts,
        }
        .into(),
        attempts: 0,
    }))
}

pub async fn check_addon_health<T>(
    transport: &T,
    manifest: &AddonManifest,
    request_id: impl Into<String>,
) -> AddonClientResult<AddonHealthCheckResponse>
where
    T: AddonTransport,
{
    validate_manifest(manifest)?;
    let request_id = request_id.into();
    let timeout_ms = manifest.default_timeout_ms.unwrap_or(10_000);
    let envelope = AddonHealthCheckRequest {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        manifest_id: manifest.id.clone(),
        request_id: request_id.clone(),
        expected_addon_version: manifest.version.clone(),
        expected_resource_count: manifest.resources.len(),
    };
    let body =
        serde_json::to_string(&envelope).map_err(|err| AddonManifestError::InvalidEnvelope {
            message: format!("failed to serialize addon health request: {err}"),
        })?;
    let response = transport
        .post(AddonHttpRequest {
            url: resource_url(&manifest.base_url, "/health"),
            headers: vec![
                ("content-type".to_owned(), "application/json".to_owned()),
                (
                    "x-taru-addon-protocol-version".to_owned(),
                    ADDON_PROTOCOL_VERSION.to_owned(),
                ),
                ("x-taru-addon-id".to_owned(), manifest.id.clone()),
                (
                    "x-taru-addon-operation".to_owned(),
                    "health-check".to_owned(),
                ),
                ("x-taru-request-id".to_owned(), request_id),
            ],
            body,
            timeout_ms,
        })
        .await?;

    if !(200..300).contains(&response.status) {
        return Err(AddonClientError::HttpStatus {
            status: response.status,
            retryable: is_retryable_http_status(response.status),
        });
    }

    let envelope =
        serde_json::from_str::<AddonHealthCheckResponse>(&response.body).map_err(|err| {
            AddonManifestError::InvalidEnvelope {
                message: format!("failed to parse addon health response: {err}"),
            }
        })?;
    validate_health_check_response(&envelope, manifest)?;

    Ok(envelope)
}

fn resource_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

impl AddonClientError {
    #[must_use]
    fn is_retryable(&self) -> bool {
        match self {
            Self::Http { .. } => true,
            Self::HttpStatus { retryable, .. } => *retryable,
            Self::Protocol(_) => false,
        }
    }
}

fn is_retryable_http_status(status: u16) -> bool {
    status == 408 || status == 429 || (500..600).contains(&status)
}

fn resource_call_setup_failure(error: impl Into<AddonClientError>) -> AddonResourceCallFailure {
    AddonResourceCallFailure {
        error: error.into(),
        attempts: 0,
    }
}

impl AddonClientError {
    #[must_use]
    pub const fn http_status(&self) -> Option<u16> {
        match self {
            Self::HttpStatus { status, .. } => Some(*status),
            Self::Protocol(_) | Self::Http { .. } => None,
        }
    }

    #[must_use]
    pub const fn was_retryable_http_status(&self) -> bool {
        match self {
            Self::HttpStatus { retryable, .. } => *retryable,
            Self::Protocol(_) | Self::Http { .. } => false,
        }
    }
}

impl AddonClientError {
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Protocol(_) => "protocol",
            Self::HttpStatus { .. } => "http_status",
            Self::Http { .. } => "http",
        }
    }
}

#[cfg(test)]
fn assert_error_shape(err: &AddonClientError) {
    match err {
        AddonClientError::HttpStatus { status, retryable } => {
            assert_eq!(*retryable, is_retryable_http_status(*status));
        }
        AddonClientError::Protocol(_) | AddonClientError::Http { .. } => {}
    }
}

#[cfg(test)]
mod client_error_tests {
    use super::*;

    #[test]
    fn http_status_error_records_retryability() {
        assert_error_shape(&AddonClientError::HttpStatus {
            status: 500,
            retryable: true,
        });
        assert_error_shape(&AddonClientError::HttpStatus {
            status: 400,
            retryable: false,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use taru_addon_protocol::{ADDON_PROTOCOL_VERSION, AddonArtifact, AddonResourceDeclaration};

    use super::*;

    #[tokio::test]
    async fn calls_resource_with_bearer_auth_and_validates_response() {
        let manifest = valid_manifest();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: response_json(&manifest, "request-1"),
        }));

        let response = call_addon_resource(
            &transport,
            &manifest,
            AddonResource::Metadata,
            &[
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            "request-1",
            serde_json::json!({"item_id":"item-1"}),
            Some("token-1"),
        )
        .await
        .unwrap();

        assert_eq!(response.payload["title"], "The Matrix");
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://example.test/addon/metadata".to_owned()
        );
        assert_eq!(
            header_value(&requests[0], "authorization"),
            Some("Bearer token-1")
        );
        assert_eq!(header_value(&requests[0], "x-taru-attempt"), Some("1"));
        assert_eq!(requests[0].timeout_ms, 5_000);
        assert!(requests[0].body.contains("\"request_id\":\"request-1\""));
    }

    #[tokio::test]
    async fn retries_retryable_errors_with_the_same_request_id() {
        let manifest = valid_manifest();
        let transport = MockTransport::default();
        transport.push_response(Err(AddonClientError::Http {
            message: "temporary network failure".to_owned(),
        }));
        transport.push_response(Ok(AddonHttpResponse {
            status: 200,
            body: response_json(&manifest, "request-2"),
        }));

        let response = call_addon_resource(
            &transport,
            &manifest,
            AddonResource::Metadata,
            &[
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            "request-2",
            serde_json::json!({"item_id":"item-1"}),
            Some("token-1"),
        )
        .await
        .unwrap();

        assert_eq!(response.request_id, "request-2");
        let requests = transport.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].body, requests[1].body);
        assert_eq!(header_value(&requests[0], "x-taru-attempt"), Some("1"));
        assert_eq!(header_value(&requests[1], "x-taru-attempt"), Some("2"));
    }

    #[tokio::test]
    async fn does_not_retry_non_retryable_http_status() {
        let manifest = valid_manifest();
        let transport = MockTransport::default();
        transport.push_response(Ok(AddonHttpResponse {
            status: 400,
            body: "{}".to_owned(),
        }));
        transport.push_response(Ok(AddonHttpResponse {
            status: 200,
            body: response_json(&manifest, "request-3"),
        }));

        let err = call_addon_resource(
            &transport,
            &manifest,
            AddonResource::Metadata,
            &[
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            "request-3",
            serde_json::json!({"item_id":"item-1"}),
            Some("token-1"),
        )
        .await
        .unwrap_err();

        assert_eq!(
            err,
            AddonClientError::HttpStatus {
                status: 400,
                retryable: false
            }
        );
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn rejects_invalid_response_mapping() {
        let manifest = valid_manifest();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: response_json(&manifest, "different-request"),
        }));

        let err = call_addon_resource(
            &transport,
            &manifest,
            AddonResource::Metadata,
            &[
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            "request-4",
            serde_json::json!({"item_id":"item-1"}),
            Some("token-1"),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            AddonClientError::Protocol(AddonManifestError::InvalidEnvelope { .. })
        ));
    }

    #[tokio::test]
    async fn requires_auth_token_for_authenticated_addons() {
        let manifest = valid_manifest();
        let transport = MockTransport::default();

        let err = call_addon_resource(
            &transport,
            &manifest,
            AddonResource::Metadata,
            &[
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            "request-5",
            serde_json::json!({"item_id":"item-1"}),
            None,
        )
        .await
        .unwrap_err();

        assert_eq!(
            err,
            AddonClientError::Protocol(AddonManifestError::MissingAuthToken {
                auth: AddonAuth::Bearer
            })
        );
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn checks_health_without_auth_or_resource_payload() {
        let manifest = valid_manifest();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: serde_json::to_string(&AddonHealthCheckResponse {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                manifest_id: manifest.id.clone(),
                status: taru_addon_protocol::AddonHealthStatus::Ok,
                checked_at: "2026-05-21T12:00:00.000Z".to_owned(),
                manifest: taru_addon_protocol::AddonHealthManifestFacts {
                    addon_version: manifest.version.clone(),
                    resource_count: manifest.resources.len(),
                },
                diagnostics: serde_json::json!({"safe_note": "ok"}),
            })
            .unwrap(),
        }));

        let response = check_addon_health(&transport, &manifest, "health-1")
            .await
            .unwrap();

        assert_eq!(response.manifest_id, manifest.id);
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://example.test/addon/health".to_owned()
        );
        assert_eq!(
            header_value(&requests[0], "x-taru-addon-operation"),
            Some("health-check")
        );
        assert_eq!(header_value(&requests[0], "authorization"), None);
        assert_eq!(header_value(&requests[0], "x-taru-addon-secret"), None);
        assert!(requests[0].body.contains("\"manifest_id\":\"example\""));
        assert!(!requests[0].body.contains("\"payload\""));
    }

    fn valid_manifest() -> AddonManifest {
        AddonManifest {
            id: "example".to_owned(),
            name: "Example".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            description: None,
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::Metadata,
                path: "/metadata".to_owned(),
                input_schema: Some("taru.metadata.request.v1".to_owned()),
                output_schema: Some("taru.metadata.response.v1".to_owned()),
                required_scopes: vec![
                    AddonScope::ItemMetadataRead,
                    AddonScope::ItemMetadataSuggest,
                ],
                timeout_ms: Some(5_000),
                max_attempts: Some(2),
            }],
            entry_points: Vec::new(),
            hosted_pages: Vec::new(),
            configuration_schema: None,
            secret_reference_fields: Vec::new(),
            event_subscriptions: Vec::new(),
            tasks: Vec::new(),
            auth: AddonAuth::Bearer,
            default_timeout_ms: Some(10_000),
            default_max_attempts: Some(2),
            scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
        }
    }

    fn response_json(manifest: &AddonManifest, request_id: &str) -> String {
        serde_json::to_string(&AddonResourceResponse {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: manifest.id.clone(),
            resource: AddonResource::Metadata,
            request_id: request_id.to_owned(),
            payload: serde_json::json!({"title":"The Matrix"}),
            artifacts: vec![AddonArtifact {
                kind: "metadata_suggestion".to_owned(),
                payload: serde_json::json!({"title":"The Matrix"}),
            }],
        })
        .unwrap()
    }

    fn header_value<'a>(request: &'a AddonHttpRequest, name: &str) -> Option<&'a str> {
        request
            .headers
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, value)| value.as_str())
    }

    #[derive(Clone, Default)]
    struct MockTransport {
        responses: Arc<Mutex<VecDeque<AddonClientResult<AddonHttpResponse>>>>,
        requests: Arc<Mutex<Vec<AddonHttpRequest>>>,
    }

    impl MockTransport {
        fn with_response(response: AddonClientResult<AddonHttpResponse>) -> Self {
            let transport = Self::default();
            transport.push_response(response);
            transport
        }

        fn push_response(&self, response: AddonClientResult<AddonHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<AddonHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl AddonTransport for MockTransport {
        async fn post(&self, request: AddonHttpRequest) -> AddonClientResult<AddonHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(AddonClientError::Http {
                        message: "mock transport response queue was empty".to_owned(),
                    })
                })
        }
    }
}
