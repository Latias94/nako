use std::{
    sync::Arc,
    sync::Mutex as StdMutex,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

use reqwest::header::HeaderMap;
use serde::Serialize;
use taru_core::{Result, TaruError};
use time::OffsetDateTime;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{sleep, timeout};

use crate::providers::{
    provider_parse_error, provider_request_error, retry_delay, truncate_message,
};

const DEFAULT_PROVIDER_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_PROVIDER_MAX_ATTEMPTS: u32 = 2;
const DEFAULT_PROVIDER_MIN_INTERVAL_MS: u64 = 250;
const DEFAULT_PROVIDER_CONCURRENCY: usize = 1;
const DEFAULT_PROVIDER_CIRCUIT_BREAKER_FAILURES: u32 = 5;
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
    last_error: Arc<StdMutex<Option<String>>>,
    last_rate_limit_wait_ms: Arc<AtomicU64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MetadataHttpRuntimeStatus {
    pub circuit_open: bool,
    pub consecutive_failures: u64,
    pub last_error: Option<String>,
    pub last_rate_limit_wait_ms: u64,
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
                    message: format!("invalid metadata provider proxy configuration: {err}"),
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
            last_error: Arc::new(StdMutex::new(None)),
            last_rate_limit_wait_ms: Arc::new(AtomicU64::new(0)),
        })
    }

    #[must_use]
    pub fn config(&self) -> &MetadataHttpRuntimeConfig {
        &self.config
    }

    #[must_use]
    pub fn status(&self) -> MetadataHttpRuntimeStatus {
        MetadataHttpRuntimeStatus {
            circuit_open: self.circuit_open.load(Ordering::SeqCst),
            consecutive_failures: self.consecutive_failures.load(Ordering::SeqCst),
            last_error: self.last_error.lock().ok().and_then(|error| error.clone()),
            last_rate_limit_wait_ms: self.last_rate_limit_wait_ms.load(Ordering::SeqCst),
        }
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
                self.circuit_open.store(false, Ordering::SeqCst);
                self.set_last_error(None);
                let value = match serde_json::from_str(&text) {
                    Ok(value) => value,
                    Err(err) => {
                        let error = provider_parse_error(provider, operation, err);
                        self.record_failure(Some(error.to_string()));
                        return Err(error);
                    }
                };
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
                self.record_failure(Some(error.to_string()));
                return Err(error);
            }

            last_error = Some(error);
            if attempt < attempts {
                sleep(retry_delay(attempt)).await;
            }
        }

        let error = last_error.unwrap_or_else(|| TaruError::Provider {
            provider: provider.to_owned(),
            message: format!("{operation} failed without a provider response"),
        });
        self.record_failure(Some(error.to_string()));
        Err(error)
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
            self.last_rate_limit_wait_ms
                .store(duration_millis(wait), Ordering::SeqCst);
            sleep(wait).await;
        } else {
            self.last_rate_limit_wait_ms.store(0, Ordering::SeqCst);
        }
        *next_allowed = OffsetDateTime::now_utc() + min_interval;

        Ok(())
    }

    fn record_failure(&self, error: Option<String>) {
        self.set_last_error(error);
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        let threshold = u64::from(self.config.circuit_breaker_failures);
        if threshold > 0 && failures >= threshold {
            self.circuit_open.store(true, Ordering::SeqCst);
        }
    }

    fn set_last_error(&self, error: Option<String>) {
        if let Ok(mut last_error) = self.last_error.lock() {
            *last_error = error;
        }
    }
}

fn default_metadata_user_agent() -> String {
    format!("taru/{}", env!("CARGO_PKG_VERSION"))
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
