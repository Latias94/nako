use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    fmt,
    sync::Arc,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};
use taru_catalog::hydrate_item_catalog;
use taru_core::{
    CanonicalMetadata, CatalogRepository, CollectionRef, ContentRating, Credit, CreditRole,
    ExternalId, ExternalProvider, ImageKind, ImageRef, JobId, MediaItem, MediaItemId, MediaKind,
    MediaRepository, MetadataField, MetadataFieldLock, MetadataMatchKind, MetadataProfile,
    MetadataProviderAttemptId, MetadataProviderAttemptStatus, MetadataProviderErrorClass,
    MetadataRefreshMode, MetadataRepository, MetadataSource, NewMetadataProviderAttempt,
    ProviderRawResponse, Result, StudioRef, TaruError,
};
use taru_search::SearchIndex;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{sleep, timeout};

const DEFAULT_TMDB_API_BASE_URL: &str = "https://api.themoviedb.org/3";
const DEFAULT_TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original";
const DEFAULT_TMDB_LANGUAGE: &str = "en-US";
const DEFAULT_BANGUMI_API_BASE_URL: &str = "https://api.bgm.tv";
const DEFAULT_BANGUMI_IMAGE_BASE_URL: &str = "https://lain.bgm.tv";
const DEFAULT_DOUBAN_API_BASE_URL: &str = "https://api.douban.com/v2";
const DEFAULT_PROVIDER_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_PROVIDER_MAX_ATTEMPTS: u32 = 2;
const DEFAULT_PROVIDER_MIN_INTERVAL_MS: u64 = 250;
const DEFAULT_PROVIDER_CONCURRENCY: usize = 1;
const DEFAULT_PROVIDER_CIRCUIT_BREAKER_FAILURES: u32 = 5;
const TMDB_PROVIDER_NAME: &str = "tmdb";
const BANGUMI_PROVIDER_NAME: &str = "bangumi";
const DOUBAN_PROVIDER_NAME: &str = "douban";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataLookup {
    pub kind: Option<MediaKind>,
    pub title: String,
    pub year: Option<u16>,
    pub language: Option<String>,
    pub external_ids: Vec<ExternalId>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MetadataCandidate {
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub score: f32,
    pub metadata: CanonicalMetadata,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataFetchRequest {
    pub kind: MediaKind,
    pub provider_key: String,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataFetchResult {
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub metadata: CanonicalMetadata,
    pub raw_json: String,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn provider(&self) -> ExternalProvider;

    fn provider_name(&self) -> &'static str;

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>>;

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataHttpRuntimeConfig {
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub min_interval_ms: u64,
    pub concurrency: usize,
    pub user_agent: String,
    pub proxy: Option<String>,
    pub circuit_breaker_failures: u32,
}

impl Default for MetadataHttpRuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_PROVIDER_TIMEOUT_MS,
            max_attempts: DEFAULT_PROVIDER_MAX_ATTEMPTS,
            min_interval_ms: DEFAULT_PROVIDER_MIN_INTERVAL_MS,
            concurrency: DEFAULT_PROVIDER_CONCURRENCY,
            user_agent: default_metadata_user_agent(),
            proxy: None,
            circuit_breaker_failures: DEFAULT_PROVIDER_CIRCUIT_BREAKER_FAILURES,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MetadataHttpRuntime {
    client: reqwest::Client,
    config: MetadataHttpRuntimeConfig,
    permits: Arc<Semaphore>,
    throttle: Arc<Mutex<OffsetDateTime>>,
    consecutive_failures: Arc<AtomicU64>,
    circuit_open: Arc<AtomicBool>,
}

#[derive(Clone, Debug)]
pub struct MetadataHttpJsonResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

impl MetadataHttpRuntime {
    pub fn new(config: MetadataHttpRuntimeConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .user_agent(config.user_agent.clone())
            .timeout(Duration::from_millis(config.timeout_ms));

        if let Some(proxy) = config
            .proxy
            .as_ref()
            .filter(|proxy| !proxy.trim().is_empty())
        {
            builder = builder.proxy(reqwest::Proxy::all(proxy).map_err(|err| {
                TaruError::InvalidInput {
                    message: format!("invalid metadata provider proxy {proxy}: {err}"),
                }
            })?);
        }

        let client = builder.build().map_err(|err| TaruError::InvalidInput {
            message: format!("failed to build metadata provider HTTP client: {err}"),
        })?;
        let concurrency = config.concurrency.max(1);

        Ok(Self {
            client,
            config,
            permits: Arc::new(Semaphore::new(concurrency)),
            throttle: Arc::new(Mutex::new(OffsetDateTime::UNIX_EPOCH)),
            consecutive_failures: Arc::new(AtomicU64::new(0)),
            circuit_open: Arc::new(AtomicBool::new(false)),
        })
    }

    #[must_use]
    pub fn config(&self) -> &MetadataHttpRuntimeConfig {
        &self.config
    }

    pub async fn get_json(
        &self,
        provider: &'static str,
        operation: &str,
        url: String,
        query: &[(String, String)],
        headers: HeaderMap,
    ) -> Result<serde_json::Value> {
        let client = self.client.clone();
        self.execute_json(provider, operation, move || {
            client
                .get(url.clone())
                .query(query)
                .headers(headers.clone())
        })
        .await
    }

    pub async fn get_json_response(
        &self,
        provider: &'static str,
        operation: &str,
        url: String,
        query: &[(String, String)],
        headers: HeaderMap,
    ) -> Result<MetadataHttpJsonResponse> {
        let client = self.client.clone();
        self.execute_json_with_status(provider, operation, move || {
            client
                .get(url.clone())
                .query(query)
                .headers(headers.clone())
        })
        .await
        .map(|(status, body)| MetadataHttpJsonResponse {
            status: status.as_u16(),
            body,
        })
    }

    pub async fn post_json<B>(
        &self,
        provider: &'static str,
        operation: &str,
        url: String,
        query: &[(String, String)],
        headers: HeaderMap,
        body: &B,
    ) -> Result<serde_json::Value>
    where
        B: Serialize + Send + Sync,
    {
        let client = self.client.clone();
        self.execute_json(provider, operation, move || {
            client
                .post(url.clone())
                .query(query)
                .headers(headers.clone())
                .json(body)
        })
        .await
    }

    async fn execute_json<F>(
        &self,
        provider: &'static str,
        operation: &str,
        request_factory: F,
    ) -> Result<serde_json::Value>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        self.execute_json_with_status(provider, operation, request_factory)
            .await
            .map(|(_, value)| value)
    }

    async fn execute_json_with_status<F>(
        &self,
        provider: &'static str,
        operation: &str,
        request_factory: F,
    ) -> Result<(reqwest::StatusCode, serde_json::Value)>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        if self.circuit_open.load(Ordering::SeqCst) {
            return Err(TaruError::Provider {
                provider: provider.to_owned(),
                message: "metadata provider circuit breaker is open".to_owned(),
            });
        }

        let _permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::Provider {
                    provider: provider.to_owned(),
                    message: format!("metadata provider concurrency limiter is unavailable: {err}"),
                })?;
        let attempts = self.config.max_attempts.max(1);
        let mut last_error = None;

        for attempt in 1..=attempts {
            self.wait_for_rate_limit().await?;

            let send_result = timeout(
                Duration::from_millis(self.config.timeout_ms),
                request_factory().send(),
            )
            .await;
            let response = match send_result {
                Ok(Ok(response)) => response,
                Ok(Err(err)) => {
                    last_error = Some(provider_request_error(provider, err));
                    if attempt < attempts {
                        sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    break;
                }
                Err(_) => {
                    last_error = Some(TaruError::Provider {
                        provider: provider.to_owned(),
                        message: format!(
                            "{operation} timed out after {}ms",
                            self.config.timeout_ms
                        ),
                    });
                    if attempt < attempts {
                        sleep(retry_delay(attempt)).await;
                        continue;
                    }
                    break;
                }
            };

            let status = response.status();
            let text = response
                .text()
                .await
                .map_err(|err| provider_request_error(provider, err))?;

            if status.is_success() {
                self.consecutive_failures.store(0, Ordering::SeqCst);
                let value = serde_json::from_str(&text)
                    .map_err(|err| provider_parse_error(provider, operation, err))?;
                return Ok((status, value));
            }

            let error = TaruError::Provider {
                provider: provider.to_owned(),
                message: format!(
                    "{operation} returned HTTP {status}: {}",
                    truncate_message(&text, 240)
                ),
            };

            if !status.is_server_error() && status.as_u16() != 429 {
                self.record_failure();
                return Err(error);
            }

            last_error = Some(error);
            if attempt < attempts {
                sleep(retry_delay(attempt)).await;
            }
        }

        self.record_failure();
        Err(last_error.unwrap_or_else(|| TaruError::Provider {
            provider: provider.to_owned(),
            message: format!("{operation} failed without a provider response"),
        }))
    }

    async fn wait_for_rate_limit(&self) -> Result<()> {
        let min_interval = Duration::from_millis(self.config.min_interval_ms);
        if min_interval.is_zero() {
            return Ok(());
        }

        let mut next_allowed = self.throttle.lock().await;
        let now = OffsetDateTime::now_utc();
        if *next_allowed > now {
            let wait = (*next_allowed - now)
                .try_into()
                .unwrap_or_else(|_| Duration::from_millis(self.config.min_interval_ms));
            sleep(wait).await;
        }
        *next_allowed = OffsetDateTime::now_utc() + min_interval;

        Ok(())
    }

    fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        let threshold = u64::from(self.config.circuit_breaker_failures);
        if threshold > 0 && failures >= threshold {
            self.circuit_open.store(true, Ordering::SeqCst);
        }
    }
}

fn default_metadata_user_agent() -> String {
    format!("taru/{}", env!("CARGO_PKG_VERSION"))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshJobInput {
    pub item_id: MediaItemId,
    pub provider: Option<ExternalProvider>,
    pub force: bool,
    pub language: Option<String>,
    pub refresh_mode: MetadataRefreshMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshRequest {
    pub job_id: JobId,
    pub item_id: MediaItemId,
    pub profile: MetadataProfile,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshSummary {
    pub job_id: JobId,
    pub item_id: MediaItemId,
    pub provider: ExternalProvider,
    pub selected_provider: ExternalProvider,
    pub provider_key: String,
    pub matched_by: MetadataMatchKind,
    pub refresh_mode: MetadataRefreshMode,
    pub updated: bool,
    pub attempted_providers: Vec<MetadataProviderAttempt>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderAttempt {
    pub provider: ExternalProvider,
    pub status: MetadataProviderAttemptStatus,
    pub message: Option<String>,
    pub provider_key: Option<String>,
    pub matched_by: Option<MetadataMatchKind>,
    pub error_class: Option<MetadataProviderErrorClass>,
}

#[derive(Clone, Default)]
pub struct MetadataProviderRegistry {
    providers: HashMap<ExternalProvider, RegisteredMetadataProvider>,
}

impl MetadataProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, provider: P) -> &mut Self
    where
        P: MetadataProvider + 'static,
    {
        let provider_id = provider.provider();
        self.providers.insert(
            provider_id,
            RegisteredMetadataProvider::Available(Arc::new(provider)),
        );
        self
    }

    #[must_use]
    pub fn with_provider<P>(mut self, provider: P) -> Self
    where
        P: MetadataProvider + 'static,
    {
        self.register(provider);
        self
    }

    pub fn register_arc(
        &mut self,
        provider_id: ExternalProvider,
        provider: Arc<dyn MetadataProvider>,
    ) -> &mut Self {
        self.providers
            .insert(provider_id, RegisteredMetadataProvider::Available(provider));
        self
    }

    pub fn register_disabled(
        &mut self,
        provider: ExternalProvider,
        reason: impl Into<String>,
    ) -> &mut Self {
        self.providers.insert(
            provider,
            RegisteredMetadataProvider::Disabled {
                reason: reason.into(),
            },
        );
        self
    }

    pub fn register_unavailable(
        &mut self,
        provider: ExternalProvider,
        reason: impl Into<String>,
    ) -> &mut Self {
        self.providers.insert(
            provider,
            RegisteredMetadataProvider::Unavailable {
                reason: reason.into(),
            },
        );
        self
    }

    fn get(&self, provider: &ExternalProvider) -> Option<&RegisteredMetadataProvider> {
        self.providers.get(provider)
    }
}

impl fmt::Debug for MetadataProviderRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MetadataProviderRegistry")
            .field("providers", &self.providers)
            .finish()
    }
}

#[derive(Clone)]
enum RegisteredMetadataProvider {
    Available(Arc<dyn MetadataProvider>),
    Disabled { reason: String },
    Unavailable { reason: String },
}

impl fmt::Debug for RegisteredMetadataProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available(provider) => formatter
                .debug_struct("Available")
                .field("provider", &provider.provider())
                .field("provider_name", &provider.provider_name())
                .finish(),
            Self::Disabled { reason } => formatter
                .debug_struct("Disabled")
                .field("reason", reason)
                .finish(),
            Self::Unavailable { reason } => formatter
                .debug_struct("Unavailable")
                .field("reason", reason)
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct MetadataStrategyExecutor<R> {
    registry: MetadataProviderRegistry,
    repository: R,
}

impl<R> MetadataStrategyExecutor<R> {
    pub fn new(registry: MetadataProviderRegistry, repository: R) -> Self {
        Self {
            registry,
            repository,
        }
    }

    #[must_use]
    pub fn registry(&self) -> &MetadataProviderRegistry {
        &self.registry
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R> MetadataStrategyExecutor<R>
where
    R: CatalogRepository + MediaRepository + MetadataRepository + SearchIndex,
{
    pub async fn refresh_item(
        &self,
        request: MetadataRefreshRequest,
    ) -> Result<MetadataRefreshSummary> {
        validate_refresh_profile(&request.profile)?;

        let existing = self
            .repository
            .get_media_item(request.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: request.item_id.to_string(),
            })?;
        let mut attempts = Vec::new();

        for provider_id in &request.profile.metadata_providers {
            match self.registry.get(provider_id) {
                Some(RegisteredMetadataProvider::Available(provider)) => {
                    let started_at = now_utc_string()?;
                    let result = refresh_existing_with_provider(
                        provider.as_ref(),
                        &self.repository,
                        &request,
                        &existing,
                    )
                    .await;
                    let finished_at = now_utc_string()?;
                    let attempt = attempt_from_result(provider_id.clone(), &result);
                    persist_metadata_attempt(
                        &self.repository,
                        request.job_id,
                        request.item_id,
                        &attempt,
                        started_at,
                        finished_at,
                    )
                    .await?;
                    attempts.push(attempt);

                    match result {
                        Ok(success) => {
                            hydrate_item_catalog(
                                &self.repository,
                                success.item_id,
                                MetadataSource::Provider(success.provider.clone()),
                            )
                            .await?;

                            return Ok(success.into_summary(request.job_id, attempts));
                        }
                        Err(MetadataProviderRefreshError::NoMatch(_))
                        | Err(MetadataProviderRefreshError::ProviderFailed(_)) => {}
                        Err(MetadataProviderRefreshError::Fatal(err)) => return Err(err),
                    }
                }
                Some(RegisteredMetadataProvider::Disabled { reason }) => {
                    let now = now_utc_string()?;
                    let attempt = skipped_attempt(
                        provider_id.clone(),
                        MetadataProviderAttemptStatus::SkippedDisabled,
                        reason.clone(),
                    );
                    persist_metadata_attempt(
                        &self.repository,
                        request.job_id,
                        request.item_id,
                        &attempt,
                        now.clone(),
                        now,
                    )
                    .await?;
                    attempts.push(attempt);
                }
                Some(RegisteredMetadataProvider::Unavailable { reason }) => {
                    let now = now_utc_string()?;
                    let attempt = skipped_attempt(
                        provider_id.clone(),
                        MetadataProviderAttemptStatus::SkippedUnavailable,
                        reason.clone(),
                    );
                    persist_metadata_attempt(
                        &self.repository,
                        request.job_id,
                        request.item_id,
                        &attempt,
                        now.clone(),
                        now,
                    )
                    .await?;
                    attempts.push(attempt);
                }
                None => {
                    let now = now_utc_string()?;
                    let attempt = skipped_attempt(
                        provider_id.clone(),
                        MetadataProviderAttemptStatus::NotImplemented,
                        "metadata provider is not registered".to_owned(),
                    );
                    persist_metadata_attempt(
                        &self.repository,
                        request.job_id,
                        request.item_id,
                        &attempt,
                        now.clone(),
                        now,
                    )
                    .await?;
                    attempts.push(attempt);
                }
            }
        }

        Err(TaruError::Provider {
            provider: "metadata_strategy".to_owned(),
            message: format!(
                "metadata refresh exhausted all providers for item {}: {}",
                request.item_id,
                summarize_attempts(&attempts)
            ),
        })
    }
}

#[derive(Debug)]
pub struct MetadataRefreshService<P, R> {
    provider: P,
    repository: R,
}

impl<P, R> MetadataRefreshService<P, R> {
    pub fn new(provider: P, repository: R) -> Self {
        Self {
            provider,
            repository,
        }
    }

    #[must_use]
    pub fn provider(&self) -> &P {
        &self.provider
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<P, R> MetadataRefreshService<P, R>
where
    P: MetadataProvider,
    R: CatalogRepository + MediaRepository + MetadataRepository + SearchIndex,
{
    pub async fn refresh_item(
        &self,
        request: MetadataRefreshRequest,
    ) -> Result<MetadataRefreshSummary> {
        if !request
            .profile
            .metadata_providers
            .contains(&self.provider.provider())
        {
            return Err(TaruError::Unsupported(
                "metadata refresh profile does not enable this provider",
            ));
        }

        validate_refresh_profile(&request.profile)?;

        let existing = self
            .repository
            .get_media_item(request.item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: request.item_id.to_string(),
            })?;

        let started_at = now_utc_string()?;
        let result =
            refresh_existing_with_provider(&self.provider, &self.repository, &request, &existing)
                .await;
        let finished_at = now_utc_string()?;
        let attempt = attempt_from_result(self.provider.provider(), &result);
        persist_metadata_attempt(
            &self.repository,
            request.job_id,
            request.item_id,
            &attempt,
            started_at,
            finished_at,
        )
        .await?;
        let success = result.map_err(MetadataProviderRefreshError::into_error)?;
        hydrate_item_catalog(
            &self.repository,
            success.item_id,
            MetadataSource::Provider(success.provider.clone()),
        )
        .await?;

        Ok(success.into_summary(request.job_id, vec![attempt]))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MetadataProviderRefreshSuccess {
    item_id: MediaItemId,
    provider: ExternalProvider,
    provider_key: String,
    matched_by: MetadataMatchKind,
    refresh_mode: MetadataRefreshMode,
    updated: bool,
}

impl MetadataProviderRefreshSuccess {
    fn into_summary(
        self,
        job_id: JobId,
        attempted_providers: Vec<MetadataProviderAttempt>,
    ) -> MetadataRefreshSummary {
        MetadataRefreshSummary {
            job_id,
            item_id: self.item_id,
            provider: self.provider.clone(),
            selected_provider: self.provider,
            provider_key: self.provider_key,
            matched_by: self.matched_by,
            refresh_mode: self.refresh_mode,
            updated: self.updated,
            attempted_providers,
        }
    }
}

async fn persist_metadata_attempt<R>(
    repository: &R,
    job_id: JobId,
    item_id: MediaItemId,
    attempt: &MetadataProviderAttempt,
    started_at: String,
    finished_at: String,
) -> Result<()>
where
    R: MetadataRepository,
{
    repository
        .insert_metadata_provider_attempt(NewMetadataProviderAttempt {
            id: MetadataProviderAttemptId::new(),
            job_id,
            item_id,
            provider: attempt.provider.clone(),
            status: attempt.status,
            provider_key: attempt.provider_key.clone(),
            matched_by: attempt.matched_by,
            started_at,
            finished_at,
            error_class: attempt.error_class,
            message: attempt.message.clone(),
        })
        .await
}

fn attempt_from_result(
    provider: ExternalProvider,
    result: &std::result::Result<MetadataProviderRefreshSuccess, MetadataProviderRefreshError>,
) -> MetadataProviderAttempt {
    match result {
        Ok(success) => MetadataProviderAttempt {
            provider,
            status: MetadataProviderAttemptStatus::Succeeded,
            message: None,
            provider_key: Some(success.provider_key.clone()),
            matched_by: Some(success.matched_by),
            error_class: None,
        },
        Err(MetadataProviderRefreshError::NoMatch(message)) => MetadataProviderAttempt {
            provider,
            status: MetadataProviderAttemptStatus::NoMatch,
            message: Some(message.clone()),
            provider_key: None,
            matched_by: None,
            error_class: Some(MetadataProviderErrorClass::NoMatch),
        },
        Err(MetadataProviderRefreshError::ProviderFailed(message)) => MetadataProviderAttempt {
            provider,
            status: MetadataProviderAttemptStatus::Failed,
            message: Some(message.clone()),
            provider_key: None,
            matched_by: None,
            error_class: Some(classify_provider_failure_message(message)),
        },
        Err(MetadataProviderRefreshError::Fatal(err)) => MetadataProviderAttempt {
            provider,
            status: MetadataProviderAttemptStatus::Failed,
            message: Some(err.to_string()),
            provider_key: None,
            matched_by: None,
            error_class: Some(classify_provider_error_class(err)),
        },
    }
}

fn classify_provider_failure_message(message: &str) -> MetadataProviderErrorClass {
    classify_provider_error_class(&TaruError::Provider {
        provider: "metadata_provider".to_owned(),
        message: message.to_owned(),
    })
}

fn skipped_attempt(
    provider: ExternalProvider,
    status: MetadataProviderAttemptStatus,
    message: String,
) -> MetadataProviderAttempt {
    let error_class = match status {
        MetadataProviderAttemptStatus::SkippedDisabled
        | MetadataProviderAttemptStatus::SkippedUnavailable => {
            Some(MetadataProviderErrorClass::Unavailable)
        }
        MetadataProviderAttemptStatus::NotImplemented => {
            Some(MetadataProviderErrorClass::Unsupported)
        }
        MetadataProviderAttemptStatus::NoMatch => Some(MetadataProviderErrorClass::NoMatch),
        MetadataProviderAttemptStatus::Failed => Some(MetadataProviderErrorClass::Unknown),
        MetadataProviderAttemptStatus::Succeeded => None,
    };

    MetadataProviderAttempt {
        provider,
        status,
        message: Some(message),
        provider_key: None,
        matched_by: None,
        error_class,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MetadataProviderRefreshError {
    NoMatch(String),
    ProviderFailed(String),
    Fatal(TaruError),
}

impl MetadataProviderRefreshError {
    fn into_error(self) -> TaruError {
        match self {
            Self::NoMatch(message) => TaruError::NotFound {
                entity: "metadata_candidate",
                id: message,
            },
            Self::ProviderFailed(message) => TaruError::Provider {
                provider: "metadata_provider".to_owned(),
                message,
            },
            Self::Fatal(err) => err,
        }
    }
}

async fn refresh_existing_with_provider<P, R>(
    provider: &P,
    repository: &R,
    request: &MetadataRefreshRequest,
    existing: &MediaItem,
) -> std::result::Result<MetadataProviderRefreshSuccess, MetadataProviderRefreshError>
where
    P: MetadataProvider + ?Sized,
    R: MediaRepository + MetadataRepository,
{
    let (provider_key, matched_by) = resolve_provider_key(provider, request, existing).await?;
    let fetched = provider
        .fetch(MetadataFetchRequest {
            kind: existing.kind,
            provider_key: provider_key.clone(),
            language: request.profile.language.clone(),
        })
        .await
        .map_err(classify_provider_error)?;

    let provider_id = provider.provider();
    if fetched.provider != provider_id {
        return Err(MetadataProviderRefreshError::ProviderFailed(format!(
            "provider {} returned metadata for {}",
            provider_label(&provider_id),
            provider_label(&fetched.provider)
        )));
    }

    let locks = repository
        .list_field_locks(existing.id)
        .await
        .map_err(MetadataProviderRefreshError::Fatal)?;
    let policy = MetadataMergePolicy::from_locks_and_mode(&locks, request.profile.refresh_mode);
    let merged_metadata = policy.merge(&existing.metadata, &fetched.metadata);
    let updated = merged_metadata != existing.metadata;
    let updated_item = MediaItem {
        metadata: merged_metadata,
        ..existing.clone()
    };

    repository
        .upsert_media_item(&updated_item)
        .await
        .map_err(MetadataProviderRefreshError::Fatal)?;
    repository
        .upsert_provider_raw_response(&ProviderRawResponse {
            item_id: updated_item.id,
            provider: fetched.provider.clone(),
            provider_key: fetched.provider_key.clone(),
            fetched_at: now_utc_string().map_err(MetadataProviderRefreshError::Fatal)?,
            body_json: fetched.raw_json,
        })
        .await
        .map_err(MetadataProviderRefreshError::Fatal)?;

    Ok(MetadataProviderRefreshSuccess {
        item_id: updated_item.id,
        provider: fetched.provider,
        provider_key: fetched.provider_key,
        matched_by,
        refresh_mode: request.profile.refresh_mode,
        updated,
    })
}

async fn resolve_provider_key<P>(
    provider: &P,
    request: &MetadataRefreshRequest,
    item: &MediaItem,
) -> std::result::Result<(String, MetadataMatchKind), MetadataProviderRefreshError>
where
    P: MetadataProvider + ?Sized,
{
    let provider_id = provider.provider();
    if let Some(external_id) = item
        .metadata
        .external_ids
        .iter()
        .find(|external_id| external_id.provider == provider_id)
    {
        return Ok((external_id.value.clone(), MetadataMatchKind::ExternalId));
    }

    let lookup = MetadataLookup {
        kind: Some(item.kind),
        title: item.metadata.title.clone(),
        year: release_year(item.metadata.release_date.as_deref()),
        language: request.profile.language.clone(),
        external_ids: item.metadata.external_ids.clone(),
    };
    let candidates = provider
        .search(lookup)
        .await
        .map_err(classify_provider_error)?;
    let candidate = candidates
        .into_iter()
        .filter(|candidate| candidate.provider == provider_id)
        .max_by(|left, right| left.score.total_cmp(&right.score))
        .ok_or_else(|| {
            MetadataProviderRefreshError::NoMatch(format!(
                "{} returned no metadata candidate for item {}",
                provider_label(&provider_id),
                item.id
            ))
        })?;

    Ok((candidate.provider_key, MetadataMatchKind::Search))
}

fn validate_refresh_profile(profile: &MetadataProfile) -> Result<()> {
    if profile.refresh_mode == MetadataRefreshMode::None {
        return Err(TaruError::Unsupported(
            "metadata refresh profile disables metadata refresh",
        ));
    }

    if profile.refresh_mode == MetadataRefreshMode::ValidationOnly {
        return Err(TaruError::Unsupported(
            "metadata refresh validation-only mode is not implemented yet",
        ));
    }

    if profile.metadata_providers.is_empty() {
        return Err(TaruError::InvalidInput {
            message: "library metadata profile does not enable any metadata provider".to_owned(),
        });
    }

    Ok(())
}

fn classify_provider_error(error: TaruError) -> MetadataProviderRefreshError {
    match error {
        TaruError::NotFound { .. } => MetadataProviderRefreshError::NoMatch(error.to_string()),
        TaruError::Unsupported(_)
        | TaruError::InvalidInput { .. }
        | TaruError::Conflict { .. }
        | TaruError::Provider { .. } => {
            MetadataProviderRefreshError::ProviderFailed(error.to_string())
        }
        TaruError::Storage { .. } | TaruError::Database { .. } => {
            MetadataProviderRefreshError::Fatal(error)
        }
    }
}

fn classify_provider_error_class(error: &TaruError) -> MetadataProviderErrorClass {
    match error {
        TaruError::NotFound { .. } => MetadataProviderErrorClass::NoMatch,
        TaruError::Unsupported(_) => MetadataProviderErrorClass::Unsupported,
        TaruError::InvalidInput { .. } | TaruError::Conflict { .. } => {
            MetadataProviderErrorClass::Unknown
        }
        TaruError::Provider { message, .. } => {
            let lower = message.to_ascii_lowercase();
            if lower.contains("timeout") || lower.contains("timed out") {
                MetadataProviderErrorClass::Timeout
            } else if lower.contains("429") || lower.contains("rate") {
                MetadataProviderErrorClass::RateLimited
            } else if lower.contains("401") || lower.contains("403") || lower.contains("auth") {
                MetadataProviderErrorClass::Auth
            } else if lower.contains("http") {
                MetadataProviderErrorClass::HttpStatus
            } else if lower.contains("parse") || lower.contains("json") {
                MetadataProviderErrorClass::Parse
            } else {
                MetadataProviderErrorClass::Network
            }
        }
        TaruError::Storage { .. } | TaruError::Database { .. } => {
            MetadataProviderErrorClass::Unknown
        }
    }
}

fn summarize_attempts(attempts: &[MetadataProviderAttempt]) -> String {
    if attempts.is_empty() {
        return "no providers were attempted".to_owned();
    }

    attempts
        .iter()
        .map(|attempt| {
            let detail = attempt
                .message
                .as_deref()
                .filter(|message| !message.trim().is_empty())
                .unwrap_or("no detail");
            format!(
                "{}={} ({detail})",
                provider_label(&attempt.provider),
                attempt_status_label(attempt.status)
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn attempt_status_label(status: MetadataProviderAttemptStatus) -> &'static str {
    status.as_str()
}

fn provider_label(provider: &ExternalProvider) -> String {
    match provider {
        ExternalProvider::Tmdb => "tmdb".to_owned(),
        ExternalProvider::Douban => "douban".to_owned(),
        ExternalProvider::Bangumi => "bangumi".to_owned(),
        ExternalProvider::Imdb => "imdb".to_owned(),
        ExternalProvider::Local => "local".to_owned(),
        ExternalProvider::Other(value) => format!("other:{value}"),
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MetadataMergePolicy {
    locked_fields: HashSet<MetadataField>,
    mode: MetadataRefreshMode,
}

impl MetadataMergePolicy {
    #[must_use]
    pub fn from_locks(locks: &[MetadataFieldLock]) -> Self {
        Self::from_locks_and_mode(locks, MetadataRefreshMode::FullRefresh)
    }

    #[must_use]
    pub fn from_locks_and_mode(locks: &[MetadataFieldLock], mode: MetadataRefreshMode) -> Self {
        Self {
            locked_fields: locks
                .iter()
                .filter(|lock| lock.locked)
                .map(|lock| lock.field)
                .collect(),
            mode,
        }
    }

    #[must_use]
    pub fn merge(
        &self,
        existing: &CanonicalMetadata,
        incoming: &CanonicalMetadata,
    ) -> CanonicalMetadata {
        let mut merged = existing.clone();

        if self.should_replace_text(MetadataField::Title, &existing.title) {
            merged.title = incoming.title.clone();
        }
        if self.should_replace_option(MetadataField::OriginalTitle, &existing.original_title) {
            merged.original_title = incoming.original_title.clone();
        }
        if self.should_replace_option(MetadataField::SortTitle, &existing.sort_title) {
            merged.sort_title = incoming.sort_title.clone();
        }
        if self.should_replace_option(MetadataField::Overview, &existing.overview) {
            merged.overview = incoming.overview.clone();
        }
        if self.should_replace_option(MetadataField::ReleaseDate, &existing.release_date) {
            merged.release_date = incoming.release_date.clone();
        }
        if self.should_replace_option(MetadataField::RuntimeMinutes, &existing.runtime_minutes) {
            merged.runtime_minutes = incoming.runtime_minutes;
        }
        if self.should_replace_option(MetadataField::Tagline, &existing.tagline) {
            merged.tagline = incoming.tagline.clone();
        }
        if self.should_replace_list(MetadataField::Genres, &existing.genres) {
            merged.genres = incoming.genres.clone();
        }
        if self.should_replace_list(MetadataField::Tags, &existing.tags) {
            merged.tags = incoming.tags.clone();
        }
        if self.should_replace_list(MetadataField::Ratings, &existing.ratings) {
            merged.ratings = incoming.ratings.clone();
        }
        if self.should_replace_list(MetadataField::Images, &existing.images) {
            merged.images = incoming.images.clone();
        }
        if self.should_replace_list(MetadataField::Credits, &existing.credits) {
            merged.credits = incoming.credits.clone();
        }
        if self.should_replace_list(MetadataField::Collections, &existing.collections) {
            merged.collections = incoming.collections.clone();
        }
        if self.should_replace_list(MetadataField::Studios, &existing.studios) {
            merged.studios = incoming.studios.clone();
        }
        if self.should_replace_list(MetadataField::ExternalIds, &existing.external_ids) {
            merged.external_ids = incoming.external_ids.clone();
        }

        merged
    }

    fn is_locked(&self, field: MetadataField) -> bool {
        self.locked_fields.contains(&field)
    }

    fn should_replace_text(&self, field: MetadataField, existing: &str) -> bool {
        !self.is_locked(field)
            && (self.mode != MetadataRefreshMode::MissingOnly || existing.is_empty())
    }

    fn should_replace_option<T>(&self, field: MetadataField, existing: &Option<T>) -> bool {
        !self.is_locked(field)
            && (self.mode != MetadataRefreshMode::MissingOnly || existing.is_none())
    }

    fn should_replace_list<T>(&self, field: MetadataField, existing: &[T]) -> bool {
        !self.is_locked(field)
            && (self.mode != MetadataRefreshMode::MissingOnly || existing.is_empty())
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TmdbProviderConfig {
    pub read_access_token: String,
    pub api_base_url: String,
    pub image_base_url: String,
    pub language: String,
    pub include_adult: bool,
    pub runtime: MetadataHttpRuntimeConfig,
}

impl TmdbProviderConfig {
    #[must_use]
    pub fn new(read_access_token: impl Into<String>) -> Self {
        Self {
            read_access_token: read_access_token.into(),
            api_base_url: DEFAULT_TMDB_API_BASE_URL.to_owned(),
            image_base_url: DEFAULT_TMDB_IMAGE_BASE_URL.to_owned(),
            language: DEFAULT_TMDB_LANGUAGE.to_owned(),
            include_adult: false,
            runtime: MetadataHttpRuntimeConfig::default(),
        }
    }
}

impl fmt::Debug for TmdbProviderConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TmdbProviderConfig")
            .field("read_access_token", &"<redacted>")
            .field("api_base_url", &self.api_base_url)
            .field("image_base_url", &self.image_base_url)
            .field("language", &self.language)
            .field("include_adult", &self.include_adult)
            .field("runtime", &self.runtime)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct TmdbMetadataProvider {
    runtime: MetadataHttpRuntime,
    config: TmdbProviderConfig,
}

impl TmdbMetadataProvider {
    pub fn new(config: TmdbProviderConfig) -> Result<Self> {
        let runtime = MetadataHttpRuntime::new(config.runtime.clone())?;
        Ok(Self { runtime, config })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn language(&self, override_language: Option<&str>) -> String {
        override_language
            .filter(|language| !language.trim().is_empty())
            .unwrap_or(&self.config.language)
            .to_owned()
    }
}

#[async_trait]
impl MetadataProvider for TmdbMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        ExternalProvider::Tmdb
    }

    fn provider_name(&self) -> &'static str {
        TMDB_PROVIDER_NAME
    }

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
        if !lookup
            .kind
            .is_none_or(|kind| kind == MediaKind::Movie || kind == MediaKind::Unknown)
        {
            return Err(TaruError::Unsupported(
                "TMDB provider search currently supports movie lookups only",
            ));
        }

        let mut query = vec![
            ("query".to_owned(), lookup.title.clone()),
            (
                "include_adult".to_owned(),
                self.config.include_adult.to_string(),
            ),
            (
                "language".to_owned(),
                self.language(lookup.language.as_deref()),
            ),
            ("page".to_owned(), "1".to_owned()),
        ];

        if let Some(year) = lookup.year {
            query.push(("primary_release_year".to_owned(), year.to_string()));
        }

        let value = self
            .runtime
            .get_json(
                TMDB_PROVIDER_NAME,
                "search movie",
                self.endpoint("search/movie"),
                &query,
                bearer_headers(&self.config.read_access_token)?,
            )
            .await?;
        let search: TmdbSearchResponse =
            serde_json::from_value(value).map_err(|err| tmdb_parse_error("search movie", err))?;

        let candidates = search
            .results
            .into_iter()
            .map(|result| {
                let score = tmdb_search_score(&lookup, &result);
                MetadataCandidate {
                    provider: ExternalProvider::Tmdb,
                    provider_key: result.id.to_string(),
                    score,
                    metadata: tmdb_search_result_to_metadata(result, &self.config.image_base_url),
                }
            })
            .collect();

        Ok(candidates)
    }

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
        if request.kind != MediaKind::Movie {
            return Err(TaruError::Unsupported(
                "TMDB provider fetch currently supports movie metadata only",
            ));
        }

        let query = [
            (
                "language".to_owned(),
                self.language(request.language.as_deref()),
            ),
            (
                "append_to_response".to_owned(),
                "credits,images,release_dates,external_ids".to_owned(),
            ),
        ];
        let value = self
            .runtime
            .get_json(
                TMDB_PROVIDER_NAME,
                "movie details",
                self.endpoint(&format!("movie/{}", request.provider_key)),
                &query,
                bearer_headers(&self.config.read_access_token)?,
            )
            .await?;
        let raw_json = serde_json::to_string(&value)
            .map_err(|err| tmdb_parse_error("serialize movie details", err))?;
        let details: TmdbMovieDetails =
            serde_json::from_value(value).map_err(|err| tmdb_parse_error("movie details", err))?;

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key: details.id.to_string(),
            metadata: tmdb_movie_details_to_metadata(details, &self.config.image_base_url),
            raw_json,
        })
    }
}

#[derive(Clone, Debug)]
pub struct BangumiProviderConfig {
    pub access_token: Option<String>,
    pub api_base_url: String,
    pub image_base_url: String,
    pub include_nsfw: bool,
    pub runtime: MetadataHttpRuntimeConfig,
}

impl Default for BangumiProviderConfig {
    fn default() -> Self {
        Self {
            access_token: None,
            api_base_url: DEFAULT_BANGUMI_API_BASE_URL.to_owned(),
            image_base_url: DEFAULT_BANGUMI_IMAGE_BASE_URL.to_owned(),
            include_nsfw: false,
            runtime: MetadataHttpRuntimeConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BangumiMetadataProvider {
    runtime: MetadataHttpRuntime,
    config: BangumiProviderConfig,
}

impl BangumiMetadataProvider {
    pub fn new(config: BangumiProviderConfig) -> Result<Self> {
        let runtime = MetadataHttpRuntime::new(config.runtime.clone())?;
        Ok(Self { runtime, config })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn headers(&self) -> Result<HeaderMap> {
        self.config
            .access_token
            .as_deref()
            .filter(|token| !token.trim().is_empty())
            .map(bearer_headers)
            .unwrap_or_else(|| Ok(HeaderMap::new()))
    }
}

#[async_trait]
impl MetadataProvider for BangumiMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        ExternalProvider::Bangumi
    }

    fn provider_name(&self) -> &'static str {
        BANGUMI_PROVIDER_NAME
    }

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
        if !lookup.kind.is_none_or(|kind| {
            matches!(
                kind,
                MediaKind::Movie
                    | MediaKind::Series
                    | MediaKind::Season
                    | MediaKind::Episode
                    | MediaKind::Unknown
            )
        }) {
            return Err(TaruError::Unsupported(
                "Bangumi provider supports video metadata lookups only",
            ));
        }

        let query = vec![
            ("limit".to_owned(), "10".to_owned()),
            ("offset".to_owned(), "0".to_owned()),
        ];
        let body = BangumiSearchRequest {
            keyword: lookup.title.clone(),
            sort: "match".to_owned(),
            filter: BangumiSearchFilter {
                subject_type: Some(vec![2]),
                nsfw: Some(self.config.include_nsfw),
            },
        };
        let value = self
            .runtime
            .post_json(
                BANGUMI_PROVIDER_NAME,
                "search subjects",
                self.endpoint("v0/search/subjects"),
                &query,
                self.headers()?,
                &body,
            )
            .await?;
        let search: BangumiSearchResponse = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(BANGUMI_PROVIDER_NAME, "search subjects", err))?;

        Ok(search
            .data
            .into_iter()
            .map(|subject| {
                let score = bangumi_search_score(&lookup, &subject);
                MetadataCandidate {
                    provider: ExternalProvider::Bangumi,
                    provider_key: subject.id.to_string(),
                    score,
                    metadata: bangumi_subject_to_metadata(subject, &self.config.image_base_url),
                }
            })
            .collect())
    }

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
        if !matches!(
            request.kind,
            MediaKind::Movie
                | MediaKind::Series
                | MediaKind::Season
                | MediaKind::Episode
                | MediaKind::Unknown
        ) {
            return Err(TaruError::Unsupported(
                "Bangumi provider supports video metadata only",
            ));
        }

        let value = self
            .runtime
            .get_json(
                BANGUMI_PROVIDER_NAME,
                "subject details",
                self.endpoint(&format!("v0/subjects/{}", request.provider_key)),
                &[],
                self.headers()?,
            )
            .await?;
        let raw_json = serde_json::to_string(&value).map_err(|err| {
            provider_parse_error(BANGUMI_PROVIDER_NAME, "serialize subject details", err)
        })?;
        let details: BangumiSubject = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(BANGUMI_PROVIDER_NAME, "subject details", err))?;

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Bangumi,
            provider_key: details.id.to_string(),
            metadata: bangumi_subject_to_metadata(details, &self.config.image_base_url),
            raw_json,
        })
    }
}

#[derive(Clone, Debug)]
pub struct DoubanProviderConfig {
    pub api_key: Option<String>,
    pub api_base_url: String,
    pub image_base_url: Option<String>,
    pub runtime: MetadataHttpRuntimeConfig,
    pub headers: Vec<(String, String)>,
}

impl Default for DoubanProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base_url: DEFAULT_DOUBAN_API_BASE_URL.to_owned(),
            image_base_url: None,
            runtime: MetadataHttpRuntimeConfig::default(),
            headers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DoubanMetadataProvider {
    runtime: MetadataHttpRuntime,
    config: DoubanProviderConfig,
}

impl DoubanMetadataProvider {
    pub fn new(config: DoubanProviderConfig) -> Result<Self> {
        let runtime = MetadataHttpRuntime::new(config.runtime.clone())?;
        Ok(Self { runtime, config })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.api_base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn query(&self) -> Vec<(String, String)> {
        api_key_query("apikey", &self.config.api_key)
    }

    fn headers(&self) -> Result<HeaderMap> {
        header_map_from_pairs(&self.config.headers)
    }
}

#[async_trait]
impl MetadataProvider for DoubanMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        ExternalProvider::Douban
    }

    fn provider_name(&self) -> &'static str {
        DOUBAN_PROVIDER_NAME
    }

    async fn search(&self, lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
        if !lookup.kind.is_none_or(|kind| {
            matches!(
                kind,
                MediaKind::Movie
                    | MediaKind::Series
                    | MediaKind::Season
                    | MediaKind::Episode
                    | MediaKind::Unknown
            )
        }) {
            return Err(TaruError::Unsupported(
                "Douban provider supports video metadata lookups only",
            ));
        }

        let mut query = self.query();
        query.push(("q".to_owned(), lookup.title.clone()));
        query.push(("start".to_owned(), "0".to_owned()));
        query.push(("count".to_owned(), "10".to_owned()));
        let value = self
            .runtime
            .get_json(
                DOUBAN_PROVIDER_NAME,
                "search movies",
                self.endpoint("movie/search"),
                &query,
                self.headers()?,
            )
            .await?;
        let search: DoubanSearchResponse = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(DOUBAN_PROVIDER_NAME, "search movies", err))?;

        Ok(search
            .subjects
            .into_iter()
            .map(|subject| {
                let score = douban_search_score(&lookup, &subject);
                MetadataCandidate {
                    provider: ExternalProvider::Douban,
                    provider_key: subject.id.clone(),
                    score,
                    metadata: douban_subject_to_metadata(
                        subject,
                        self.config.image_base_url.as_deref(),
                    ),
                }
            })
            .collect())
    }

    async fn fetch(&self, request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
        if !matches!(
            request.kind,
            MediaKind::Movie
                | MediaKind::Series
                | MediaKind::Season
                | MediaKind::Episode
                | MediaKind::Unknown
        ) {
            return Err(TaruError::Unsupported(
                "Douban provider supports video metadata only",
            ));
        }

        let value = self
            .runtime
            .get_json(
                DOUBAN_PROVIDER_NAME,
                "movie details",
                self.endpoint(&format!("movie/subject/{}", request.provider_key)),
                &self.query(),
                self.headers()?,
            )
            .await?;
        let raw_json = serde_json::to_string(&value).map_err(|err| {
            provider_parse_error(DOUBAN_PROVIDER_NAME, "serialize movie details", err)
        })?;
        let details: DoubanSubject = serde_json::from_value(value)
            .map_err(|err| provider_parse_error(DOUBAN_PROVIDER_NAME, "movie details", err))?;

        Ok(MetadataFetchResult {
            provider: ExternalProvider::Douban,
            provider_key: details.id.clone(),
            metadata: douban_subject_to_metadata(details, self.config.image_base_url.as_deref()),
            raw_json,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    #[serde(default)]
    results: Vec<TmdbMovieSearchResult>,
}

#[derive(Debug, Deserialize)]
struct TmdbMovieSearchResult {
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: Option<String>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    poster_path: Option<String>,
    #[serde(default)]
    backdrop_path: Option<String>,
    #[serde(default)]
    popularity: f32,
}

#[derive(Debug, Deserialize)]
struct TmdbMovieDetails {
    id: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: Option<String>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    runtime: Option<u32>,
    #[serde(default)]
    tagline: Option<String>,
    #[serde(default)]
    genres: Vec<TmdbGenre>,
    #[serde(default)]
    belongs_to_collection: Option<TmdbCollection>,
    #[serde(default)]
    production_companies: Vec<TmdbProductionCompany>,
    #[serde(default)]
    poster_path: Option<String>,
    #[serde(default)]
    backdrop_path: Option<String>,
    #[serde(default)]
    imdb_id: Option<String>,
    #[serde(default)]
    credits: Option<TmdbCredits>,
    #[serde(default)]
    images: Option<TmdbImages>,
    #[serde(default)]
    release_dates: Option<TmdbReleaseDates>,
    #[serde(default)]
    external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Deserialize)]
struct TmdbGenre {
    name: String,
}

#[derive(Debug, Deserialize)]
struct TmdbCollection {
    id: u64,
    name: String,
    #[serde(default)]
    poster_path: Option<String>,
    #[serde(default)]
    backdrop_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TmdbProductionCompany {
    id: u64,
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbCredits {
    #[serde(default)]
    cast: Vec<TmdbCastMember>,
    #[serde(default)]
    crew: Vec<TmdbCrewMember>,
}

#[derive(Debug, Deserialize)]
struct TmdbCastMember {
    #[serde(default)]
    id: Option<u64>,
    name: String,
    #[serde(default)]
    character: Option<String>,
    #[serde(default)]
    order: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TmdbCrewMember {
    #[serde(default)]
    id: Option<u64>,
    name: String,
    #[serde(default)]
    job: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbImages {
    #[serde(default)]
    posters: Vec<TmdbImage>,
    #[serde(default)]
    backdrops: Vec<TmdbImage>,
    #[serde(default)]
    logos: Vec<TmdbImage>,
}

#[derive(Debug, Deserialize)]
struct TmdbImage {
    file_path: String,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    iso_639_1: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbReleaseDates {
    #[serde(default)]
    results: Vec<TmdbCountryReleaseDates>,
}

#[derive(Debug, Deserialize)]
struct TmdbCountryReleaseDates {
    iso_3166_1: String,
    #[serde(default)]
    release_dates: Vec<TmdbReleaseDate>,
}

#[derive(Debug, Deserialize)]
struct TmdbReleaseDate {
    #[serde(default)]
    certification: String,
}

#[derive(Debug, Default, Deserialize)]
struct TmdbExternalIds {
    #[serde(default)]
    imdb_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct BangumiSearchRequest {
    keyword: String,
    sort: String,
    filter: BangumiSearchFilter,
}

#[derive(Debug, Serialize)]
struct BangumiSearchFilter {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    subject_type: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nsfw: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct BangumiSearchResponse {
    #[serde(default)]
    data: Vec<BangumiSubject>,
}

#[derive(Debug, Deserialize)]
struct BangumiSubject {
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    name_cn: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    images: Option<BangumiImages>,
    #[serde(default)]
    infobox: Vec<BangumiInfoBoxItem>,
    #[serde(default)]
    tags: Vec<BangumiTag>,
    #[serde(default)]
    rating: Option<BangumiRating>,
}

#[derive(Debug, Deserialize)]
struct BangumiImages {
    #[serde(default)]
    large: Option<String>,
    #[serde(default)]
    common: Option<String>,
    #[serde(default)]
    medium: Option<String>,
    #[serde(default)]
    small: Option<String>,
    #[serde(default)]
    grid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BangumiInfoBoxItem {
    key: String,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct BangumiTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BangumiRating {
    #[serde(default)]
    score: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct DoubanSearchResponse {
    #[serde(default)]
    subjects: Vec<DoubanSubject>,
}

#[derive(Debug, Deserialize)]
struct DoubanSubject {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    original_title: Option<String>,
    #[serde(default)]
    alt_title: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    year: Option<String>,
    #[serde(default)]
    images: Option<DoubanImages>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(default)]
    countries: Vec<String>,
    #[serde(default)]
    casts: Vec<DoubanPerson>,
    #[serde(default)]
    directors: Vec<DoubanPerson>,
    #[serde(default)]
    writers: Vec<DoubanPerson>,
    #[serde(default)]
    rating: Option<DoubanRating>,
}

#[derive(Debug, Deserialize)]
struct DoubanImages {
    #[serde(default)]
    small: Option<String>,
    #[serde(default)]
    medium: Option<String>,
    #[serde(default)]
    large: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DoubanPerson {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct DoubanRating {
    #[serde(default)]
    average: Option<f32>,
}

fn provider_request_error(provider: &str, error: reqwest::Error) -> TaruError {
    TaruError::Provider {
        provider: provider.to_owned(),
        message: error.to_string(),
    }
}

fn tmdb_parse_error(operation: &str, error: impl ToString) -> TaruError {
    provider_parse_error(TMDB_PROVIDER_NAME, operation, error)
}

fn provider_parse_error(provider: &str, operation: &str, error: impl ToString) -> TaruError {
    TaruError::Provider {
        provider: provider.to_owned(),
        message: format!(
            "failed to parse {provider} {operation} response: {}",
            error.to_string()
        ),
    }
}

fn bearer_headers(token: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|err| {
        TaruError::InvalidInput {
            message: format!("invalid bearer token for metadata provider header: {err}"),
        }
    })?;
    headers.insert(AUTHORIZATION, value);
    Ok(headers)
}

fn api_key_query(name: &str, value: &Option<String>) -> Vec<(String, String)> {
    value
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| vec![(name.to_owned(), value.clone())])
        .unwrap_or_default()
}

fn header_map_from_pairs(pairs: &[(String, String)]) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        let name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|err| TaruError::InvalidInput {
                message: format!("invalid metadata provider header name {name}: {err}"),
            })?;
        let value = HeaderValue::from_str(value).map_err(|err| TaruError::InvalidInput {
            message: format!("invalid metadata provider header value for {name}: {err}"),
        })?;
        headers.insert(name, value);
    }
    Ok(headers)
}

fn retry_delay(attempt: u32) -> Duration {
    Duration::from_millis(100 * u64::from(attempt))
}

fn truncate_message(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    value.chars().take(max_chars).collect::<String>()
}

fn tmdb_search_score(lookup: &MetadataLookup, result: &TmdbMovieSearchResult) -> f32 {
    let mut score = 0.50;

    if result.title.eq_ignore_ascii_case(&lookup.title)
        || result
            .original_title
            .as_ref()
            .is_some_and(|title| title.eq_ignore_ascii_case(&lookup.title))
    {
        score += 0.25;
    }

    if lookup.year.is_some_and(|year| {
        result
            .release_date
            .as_deref()
            .and_then(|value| release_year(Some(value)))
            .is_some_and(|release_year| release_year == year)
    }) {
        score += 0.20;
    }

    score + (result.popularity.clamp(0.0, 100.0) / 2_000.0)
}

fn tmdb_search_result_to_metadata(
    result: TmdbMovieSearchResult,
    image_base_url: &str,
) -> CanonicalMetadata {
    let mut images = Vec::new();
    push_image_path(
        &mut images,
        ImageKind::Poster,
        result.poster_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );
    push_image_path(
        &mut images,
        ImageKind::Backdrop,
        result.backdrop_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );

    CanonicalMetadata {
        title: result.title,
        original_title: result.original_title,
        overview: result.overview,
        release_date: result.release_date,
        images,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: result.id.to_string(),
        }],
        ..CanonicalMetadata::default()
    }
}

fn tmdb_movie_details_to_metadata(
    details: TmdbMovieDetails,
    image_base_url: &str,
) -> CanonicalMetadata {
    let mut external_ids = vec![ExternalId {
        provider: ExternalProvider::Tmdb,
        value: details.id.to_string(),
    }];
    let imdb_id = details
        .external_ids
        .as_ref()
        .and_then(|ids| ids.imdb_id.as_ref())
        .or(details.imdb_id.as_ref())
        .filter(|value| !value.trim().is_empty());

    if let Some(imdb_id) = imdb_id {
        external_ids.push(ExternalId {
            provider: ExternalProvider::Imdb,
            value: imdb_id.clone(),
        });
    }

    let mut images = Vec::new();
    push_image_path(
        &mut images,
        ImageKind::Poster,
        details.poster_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );
    push_image_path(
        &mut images,
        ImageKind::Backdrop,
        details.backdrop_path.as_deref(),
        image_base_url,
        None,
        None,
        None,
    );

    if let Some(collection) = details.belongs_to_collection.as_ref() {
        push_image_path(
            &mut images,
            ImageKind::Poster,
            collection.poster_path.as_deref(),
            image_base_url,
            None,
            None,
            None,
        );
        push_image_path(
            &mut images,
            ImageKind::Backdrop,
            collection.backdrop_path.as_deref(),
            image_base_url,
            None,
            None,
            None,
        );
    }

    if let Some(tmdb_images) = details.images.as_ref() {
        for image in &tmdb_images.posters {
            push_tmdb_image(&mut images, ImageKind::Poster, image, image_base_url);
        }
        for image in &tmdb_images.backdrops {
            push_tmdb_image(&mut images, ImageKind::Backdrop, image, image_base_url);
        }
        for image in &tmdb_images.logos {
            push_tmdb_image(&mut images, ImageKind::Logo, image, image_base_url);
        }
    }

    CanonicalMetadata {
        title: details.title,
        original_title: details.original_title,
        overview: details.overview,
        release_date: details.release_date,
        runtime_minutes: details.runtime,
        tagline: details.tagline,
        genres: details
            .genres
            .into_iter()
            .map(|genre| genre.name)
            .filter(|name| !name.trim().is_empty())
            .collect(),
        ratings: ratings_from_release_dates(details.release_dates.as_ref()),
        images,
        credits: credits_from_tmdb(details.credits.unwrap_or_default()),
        collections: details
            .belongs_to_collection
            .into_iter()
            .filter(|collection| !collection.name.trim().is_empty())
            .map(|collection| CollectionRef {
                name: collection.name,
                overview: None,
                sort_order: None,
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: collection.id.to_string(),
                }],
            })
            .collect(),
        studios: details
            .production_companies
            .into_iter()
            .filter(|company| !company.name.trim().is_empty())
            .map(|company| StudioRef {
                name: company.name,
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: company.id.to_string(),
                }],
            })
            .collect(),
        external_ids,
        ..CanonicalMetadata::default()
    }
}

fn bangumi_search_score(lookup: &MetadataLookup, subject: &BangumiSubject) -> f32 {
    let mut score = 0.50;
    if subject.name.eq_ignore_ascii_case(&lookup.title)
        || subject.name_cn.eq_ignore_ascii_case(&lookup.title)
    {
        score += 0.30;
    }
    if lookup.year.is_some_and(|year| {
        subject
            .date
            .as_deref()
            .and_then(|value| release_year(Some(value)))
            .is_some_and(|release_year| release_year == year)
    }) {
        score += 0.15;
    }
    score
        + subject
            .rating
            .as_ref()
            .and_then(|rating| rating.score)
            .unwrap_or(0.0)
            / 200.0
}

fn bangumi_subject_to_metadata(subject: BangumiSubject, image_base_url: &str) -> CanonicalMetadata {
    let mut images = Vec::new();
    if let Some(subject_images) = subject.images.as_ref() {
        for uri in [
            subject_images.large.as_deref(),
            subject_images.common.as_deref(),
            subject_images.medium.as_deref(),
            subject_images.small.as_deref(),
            subject_images.grid.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            push_provider_image_uri(
                &mut images,
                ImageKind::Poster,
                Some(uri),
                image_base_url,
                ExternalProvider::Bangumi,
                None,
                None,
                None,
            );
        }
    }

    let studios = bangumi_infobox_strings(&subject.infobox, &["动画制作", "制作", "製作"])
        .into_iter()
        .map(|name| StudioRef {
            name,
            external_ids: Vec::new(),
        })
        .collect();
    let tags = subject
        .tags
        .into_iter()
        .map(|tag| tag.name)
        .filter(|name| !name.trim().is_empty())
        .collect();

    CanonicalMetadata {
        title: first_non_empty(&[Some(subject.name_cn.as_str()), Some(subject.name.as_str())])
            .unwrap_or_default(),
        original_title: non_empty_string(subject.name),
        overview: subject.summary.filter(|value| !value.trim().is_empty()),
        release_date: subject.date.filter(|value| !value.trim().is_empty()),
        runtime_minutes: None,
        tags,
        ratings: subject
            .rating
            .and_then(|rating| rating.score)
            .map(|score| ContentRating {
                source: "Bangumi:score".to_owned(),
                value: score.to_string(),
            })
            .into_iter()
            .collect(),
        images,
        studios,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Bangumi,
            value: subject.id.to_string(),
        }],
        ..CanonicalMetadata::default()
    }
}

