use std::time::Duration;

use async_trait::async_trait;
use nako_addon_protocol::{
    ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA, ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA,
    ADDON_RUNTIME_ACCESS_CHECK_PATH, ADDON_RUNTIME_SIDE_EFFECTS_PATH, AddonAccessCheckRequest,
    AddonAccessCheckResponse, AddonAuth, AddonEventRequest, AddonEventResponse,
    AddonHealthCheckRequest, AddonHealthCheckResponse, AddonManifest, AddonManifestError,
    AddonPermission, AddonResource, AddonResourceRequest, AddonResourceResponse,
    AddonResourceSearchRequest, AddonResourceSearchResponse, AddonScope, AddonSideEffectResponse,
    AddonSideEffectTargetKind, AddonTaskRequest, AddonTaskResponse, SubmitAddonArtworkWriteRequest,
    SubmitAddonMetadataWriteRequest, SubmitAddonSideEffectRequest,
    ensure_event_subscription_scope_grant, ensure_scope_grant, ensure_task_scope_grant,
    validate_event_response, validate_health_check_response, validate_manifest,
    validate_resource_response, validate_task_response,
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
    InvalidRequest { message: String },
    InvalidResponse { message: String },
    UnsafeRequestBody,
    HttpStatus { status: u16, retryable: bool },
    Http { message: String },
}

