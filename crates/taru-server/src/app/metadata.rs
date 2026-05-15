use std::env;

use serde::Serialize;
use taru_api::{
    MetadataProviderAttemptsResponse, MetadataProviderDiagnostic, MetadataProviderDiagnosticStatus,
    MetadataProviderDiagnosticsResponse, MetadataProviderRuntimeDiagnostic,
    MetadataRawResponsesResponse, PageInfo,
};
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, ExternalProvider, Job, JobId, JobKind,
    JobRepository, MediaItemId, MediaRepository, MetadataProfile, MetadataRepository, NewJob,
    NewOutboxEvent, PageRequest, Result, TaruError,
};
use taru_metadata::{
    BangumiMetadataProvider, BangumiProviderConfig, DoubanMetadataProvider, DoubanProviderConfig,
    MetadataHttpRuntimeConfig, MetadataProvider as _, MetadataProviderRegistry,
    MetadataRefreshJobInput, MetadataRefreshRequest, MetadataRefreshSummary,
    MetadataStrategyExecutor, TmdbMetadataProvider, TmdbProviderConfig,
};
use tracing::{Instrument, error, info, info_span, warn};

use super::TaruApp;
use crate::config::{
    MetadataProviderConfig, MetadataProviderHeaderConfig, MetadataProviderRuntimeConfig,
    TmdbMetadataConfig,
};

#[derive(Clone, Debug, Serialize)]
pub struct MetadataRefreshCommandOutput {
    pub job: Job,
    pub refresh: MetadataRefreshSummary,
}

impl TaruApp {
    pub async fn enqueue_metadata_refresh(&self, item_id: MediaItemId) -> Result<Job> {
        let job = self.create_metadata_refresh_job(item_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_metadata_refresh_job(job_id, item_id).await;
            }
            .instrument(info_span!(
                "metadata_refresh_background_job",
                job_id = %job_id,
                item_id = %item_id,
                resource_class = "metadata.tmdb"
            )),
        );