fn douban_search_score(lookup: &MetadataLookup, subject: &DoubanSubject) -> f32 {
    let mut score = 0.50;
    if subject.title.eq_ignore_ascii_case(&lookup.title)
        || subject
            .original_title
            .as_ref()
            .is_some_and(|title| title.eq_ignore_ascii_case(&lookup.title))
    {
        score += 0.30;
    }
    if lookup.year.is_some_and(|year| {
        subject
            .year
            .as_deref()
            .and_then(|value| release_year(Some(value)))
            .is_some_and(|release_year| release_year == year)
    }) {
        score += 0.15;
    }
    score
        + subject
            .rating
            .as_ref()
            .and_then(|rating| rating.average)
            .unwrap_or(0.0)
            / 200.0
}

fn douban_subject_to_metadata(
    subject: DoubanSubject,
    image_base_url: Option<&str>,
) -> CanonicalMetadata {
    let mut images = Vec::new();
    if let Some(subject_images) = subject.images.as_ref() {
        for uri in [
            subject_images.large.as_deref(),
            subject_images.medium.as_deref(),
            subject_images.small.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            push_provider_image_uri(
                &mut images,
                ImageKind::Poster,
                Some(uri),
                image_base_url.unwrap_or_default(),
                ExternalProvider::Douban,
                None,
                None,
                None,
            );
        }
    }

    let mut credits = Vec::new();
    for person in subject.directors {
        push_douban_credit(&mut credits, person, CreditRole::Director);
    }
    for person in subject.writers {
        push_douban_credit(&mut credits, person, CreditRole::Writer);
    }
    for (order, person) in subject.casts.into_iter().enumerate() {
        let mut credit = douban_person_credit(person, CreditRole::Actor);
        credit.order = u32::try_from(order).ok();
        credits.push(credit);
    }

    let release_date = subject
        .year
        .as_ref()
        .filter(|year| year.len() == 4 && year.chars().all(|character| character.is_ascii_digit()))
        .map(|year| format!("{year}-01-01"));

    CanonicalMetadata {
        title: subject.title,
        original_title: subject.original_title.or(subject.alt_title),
        overview: subject.summary.filter(|value| !value.trim().is_empty()),
        release_date,
        genres: subject
            .genres
            .into_iter()
            .filter(|genre| !genre.trim().is_empty())
            .collect(),
        tags: subject
            .countries
            .into_iter()
            .filter(|country| !country.trim().is_empty())
            .collect(),
        ratings: subject
            .rating
            .and_then(|rating| rating.average)
            .map(|score| ContentRating {
                source: "Douban:score".to_owned(),
                value: score.to_string(),
            })
            .into_iter()
            .collect(),
        images,
        credits,
        external_ids: vec![ExternalId {
            provider: ExternalProvider::Douban,
            value: subject.id,
        }],
        ..CanonicalMetadata::default()
    }
}