impl std::fmt::Display for AddonClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protocol(err) => write!(formatter, "{err}"),
            Self::InvalidRequest { message } => {
                write!(formatter, "addon client invalid request: {message}")
            }
            Self::InvalidResponse { message } => {
                write!(formatter, "addon client invalid response: {message}")
            }
            Self::UnsafeRequestBody => {
                write!(
                    formatter,
                    "addon client request body contained token material"
                )
            }
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
pub struct AddonResourceSearchCallOutcome {
    pub response: AddonResourceSearchResponse,
    pub http_status: u16,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonResourceCallFailure {
    pub error: AddonClientError,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonTaskCallRequest {
    pub task_id: String,
    pub job_id: String,
    pub request_id: String,
    pub attempt: u32,
    pub retry_of_job_id: Option<String>,
    pub library_id: Option<String>,
    pub source_id: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonTaskCallOutcome {
    pub response: AddonTaskResponse,
    pub http_status: u16,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonTaskCallFailure {
    pub error: AddonClientError,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonEventCallRequest {
    pub subscription_id: String,
    pub event_id: String,
    pub event_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub occurred_at: String,
    pub attempt: u32,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonEventCallOutcome {
    pub response: AddonEventResponse,
    pub http_status: u16,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonEventCallFailure {
    pub error: AddonClientError,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NakoRuntimeClientConfig {
    pub base_url: String,
    pub addon_token: String,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct NakoRuntimeClient<T = ReqwestAddonTransport> {
    config: NakoRuntimeClientConfig,
    transport: T,
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

        let response = builder.send().await.map_err(addon_http_error)?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(addon_http_error)?;

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
    let protocol_version = manifest.protocol_version.clone();
    let envelope = AddonResourceRequest {
        protocol_version: protocol_version.clone(),
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
        ("x-nako-addon-protocol-version".to_owned(), protocol_version),
        ("x-nako-addon-id".to_owned(), manifest.id.clone()),
        (
            "x-nako-addon-resource".to_owned(),
            resource.as_str().to_owned(),
        ),
        ("x-nako-request-id".to_owned(), request_id.clone()),
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
            headers.push(("x-nako-addon-secret".to_owned(), token.to_owned()));
        }
    }

    let mut last_error = None;
    for attempt in 1..=max_attempts {
        let mut attempt_headers = headers.clone();
        attempt_headers.push(("x-nako-attempt".to_owned(), attempt.to_string()));
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

pub async fn call_addon_resource_search<T>(
    transport: &T,
    manifest: &AddonManifest,
    granted_scopes: &[AddonScope],
    request_id: impl Into<String>,
    request: AddonResourceSearchRequest,
    bearer_token: Option<&str>,
) -> AddonClientResult<AddonResourceSearchResponse>
where
    T: AddonTransport,
{
    call_addon_resource_search_with_outcome(
        transport,
        manifest,
        granted_scopes,
        request_id,
        request,
        bearer_token,
    )
    .await
    .map(|outcome| outcome.response)
    .map_err(|failure| failure.error)
}

pub async fn call_addon_resource_search_with_outcome<T>(
    transport: &T,
    manifest: &AddonManifest,
    granted_scopes: &[AddonScope],
    request_id: impl Into<String>,
    request: AddonResourceSearchRequest,
    bearer_token: Option<&str>,
) -> Result<AddonResourceSearchCallOutcome, AddonResourceCallFailure>
where
    T: AddonTransport,
{
    ensure_resource_search_scope_contract(manifest, granted_scopes)
        .map_err(resource_call_setup_failure)?;
    if request.schema != ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA {
        return Err(resource_call_setup_failure(
            AddonClientError::InvalidRequest {
                message: format!(
                    "resource_search request schema {} did not match {}",
                    request.schema, ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA
                ),
            },
        ));
    }
    let payload = serde_json::to_value(request)
        .map_err(invalid_resource_search_request_envelope)
        .map_err(resource_call_setup_failure)?;
    let outcome = call_addon_resource_with_outcome(
        transport,
        manifest,
        AddonResource::ResourceSearch,
        granted_scopes,
        request_id,
        payload,
        bearer_token,
    )
    .await?;
    let attempts = outcome.attempts;
    let response = serde_json::from_value::<AddonResourceSearchResponse>(outcome.response.payload)
        .map_err(|error| AddonResourceCallFailure {
            error: AddonClientError::InvalidResponse {
                message: format!("failed to parse resource_search response payload: {error}"),
            },
            attempts,
        })?;
    if response.schema != ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA {
        return Err(AddonResourceCallFailure {
            error: AddonClientError::InvalidResponse {
                message: format!(
                    "resource_search response schema {} did not match {}",
                    response.schema, ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA
                ),
            },
            attempts,
        });
    }

    Ok(AddonResourceSearchCallOutcome {
        response,
        http_status: outcome.http_status,
        attempts,
    })
}

pub async fn call_addon_task_with_outcome<T>(
    transport: &T,
    manifest: &AddonManifest,
    granted_scopes: &[AddonScope],
    request: AddonTaskCallRequest,
    bearer_token: Option<&str>,
) -> Result<AddonTaskCallOutcome, AddonTaskCallFailure>
where
    T: AddonTransport,
{
    validate_manifest(manifest).map_err(task_call_setup_failure)?;
    ensure_task_scope_grant(manifest, &request.task_id, granted_scopes)
        .map_err(task_call_setup_failure)?;
    let declaration = manifest
        .tasks
        .iter()
        .find(|candidate| candidate.id == request.task_id)
        .ok_or_else(|| AddonManifestError::TaskNotDeclared {
            task_id: request.task_id.clone(),
        })
        .map_err(task_call_setup_failure)?;
    let timeout_ms = declaration
        .timeout_ms
        .or(manifest.default_timeout_ms)
        .unwrap_or(10_000);
    let protocol_version = manifest.protocol_version.clone();
    let envelope = AddonTaskRequest {
        protocol_version: protocol_version.clone(),
        addon_id: manifest.id.clone(),
        task_id: request.task_id.clone(),
        job_id: request.job_id.clone(),
        request_id: request.request_id.clone(),
        attempt: request.attempt,
        retry_of_job_id: request.retry_of_job_id.clone(),
        library_id: request.library_id.clone(),
        source_id: request.source_id.clone(),
        payload: request.payload,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|err| AddonManifestError::InvalidEnvelope {
            message: format!("failed to serialize addon task request: {err}"),
        })
        .map_err(task_call_setup_failure)?;
    let mut headers = vec![
        ("content-type".to_owned(), "application/json".to_owned()),
        ("x-nako-addon-protocol-version".to_owned(), protocol_version),
        ("x-nako-addon-id".to_owned(), manifest.id.clone()),
        (
            "x-nako-addon-operation".to_owned(),
            "task-dispatch".to_owned(),
        ),
        ("x-nako-addon-task".to_owned(), request.task_id.clone()),
        ("x-nako-job-id".to_owned(), request.job_id.clone()),
        ("x-nako-request-id".to_owned(), request.request_id.clone()),
    ];
    match manifest.auth {
        AddonAuth::None => {}
        AddonAuth::Bearer => {
            let token = bearer_token
                .ok_or(AddonManifestError::MissingAuthToken {
                    auth: AddonAuth::Bearer,
                })
                .map_err(task_call_setup_failure)?;
            headers.push(("authorization".to_owned(), format!("Bearer {token}")));
        }
        AddonAuth::SharedSecret => {
            let token = bearer_token
                .ok_or(AddonManifestError::MissingAuthToken {
                    auth: AddonAuth::SharedSecret,
                })
                .map_err(task_call_setup_failure)?;
            headers.push(("x-nako-addon-secret".to_owned(), token.to_owned()));
        }
    }

    let dispatch_attempt = 1;
    {
        let mut attempt_headers = headers.clone();
        attempt_headers.push(("x-nako-attempt".to_owned(), dispatch_attempt.to_string()));
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
            Err(err) => {
                return Err(AddonTaskCallFailure {
                    error: err,
                    attempts: dispatch_attempt,
                });
            }
        };

        if !(200..300).contains(&response.status) {
            return Err(AddonTaskCallFailure {
                error: AddonClientError::HttpStatus {
                    status: response.status,
                    retryable: is_retryable_http_status(response.status),
                },
                attempts: dispatch_attempt,
            });
        }

        let envelope = serde_json::from_str::<AddonTaskResponse>(&response.body)
            .map_err(|err| AddonManifestError::InvalidEnvelope {
                message: format!("failed to parse addon task response: {err}"),
            })
            .map_err(|error| AddonTaskCallFailure {
                error: error.into(),
                attempts: dispatch_attempt,
            })?;
        validate_task_response(
            &envelope,
            manifest,
            &request.task_id,
            &request.job_id,
            &request.request_id,
        )
        .map_err(|error| AddonTaskCallFailure {
            error: error.into(),
            attempts: dispatch_attempt,
        })?;

        Ok(AddonTaskCallOutcome {
            response: envelope,
            http_status: response.status,
            attempts: dispatch_attempt,
        })
    }
}

pub async fn call_addon_event_with_outcome<T>(
    transport: &T,
    manifest: &AddonManifest,
    granted_scopes: &[AddonScope],
    request: AddonEventCallRequest,
    bearer_token: Option<&str>,
) -> Result<AddonEventCallOutcome, AddonEventCallFailure>
where
    T: AddonTransport,
{
    validate_manifest(manifest).map_err(event_call_setup_failure)?;
    ensure_event_subscription_scope_grant(manifest, &request.subscription_id, granted_scopes)
        .map_err(event_call_setup_failure)?;
    let declaration = manifest
        .event_subscriptions
        .iter()
        .find(|candidate| candidate.id == request.subscription_id)
        .ok_or_else(|| AddonManifestError::EventSubscriptionNotDeclared {
            subscription_id: request.subscription_id.clone(),
        })
        .map_err(event_call_setup_failure)?;
    if declaration.event_kind != request.event_kind {
        return Err(event_call_setup_failure(
            AddonManifestError::InvalidEnvelope {
                message: format!(
                    "event subscription {} declares {} but request used {}",
                    declaration.id, declaration.event_kind, request.event_kind
                ),
            },
        ));
    }

    let timeout_ms = manifest.default_timeout_ms.unwrap_or(10_000);
    let protocol_version = manifest.protocol_version.clone();
    let envelope = AddonEventRequest {
        protocol_version: protocol_version.clone(),
        addon_id: manifest.id.clone(),
        subscription_id: request.subscription_id.clone(),
        event_id: request.event_id.clone(),
        event_kind: request.event_kind.clone(),
        subject_kind: request.subject_kind.clone(),
        subject_id: request.subject_id.clone(),
        occurred_at: request.occurred_at.clone(),
        attempt: request.attempt,
        payload: request.payload,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|err| AddonManifestError::InvalidEnvelope {
            message: format!("failed to serialize addon event request: {err}"),
        })
        .map_err(event_call_setup_failure)?;
    let mut headers = vec![
        ("content-type".to_owned(), "application/json".to_owned()),
        ("x-nako-addon-protocol-version".to_owned(), protocol_version),
        ("x-nako-addon-id".to_owned(), manifest.id.clone()),
        (
            "x-nako-addon-operation".to_owned(),
            "event-delivery".to_owned(),
        ),
        (
            "x-nako-addon-event-subscription".to_owned(),
            request.subscription_id.clone(),
        ),
        ("x-nako-event-id".to_owned(), request.event_id.clone()),
        ("x-nako-event-kind".to_owned(), request.event_kind.clone()),
        ("x-nako-attempt".to_owned(), request.attempt.to_string()),
    ];
    match manifest.auth {
        AddonAuth::None => {}
        AddonAuth::Bearer => {
            let token = bearer_token
                .ok_or(AddonManifestError::MissingAuthToken {
                    auth: AddonAuth::Bearer,
                })
                .map_err(event_call_setup_failure)?;
            headers.push(("authorization".to_owned(), format!("Bearer {token}")));
        }
        AddonAuth::SharedSecret => {
            let token = bearer_token
                .ok_or(AddonManifestError::MissingAuthToken {
                    auth: AddonAuth::SharedSecret,
                })
                .map_err(event_call_setup_failure)?;
            headers.push(("x-nako-addon-secret".to_owned(), token.to_owned()));
        }
    }

    let dispatch_attempt = 1;
    let response = transport
        .post(AddonHttpRequest {
            url: resource_url(&manifest.base_url, &declaration.path),
            headers,
            body,
            timeout_ms,
        })
        .await
        .map_err(|err| AddonEventCallFailure {
            error: err,
            attempts: dispatch_attempt,
        })?;

    if !(200..300).contains(&response.status) {
        return Err(AddonEventCallFailure {
            error: AddonClientError::HttpStatus {
                status: response.status,
                retryable: is_retryable_http_status(response.status),
            },
            attempts: dispatch_attempt,
        });
    }

    let envelope = serde_json::from_str::<AddonEventResponse>(&response.body)
        .map_err(|err| AddonManifestError::InvalidEnvelope {
            message: format!("failed to parse addon event response: {err}"),
        })
        .map_err(|error| AddonEventCallFailure {
            error: error.into(),
            attempts: dispatch_attempt,
        })?;
    validate_event_response(
        &envelope,
        manifest,
        &request.subscription_id,
        &request.event_id,
    )
    .map_err(|error| AddonEventCallFailure {
        error: error.into(),
        attempts: dispatch_attempt,
    })?;

    Ok(AddonEventCallOutcome {
        response: envelope,
        http_status: response.status,
        attempts: dispatch_attempt,
    })
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
    let protocol_version = manifest.protocol_version.clone();
    let envelope = AddonHealthCheckRequest {
        protocol_version: protocol_version.clone(),
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
                ("x-nako-addon-protocol-version".to_owned(), protocol_version),
                ("x-nako-addon-id".to_owned(), manifest.id.clone()),
                (
                    "x-nako-addon-operation".to_owned(),
                    "health-check".to_owned(),
                ),
                ("x-nako-request-id".to_owned(), request_id),
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

impl NakoRuntimeClient<ReqwestAddonTransport> {
    #[must_use]
    pub fn new(config: NakoRuntimeClientConfig) -> Self {
        Self::with_transport(config, ReqwestAddonTransport::default())
    }
}

impl<T> NakoRuntimeClient<T>
where
    T: AddonTransport,
{
    #[must_use]
    pub const fn with_transport(config: NakoRuntimeClientConfig, transport: T) -> Self {
        Self { config, transport }
    }

    pub async fn access_check(
        &self,
        request: AddonAccessCheckRequest,
    ) -> AddonClientResult<AddonAccessCheckResponse> {
        self.post_runtime_json(ADDON_RUNTIME_ACCESS_CHECK_PATH, &request)
            .await
    }

    pub async fn submit_side_effect(
        &self,
        request: SubmitAddonSideEffectRequest,
    ) -> AddonClientResult<AddonSideEffectResponse> {
        self.post_runtime_json(ADDON_RUNTIME_SIDE_EFFECTS_PATH, &request)
            .await
    }

    pub async fn submit_metadata_write(
        &self,
        request: SubmitAddonMetadataWriteRequest,
    ) -> AddonClientResult<AddonSideEffectResponse> {
        let payload =
            serde_json::to_value(&request.patch).map_err(invalid_runtime_request_envelope)?;
        self.submit_side_effect(SubmitAddonSideEffectRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: request.library_id,
            target: request.target,
            idempotency_key: request.idempotency_key,
            provenance: request.provenance,
            payload,
        })
        .await
    }

    pub async fn submit_artwork_write(
        &self,
        request: SubmitAddonArtworkWriteRequest,
    ) -> AddonClientResult<AddonSideEffectResponse> {
        if request.target.kind != AddonSideEffectTargetKind::MediaItem {
            return Err(invalid_runtime_request(
                "artwork_write target must be media_item",
            ));
        }
        let payload =
            serde_json::to_value(&request.artwork).map_err(invalid_runtime_request_envelope)?;
        self.submit_side_effect(SubmitAddonSideEffectRequest {
            permission: AddonPermission::ArtworkWrite,
            library_id: request.library_id,
            target: request.target,
            idempotency_key: request.idempotency_key,
            provenance: request.provenance,
            payload,
        })
        .await
    }

    async fn post_runtime_json<B, R>(&self, path: &str, body: &B) -> AddonClientResult<R>
    where
        B: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let body = serde_json::to_string(body).map_err(invalid_runtime_request_envelope)?;
        if !self.config.addon_token.trim().is_empty() && body.contains(&self.config.addon_token) {
            return Err(AddonClientError::UnsafeRequestBody);
        }

        let response = self
            .transport
            .post(AddonHttpRequest {
                url: resource_url(&self.config.base_url, path),
                headers: vec![
                    ("accept".to_owned(), "application/json".to_owned()),
                    ("content-type".to_owned(), "application/json".to_owned()),
                    (
                        "authorization".to_owned(),
                        format!("Bearer {}", self.config.addon_token),
                    ),
                ],
                body,
                timeout_ms: self.config.timeout_ms,
            })
            .await?;

        if !(200..300).contains(&response.status) {
            return Err(AddonClientError::HttpStatus {
                status: response.status,
                retryable: is_retryable_http_status(response.status),
            });
        }

        serde_json::from_str(&response.body).map_err(runtime_response_envelope_error)
    }
}

fn resource_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

fn invalid_runtime_request(message: impl Into<String>) -> AddonClientError {
    AddonClientError::InvalidRequest {
        message: message.into(),
    }
}

fn invalid_runtime_request_envelope(error: serde_json::Error) -> AddonClientError {
    invalid_runtime_request(format!("failed to serialize Nako runtime request: {error}"))
}

fn runtime_response_envelope_error(error: serde_json::Error) -> AddonClientError {
    AddonClientError::InvalidResponse {
        message: format!("failed to parse Nako runtime response: {error}"),
    }
}

fn addon_http_error(error: reqwest::Error) -> AddonClientError {
    AddonClientError::Http {
        message: safe_error_text(&error.without_url().to_string()),
    }
}

fn safe_error_text(value: &str) -> String {
    value.replace(['\r', '\n'], " ").chars().take(240).collect()
}

impl AddonClientError {
    #[must_use]
    fn is_retryable(&self) -> bool {
        match self {
            Self::Http { .. } => true,
            Self::HttpStatus { retryable, .. } => *retryable,
            Self::Protocol(_)
            | Self::InvalidRequest { .. }
            | Self::InvalidResponse { .. }
            | Self::UnsafeRequestBody => false,
        }
    }
}

fn is_retryable_http_status(status: u16) -> bool {
    status == 408 || status == 429 || (500..600).contains(&status)
}

fn ensure_resource_search_scope_contract(
    manifest: &AddonManifest,
    granted_scopes: &[AddonScope],
) -> Result<(), AddonManifestError> {
    validate_manifest(manifest)?;
    let declaration = manifest
        .resources
        .iter()
        .find(|candidate| candidate.kind == AddonResource::ResourceSearch)
        .ok_or(AddonManifestError::ResourceNotDeclared {
            resource: AddonResource::ResourceSearch,
        })?;
    if !declaration
        .required_scopes
        .contains(&AddonScope::AcquisitionSearchRead)
    {
        return Err(AddonManifestError::MissingDeclaredScope {
            resource: AddonResource::ResourceSearch,
            scope: AddonScope::AcquisitionSearchRead,
        });
    }

    ensure_scope_grant(manifest, AddonResource::ResourceSearch, granted_scopes)
}

fn invalid_resource_search_request_envelope(error: serde_json::Error) -> AddonManifestError {
    AddonManifestError::InvalidEnvelope {
        message: format!("failed to serialize resource_search request payload: {error}"),
    }
}

fn resource_call_setup_failure(error: impl Into<AddonClientError>) -> AddonResourceCallFailure {
    AddonResourceCallFailure {
        error: error.into(),
        attempts: 0,
    }
}

fn task_call_setup_failure(error: impl Into<AddonClientError>) -> AddonTaskCallFailure {
    AddonTaskCallFailure {
        error: error.into(),
        attempts: 0,
    }
}

fn event_call_setup_failure(error: impl Into<AddonClientError>) -> AddonEventCallFailure {
    AddonEventCallFailure {
        error: error.into(),
        attempts: 0,
    }
}

impl AddonClientError {
    #[must_use]
    pub const fn http_status(&self) -> Option<u16> {
        match self {
            Self::HttpStatus { status, .. } => Some(*status),
            Self::Protocol(_)
            | Self::InvalidRequest { .. }
            | Self::InvalidResponse { .. }
            | Self::UnsafeRequestBody
            | Self::Http { .. } => None,
        }
    }

    #[must_use]
    pub const fn was_retryable_http_status(&self) -> bool {
        match self {
            Self::HttpStatus { retryable, .. } => *retryable,
            Self::Protocol(_)
            | Self::InvalidRequest { .. }
            | Self::InvalidResponse { .. }
            | Self::UnsafeRequestBody
            | Self::Http { .. } => false,
        }
    }

    #[must_use]
    pub const fn safe_code(&self) -> &'static str {
        match self {
            Self::Protocol(_) | Self::InvalidRequest { .. } => "invalid_request",
            Self::InvalidResponse { .. } => "invalid_response",
            Self::UnsafeRequestBody => "unsafe_request_body",
            Self::Http { .. } => "transport_error",
            Self::HttpStatus { status, .. } => match *status {
                400..=499 => "http_client_error",
                500..=599 => "http_server_error",
                _ => "http_status_error",
            },
        }
    }
}

impl AddonClientError {
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Protocol(_) => "protocol",
            Self::InvalidRequest { .. } => "invalid_request",
            Self::InvalidResponse { .. } => "invalid_response",
            Self::UnsafeRequestBody => "unsafe_request_body",
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
        AddonClientError::Protocol(_)
        | AddonClientError::InvalidRequest { .. }
        | AddonClientError::InvalidResponse { .. }
        | AddonClientError::UnsafeRequestBody
        | AddonClientError::Http { .. } => {}
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
        collections::{BTreeMap, VecDeque},
        sync::{Arc, Mutex},
    };

    use nako_addon_protocol::{
        ADDON_PROTOCOL_VERSION, ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA,
        ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA, AddonArtifact, AddonEventSubscriptionDeclaration,
        AddonMergedResourceLink, AddonResourceDeclaration, AddonResourceLink,
        AddonResourceLinkType, AddonResourceSearchIntent, AddonResourceSearchProviderExecution,
        AddonResourceSearchProviderFinality, AddonResourceSearchProviderStatus,
        AddonResourceSearchResult, AddonTaskDeclaration,
    };

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
        assert_eq!(header_value(&requests[0], "x-nako-attempt"), Some("1"));
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
        assert_eq!(header_value(&requests[0], "x-nako-attempt"), Some("1"));
        assert_eq!(header_value(&requests[1], "x-nako-attempt"), Some("2"));
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
    async fn resource_search_helper_calls_declared_path_with_typed_contract() {
        let manifest = resource_search_manifest();
        let payload = resource_search_response_payload();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: resource_search_response_json(&manifest, "resource-search-1", payload.clone()),
        }));

        let outcome = call_addon_resource_search_with_outcome(
            &transport,
            &manifest,
            &[AddonScope::AcquisitionSearchRead],
            "resource-search-1",
            resource_search_request(),
            Some("token-1"),
        )
        .await
        .unwrap();

        assert_eq!(outcome.http_status, 200);
        assert_eq!(outcome.attempts, 1);
        assert_eq!(outcome.response, payload);
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://example.test/addon/resource-search".to_owned()
        );
        assert_eq!(
            header_value(&requests[0], "x-nako-addon-resource"),
            Some("resource_search")
        );
        assert_eq!(
            header_value(&requests[0], "authorization"),
            Some("Bearer token-1")
        );
        assert_eq!(requests[0].timeout_ms, 6_000);

        let body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(body["resource"], "resource_search");
        assert_eq!(
            body["payload"]["schema"],
            ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA
        );
        assert_eq!(body["payload"]["intent"]["kind"], "media_title");
        assert_eq!(body["payload"]["link_types"], serde_json::json!(["quark"]));
    }

    #[tokio::test]
    async fn resource_search_helper_requires_granted_read_scope_before_http() {
        let manifest = resource_search_manifest();
        let transport = MockTransport::default();

        let err = call_addon_resource_search_with_outcome(
            &transport,
            &manifest,
            &[],
            "resource-search-2",
            resource_search_request(),
            Some("token-1"),
        )
        .await
        .unwrap_err();

        assert_eq!(err.attempts, 0);
        assert!(matches!(
            err.error,
            AddonClientError::Protocol(AddonManifestError::MissingDeclaredScope {
                resource: AddonResource::ResourceSearch,
                scope: AddonScope::AcquisitionSearchRead,
            })
        ));
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn resource_search_helper_requires_manifest_read_scope_contract() {
        let mut manifest = resource_search_manifest();
        manifest.resources[0].required_scopes.clear();
        let transport = MockTransport::default();

        let err = call_addon_resource_search_with_outcome(
            &transport,
            &manifest,
            &[AddonScope::AcquisitionSearchRead],
            "resource-search-3",
            resource_search_request(),
            Some("token-1"),
        )
        .await
        .unwrap_err();

        assert_eq!(err.attempts, 0);
        assert!(matches!(
            err.error,
            AddonClientError::Protocol(AddonManifestError::MissingDeclaredScope {
                resource: AddonResource::ResourceSearch,
                scope: AddonScope::AcquisitionSearchRead,
            })
        ));
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn resource_search_helper_rejects_wrong_request_schema_before_http() {
        let manifest = resource_search_manifest();
        let transport = MockTransport::default();
        let mut request = resource_search_request();
        request.schema = "nako.addon.resource_search.request.v0".to_owned();

        let err = call_addon_resource_search_with_outcome(
            &transport,
            &manifest,
            &[AddonScope::AcquisitionSearchRead],
            "resource-search-4",
            request,
            Some("token-1"),
        )
        .await
        .unwrap_err();

        assert_eq!(err.attempts, 0);
        assert!(matches!(err.error, AddonClientError::InvalidRequest { .. }));
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn resource_search_helper_rejects_wrong_response_schema() {
        let manifest = resource_search_manifest();
        let mut payload = resource_search_response_payload();
        payload.schema = "nako.addon.resource_search.response.v0".to_owned();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: resource_search_response_json(&manifest, "resource-search-5", payload),
        }));

        let err = call_addon_resource_search_with_outcome(
            &transport,
            &manifest,
            &[AddonScope::AcquisitionSearchRead],
            "resource-search-5",
            resource_search_request(),
            Some("token-1"),
        )
        .await
        .unwrap_err();

        assert_eq!(err.attempts, 1);
        assert!(matches!(
            err.error,
            AddonClientError::InvalidResponse { .. }
        ));
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn resource_search_helper_rejects_invalid_typed_response_payload() {
        let manifest = resource_search_manifest();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: serde_json::to_string(&AddonResourceResponse {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: manifest.id.clone(),
                resource: AddonResource::ResourceSearch,
                request_id: "resource-search-6".to_owned(),
                payload: serde_json::json!({"schema": ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA}),
                artifacts: Vec::new(),
            })
            .unwrap(),
        }));

        let err = call_addon_resource_search_with_outcome(
            &transport,
            &manifest,
            &[AddonScope::AcquisitionSearchRead],
            "resource-search-6",
            resource_search_request(),
            Some("token-1"),
        )
        .await
        .unwrap_err();

        assert_eq!(err.attempts, 1);
        assert!(matches!(
            err.error,
            AddonClientError::InvalidResponse { .. }
        ));
        assert_eq!(transport.requests().len(), 1);
    }

    #[tokio::test]
    async fn checks_health_without_auth_or_resource_payload() {
        let manifest = valid_manifest();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: serde_json::to_string(&AddonHealthCheckResponse {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                manifest_id: manifest.id.clone(),
                status: nako_addon_protocol::AddonHealthStatus::Ok,
                checked_at: "2026-05-21T12:00:00.000Z".to_owned(),
                manifest: nako_addon_protocol::AddonHealthManifestFacts {
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
            header_value(&requests[0], "x-nako-addon-operation"),
            Some("health-check")
        );
        assert_eq!(header_value(&requests[0], "authorization"), None);
        assert_eq!(header_value(&requests[0], "x-nako-addon-secret"), None);
        assert!(requests[0].body.contains("\"manifest_id\":\"example\""));
        assert!(!requests[0].body.contains("\"payload\""));
    }

    #[tokio::test]
    async fn calls_declared_task_path_with_host_owned_run_envelope() {
        let manifest = valid_manifest_with_task();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: serde_json::to_string(&AddonTaskResponse {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: manifest.id.clone(),
                task_id: "bulk-task".to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "task-request-1".to_owned(),
                output: serde_json::json!({"accepted": 2}),
            })
            .unwrap(),
        }));

        let outcome = call_addon_task_with_outcome(
            &transport,
            &manifest,
            &[AddonScope::AutomationRun],
            AddonTaskCallRequest {
                task_id: "bulk-task".to_owned(),
                job_id: "job-1".to_owned(),
                request_id: "task-request-1".to_owned(),
                attempt: 2,
                retry_of_job_id: Some("job-0".to_owned()),
                library_id: Some("library-1".to_owned()),
                source_id: Some("source-1".to_owned()),
                payload: serde_json::json!({"mode": "missing-only"}),
            },
            Some("token-1"),
        )
        .await
        .unwrap();

        assert_eq!(outcome.response.output["accepted"], 2);
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://example.test/addon/tasks/bulk".to_owned()
        );
        assert_eq!(
            header_value(&requests[0], "x-nako-addon-task"),
            Some("bulk-task")
        );
        assert_eq!(header_value(&requests[0], "x-nako-job-id"), Some("job-1"));
        assert_eq!(
            header_value(&requests[0], "x-nako-addon-operation"),
            Some("task-dispatch")
        );
        assert_eq!(
            header_value(&requests[0], "authorization"),
            Some("Bearer token-1")
        );
        assert_eq!(requests[0].timeout_ms, 7_000);
        assert!(requests[0].body.contains("\"task_id\":\"bulk-task\""));
        assert!(requests[0].body.contains("\"retry_of_job_id\":\"job-0\""));
        assert!(requests[0].body.contains("\"mode\":\"missing-only\""));
    }

    #[tokio::test]
    async fn calls_declared_event_subscription_path_with_event_envelope() {
        let manifest = valid_manifest_with_event_subscription();
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 202,
            body: serde_json::to_string(&AddonEventResponse {
                protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
                addon_id: manifest.id.clone(),
                subscription_id: "library-scanned".to_owned(),
                event_id: "event-1".to_owned(),
                output: serde_json::json!({"queued": true}),
            })
            .unwrap(),
        }));

        let outcome = call_addon_event_with_outcome(
            &transport,
            &manifest,
            &[AddonScope::WebhookEventRead],
            AddonEventCallRequest {
                subscription_id: "library-scanned".to_owned(),
                event_id: "event-1".to_owned(),
                event_kind: "library.scanned".to_owned(),
                subject_kind: "library".to_owned(),
                subject_id: "library-1".to_owned(),
                occurred_at: "2026-05-25T00:00:00.000Z".to_owned(),
                attempt: 2,
                payload: serde_json::json!({"library_id": "library-1"}),
            },
            None,
        )
        .await
        .unwrap();

        assert_eq!(outcome.http_status, 202);
        assert_eq!(outcome.response.output["queued"], true);
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://example.test/addon/events/library-scanned".to_owned()
        );
        assert_eq!(
            header_value(&requests[0], "x-nako-addon-operation"),
            Some("event-delivery")
        );
        assert_eq!(
            header_value(&requests[0], "x-nako-addon-event-subscription"),
            Some("library-scanned")
        );
        assert_eq!(
            header_value(&requests[0], "x-nako-event-kind"),
            Some("library.scanned")
        );
        assert_eq!(header_value(&requests[0], "x-nako-attempt"), Some("2"));
        assert!(
            requests[0]
                .body
                .contains("\"subscription_id\":\"library-scanned\"")
        );
        assert!(requests[0].body.contains("\"event_id\":\"event-1\""));
        assert!(requests[0].body.contains("\"library_id\":\"library-1\""));
    }

    #[tokio::test]
    async fn runtime_access_check_sends_bearer_token_only_in_header() {
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: serde_json::json!({
                "addon_id": "addon-1",
                "token_id": "token-1",
                "permission": "metadata_write",
                "library_id": "library-1",
                "allowed": true
            })
            .to_string(),
        }));
        let client = runtime_client(transport.clone());

        let response = client
            .access_check(AddonAccessCheckRequest {
                permission: AddonPermission::MetadataWrite,
                library_id: Some("library-1".to_owned()),
            })
            .await
            .unwrap();

        assert!(response.allowed);
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].url,
            "https://nako.example/addon/v1/access-check"
        );
        assert_eq!(
            header_value(&requests[0], "authorization"),
            Some("Bearer addon-token-secret")
        );
        assert!(!requests[0].body.contains("addon-token-secret"));
        assert!(
            requests[0]
                .body
                .contains("\"permission\":\"metadata_write\"")
        );
    }

    #[tokio::test]
    async fn runtime_side_effect_submission_parses_version_tolerant_summary() {
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: serde_json::json!({
                "side_effect": {
                    "id": "effect-1",
                    "addon_id": "addon-1",
                    "token_id": "token-1",
                    "permission": "metadata_write",
                    "library_id": "library-1",
                    "target": {"kind": "media_source", "id": "source-1"},
                    "idempotency_key": "metadata-demo-1",
                    "validation_status": "accepted",
                    "safe_error_code": null,
                    "apply_status": "applied",
                    "apply_error_code": null,
                    "applied_item_id": "item-1",
                    "applied_source": "addon:addon-1",
                    "apply_report": null,
                    "applied_at": "2026-05-24T09:00:00Z",
                    "created_at": "2026-05-24T09:00:00Z"
                },
                "idempotent_replay": false
            })
            .to_string(),
        }));
        let client = runtime_client(transport.clone());

        let response = client
            .submit_side_effect(SubmitAddonSideEffectRequest {
                permission: AddonPermission::MetadataWrite,
                library_id: "library-1".to_owned(),
                target: nako_addon_protocol::AddonSideEffectTarget {
                    kind: AddonSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "metadata-demo-1".to_owned(),
                provenance: serde_json::json!({"origin": "official-addon"}),
                payload: serde_json::json!({"title": "Demo"}),
            })
            .await
            .unwrap();

        assert_eq!(response.side_effect.apply_status, "applied");
        assert_eq!(
            response.side_effect.applied_item_id.as_deref(),
            Some("item-1")
        );
        let requests = transport.requests();
        assert_eq!(
            requests[0].url,
            "https://nako.example/addon/v1/side-effects"
        );
        assert_eq!(
            header_value(&requests[0], "authorization"),
            Some("Bearer addon-token-secret")
        );
        assert!(!requests[0].body.contains("addon-token-secret"));
        assert!(
            requests[0]
                .body
                .contains("\"idempotency_key\":\"metadata-demo-1\"")
        );
    }

    #[tokio::test]
    async fn runtime_metadata_write_serializes_patch_under_side_effect_payload() {
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 200,
            body: runtime_side_effect_response_json("metadata_write", "media_source"),
        }));
        let client = runtime_client(transport.clone());

        client
            .submit_metadata_write(SubmitAddonMetadataWriteRequest {
                library_id: "library-1".to_owned(),
                target: nako_addon_protocol::AddonSideEffectTarget {
                    kind: AddonSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "metadata-demo-2".to_owned(),
                provenance: serde_json::json!({"origin": "official-addon"}),
                patch: nako_addon_protocol::AddonMetadataPatch {
                    title: Some("The Matrix".to_owned()),
                    ..nako_addon_protocol::AddonMetadataPatch::default()
                },
            })
            .await
            .unwrap();

        let requests = transport.requests();
        let body: serde_json::Value = serde_json::from_str(&requests[0].body).unwrap();
        assert_eq!(body["permission"], "metadata_write");
        assert_eq!(body["target"]["kind"], "media_source");
        assert_eq!(body["payload"]["title"], "The Matrix");
        assert_eq!(body["payload"]["overview"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn runtime_artwork_write_rejects_non_media_item_targets_before_http() {
        let transport = MockTransport::default();
        let client = runtime_client(transport.clone());

        let error = client
            .submit_artwork_write(SubmitAddonArtworkWriteRequest {
                library_id: "library-1".to_owned(),
                target: nako_addon_protocol::AddonSideEffectTarget {
                    kind: AddonSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "artwork-demo-1".to_owned(),
                provenance: serde_json::json!({"origin": "official-addon"}),
                artwork: nako_addon_protocol::AddonArtworkWritePayload {
                    intent: nako_addon_protocol::AddonArtworkIntent::ProposeArtwork,
                    kind: nako_addon_protocol::AddonArtworkKind::Poster,
                    source: nako_addon_protocol::AddonArtworkSourcePayload {
                        kind: nako_addon_protocol::AddonArtworkSourceKind::RemoteUrl,
                        url: "https://example.test/poster.jpg".to_owned(),
                    },
                    language: None,
                    width: None,
                    height: None,
                },
            })
            .await
            .unwrap_err();

        assert!(matches!(error, AddonClientError::InvalidRequest { .. }));
        assert_eq!(error.safe_code(), "invalid_request");
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn runtime_request_rejects_body_token_material_before_http() {
        let transport = MockTransport::default();
        let client = runtime_client(transport.clone());

        let error = client
            .submit_side_effect(SubmitAddonSideEffectRequest {
                permission: AddonPermission::MetadataWrite,
                library_id: "library-1".to_owned(),
                target: nako_addon_protocol::AddonSideEffectTarget {
                    kind: AddonSideEffectTargetKind::MediaSource,
                    id: "source-1".to_owned(),
                },
                idempotency_key: "metadata-demo-token".to_owned(),
                provenance: serde_json::json!({"origin": "official-addon"}),
                payload: serde_json::json!({"leak": "addon-token-secret"}),
            })
            .await
            .unwrap_err();

        assert_eq!(error, AddonClientError::UnsafeRequestBody);
        assert_eq!(error.safe_code(), "unsafe_request_body");
        assert!(transport.requests().is_empty());
    }

    #[tokio::test]
    async fn runtime_http_errors_do_not_expose_response_bodies() {
        let transport = MockTransport::with_response(Ok(AddonHttpResponse {
            status: 403,
            body: "forbidden: addon-token-secret".to_owned(),
        }));
        let client = runtime_client(transport);

        let error = client
            .access_check(AddonAccessCheckRequest {
                permission: AddonPermission::MetadataWrite,
                library_id: None,
            })
            .await
            .unwrap_err();

        assert_eq!(
            error,
            AddonClientError::HttpStatus {
                status: 403,
                retryable: false
            }
        );
        assert_eq!(error.safe_code(), "http_client_error");
        assert!(!error.to_string().contains("addon-token-secret"));
    }

    #[tokio::test]
    async fn reqwest_transport_errors_do_not_expose_request_url_or_query_tokens() {
        let error = reqwest::Client::new()
            .get("http://127.0.0.1:1/addon/v1/access-check?token=addon-token-secret")
            .timeout(Duration::from_millis(50))
            .send()
            .await
            .unwrap_err();

        let error = addon_http_error(error);
        let message = error.to_string();

        assert_eq!(error.safe_code(), "transport_error");
        assert!(!message.contains("addon-token-secret"));
        assert!(!message.contains("/addon/v1/access-check"));
        assert!(!message.contains("127.0.0.1:1"));
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
                input_schema: Some("nako.metadata.request.v1".to_owned()),
                output_schema: Some("nako.metadata.response.v1".to_owned()),
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

    fn resource_search_manifest() -> AddonManifest {
        AddonManifest {
            id: "resource-search".to_owned(),
            name: "Resource Search".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            description: None,
            resources: vec![AddonResourceDeclaration {
                kind: AddonResource::ResourceSearch,
                path: "/resource-search".to_owned(),
                input_schema: Some(ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned()),
                output_schema: Some(ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned()),
                required_scopes: vec![AddonScope::AcquisitionSearchRead],
                timeout_ms: Some(6_000),
                max_attempts: Some(1),
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
            scopes: vec![AddonScope::AcquisitionSearchRead],
        }
    }

    fn resource_search_request() -> AddonResourceSearchRequest {
        AddonResourceSearchRequest {
            schema: ADDON_RESOURCE_SEARCH_REQUEST_SCHEMA.to_owned(),
            intent: AddonResourceSearchIntent::MediaTitle {
                title: "Demo Movie".to_owned(),
                year: Some(2026),
                media_kind: Some("movie".to_owned()),
            },
            query: "Demo Movie 2026".to_owned(),
            limit: Some(10),
            sources: vec!["pansou_compatible".to_owned()],
            link_types: vec![AddonResourceLinkType::Quark],
            refresh: false,
            context: serde_json::json!({"library_id": "library-1"}),
        }
    }

    fn resource_search_response_payload() -> AddonResourceSearchResponse {
        let link = AddonResourceLink {
            url: "https://pan.quark.cn/s/demo".to_owned(),
            normalized_url: "https://pan.quark.cn/s/demo".to_owned(),
            link_type: AddonResourceLinkType::Quark,
            source: "pansou:movies".to_owned(),
            password: Some("secret-code".to_owned()),
            note: None,
        };
        let mut merged_by_type = BTreeMap::new();
        merged_by_type.insert(
            AddonResourceLinkType::Quark,
            vec![AddonMergedResourceLink {
                url: link.url.clone(),
                normalized_url: link.normalized_url.clone(),
                link_type: AddonResourceLinkType::Quark,
                password: Some("secret-code".to_owned()),
                note: None,
                sources: vec!["pansou:movies".to_owned()],
            }],
        );

        AddonResourceSearchResponse {
            schema: ADDON_RESOURCE_SEARCH_RESPONSE_SCHEMA.to_owned(),
            query: "Demo Movie 2026".to_owned(),
            intent: AddonResourceSearchIntent::FreeText {
                text: "Demo Movie 2026".to_owned(),
            },
            total: 1,
            results: vec![AddonResourceSearchResult {
                id: "result-1".to_owned(),
                title: "Demo Movie".to_owned(),
                source: "pansou:movies".to_owned(),
                content: Some("candidate".to_owned()),
                links: vec![link],
                tags: vec!["demo".to_owned()],
                images: Vec::new(),
                score: 900,
            }],
            merged_by_type,
            provider_executions: vec![AddonResourceSearchProviderExecution {
                provider_id: "pansou_compatible".to_owned(),
                status: AddonResourceSearchProviderStatus::Ok,
                result_count: 1,
                finality: AddonResourceSearchProviderFinality::Complete,
                safe_message: None,
            }],
        }
    }

    fn resource_search_response_json(
        manifest: &AddonManifest,
        request_id: &str,
        payload: AddonResourceSearchResponse,
    ) -> String {
        serde_json::to_string(&AddonResourceResponse {
            protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
            addon_id: manifest.id.clone(),
            resource: AddonResource::ResourceSearch,
            request_id: request_id.to_owned(),
            payload: serde_json::to_value(payload).unwrap(),
            artifacts: Vec::new(),
        })
        .unwrap()
    }

    fn valid_manifest_with_task() -> AddonManifest {
        let mut manifest = valid_manifest();
        manifest.tasks = vec![
            AddonTaskDeclaration::new(
                "bulk-task",
                "Bulk Task",
                "/tasks/bulk",
                vec![AddonScope::AutomationRun],
            )
            .with_execution_bounds(Some(7_000), Some(3)),
        ];
        manifest.scopes.push(AddonScope::AutomationRun);
        manifest
    }

    fn valid_manifest_with_event_subscription() -> AddonManifest {
        let mut manifest = valid_manifest();
        manifest.auth = AddonAuth::None;
        manifest.event_subscriptions = vec![AddonEventSubscriptionDeclaration::new(
            "library-scanned",
            "library.scanned",
            "/events/library-scanned",
            vec![AddonScope::WebhookEventRead],
            serde_json::Value::Null,
        )];
        manifest.scopes.push(AddonScope::WebhookEventRead);
        manifest
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

    fn runtime_client(transport: MockTransport) -> NakoRuntimeClient<MockTransport> {
        NakoRuntimeClient::with_transport(
            NakoRuntimeClientConfig {
                base_url: "https://nako.example".to_owned(),
                addon_token: "addon-token-secret".to_owned(),
                timeout_ms: 9_000,
            },
            transport,
        )
    }

    fn runtime_side_effect_response_json(permission: &str, target_kind: &str) -> String {
        serde_json::json!({
            "side_effect": {
                "id": "effect-1",
                "permission": permission,
                "library_id": "library-1",
                "target": {"kind": target_kind, "id": "source-1"},
                "idempotency_key": "demo-1",
                "validation_status": "accepted",
                "safe_error_code": null,
                "apply_status": "applied",
                "apply_error_code": null,
                "applied_item_id": "item-1",
                "applied_source": "addon:addon-1",
                "apply_report": null
            },
            "idempotent_replay": false
        })
        .to_string()
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