        Ok(job)
    }

    pub async fn refresh_item_metadata(
        &self,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshCommandOutput> {
        let job = self.create_metadata_refresh_job(item_id).await?;
        self.execute_metadata_refresh_job(job.id, item_id).await
    }

    pub(super) async fn create_metadata_refresh_job(&self, item_id: MediaItemId) -> Result<Job> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let library = self.library_for_item(item_id).await?;
        let profile = self.effective_metadata_profile(&library, item.kind)?;
        let provider = self.first_metadata_provider(&profile)?;
        let input = MetadataRefreshJobInput {
            item_id,
            provider: Some(provider.clone()),
            force: false,
            language: profile.language.clone(),
            refresh_mode: profile.refresh_mode,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize metadata refresh job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: format!("metadata.{}", provider_resource_name(&provider)),
                library_id: Some(library.id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn finish_metadata_refresh_job(&self, job_id: JobId, item_id: MediaItemId) {
        match self.execute_metadata_refresh_job(job_id, item_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    item_id = %item_id,
                    provider_key = %output.refresh.provider_key,
                    status = ?output.job.status,
                    "metadata refresh job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    item_id = %item_id,
                    error = %err,
                    "metadata refresh job failed"
                );
            }
        }
    }

    async fn execute_metadata_refresh_job(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshCommandOutput> {
        let permit = self
            .inner
            .metadata_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("metadata concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_metadata_refresh(job_id, item_id).await {
            Ok(refresh) => {
                let summary_json =
                    serde_json::to_string(&refresh).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize metadata refresh job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;
                self.record_metadata_refreshed_event(job_id, item_id, &refresh)
                    .await;

                Ok(MetadataRefreshCommandOutput { job, refresh })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        item_id = %item_id,
                        error = %update_err,
                        "failed to persist failed metadata refresh job state"
                    );
                }

                Err(err)
            }
        }
    }

    pub(super) async fn record_metadata_refreshed_event(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
        refresh: &MetadataRefreshSummary,
    ) {
        let library_id = match self.library_for_item(item_id).await {
            Ok(library) => Some(library.id),
            Err(err) => {
                warn!(
                    job_id = %job_id,
                    item_id = %item_id,
                    error = %err,
                    "failed to resolve library while recording metadata event"
                );
                None
            }
        };
        let payload = serde_json::json!({
            "job_id": job_id,
            "item_id": item_id,
            "provider": &refresh.provider,
            "selected_provider": &refresh.selected_provider,
            "provider_key": &refresh.provider_key,
            "matched_by": refresh.matched_by,
            "refresh_mode": refresh.refresh_mode,
            "updated": refresh.updated,
            "attempted_providers": refresh.attempted_providers.len(),
        });
        self.record_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::ItemMetadataRefreshed,
            subject: DomainEventSubject::Item(item_id),
            library_id,
            source_id: None,
            idempotency_key: format!("item.metadata_refreshed:{job_id}:{item_id}"),
            payload_json: payload.to_string(),
        })
        .await;
    }

    pub(super) async fn run_metadata_refresh(
        &self,
        job_id: JobId,
        item_id: MediaItemId,
    ) -> Result<MetadataRefreshSummary> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let library = self.library_for_item(item_id).await?;
        let profile = self.effective_metadata_profile(&library, item.kind)?;
        let registry = self.metadata_provider_registry();
        let executor = MetadataStrategyExecutor::new(registry, self.inner.store.clone());

        executor
            .refresh_item(MetadataRefreshRequest {
                job_id,
                item_id,
                profile,
                force: false,
            })
            .await
    }

    pub async fn list_metadata_provider_attempts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<MetadataProviderAttemptsResponse> {
        self.ensure_metadata_item_exists(item_id).await?;
        let attempts = self
            .inner
            .store
            .list_metadata_provider_attempts_for_item(item_id, page)
            .await?;

        Ok(MetadataProviderAttemptsResponse {
            item_id,
            page: PageInfo::new(page, attempts.len()),
            attempts,
        })
    }

    pub async fn list_provider_raw_responses_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<MetadataRawResponsesResponse> {
        self.ensure_metadata_item_exists(item_id).await?;
        let responses = self
            .inner
            .store
            .list_provider_raw_responses(item_id, page)
            .await?;

        Ok(MetadataRawResponsesResponse {
            item_id,
            page: PageInfo::new(page, responses.len()),
            responses,
        })
    }

    pub fn list_metadata_provider_diagnostics(&self) -> MetadataProviderDiagnosticsResponse {
        MetadataProviderDiagnosticsResponse {
            providers: self.metadata_provider_diagnostics(),
        }
    }

    async fn ensure_metadata_item_exists(&self, item_id: MediaItemId) -> Result<()> {
        self.inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;

        Ok(())
    }

    fn effective_metadata_profile(
        &self,
        library: &taru_core::Library,
        item_kind: taru_core::MediaKind,
    ) -> Result<MetadataProfile> {
        let mut profile = library.options.metadata_profile.clone();

        if !profile.item_kinds.is_empty()
            && !profile.item_kinds.contains(&item_kind)
            && !profile.item_kinds.contains(&taru_core::MediaKind::Unknown)
        {
            return Err(TaruError::Unsupported(
                "library metadata profile does not apply to this item kind",
            ));
        }

        if profile.language.is_none() && !self.config().metadata.tmdb.language.trim().is_empty() {
            profile.language = Some(self.config().metadata.tmdb.language.clone());
        }

        Ok(profile)
    }

    fn first_metadata_provider(&self, profile: &MetadataProfile) -> Result<ExternalProvider> {
        let Some(provider) = profile.metadata_providers.first().cloned() else {
            return Err(TaruError::InvalidInput {
                message: "library metadata profile does not enable any metadata provider"
                    .to_owned(),
            });
        };

        Ok(provider)
    }

    fn metadata_provider_registry(&self) -> MetadataProviderRegistry {
        let mut registry = MetadataProviderRegistry::new();

        if self.config().metadata.providers.is_empty() {
            register_legacy_tmdb_provider(
                &mut registry,
                &self.config().metadata.tmdb,
                &self.config().metadata.runtime,
            );
            return registry;
        }

        for provider in &self.config().metadata.providers {
            match self.build_metadata_provider(provider) {
                Ok(BuiltMetadataProvider::Tmdb(provider)) => {
                    registry.register(provider);
                }
                Ok(BuiltMetadataProvider::Bangumi(provider)) => {
                    registry.register(provider);
                }
                Ok(BuiltMetadataProvider::Douban(provider)) => {
                    registry.register(provider);
                }
                Err(MetadataProviderBuildError::Disabled(provider, message)) => {
                    registry.register_disabled(provider, message);
                }
                Err(MetadataProviderBuildError::Unavailable(provider, message)) => {
                    registry.register_unavailable(provider, message);
                }
            }
        }

        registry
    }

    fn metadata_provider_diagnostics(&self) -> Vec<MetadataProviderDiagnostic> {
        if self.config().metadata.providers.is_empty() {
            return vec![self.legacy_tmdb_provider_diagnostic()];
        }

        self.config()
            .metadata
            .providers
            .iter()
            .map(|settings| self.configured_provider_diagnostic(settings))
            .collect()
    }

    fn legacy_tmdb_provider_diagnostic(&self) -> MetadataProviderDiagnostic {
        let runtime = provider_runtime_diagnostic(&self.config().metadata.runtime);

        match build_legacy_tmdb_provider(
            &self.config().metadata.tmdb,
            &self.config().metadata.runtime,
        ) {
            Ok(provider) => available_provider_diagnostic(
                provider.provider(),
                Some(provider.provider_name().to_owned()),
                runtime,
            ),
            Err(err) => build_error_diagnostic(err, runtime),
        }
    }

    fn configured_provider_diagnostic(
        &self,
        settings: &MetadataProviderConfig,
    ) -> MetadataProviderDiagnostic {
        let runtime = provider_runtime_diagnostic(
            settings
                .runtime
                .as_ref()
                .unwrap_or(&self.config().metadata.runtime),
        );

        match self.build_metadata_provider(settings) {
            Ok(provider) => available_provider_diagnostic(
                provider.provider(),
                Some(provider.provider_name().to_owned()),
                runtime,
            ),
            Err(err) => build_error_diagnostic(err, runtime),
        }
    }

    fn build_metadata_provider(
        &self,
        settings: &MetadataProviderConfig,
    ) -> std::result::Result<BuiltMetadataProvider, MetadataProviderBuildError> {
        if !settings.enabled {
            return Err(MetadataProviderBuildError::Disabled(
                settings.provider.clone(),
                format!(
                    "{} metadata provider is disabled in config",
                    provider_resource_name(&settings.provider).to_uppercase()
                ),
            ));
        }

        match settings.provider {
            ExternalProvider::Tmdb => {
                build_tmdb_provider(settings, &self.config().metadata.runtime)
                    .map(BuiltMetadataProvider::Tmdb)
            }
            ExternalProvider::Bangumi => {
                build_bangumi_provider(settings, &self.config().metadata.runtime)
                    .map(BuiltMetadataProvider::Bangumi)
            }
            ExternalProvider::Douban => {
                build_douban_provider(settings, &self.config().metadata.runtime)
                    .map(BuiltMetadataProvider::Douban)
            }
            _ => Err(MetadataProviderBuildError::Unavailable(
                settings.provider.clone(),
                format!(
                    "{} metadata provider is not implemented",
                    provider_resource_name(&settings.provider)
                ),
            )),
        }
    }
}