fn bangumi_infobox_strings(items: &[BangumiInfoBoxItem], keys: &[&str]) -> Vec<String> {
    items
        .iter()
        .filter(|item| keys.iter().any(|key| item.key == *key))
        .flat_map(|item| metadata_strings_from_json(&item.value))
        .collect()
}

fn metadata_strings_from_json(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::String(value) => non_empty_string(value.clone()).into_iter().collect(),
        serde_json::Value::Array(values) => values
            .iter()
            .flat_map(metadata_strings_from_json)
            .collect::<Vec<_>>(),
        serde_json::Value::Object(map) => map
            .get("v")
            .or_else(|| map.get("value"))
            .into_iter()
            .flat_map(metadata_strings_from_json)
            .collect(),
        _ => Vec::new(),
    }
}

fn push_douban_credit(credits: &mut Vec<Credit>, person: DoubanPerson, role: CreditRole) {
    if person.name.trim().is_empty() {
        return;
    }
    credits.push(douban_person_credit(person, role));
}

fn douban_person_credit(person: DoubanPerson, role: CreditRole) -> Credit {
    Credit {
        name: person.name,
        role,
        character: None,
        order: None,
        external_ids: person
            .id
            .filter(|id| !id.trim().is_empty())
            .map(|id| ExternalId {
                provider: ExternalProvider::Douban,
                value: id,
            })
            .into_iter()
            .collect(),
    }
}

