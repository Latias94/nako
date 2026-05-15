use std::{collections::HashSet, fmt, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub const ADDON_PROTOCOL_VERSION: &str = "2026-05-15";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub protocol_version: String,
    pub base_url: String,
    pub description: Option<String>,
    #[serde(default)]
    pub resources: Vec<AddonResourceDeclaration>,
    pub auth: AddonAuth,
    #[serde(default)]
    pub default_timeout_ms: Option<u64>,
    #[serde(default)]
    pub default_max_attempts: Option<u32>,
    #[serde(default)]
    pub scopes: Vec<AddonScope>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceDeclaration {
    pub kind: AddonResource,
    pub path: String,
    #[serde(default)]
    pub input_schema: Option<String>,
    #[serde(default)]
    pub output_schema: Option<String>,
    #[serde(default)]
    pub required_scopes: Vec<AddonScope>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_attempts: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonResource {
    Catalog,
    Metadata,
    Image,
    Stream,
    Subtitle,
    Recommendation,
    Automation,
    Webhook,
}

impl AddonResource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Metadata => "metadata",
            Self::Image => "image",
            Self::Stream => "stream",
            Self::Subtitle => "subtitle",
            Self::Recommendation => "recommendation",
            Self::Automation => "automation",
            Self::Webhook => "webhook",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonScope {
    CatalogRead,
    ItemMetadataRead,
    ItemMetadataSuggest,
    ImageRead,
    SubtitleRead,
    StreamUrlRead,
    RecommendationWrite,
    AutomationRun,
    WebhookEventRead,
}

impl AddonScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CatalogRead => "catalog_read",
            Self::ItemMetadataRead => "item_metadata_read",
            Self::ItemMetadataSuggest => "item_metadata_suggest",
            Self::ImageRead => "image_read",
            Self::SubtitleRead => "subtitle_read",
            Self::StreamUrlRead => "stream_url_read",
            Self::RecommendationWrite => "recommendation_write",
            Self::AutomationRun => "automation_run",
            Self::WebhookEventRead => "webhook_event_read",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonAuth {
    #[default]
    None,
    Bearer,
    SharedSecret,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceRequest {
    pub protocol_version: String,
    pub addon_id: String,
    pub resource: AddonResource,
    pub request_id: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonResourceResponse {
    pub protocol_version: String,
    pub addon_id: String,
    pub resource: AddonResource,
    pub request_id: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub artifacts: Vec<AddonArtifact>,
}

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

#[async_trait]
pub trait AddonTransport: Send + Sync {
    async fn post(&self, request: AddonHttpRequest) -> AddonProtocolResult<AddonHttpResponse>;
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
    async fn post(&self, request: AddonHttpRequest) -> AddonProtocolResult<AddonHttpResponse> {
        let mut builder = self
            .client
            .post(&request.url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .body(request.body);

        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| AddonManifestError::Http {
                message: err.to_string(),
            })?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|err| AddonManifestError::Http {
                message: err.to_string(),
            })?;

        Ok(AddonHttpResponse { status, body })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonArtifact {
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddonManifestError {
    UnsupportedProtocolVersion {
        actual: String,
    },
    EmptyField {
        field: &'static str,
    },
    InvalidBaseUrl,
    InvalidResourcePath {
        path: String,
    },
    DuplicateResource {
        resource: AddonResource,
    },
    EmptyResources,
    MissingDeclaredScope {
        resource: AddonResource,
        scope: AddonScope,
    },
    InvalidTimeout {
        value: u64,
    },
    InvalidMaxAttempts {
        value: u32,
    },
    MissingAuthToken {
        auth: AddonAuth,
    },
    ResourceNotDeclared {
        resource: AddonResource,
    },
    HttpStatus {
        status: u16,
    },
    Http {
        message: String,
    },
    InvalidEnvelope {
        message: String,
    },
}

impl fmt::Display for AddonManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProtocolVersion { actual } => {
                write!(formatter, "unsupported addon protocol version: {actual}")
            }
            Self::EmptyField { field } => {
                write!(formatter, "addon manifest field is empty: {field}")
            }
            Self::InvalidBaseUrl => write!(formatter, "addon base_url must use http or https"),
            Self::InvalidResourcePath { path } => {
                write!(formatter, "addon resource path must be absolute: {path}")
            }
            Self::DuplicateResource { resource } => {
                write!(formatter, "duplicate addon resource: {}", resource.as_str())
            }
            Self::EmptyResources => write!(formatter, "addon manifest must declare resources"),
            Self::MissingDeclaredScope { resource, scope } => write!(
                formatter,
                "addon resource {} requires undeclared scope {}",
                resource.as_str(),
                scope.as_str()
            ),
            Self::InvalidTimeout { value } => {
                write!(
                    formatter,
                    "addon timeout_ms is outside allowed range: {value}"
                )
            }
            Self::InvalidMaxAttempts { value } => {
                write!(
                    formatter,
                    "addon max_attempts is outside allowed range: {value}"
                )
            }
            Self::MissingAuthToken { auth } => {
                write!(
                    formatter,
                    "addon auth token is required for {auth:?} authentication"
                )
            }
            Self::ResourceNotDeclared { resource } => {
                write!(
                    formatter,
                    "addon resource is not declared: {}",
                    resource.as_str()
                )
            }
            Self::HttpStatus { status } => write!(formatter, "addon returned HTTP {status}"),
            Self::Http { message } => write!(formatter, "addon HTTP call failed: {message}"),
            Self::InvalidEnvelope { message } => {
                write!(formatter, "invalid addon envelope: {message}")
            }
        }
    }
}

impl std::error::Error for AddonManifestError {}

pub type AddonProtocolResult<T> = std::result::Result<T, AddonManifestError>;

pub fn validate_manifest(manifest: &AddonManifest) -> AddonProtocolResult<()> {
    validate_non_empty(&manifest.id, "id")?;
    validate_non_empty(&manifest.name, "name")?;
    validate_non_empty(&manifest.version, "version")?;
    if manifest.protocol_version != ADDON_PROTOCOL_VERSION {
        return Err(AddonManifestError::UnsupportedProtocolVersion {
            actual: manifest.protocol_version.clone(),
        });
    }
    let Ok(base_url) = reqwest::Url::parse(&manifest.base_url) else {
        return Err(AddonManifestError::InvalidBaseUrl);
    };
    if !matches!(base_url.scheme(), "http" | "https") {
        return Err(AddonManifestError::InvalidBaseUrl);
    }
    if manifest.resources.is_empty() {
        return Err(AddonManifestError::EmptyResources);
    }
    if let Some(timeout) = manifest.default_timeout_ms {
        validate_timeout(timeout)?;
    }
    if let Some(max_attempts) = manifest.default_max_attempts {
        validate_max_attempts(max_attempts)?;
    }

    let declared_scopes = manifest.scopes.iter().copied().collect::<HashSet<_>>();
    let mut declared_resources = HashSet::new();
    for resource in &manifest.resources {
        if !declared_resources.insert(resource.kind) {
            return Err(AddonManifestError::DuplicateResource {
                resource: resource.kind,
            });
        }
        if !resource.path.starts_with('/') {
            return Err(AddonManifestError::InvalidResourcePath {
                path: resource.path.clone(),
            });
        }
        if let Some(timeout) = resource.timeout_ms {
            validate_timeout(timeout)?;
        }
        if let Some(max_attempts) = resource.max_attempts {
            validate_max_attempts(max_attempts)?;
        }
        for scope in &resource.required_scopes {
            if !declared_scopes.contains(scope) {
                return Err(AddonManifestError::MissingDeclaredScope {
                    resource: resource.kind,
                    scope: *scope,
                });
            }
        }
    }

    Ok(())
}

pub fn ensure_scope_grant(
    manifest: &AddonManifest,
    resource: AddonResource,
    granted_scopes: &[AddonScope],
) -> AddonProtocolResult<()> {
    validate_manifest(manifest)?;
    let granted = granted_scopes.iter().copied().collect::<HashSet<_>>();
    let declaration = manifest
        .resources
        .iter()
        .find(|candidate| candidate.kind == resource)
        .ok_or(AddonManifestError::ResourceNotDeclared { resource })?;

    for scope in &declaration.required_scopes {
        if !granted.contains(scope) {
            return Err(AddonManifestError::MissingDeclaredScope {
                resource,
                scope: *scope,
            });
        }
    }

    Ok(())
}

pub async fn call_addon_resource<T>(
    transport: &T,
    manifest: &AddonManifest,
    resource: AddonResource,
    granted_scopes: &[AddonScope],
    request_id: impl Into<String>,
    payload: serde_json::Value,
    bearer_token: Option<&str>,
) -> AddonProtocolResult<AddonResourceResponse>
where
    T: AddonTransport,
{
    validate_manifest(manifest)?;
    ensure_scope_grant(manifest, resource, granted_scopes)?;
    let declaration = manifest
        .resources
        .iter()
        .find(|candidate| candidate.kind == resource)
        .ok_or(AddonManifestError::ResourceNotDeclared { resource })?;
    let request_id = request_id.into();
    let timeout_ms = declaration
        .timeout_ms
        .or(manifest.default_timeout_ms)
        .unwrap_or(10_000);
    validate_timeout(timeout_ms)?;
    let max_attempts = declaration
        .max_attempts
        .or(manifest.default_max_attempts)
        .unwrap_or(1);
    validate_max_attempts(max_attempts)?;
    let envelope = AddonResourceRequest {
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        addon_id: manifest.id.clone(),
        resource,
        request_id: request_id.clone(),
        payload,
    };
    let body =
        serde_json::to_string(&envelope).map_err(|err| AddonManifestError::InvalidEnvelope {
            message: format!("failed to serialize addon request: {err}"),
        })?;
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
            let token = bearer_token.ok_or(AddonManifestError::MissingAuthToken {
                auth: AddonAuth::Bearer,
            })?;
            headers.push(("authorization".to_owned(), format!("Bearer {token}")));
        }
        AddonAuth::SharedSecret => {
            let token = bearer_token.ok_or(AddonManifestError::MissingAuthToken {
                auth: AddonAuth::SharedSecret,
            })?;
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
                last_error = Some(err);
                continue;
            }
            Err(err) => return Err(err),
        };

        if !(200..300).contains(&response.status) {
            let err = AddonManifestError::HttpStatus {
                status: response.status,
            };
            if attempt < max_attempts && err.is_retryable() {
                last_error = Some(err);
                continue;
            }
            return Err(err);
        }

        let envelope =
            serde_json::from_str::<AddonResourceResponse>(&response.body).map_err(|err| {
                AddonManifestError::InvalidEnvelope {
                    message: format!("failed to parse addon response: {err}"),
                }
            })?;
        validate_resource_response(&envelope, manifest, resource, &request_id)?;

        return Ok(envelope);
    }

    Err(
        last_error.unwrap_or(AddonManifestError::InvalidMaxAttempts {
            value: max_attempts,
        }),
    )
}

