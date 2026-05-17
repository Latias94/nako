use serde::{Deserialize, Serialize};
use taru_catalog::{CatalogHydrationPort, hydrate_item_catalog};
use taru_core::{
    ExternalProvider, JobId, LibraryItemRepository, LibraryItemState, MediaItem, MediaItemId,
    MediaKind, MediaRepository, MetadataMatchKind, MetadataProfile, MetadataProviderAttemptId,
    MetadataProviderAttemptStatus, MetadataProviderErrorClass, MetadataRefreshMode,
    MetadataRepository, MetadataSource, NewMetadataProviderAttempt, PageRequest, ProviderMapping,
    ProviderMappingId, ProviderMappingRepository, ProviderMappingStatus, ProviderRawResponse,
    ProviderSubject, ProviderSubjectId, ProviderSubjectKind, Result, TaruError,
};

use crate::{
    MetadataFetchRequest, MetadataFetchResult, MetadataLookup, MetadataMergePolicy,
    MetadataProvider, MetadataProviderRegistry,
    providers::{now_utc_string, release_year},
    registry::RegisteredMetadataProvider,
};
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
    R: CatalogHydrationPort
        + LibraryItemRepository
        + MediaRepository
        + MetadataRepository
        + ProviderMappingRepository,
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
    R: CatalogHydrationPort
        + LibraryItemRepository
        + MediaRepository
        + MetadataRepository
        + ProviderMappingRepository,
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
        Err(MetadataProviderRefreshError::ProviderFailed(message)) => {
            let error_class = classify_provider_failure_message(message);

            MetadataProviderAttempt {
                provider,
                status: attempt_status_for_error_class(error_class),
                message: Some(message.clone()),
                provider_key: None,
                matched_by: None,
                error_class: Some(error_class),
            }
        }
        Err(MetadataProviderRefreshError::Fatal(err)) => {
            let error_class = classify_provider_error_class(err);

            MetadataProviderAttempt {
                provider,
                status: attempt_status_for_error_class(error_class),
                message: Some(err.to_string()),
                provider_key: None,
                matched_by: None,
                error_class: Some(error_class),
            }
        }
    }
}

fn attempt_status_for_error_class(
    error_class: MetadataProviderErrorClass,
) -> MetadataProviderAttemptStatus {
    match error_class {
        MetadataProviderErrorClass::RateLimited => MetadataProviderAttemptStatus::RateLimited,
        _ => MetadataProviderAttemptStatus::Failed,
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
        MetadataProviderAttemptStatus::RateLimited => Some(MetadataProviderErrorClass::RateLimited),
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
    R: LibraryItemRepository + MediaRepository + MetadataRepository + ProviderMappingRepository,
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
        .apply_metadata_refresh(
            &updated_item,
            &ProviderRawResponse {
                item_id: updated_item.id,
                provider: fetched.provider.clone(),
                provider_key: fetched.provider_key.clone(),
                fetched_at: now_utc_string().map_err(MetadataProviderRefreshError::Fatal)?,
                body_json: fetched.raw_json.clone(),
            },
        )
        .await
        .map_err(MetadataProviderRefreshError::Fatal)?;
    accept_provider_mapping(repository, &updated_item, &fetched)
        .await
        .map_err(MetadataProviderRefreshError::Fatal)?;
    confirm_item_source_libraries(repository, updated_item.id)
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

async fn accept_provider_mapping<R>(
    repository: &R,
    item: &MediaItem,
    fetched: &MetadataFetchResult,
) -> Result<()>
where
    R: ProviderMappingRepository,
{
    let subject_kind = provider_subject_kind_for_item(item.kind);
    let subject = match repository
        .find_provider_subject(&fetched.provider, &subject_kind, &fetched.provider_key)
        .await?
    {
        Some(existing) => ProviderSubject {
            title: Some(fetched.metadata.title.clone()),
            release_year: release_year(fetched.metadata.release_date.as_deref()).map(i32::from),
            ..existing
        },
        None => ProviderSubject {
            id: ProviderSubjectId::new(),
            provider: fetched.provider.clone(),
            subject_kind,
            subject_key: fetched.provider_key.clone(),
            title: Some(fetched.metadata.title.clone()),
            release_year: release_year(fetched.metadata.release_date.as_deref()).map(i32::from),
            locale: None,
        },
    };
    repository.upsert_provider_subject(&subject).await?;

    let mapping_id = existing_mapping_id(repository, item.id, subject.id).await?;
    repository
        .upsert_provider_mapping(&ProviderMapping {
            id: mapping_id.unwrap_or_else(ProviderMappingId::new),
            item_id: item.id,
            subject_id: subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: Some(1_000),
            source: MetadataSource::Provider(fetched.provider.clone()),
        })
        .await
}

async fn existing_mapping_id<R>(
    repository: &R,
    item_id: MediaItemId,
    subject_id: ProviderSubjectId,
) -> Result<Option<ProviderMappingId>>
where
    R: ProviderMappingRepository,
{
    let mut offset = 0;

    loop {
        let mappings = repository
            .list_provider_mappings_for_item(
                item_id,
                PageRequest {
                    limit: PageRequest::MAX_LIMIT,
                    offset,
                },
            )
            .await?;
        let returned = mappings.len();
        if let Some(mapping) = mappings
            .into_iter()
            .find(|mapping| mapping.subject_id == subject_id)
        {
            return Ok(Some(mapping.id));
        }
        if returned < PageRequest::MAX_LIMIT as usize {
            return Ok(None);
        }
        offset += u64::from(PageRequest::MAX_LIMIT);
    }
}

async fn confirm_item_source_libraries<R>(repository: &R, item_id: MediaItemId) -> Result<()>
where
    R: LibraryItemRepository,
{
    for state in repository
        .list_library_item_states_for_item(item_id)
        .await?
    {
        repository
            .upsert_library_item_state(&LibraryItemState {
                library_id: state.library_id,
                item_id,
                provisional: false,
            })
            .await?;
    }

    Ok(())
}

fn provider_subject_kind_for_item(kind: MediaKind) -> ProviderSubjectKind {
    match kind {
        MediaKind::Movie => ProviderSubjectKind::Movie,
        MediaKind::Series => ProviderSubjectKind::Series,
        MediaKind::Season => ProviderSubjectKind::Season,
        MediaKind::Episode => ProviderSubjectKind::Episode,
        MediaKind::Collection => ProviderSubjectKind::Collection,
        MediaKind::Extra | MediaKind::Unknown => ProviderSubjectKind::Subject,
    }
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