fn provider_resource_name(provider: &ExternalProvider) -> &str {
    match provider {
        ExternalProvider::Tmdb => "tmdb",
        ExternalProvider::Douban => "douban",
        ExternalProvider::Bangumi => "bangumi",
        ExternalProvider::Imdb => "imdb",
        ExternalProvider::Local => "local",
        ExternalProvider::Other(_) => "other",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MetadataProviderBuildError {
    Disabled(ExternalProvider, String),
    Unavailable(ExternalProvider, String),
}

enum BuiltMetadataProvider {
    Tmdb(TmdbMetadataProvider),
    Bangumi(BangumiMetadataProvider),
    Douban(DoubanMetadataProvider),
}

impl BuiltMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        match self {
            Self::Tmdb(provider) => provider.provider(),
            Self::Bangumi(provider) => provider.provider(),
            Self::Douban(provider) => provider.provider(),
        }
    }

    fn provider_name(&self) -> &'static str {
        match self {
            Self::Tmdb(provider) => provider.provider_name(),
            Self::Bangumi(provider) => provider.provider_name(),
            Self::Douban(provider) => provider.provider_name(),
        }
    }
}

fn available_provider_diagnostic(
    provider: ExternalProvider,
    provider_name: Option<String>,
    runtime: MetadataProviderRuntimeDiagnostic,
) -> MetadataProviderDiagnostic {
    MetadataProviderDiagnostic {
        provider,
        status: MetadataProviderDiagnosticStatus::Available,
        provider_name,
        reason: None,
        runtime,
    }
}