pub fn validate_resource_response(
    response: &AddonResourceResponse,
    manifest: &AddonManifest,
    resource: AddonResource,
    request_id: &str,
) -> AddonProtocolResult<()> {
    if response.protocol_version != ADDON_PROTOCOL_VERSION {
        return Err(AddonManifestError::UnsupportedProtocolVersion {
            actual: response.protocol_version.clone(),
        });
    }
    if response.addon_id != manifest.id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "response addon_id {} did not match {}",
                response.addon_id, manifest.id
            ),
        });
    }
    if response.resource != resource {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "response resource {} did not match {}",
                response.resource.as_str(),
                resource.as_str()
            ),
        });
    }
    if response.request_id != request_id {
        return Err(AddonManifestError::InvalidEnvelope {
            message: format!(
                "response request_id {} did not match {request_id}",
                response.request_id
            ),
        });
    }

    Ok(())
}

fn resource_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

fn validate_non_empty(value: &str, field: &'static str) -> AddonProtocolResult<()> {
    if value.trim().is_empty() {
        Err(AddonManifestError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn validate_timeout(value: u64) -> AddonProtocolResult<()> {
    if (100..=120_000).contains(&value) {
        Ok(())
    } else {
        Err(AddonManifestError::InvalidTimeout { value })
    }
}

fn validate_max_attempts(value: u32) -> AddonProtocolResult<()> {
    if (1..=10).contains(&value) {
        Ok(())
    } else {
        Err(AddonManifestError::InvalidMaxAttempts { value })
    }
}

impl AddonManifestError {
    #[must_use]
    fn is_retryable(&self) -> bool {
        match self {
            Self::Http { .. } => true,
            Self::HttpStatus { status } => {
                *status == 408 || *status == 429 || (500..600).contains(status)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use super::*;

    #[test]
    fn validates_manifest_resource_and_scope_contract() {
        let manifest = valid_manifest();

        validate_manifest(&manifest).unwrap();
        ensure_scope_grant(
            &manifest,
            AddonResource::Metadata,
            &[
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
        )
        .unwrap();
    }

    #[test]
    fn rejects_invalid_manifest_shape() {
        let mut manifest = valid_manifest();
        manifest.protocol_version = "2020-01-01".to_owned();
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::UnsupportedProtocolVersion { .. })
        ));

        let mut manifest = valid_manifest();
        manifest.resources[0].path = "metadata".to_owned();
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::InvalidResourcePath { .. })
        ));

        let mut manifest = valid_manifest();
        manifest.resources.push(manifest.resources[0].clone());
        assert!(matches!(
            validate_manifest(&manifest),
            Err(AddonManifestError::DuplicateResource { .. })
        ));
    }

    #[test]
    fn denies_missing_scope_grants() {
        let manifest = valid_manifest();

        assert!(matches!(
            ensure_scope_grant(
                &manifest,
                AddonResource::Metadata,
                &[AddonScope::ItemMetadataRead]
            ),
            Err(AddonManifestError::MissingDeclaredScope { .. })
        ));
    }

    #[test]
    fn resource_envelopes_round_trip() {
        let request = AddonResourceRequest {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: "example".to_owned(),
            resource: AddonResource::Metadata,
            request_id: "request-1".to_owned(),
            payload: serde_json::json!({"item_id":"018f0000-0000-7000-8000-000000000001"}),
        };
        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            serde_json::from_str::<AddonResourceRequest>(&json).unwrap(),
            request
        );
    }

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
        transport.push_response(Err(AddonManifestError::Http {
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

        assert_eq!(err, AddonManifestError::HttpStatus { status: 400 });
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

        assert!(matches!(err, AddonManifestError::InvalidEnvelope { .. }));
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
            AddonManifestError::MissingAuthToken {
                auth: AddonAuth::Bearer
            }
        );
        assert!(transport.requests().is_empty());
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
        responses: Arc<Mutex<VecDeque<AddonProtocolResult<AddonHttpResponse>>>>,
        requests: Arc<Mutex<Vec<AddonHttpRequest>>>,
    }

    impl MockTransport {
        fn with_response(response: AddonProtocolResult<AddonHttpResponse>) -> Self {
            let transport = Self::default();
            transport.push_response(response);
            transport
        }

        fn push_response(&self, response: AddonProtocolResult<AddonHttpResponse>) {
            self.responses.lock().unwrap().push_back(response);
        }

        fn requests(&self) -> Vec<AddonHttpRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl AddonTransport for MockTransport {
        async fn post(&self, request: AddonHttpRequest) -> AddonProtocolResult<AddonHttpResponse> {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(AddonManifestError::Http {
                        message: "mock transport response queue was empty".to_owned(),
                    })
                })
        }
    }
}