fn ratings_from_release_dates(release_dates: Option<&TmdbReleaseDates>) -> Vec<ContentRating> {
    release_dates
        .into_iter()
        .flat_map(|dates| dates.results.iter())
        .filter_map(|country| {
            country
                .release_dates
                .iter()
                .find(|date| !date.certification.trim().is_empty())
                .map(|date| ContentRating {
                    source: format!("TMDB:{}", country.iso_3166_1),
                    value: date.certification.clone(),
                })
        })
        .collect()
}

fn credits_from_tmdb(credits: TmdbCredits) -> Vec<Credit> {
    let mut output = Vec::new();

    for member in credits.cast {
        output.push(Credit {
            name: member.name,
            role: CreditRole::Actor,
            character: member.character,
            order: member.order,
            external_ids: tmdb_person_external_ids(member.id),
        });
    }

    for member in credits.crew {
        output.push(Credit {
            name: member.name,
            role: credit_role_from_tmdb_job(member.job.as_deref()),
            character: None,
            order: None,
            external_ids: tmdb_person_external_ids(member.id),
        });
    }

    output
}

fn credit_role_from_tmdb_job(job: Option<&str>) -> CreditRole {
    match job.unwrap_or_default().to_ascii_lowercase().as_str() {
        "director" => CreditRole::Director,
        "writer" | "screenplay" | "story" => CreditRole::Writer,
        "producer" | "executive producer" => CreditRole::Producer,
        "creator" => CreditRole::Creator,
        value if value.is_empty() => CreditRole::Other("crew".to_owned()),
        value => CreditRole::Other(value.to_owned()),
    }
}

