use std::env;

use serde::Serialize;
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, ExternalProvider, Job, JobId, JobKind,
    JobRepository, MediaItemId, MediaRepository, MetadataProfile, NewJob, NewOutboxEvent, Result,
    TaruError,
};
use taru_metadata::{
    MetadataProviderRegistry, MetadataRefreshJobInput, MetadataRefreshRequest,
    MetadataRefreshSummary, MetadataStrategyExecutor, TmdbMetadataProvider, TmdbProviderConfig,
};
use tracing::{Instrument, error, info, info_span, warn};

use super::TaruApp;

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
        match self.tmdb_provider() {
            Ok(provider) => {
                registry.register(provider);
            }
            Err(TmdbProviderBuildError::Disabled(message)) => {
                registry.register_disabled(ExternalProvider::Tmdb, message);
            }
            Err(TmdbProviderBuildError::Unavailable(message)) => {
                registry.register_unavailable(ExternalProvider::Tmdb, message);
            }
        }

        registry
    }

    fn tmdb_provider(&self) -> std::result::Result<TmdbMetadataProvider, TmdbProviderBuildError> {
        let settings = &self.config().metadata.tmdb;

        if !settings.enabled {
            return Err(TmdbProviderBuildError::Disabled(
                "TMDB metadata provider is disabled in config".to_owned(),
            ));
        }

        let token = env::var(&settings.access_token_env).map_err(|err| {
            TmdbProviderBuildError::Unavailable(format!(
                "failed to read TMDB access token from environment variable {}: {err}",
                settings.access_token_env
            ))
        })?;

        if token.trim().is_empty() {
            return Err(TmdbProviderBuildError::Unavailable(format!(
                "TMDB access token environment variable {} is empty",
                settings.access_token_env
            )));
        }

        let mut config = TmdbProviderConfig::new(token);
        config.api_base_url = settings.api_base_url.clone();
        config.image_base_url = settings.image_base_url.clone();
        config.language = settings.language.clone();
        config.include_adult = settings.include_adult;

        Ok(TmdbMetadataProvider::new(config))
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
enum TmdbProviderBuildError {
    Disabled(String),
    Unavailable(String),
}