fn build_error_diagnostic(
    error: MetadataProviderBuildError,
    runtime: MetadataProviderRuntimeDiagnostic,
) -> MetadataProviderDiagnostic {
    match error {
        MetadataProviderBuildError::Disabled(provider, reason) => MetadataProviderDiagnostic {
            provider,
            status: MetadataProviderDiagnosticStatus::Disabled,
            provider_name: None,
            reason: Some(reason),
            runtime,
        },
        MetadataProviderBuildError::Unavailable(provider, reason) => MetadataProviderDiagnostic {
            provider,
            status: MetadataProviderDiagnosticStatus::Unavailable,
            provider_name: None,
            reason: Some(reason),
            runtime,
        },
    }
}

fn provider_runtime_diagnostic(
    config: &MetadataProviderRuntimeConfig,
) -> MetadataProviderRuntimeDiagnostic {
    MetadataProviderRuntimeDiagnostic {
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
        min_interval_ms: config.min_interval_ms,
        concurrency: config.concurrency,
        user_agent: config.user_agent.clone(),
        proxy_configured: config
            .proxy
            .as_ref()
            .is_some_and(|proxy| !proxy.trim().is_empty()),
        circuit_breaker_failures: config.circuit_breaker_failures,
    }
}

fn register_legacy_tmdb_provider(
    registry: &mut MetadataProviderRegistry,
    settings: &TmdbMetadataConfig,
    runtime: &MetadataProviderRuntimeConfig,
) {
    match build_legacy_tmdb_provider(settings, runtime) {
        Ok(provider) => {
            registry.register(provider);
        }
        Err(MetadataProviderBuildError::Disabled(provider, message)) => {
            registry.register_disabled(provider, message);
        }
        Err(MetadataProviderBuildError::Unavailable(provider, message)) => {
            registry.register_unavailable(provider, message);
        }
    }
}