fn tmdb_person_external_ids(id: Option<u64>) -> Vec<ExternalId> {
    id.map(|id| ExternalId {
        provider: ExternalProvider::Tmdb,
        value: id.to_string(),
    })
    .into_iter()
    .collect()
}

fn push_tmdb_image(
    images: &mut Vec<ImageRef>,
    kind: ImageKind,
    image: &TmdbImage,
    image_base_url: &str,
) {
    push_image_path(
        images,
        kind,
        Some(&image.file_path),
        image_base_url,
        image.width,
        image.height,
        image.iso_639_1.clone(),
    );
}

fn push_image_path(
    images: &mut Vec<ImageRef>,
    kind: ImageKind,
    path: Option<&str>,
    image_base_url: &str,
    width: Option<u32>,
    height: Option<u32>,
    language: Option<String>,
) {
    let Some(path) = path.filter(|path| !path.trim().is_empty()) else {
        return;
    };

    let uri = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_owned()
    } else {
        format!("{}{}", image_base_url.trim_end_matches('/'), path)
    };

    if images
        .iter()
        .any(|image| image.kind == kind && image.uri == uri)
    {
        return;
    }

    images.push(ImageRef {
        kind,
        uri,
        provider: ExternalProvider::Tmdb,
        width,
        height,
        language,
    });
}

fn push_provider_image_uri(
    images: &mut Vec<ImageRef>,
    kind: ImageKind,
    uri: Option<&str>,
    image_base_url: &str,
    provider: ExternalProvider,
    width: Option<u32>,
    height: Option<u32>,
    language: Option<String>,
) {
    let Some(uri) = uri.filter(|uri| !uri.trim().is_empty()) else {
        return;
    };
    let uri =
        if uri.starts_with("http://") || uri.starts_with("https://") || image_base_url.is_empty() {
            uri.to_owned()
        } else {
            format!("{}{}", image_base_url.trim_end_matches('/'), uri)
        };

    if images
        .iter()
        .any(|image| image.kind == kind && image.uri == uri)
    {
        return;
    }

    images.push(ImageRef {
        kind,
        uri,
        provider,
        width,
        height,
        language,
    });
}

fn release_year(value: Option<&str>) -> Option<u16> {
    let year = value?.get(0..4)?;

    if year.chars().all(|character| character.is_ascii_digit()) {
        year.parse().ok()
    } else {
        None
    }
}

fn first_non_empty(values: &[Option<&str>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .map(|value| (*value).to_owned())
}

fn non_empty_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn now_utc_string() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| TaruError::InvalidInput {
            message: format!("failed to format metadata refresh timestamp: {err}"),
        })
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicUsize, Ordering},
    };

    use axum::{
        Json, Router,
        extract::State,
        http::{HeaderMap as AxumHeaderMap, StatusCode, Uri},
        response::{IntoResponse, Response},
        routing::{get, post},
    };
    use serde_json::json;
    use taru_core::{
        CatalogRepository, JobKind, JobRepository, Library, LibraryId, LibraryOptions,
        LibraryPreset, LibraryRepository, MediaRepository, MetadataRepository, MetadataSource,
        NewJob, PageRequest, TransactionManager,
    };
    use taru_db::SqliteStore;
    use taru_search::{SearchIndex, SearchQuery};
    use tokio::{net::TcpListener, time::Instant};

    use super::*;

    #[test]
    fn merge_preserves_locked_fields() {
        let item_id = MediaItemId::new();
        let policy = MetadataMergePolicy::from_locks(&[
            MetadataFieldLock {
                item_id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            },
            MetadataFieldLock {
                item_id,
                field: MetadataField::Genres,
                locked: true,
                source: MetadataSource::Nfo,
            },
        ]);
        let existing = CanonicalMetadata {
            title: "Local Title".to_owned(),
            overview: Some("old".to_owned()),
            genres: vec!["Local".to_owned()],
            ..CanonicalMetadata::default()
        };
        let incoming = CanonicalMetadata {
            title: "Provider Title".to_owned(),
            overview: Some("new".to_owned()),
            genres: vec!["Action".to_owned()],
            tagline: Some("Wake up.".to_owned()),
            ..CanonicalMetadata::default()
        };

        let merged = policy.merge(&existing, &incoming);

        assert_eq!(merged.title, "Local Title");
        assert_eq!(merged.overview, Some("new".to_owned()));
        assert_eq!(merged.genres, vec!["Local"]);
        assert_eq!(merged.tagline, Some("Wake up.".to_owned()));
    }

    #[test]
    fn missing_only_merge_fills_empty_fields_without_replacing_existing_values() {
        let policy =
            MetadataMergePolicy::from_locks_and_mode(&[], MetadataRefreshMode::MissingOnly);
        let existing = CanonicalMetadata {
            title: "Local Title".to_owned(),
            overview: Some("old".to_owned()),
            genres: Vec::new(),
            ..CanonicalMetadata::default()
        };
        let incoming = CanonicalMetadata {
            title: "Provider Title".to_owned(),
            overview: Some("new".to_owned()),
            genres: vec!["Action".to_owned()],
            tagline: Some("Wake up.".to_owned()),
            ..CanonicalMetadata::default()
        };

        let merged = policy.merge(&existing, &incoming);

        assert_eq!(merged.title, "Local Title");
        assert_eq!(merged.overview, Some("old".to_owned()));
        assert_eq!(merged.genres, vec!["Action"]);
        assert_eq!(merged.tagline, Some("Wake up.".to_owned()));
    }

    #[test]
    fn full_refresh_replaces_unlocked_existing_values() {
        let policy =
            MetadataMergePolicy::from_locks_and_mode(&[], MetadataRefreshMode::FullRefresh);
        let existing = CanonicalMetadata {
            title: "Local Title".to_owned(),
            overview: Some("old".to_owned()),
            genres: vec!["Local".to_owned()],
            ..CanonicalMetadata::default()
        };
        let incoming = CanonicalMetadata {
            title: "Provider Title".to_owned(),
            overview: Some("new".to_owned()),
            genres: vec!["Action".to_owned()],
            ..CanonicalMetadata::default()
        };

        let merged = policy.merge(&existing, &incoming);

        assert_eq!(merged.title, "Provider Title");
        assert_eq!(merged.overview, Some("new".to_owned()));
        assert_eq!(merged.genres, vec!["Action"]);
    }

    #[tokio::test]
    async fn refresh_searches_fetches_caches_raw_and_preserves_locks() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let item = seed_movie(&store, "Local Matrix", Some("1999".to_owned()), vec![]).await;
        store
            .upsert_field_lock(&MetadataFieldLock {
                item_id: item.id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            })
            .await
            .unwrap();

        let provider = mock_provider(
            ExternalProvider::Tmdb,
            vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
            MetadataFetchResult {
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                metadata: CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    overview: Some("A hacker discovers the nature of reality.".to_owned()),
                    genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "603".to_owned(),
                    }],
                    ..CanonicalMetadata::default()
                },
                raw_json: r#"{"id":603,"title":"The Matrix"}"#.to_owned(),
            },
        );
        let search_count = provider.search_count.clone();
        let fetch_count = provider.fetch_count.clone();
        let service = MetadataRefreshService::new(provider, store.clone());
        let job_id = seed_metadata_job(&store, &item).await;

        let summary = service
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id: item.id,
                profile: MetadataProfile::from_preset(LibraryPreset::Movies),
                force: false,
            })
            .await
            .unwrap();
        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let raw = store
            .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "603")
            .await
            .unwrap()
            .unwrap();
        let persisted_attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
        let genres = store.list_genres(PageRequest::first_page()).await.unwrap();
        let hits = store
            .search(SearchQuery {
                query: "Science Fiction".to_owned(),
                facets: vec!["genre:Science Fiction".to_owned()],
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();

        assert_eq!(summary.provider_key, "603");
        assert_eq!(summary.provider, ExternalProvider::Tmdb);
        assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
        assert_eq!(summary.matched_by, MetadataMatchKind::Search);
        assert_eq!(summary.refresh_mode, MetadataRefreshMode::Default);
        assert_eq!(
            summary.attempted_providers,
            vec![MetadataProviderAttempt {
                provider: ExternalProvider::Tmdb,
                status: MetadataProviderAttemptStatus::Succeeded,
                message: None,
                provider_key: Some("603".to_owned()),
                matched_by: Some(MetadataMatchKind::Search),
                error_class: None,
            }]
        );
        assert!(summary.updated);
        assert_eq!(persisted_attempts.len(), 1);
        assert_eq!(
            persisted_attempts[0].status,
            MetadataProviderAttemptStatus::Succeeded
        );
        assert_eq!(
            persisted_attempts[0].matched_by,
            Some(MetadataMatchKind::Search)
        );
        assert_eq!(persisted_attempts[0].provider_key.as_deref(), Some("603"));
        assert_eq!(loaded.metadata.title, "Local Matrix");
        assert_eq!(
            loaded.metadata.overview,
            Some("A hacker discovers the nature of reality.".to_owned())
        );
        assert_eq!(
            loaded.metadata.genres,
            vec!["Action".to_owned(), "Science Fiction".to_owned()]
        );
        assert_eq!(raw.body_json, r#"{"id":603,"title":"The Matrix"}"#);
        assert!(genres.iter().any(|genre| genre.name == "Science Fiction"));
        assert_eq!(hits[0].item_id, item.id);
        assert_eq!(search_count.load(Ordering::SeqCst), 1);
        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn refresh_uses_existing_external_id_without_search() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item = seed_movie(
            &store,
            "The Matrix",
            Some("1999".to_owned()),
            vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "603".to_owned(),
            }],
        )
        .await;
        let provider = mock_provider(
            ExternalProvider::Tmdb,
            Vec::new(),
            MetadataFetchResult {
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                metadata: CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    runtime_minutes: Some(136),
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "603".to_owned(),
                    }],
                    ..CanonicalMetadata::default()
                },
                raw_json: r#"{"id":603,"runtime":136}"#.to_owned(),
            },
        );
        let search_count = provider.search_count.clone();
        let service = MetadataRefreshService::new(provider, store.clone());
        let job_id = seed_metadata_job(&store, &item).await;

        let summary = service
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id: item.id,
                profile: MetadataProfile::from_preset(LibraryPreset::Movies),
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.matched_by, MetadataMatchKind::ExternalId);
        assert_eq!(search_count.load(Ordering::SeqCst), 0);
        assert_eq!(
            store
                .get_media_item(item.id)
                .await
                .unwrap()
                .unwrap()
                .metadata
                .runtime_minutes,
            Some(136)
        );
    }

    #[tokio::test]
    async fn strategy_falls_back_from_unimplemented_bangumi_to_tmdb() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item = seed_movie(&store, "Anime Movie", Some("2024".to_owned()), vec![]).await;
        let job_id = seed_metadata_job(&store, &item).await;
        let tmdb = mock_provider(
            ExternalProvider::Tmdb,
            vec![mock_candidate(ExternalProvider::Tmdb, "100", "Anime Movie")],
            mock_fetch_result(
                ExternalProvider::Tmdb,
                "100",
                CanonicalMetadata {
                    title: "Anime Movie Provider Title".to_owned(),
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "100".to_owned(),
                    }],
                    ..CanonicalMetadata::default()
                },
            ),
        );
        let mut registry = MetadataProviderRegistry::new();
        registry.register(tmdb);
        let executor = MetadataStrategyExecutor::new(registry, store.clone());

        let summary = executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id: item.id,
                profile: MetadataProfile::from_preset(LibraryPreset::Anime),
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
        assert_eq!(summary.provider_key, "100");
        assert_eq!(
            attempt_statuses(&summary),
            vec![
                (
                    ExternalProvider::Bangumi,
                    MetadataProviderAttemptStatus::NotImplemented
                ),
                (
                    ExternalProvider::Tmdb,
                    MetadataProviderAttemptStatus::Succeeded
                )
            ]
        );
        let attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
        assert_eq!(attempts.len(), 2);
        assert_eq!(
            attempts[0].status,
            MetadataProviderAttemptStatus::NotImplemented
        );
        assert_eq!(attempts[1].status, MetadataProviderAttemptStatus::Succeeded);
        assert_eq!(
            store
                .get_media_item(item.id)
                .await
                .unwrap()
                .unwrap()
                .metadata
                .title,
            "Anime Movie Provider Title"
        );
    }

    #[tokio::test]
    async fn strategy_skips_disabled_provider() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item = seed_movie(&store, "The Matrix", Some("1999".to_owned()), vec![]).await;
        let job_id = seed_metadata_job(&store, &item).await;
        let tmdb = mock_provider(
            ExternalProvider::Tmdb,
            vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
            mock_fetch_result(
                ExternalProvider::Tmdb,
                "603",
                CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    ..CanonicalMetadata::default()
                },
            ),
        );
        let mut registry = MetadataProviderRegistry::new();
        registry.register_disabled(ExternalProvider::Douban, "disabled by config");
        registry.register(tmdb);
        let mut profile = MetadataProfile::from_preset(LibraryPreset::Movies);
        profile.metadata_providers = vec![ExternalProvider::Douban, ExternalProvider::Tmdb];
        let executor = MetadataStrategyExecutor::new(registry, store.clone());

        let summary = executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id: item.id,
                profile,
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
        assert_eq!(
            attempt_statuses(&summary),
            vec![
                (
                    ExternalProvider::Douban,
                    MetadataProviderAttemptStatus::SkippedDisabled
                ),
                (
                    ExternalProvider::Tmdb,
                    MetadataProviderAttemptStatus::Succeeded
                )
            ]
        );
        assert_eq!(
            store
                .list_metadata_provider_attempts(job_id)
                .await
                .unwrap()
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn strategy_fails_when_all_providers_fail() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item = seed_movie(&store, "Unknown Movie", None, vec![]).await;
        let job_id = seed_metadata_job(&store, &item).await;
        let mut tmdb = mock_provider(
            ExternalProvider::Tmdb,
            Vec::new(),
            mock_fetch_result(
                ExternalProvider::Tmdb,
                "never",
                CanonicalMetadata::default(),
            ),
        );
        tmdb.search_result = Ok(Vec::new());
        let mut registry = MetadataProviderRegistry::new();
        registry.register_unavailable(ExternalProvider::Bangumi, "credentials missing");
        registry.register(tmdb);
        let mut profile = MetadataProfile::from_preset(LibraryPreset::Anime);
        profile.metadata_providers = vec![ExternalProvider::Bangumi, ExternalProvider::Tmdb];
        let executor = MetadataStrategyExecutor::new(registry, store.clone());

        let err = executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id: item.id,
                profile,
                force: false,
            })
            .await
            .unwrap_err();

        let TaruError::Provider { provider, message } = err else {
            panic!("expected provider exhaustion error");
        };
        assert_eq!(provider, "metadata_strategy");
        assert!(message.contains("bangumi=skipped_unavailable"));
        assert!(message.contains("tmdb=no_match"));
        let attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
        assert_eq!(attempts.len(), 2);
        assert_eq!(
            attempts[0].status,
            MetadataProviderAttemptStatus::SkippedUnavailable
        );
        assert_eq!(attempts[1].status, MetadataProviderAttemptStatus::NoMatch);
    }

    #[tokio::test]
    async fn strategy_short_circuits_after_first_success() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item = seed_movie(&store, "The Matrix", Some("1999".to_owned()), vec![]).await;
        let job_id = seed_metadata_job(&store, &item).await;
        let tmdb = mock_provider(
            ExternalProvider::Tmdb,
            vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
            mock_fetch_result(
                ExternalProvider::Tmdb,
                "603",
                CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    ..CanonicalMetadata::default()
                },
            ),
        );
        let douban = mock_provider(
            ExternalProvider::Douban,
            vec![mock_candidate(
                ExternalProvider::Douban,
                "douban-1",
                "The Matrix",
            )],
            mock_fetch_result(
                ExternalProvider::Douban,
                "douban-1",
                CanonicalMetadata {
                    title: "The Matrix Douban".to_owned(),
                    ..CanonicalMetadata::default()
                },
            ),
        );
        let douban_search_count = douban.search_count.clone();
        let douban_fetch_count = douban.fetch_count.clone();
        let mut registry = MetadataProviderRegistry::new();
        registry.register(tmdb);
        registry.register(douban);
        let executor = MetadataStrategyExecutor::new(registry, store.clone());

        let summary = executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id: item.id,
                profile: MetadataProfile::from_preset(LibraryPreset::Movies),
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
        assert_eq!(summary.attempted_providers.len(), 1);
        assert_eq!(
            store
                .list_metadata_provider_attempts(job_id)
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(douban_search_count.load(Ordering::SeqCst), 0);
        assert_eq!(douban_fetch_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn strategy_preserves_locked_fields() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let item = seed_movie(&store, "Local Matrix", Some("1999".to_owned()), vec![]).await;
        let job_id = seed_metadata_job(&store, &item).await;
        store
            .upsert_field_lock(&MetadataFieldLock {
                item_id: item.id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            })
            .await
            .unwrap();
        let tmdb = mock_provider(
            ExternalProvider::Tmdb,
            vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
            mock_fetch_result(
                ExternalProvider::Tmdb,
                "603",
                CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    overview: Some("A hacker discovers the nature of reality.".to_owned()),
                    ..CanonicalMetadata::default()
                },
            ),
        );
        let mut registry = MetadataProviderRegistry::new();
        registry.register(tmdb);
        let executor = MetadataStrategyExecutor::new(registry, store.clone());

        executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id: item.id,
                profile: MetadataProfile::from_preset(LibraryPreset::Movies),
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        assert_eq!(loaded.metadata.title, "Local Matrix");
        assert_eq!(
            loaded.metadata.overview,
            Some("A hacker discovers the nature of reality.".to_owned())
        );
    }

    #[test]
    fn tmdb_movie_details_maps_core_metadata() {
        let details: TmdbMovieDetails = serde_json::from_str(
            r#"
            {
              "id": 603,
              "title": "The Matrix",
              "original_title": "The Matrix",
              "overview": "A hacker discovers the nature of reality.",
              "release_date": "1999-03-31",
              "runtime": 136,
              "tagline": "Welcome to the Real World",
              "genres": [{"id": 28, "name": "Action"}],
              "belongs_to_collection": {"id": 2344, "name": "The Matrix Collection"},
              "production_companies": [{"id": 79, "name": "Village Roadshow Pictures"}],
              "poster_path": "/poster.jpg",
              "backdrop_path": "/backdrop.jpg",
              "external_ids": {"imdb_id": "tt0133093"},
              "credits": {
                "cast": [
                  {"id": 6384, "name": "Keanu Reeves", "character": "Neo", "order": 0}
                ],
                "crew": [
                  {"id": 9339, "name": "Lana Wachowski", "job": "Director"}
                ]
              },
              "images": {
                "posters": [
                  {"file_path": "/poster.jpg", "width": 1000, "height": 1500, "iso_639_1": "en"}
                ],
                "backdrops": [],
                "logos": []
              },
              "release_dates": {
                "results": [
                  {"iso_3166_1": "US", "release_dates": [{"certification": "R"}]}
                ]
              }
            }
            "#,
        )
        .unwrap();

        let metadata = tmdb_movie_details_to_metadata(details, DEFAULT_TMDB_IMAGE_BASE_URL);

        assert_eq!(metadata.title, "The Matrix");
        assert_eq!(metadata.runtime_minutes, Some(136));
        assert_eq!(metadata.genres, vec!["Action"]);
        assert_eq!(
            metadata.ratings,
            vec![ContentRating {
                source: "TMDB:US".to_owned(),
                value: "R".to_owned()
            }]
        );
        assert!(metadata.images.iter().any(|image| {
            image.kind == ImageKind::Poster
                && image.uri == "https://image.tmdb.org/t/p/original/poster.jpg"
        }));
        assert!(metadata.credits.iter().any(|credit| {
            credit.name == "Lana Wachowski" && credit.role == CreditRole::Director
        }));
        assert_eq!(metadata.collections[0].name, "The Matrix Collection");
        assert_eq!(metadata.studios[0].name, "Village Roadshow Pictures");
        assert!(metadata.external_ids.iter().any(|external_id| {
            external_id.provider == ExternalProvider::Imdb && external_id.value == "tt0133093"
        }));
    }

    #[test]
    fn bangumi_subject_maps_core_metadata() {
        let subject: BangumiSubject = serde_json::from_str(
            r#"
            {
              "id": 8,
              "name": "Cowboy Bebop",
              "name_cn": "星际牛仔",
              "summary": "Whatever happens, happens.",
              "date": "1998-04-03",
              "images": {"large": "https://lain.bgm.tv/pic/cover/l/8.jpg"},
              "infobox": [
                {"key": "动画制作", "value": "SUNRISE"}
              ],
              "tags": [{"name": "科幻"}, {"name": "原创"}],
              "rating": {"score": 9.1}
            }
            "#,
        )
        .unwrap();

        let metadata = bangumi_subject_to_metadata(subject, DEFAULT_BANGUMI_IMAGE_BASE_URL);

        assert_eq!(metadata.title, "星际牛仔");
        assert_eq!(metadata.original_title.as_deref(), Some("Cowboy Bebop"));
        assert_eq!(metadata.release_date.as_deref(), Some("1998-04-03"));
        assert_eq!(metadata.tags, vec!["科幻", "原创"]);
        assert_eq!(metadata.studios[0].name, "SUNRISE");
        assert!(metadata.images.iter().any(|image| {
            image.provider == ExternalProvider::Bangumi
                && image.uri == "https://lain.bgm.tv/pic/cover/l/8.jpg"
        }));
        assert!(metadata.external_ids.iter().any(|external_id| {
            external_id.provider == ExternalProvider::Bangumi && external_id.value == "8"
        }));
    }

    #[test]
    fn douban_subject_maps_core_metadata() {
        let subject: DoubanSubject = serde_json::from_str(
            r#"
            {
              "id": "1292052",
              "title": "肖申克的救赎",
              "original_title": "The Shawshank Redemption",
              "summary": "Hope is a good thing.",
              "year": "1994",
              "images": {"large": "https://img.doubanio.com/view/photo/l/public/p480747492.webp"},
              "genres": ["剧情", "犯罪"],
              "countries": ["美国"],
              "directors": [{"id": "1047973", "name": "Frank Darabont"}],
              "casts": [{"id": "1054521", "name": "Tim Robbins"}],
              "rating": {"average": 9.7}
            }
            "#,
        )
        .unwrap();

        let metadata = douban_subject_to_metadata(subject, None);

        assert_eq!(metadata.title, "肖申克的救赎");
        assert_eq!(
            metadata.original_title.as_deref(),
            Some("The Shawshank Redemption")
        );
        assert_eq!(metadata.release_date.as_deref(), Some("1994-01-01"));
        assert_eq!(metadata.genres, vec!["剧情", "犯罪"]);
        assert!(metadata.credits.iter().any(|credit| {
            credit.name == "Frank Darabont" && credit.role == CreditRole::Director
        }));
        assert!(metadata.external_ids.iter().any(|external_id| {
            external_id.provider == ExternalProvider::Douban && external_id.value == "1292052"
        }));
    }

    #[tokio::test]
    async fn metadata_http_runtime_retries_and_sends_user_agent() {
        let server = MockMetadataServer::start().await;
        let runtime = MetadataHttpRuntime::new(MetadataHttpRuntimeConfig {
            max_attempts: 2,
            min_interval_ms: 0,
            user_agent: "taru-test-agent".to_owned(),
            ..MetadataHttpRuntimeConfig::default()
        })
        .unwrap();

        let body = runtime
            .get_json("mock", "flaky", server.url("/flaky"), &[], HeaderMap::new())
            .await
            .unwrap();

        assert_eq!(body["ok"], true);
        assert_eq!(server.request_count(), 2);
        assert_eq!(
            server.user_agents(),
            vec!["taru-test-agent", "taru-test-agent"]
        );
    }

    #[tokio::test]
    async fn metadata_http_runtime_rate_limits_requests() {
        let server = MockMetadataServer::start().await;
        let runtime = MetadataHttpRuntime::new(MetadataHttpRuntimeConfig {
            min_interval_ms: 40,
            max_attempts: 1,
            ..MetadataHttpRuntimeConfig::default()
        })
        .unwrap();
        let started = Instant::now();

        runtime
            .get_json("mock", "ok", server.url("/ok"), &[], HeaderMap::new())
            .await
            .unwrap();
        runtime
            .get_json("mock", "ok", server.url("/ok"), &[], HeaderMap::new())
            .await
            .unwrap();

        assert!(started.elapsed().as_millis() >= 35);
    }

    #[tokio::test]
    async fn bangumi_provider_uses_runtime_and_maps_http_response() {
        let server = MockMetadataServer::start().await;
        let provider = BangumiMetadataProvider::new(BangumiProviderConfig {
            access_token: Some("bangumi-token".to_owned()),
            api_base_url: server.base_url(),
            runtime: MetadataHttpRuntimeConfig {
                min_interval_ms: 0,
                user_agent: "taru-bangumi-test".to_owned(),
                ..MetadataHttpRuntimeConfig::default()
            },
            ..BangumiProviderConfig::default()
        })
        .unwrap();

        let candidates = provider
            .search(MetadataLookup {
                kind: Some(MediaKind::Series),
                title: "Cowboy Bebop".to_owned(),
                year: Some(1998),
                language: Some("zh-CN".to_owned()),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();
        let fetched = provider
            .fetch(MetadataFetchRequest {
                kind: MediaKind::Series,
                provider_key: candidates[0].provider_key.clone(),
                language: Some("zh-CN".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(candidates[0].provider, ExternalProvider::Bangumi);
        assert_eq!(fetched.metadata.title, "星际牛仔");
        assert!(
            server
                .authorizations()
                .iter()
                .any(|value| value == "Bearer bangumi-token")
        );
    }

    #[tokio::test]
    async fn douban_provider_uses_api_key_and_maps_http_response() {
        let server = MockMetadataServer::start().await;
        let provider = DoubanMetadataProvider::new(DoubanProviderConfig {
            api_key: Some("douban-key".to_owned()),
            api_base_url: server.base_url(),
            runtime: MetadataHttpRuntimeConfig {
                min_interval_ms: 0,
                ..MetadataHttpRuntimeConfig::default()
            },
            headers: vec![("X-Douban-Test".to_owned(), "ok".to_owned())],
            ..DoubanProviderConfig::default()
        })
        .unwrap();

        let candidates = provider
            .search(MetadataLookup {
                kind: Some(MediaKind::Movie),
                title: "肖申克的救赎".to_owned(),
                year: Some(1994),
                language: Some("zh-CN".to_owned()),
                external_ids: Vec::new(),
            })
            .await
            .unwrap();
        let fetched = provider
            .fetch(MetadataFetchRequest {
                kind: MediaKind::Movie,
                provider_key: candidates[0].provider_key.clone(),
                language: Some("zh-CN".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(candidates[0].provider, ExternalProvider::Douban);
        assert_eq!(fetched.metadata.title, "肖申克的救赎");
        assert!(
            server
                .uris()
                .iter()
                .any(|uri| uri.contains("apikey=douban-key"))
        );
        assert!(
            server
                .headers("x-douban-test")
                .iter()
                .any(|value| value == "ok")
        );
    }

    async fn seed_movie(
        store: &SqliteStore,
        title: &str,
        release_date: Option<String>,
        external_ids: Vec<ExternalId>,
    ) -> MediaItem {
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: title.to_owned(),
                release_date,
                external_ids,
                ..CanonicalMetadata::default()
            },
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        item
    }

    async fn seed_metadata_job(store: &SqliteStore, item: &MediaItem) -> JobId {
        store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: "metadata.test".to_owned(),
                library_id: None,
                source_id: None,
                input_json: Some(
                    serde_json::to_string(&MetadataRefreshJobInput {
                        item_id: item.id,
                        provider: None,
                        force: false,
                        language: None,
                        refresh_mode: MetadataRefreshMode::Default,
                    })
                    .unwrap(),
                ),
            })
            .await
            .unwrap()
            .id
    }

    struct MockMetadataProvider {
        provider: ExternalProvider,
        search_count: Arc<AtomicUsize>,
        fetch_count: Arc<AtomicUsize>,
        search_result: Result<Vec<MetadataCandidate>>,
        fetch_result: Result<MetadataFetchResult>,
    }

    #[async_trait]
    impl MetadataProvider for MockMetadataProvider {
        fn provider(&self) -> ExternalProvider {
            self.provider.clone()
        }

        fn provider_name(&self) -> &'static str {
            "mock"
        }

        async fn search(&self, _lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
            self.search_count.fetch_add(1, Ordering::SeqCst);
            self.search_result.clone()
        }

        async fn fetch(&self, _request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            self.fetch_result.clone()
        }
    }

    fn mock_provider(
        provider: ExternalProvider,
        search_candidates: Vec<MetadataCandidate>,
        fetch_result: MetadataFetchResult,
    ) -> MockMetadataProvider {
        MockMetadataProvider {
            provider,
            search_count: Arc::new(AtomicUsize::new(0)),
            fetch_count: Arc::new(AtomicUsize::new(0)),
            search_result: Ok(search_candidates),
            fetch_result: Ok(fetch_result),
        }
    }

    fn mock_candidate(
        provider: ExternalProvider,
        provider_key: &str,
        title: &str,
    ) -> MetadataCandidate {
        MetadataCandidate {
            provider,
            provider_key: provider_key.to_owned(),
            score: 0.95,
            metadata: CanonicalMetadata {
                title: title.to_owned(),
                ..CanonicalMetadata::default()
            },
        }
    }

    fn mock_fetch_result(
        provider: ExternalProvider,
        provider_key: &str,
        metadata: CanonicalMetadata,
    ) -> MetadataFetchResult {
        MetadataFetchResult {
            provider,
            provider_key: provider_key.to_owned(),
            metadata,
            raw_json: format!(r#"{{"id":"{provider_key}"}}"#),
        }
    }

    fn attempt_statuses(
        summary: &MetadataRefreshSummary,
    ) -> Vec<(ExternalProvider, MetadataProviderAttemptStatus)> {
        summary
            .attempted_providers
            .iter()
            .map(|attempt| (attempt.provider.clone(), attempt.status))
            .collect()
    }

    #[derive(Clone)]
    struct MockMetadataServer {
        base_url: String,
        state: MockMetadataState,
    }

    #[derive(Clone, Default)]
    struct MockMetadataState {
        requests: Arc<StdMutex<Vec<MockRequest>>>,
    }

    #[derive(Clone, Debug)]
    struct MockRequest {
        uri: String,
        user_agent: Option<String>,
        authorization: Option<String>,
        headers: Vec<(String, String)>,
    }

    impl MockMetadataServer {
        async fn start() -> Self {
            let state = MockMetadataState::default();
            let router = Router::new()
                .route("/ok", get(mock_ok))
                .route("/flaky", get(mock_flaky))
                .route("/v0/search/subjects", post(mock_bangumi_search))
                .route("/v0/subjects/{id}", get(mock_bangumi_subject))
                .route("/movie/search", get(mock_douban_search))
                .route("/movie/subject/{id}", get(mock_douban_subject))
                .with_state(state.clone());
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                axum::serve(listener, router).await.unwrap();
            });

            Self {
                base_url: format!("http://{addr}"),
                state,
            }
        }

        fn base_url(&self) -> String {
            self.base_url.clone()
        }

        fn url(&self, path: &str) -> String {
            format!("{}{}", self.base_url, path)
        }

        fn requests(&self) -> Vec<MockRequest> {
            self.state.requests.lock().unwrap().clone()
        }

        fn request_count(&self) -> usize {
            self.requests().len()
        }

        fn user_agents(&self) -> Vec<String> {
            self.requests()
                .into_iter()
                .filter_map(|request| request.user_agent)
                .collect()
        }

        fn authorizations(&self) -> Vec<String> {
            self.requests()
                .into_iter()
                .filter_map(|request| request.authorization)
                .collect()
        }

        fn uris(&self) -> Vec<String> {
            self.requests()
                .into_iter()
                .map(|request| request.uri)
                .collect()
        }

        fn headers(&self, name: &str) -> Vec<String> {
            self.requests()
                .into_iter()
                .flat_map(|request| request.headers.into_iter())
                .filter(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
                .map(|(_, value)| value)
                .collect()
        }
    }

    fn record_request(state: &MockMetadataState, headers: &AxumHeaderMap, uri: &Uri) -> usize {
        let request = MockRequest {
            uri: uri.to_string(),
            user_agent: headers
                .get("user-agent")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned),
            headers: headers
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| (name.as_str().to_owned(), value.to_owned()))
                })
                .collect(),
        };
        let mut requests = state.requests.lock().unwrap();
        requests.push(request);
        requests.len()
    }

    async fn mock_ok(
        State(state): State<MockMetadataState>,
        headers: AxumHeaderMap,
        uri: Uri,
    ) -> Json<serde_json::Value> {
        record_request(&state, &headers, &uri);
        Json(json!({"ok": true}))
    }

    async fn mock_flaky(
        State(state): State<MockMetadataState>,
        headers: AxumHeaderMap,
        uri: Uri,
    ) -> Response {
        let count = record_request(&state, &headers, &uri);
        if count == 1 {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "retry"})),
            )
                .into_response();
        }

        Json(json!({"ok": true})).into_response()
    }

    async fn mock_bangumi_search(
        State(state): State<MockMetadataState>,
        headers: AxumHeaderMap,
        uri: Uri,
    ) -> Json<serde_json::Value> {
        record_request(&state, &headers, &uri);
        Json(json!({
            "data": [{
                "id": 8,
                "name": "Cowboy Bebop",
                "name_cn": "星际牛仔",
                "summary": "Whatever happens, happens.",
                "date": "1998-04-03",
                "images": {"large": "https://lain.bgm.tv/pic/cover/l/8.jpg"},
                "tags": [{"name": "科幻"}],
                "rating": {"score": 9.1}
            }]
        }))
    }

    async fn mock_bangumi_subject(
        State(state): State<MockMetadataState>,
        headers: AxumHeaderMap,
        uri: Uri,
    ) -> Json<serde_json::Value> {
        record_request(&state, &headers, &uri);
        Json(json!({
            "id": 8,
            "name": "Cowboy Bebop",
            "name_cn": "星际牛仔",
            "summary": "Whatever happens, happens.",
            "date": "1998-04-03",
            "images": {"large": "https://lain.bgm.tv/pic/cover/l/8.jpg"},
            "infobox": [{"key": "动画制作", "value": "SUNRISE"}],
            "tags": [{"name": "科幻"}],
            "rating": {"score": 9.1}
        }))
    }

    async fn mock_douban_search(
        State(state): State<MockMetadataState>,
        headers: AxumHeaderMap,
        uri: Uri,
    ) -> Json<serde_json::Value> {
        record_request(&state, &headers, &uri);
        Json(json!({
            "subjects": [{
                "id": "1292052",
                "title": "肖申克的救赎",
                "original_title": "The Shawshank Redemption",
                "summary": "Hope is a good thing.",
                "year": "1994",
                "images": {"large": "https://img.doubanio.com/view/photo/l/public/p480747492.webp"},
                "genres": ["剧情", "犯罪"],
                "rating": {"average": 9.7}
            }]
        }))
    }

    async fn mock_douban_subject(
        State(state): State<MockMetadataState>,
        headers: AxumHeaderMap,
        uri: Uri,
    ) -> Json<serde_json::Value> {
        record_request(&state, &headers, &uri);
        Json(json!({
            "id": "1292052",
            "title": "肖申克的救赎",
            "original_title": "The Shawshank Redemption",
            "summary": "Hope is a good thing.",
            "year": "1994",
            "images": {"large": "https://img.doubanio.com/view/photo/l/public/p480747492.webp"},
            "genres": ["剧情", "犯罪"],
            "countries": ["美国"],
            "directors": [{"id": "1047973", "name": "Frank Darabont"}],
            "casts": [{"id": "1054521", "name": "Tim Robbins"}],
            "rating": {"average": 9.7}
        }))
    }
}