fn build_legacy_tmdb_provider(
    settings: &TmdbMetadataConfig,
    runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<TmdbMetadataProvider, MetadataProviderBuildError> {
    if !settings.enabled {
        return Err(MetadataProviderBuildError::Disabled(
            ExternalProvider::Tmdb,
            "TMDB metadata provider is disabled in config".to_owned(),
        ));
    }

    let token = resolve_required_secret(
        ExternalProvider::Tmdb,
        &settings.access_token_env,
        "access token",
    )?;
    let mut config = TmdbProviderConfig::new(token);
    config.api_base_url = settings.api_base_url.clone();
    config.image_base_url = settings.image_base_url.clone();
    config.language = settings.language.clone();
    config.include_adult = settings.include_adult;
    config.runtime = runtime_config(runtime);

    TmdbMetadataProvider::new(config).map_err(|err| {
        MetadataProviderBuildError::Unavailable(ExternalProvider::Tmdb, err.to_string())
    })
}

fn build_tmdb_provider(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<TmdbMetadataProvider, MetadataProviderBuildError> {
    let token_env = settings
        .token_env
        .as_deref()
        .unwrap_or("TMDB_READ_ACCESS_TOKEN");
    let token = resolve_required_secret(ExternalProvider::Tmdb, token_env, "access token")?;
    let mut config = TmdbProviderConfig::new(token);
    if let Some(api_base_url) = settings.api_base_url.as_ref() {
        config.api_base_url = api_base_url.clone();
    }
    if let Some(image_base_url) = settings.image_base_url.as_ref() {
        config.image_base_url = image_base_url.clone();
    }
    if let Some(language) = settings.language.as_ref() {
        config.language = language.clone();
    }
    config.include_adult = settings.include_adult;
    config.runtime = runtime_config(settings.runtime.as_ref().unwrap_or(inherited_runtime));

    TmdbMetadataProvider::new(config).map_err(|err| {
        MetadataProviderBuildError::Unavailable(ExternalProvider::Tmdb, err.to_string())
    })
}

fn build_bangumi_provider(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<BangumiMetadataProvider, MetadataProviderBuildError> {
    let access_token = settings
        .token_env
        .as_deref()
        .map(|env_name| {
            resolve_required_secret(ExternalProvider::Bangumi, env_name, "access token")
        })
        .transpose()?;
    let mut config = BangumiProviderConfig {
        access_token,
        include_nsfw: settings.include_adult,
        runtime: runtime_config(settings.runtime.as_ref().unwrap_or(inherited_runtime)),
        ..BangumiProviderConfig::default()
    };
    if let Some(api_base_url) = settings.api_base_url.as_ref() {
        config.api_base_url = api_base_url.clone();
    }
    if let Some(image_base_url) = settings.image_base_url.as_ref() {
        config.image_base_url = image_base_url.clone();
    }

    BangumiMetadataProvider::new(config).map_err(|err| {
        MetadataProviderBuildError::Unavailable(ExternalProvider::Bangumi, err.to_string())
    })
}

fn build_douban_provider(
    settings: &MetadataProviderConfig,
    inherited_runtime: &MetadataProviderRuntimeConfig,
) -> std::result::Result<DoubanMetadataProvider, MetadataProviderBuildError> {
    let api_key = settings
        .api_key_env
        .as_deref()
        .map(|env_name| resolve_required_secret(ExternalProvider::Douban, env_name, "API key"))
        .transpose()?;
    let mut config = DoubanProviderConfig {
        api_key,
        image_base_url: settings.image_base_url.clone(),
        runtime: runtime_config(settings.runtime.as_ref().unwrap_or(inherited_runtime)),
        headers: resolve_headers(ExternalProvider::Douban, &settings.headers)?,
        ..DoubanProviderConfig::default()
    };
    if let Some(api_base_url) = settings.api_base_url.as_ref() {
        config.api_base_url = api_base_url.clone();
    }

    DoubanMetadataProvider::new(config).map_err(|err| {
        MetadataProviderBuildError::Unavailable(ExternalProvider::Douban, err.to_string())
    })
}

fn runtime_config(config: &MetadataProviderRuntimeConfig) -> MetadataHttpRuntimeConfig {
    MetadataHttpRuntimeConfig {
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
        min_interval_ms: config.min_interval_ms,
        concurrency: config.concurrency,
        user_agent: config.user_agent.clone(),
        proxy: config.proxy.clone(),
        circuit_breaker_failures: config.circuit_breaker_failures,
    }
}

fn resolve_required_secret(
    provider: ExternalProvider,
    env_name: &str,
    label: &str,
) -> std::result::Result<String, MetadataProviderBuildError> {
    let value = env::var(env_name).map_err(|err| {
        MetadataProviderBuildError::Unavailable(
            provider.clone(),
            format!(
                "failed to read {} {label} from environment variable {env_name}: {err}",
                provider_resource_name(&provider).to_uppercase()
            ),
        )
    })?;

    if value.trim().is_empty() {
        return Err(MetadataProviderBuildError::Unavailable(
            provider.clone(),
            format!(
                "{} {label} environment variable {env_name} is empty",
                provider_resource_name(&provider).to_uppercase()
            ),
        ));
    }

    Ok(value)
}

fn resolve_headers(
    provider: ExternalProvider,
    headers: &[MetadataProviderHeaderConfig],
) -> std::result::Result<Vec<(String, String)>, MetadataProviderBuildError> {
    headers
        .iter()
        .map(|header| {
            let value = match (&header.value, &header.value_env) {
                (Some(value), None) => Ok(value.clone()),
                (None, Some(env_name)) => resolve_required_secret(
                    provider.clone(),
                    env_name,
                    &format!("header {}", header.name),
                ),
                (Some(_), Some(_)) => Err(MetadataProviderBuildError::Unavailable(
                    provider.clone(),
                    format!(
                        "{} metadata provider header {} cannot set both value and value_env",
                        provider_resource_name(&provider).to_uppercase(),
                        header.name
                    ),
                )),
                (None, None) => Err(MetadataProviderBuildError::Unavailable(
                    provider.clone(),
                    format!(
                        "{} metadata provider header {} must set value or value_env",
                        provider_resource_name(&provider).to_uppercase(),
                        header.name
                    ),
                )),
            }?;

            Ok((header.name.clone(), value))
        })
        .collect()
}
