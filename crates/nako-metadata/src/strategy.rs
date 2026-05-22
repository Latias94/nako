use async_trait::async_trait;
use nako_catalog::{CatalogHydrationPort, hydrate_item_catalog};
use nako_core::{
    ExternalProvider, JobId, MediaItem, MediaItemId, MediaKind, MediaRepository,
    MetadataCandidateSubject, MetadataFieldLock, MetadataMatchKind, MetadataProfile,
    MetadataProviderAttemptStatus, MetadataProviderErrorClass, MetadataRefreshMode,
    MetadataRefreshPersistenceCommit, MetadataRefreshProviderMappingCommit, MetadataRepository,
    MetadataSource, NakoError, NewMetadataProviderAttempt, PageRequest, ProviderMappingId,
    ProviderMappingRepository, ProviderRawResponse, ProviderSubject, ProviderSubjectId,
    ProviderSubjectKind, Result,
};
use serde::{Deserialize, Serialize};

use crate::{
    MetadataProvider, MetadataProviderRegistry,
    provider_attempt::{
        MetadataProviderRefreshError, record_skipped_provider_attempt,
        run_available_provider_attempt, summarize_attempts,
    },
    providers::release_year,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshSnapshot {
    pub item: MediaItem,
    pub field_locks: Vec<MetadataFieldLock>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRefreshCommit {
    pub item: MediaItem,
    pub raw_response: ProviderRawResponse,
    pub provider_subject: Option<MetadataCandidateSubject>,
}

#[async_trait]
pub trait MetadataRefreshPort: Send + Sync {
    async fn load_refresh_snapshot(&self, item_id: MediaItemId) -> Result<MetadataRefreshSnapshot>;

    async fn commit_refresh(&self, commit: MetadataRefreshCommit) -> Result<()>;
}

#[async_trait]
pub trait MetadataAttemptPort: Send + Sync {
    async fn record_metadata_attempt(&self, attempt: NewMetadataProviderAttempt) -> Result<()>;
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
    R: CatalogHydrationPort + MetadataRefreshPort + MetadataAttemptPort,
{
    pub async fn refresh_item(
        &self,
        request: MetadataRefreshRequest,
    ) -> Result<MetadataRefreshSummary> {
        validate_refresh_profile(&request.profile)?;

        let snapshot = self
            .repository
            .load_refresh_snapshot(request.item_id)
            .await?;
        let mut attempts = Vec::new();

        for provider_id in &request.profile.metadata_providers {
            match self.registry.get(provider_id) {
                Some(RegisteredMetadataProvider::Available(provider)) => {
                    let outcome = run_available_provider_attempt(
                        &self.repository,
                        provider.as_ref(),
                        &request,
                        &snapshot,
                    )
                    .await?;
                    attempts.push(outcome.attempt);

                    match outcome.result {
                        Ok(success) => {
                            self.repository
                                .commit_refresh(success.commit.clone())
                                .await?;
                            hydrate_item_catalog(
                                &self.repository,
                                success.commit.item.id,
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
                    let attempt = record_skipped_provider_attempt(
                        &self.repository,
                        request.job_id,
                        request.item_id,
                        provider_id.clone(),
                        MetadataProviderAttemptStatus::SkippedDisabled,
                        reason.clone(),
                    )
                    .await?;
                    attempts.push(attempt);
                }
                Some(RegisteredMetadataProvider::Unavailable { reason }) => {
                    let attempt = record_skipped_provider_attempt(
                        &self.repository,
                        request.job_id,
                        request.item_id,
                        provider_id.clone(),
                        MetadataProviderAttemptStatus::SkippedUnavailable,
                        reason.clone(),
                    )
                    .await?;
                    attempts.push(attempt);
                }
                None => {
                    let attempt = record_skipped_provider_attempt(
                        &self.repository,
                        request.job_id,
                        request.item_id,
                        provider_id.clone(),
                        MetadataProviderAttemptStatus::NotImplemented,
                        "metadata provider is not registered".to_owned(),
                    )
                    .await?;
                    attempts.push(attempt);
                }
            }
        }

        Err(NakoError::Provider {
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
    R: CatalogHydrationPort + MetadataRefreshPort + MetadataAttemptPort,
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
            return Err(NakoError::Unsupported(
                "metadata refresh profile does not enable this provider",
            ));
        }

        validate_refresh_profile(&request.profile)?;

        let snapshot = self
            .repository
            .load_refresh_snapshot(request.item_id)
            .await?;

        let outcome =
            run_available_provider_attempt(&self.repository, &self.provider, &request, &snapshot)
                .await?;
        let success = outcome
            .result
            .map_err(MetadataProviderRefreshError::into_error)?;
        self.repository
            .commit_refresh(success.commit.clone())
            .await?;
        hydrate_item_catalog(
            &self.repository,
            success.commit.item.id,
            MetadataSource::Provider(success.provider.clone()),
        )
        .await?;

        Ok(success.into_summary(request.job_id, vec![outcome.attempt]))
    }
}

#[async_trait]
impl<T> MetadataRefreshPort for T
where
    T: MediaRepository + MetadataRepository + ProviderMappingRepository,
{
    async fn load_refresh_snapshot(&self, item_id: MediaItemId) -> Result<MetadataRefreshSnapshot> {
        let item = self
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let field_locks = self.list_field_locks(item.id).await?;

        Ok(MetadataRefreshSnapshot { item, field_locks })
    }

    async fn commit_refresh(&self, commit: MetadataRefreshCommit) -> Result<()> {
        let provider_mapping = accepted_provider_mapping_commit(
            self,
            &commit.item,
            &commit.raw_response,
            commit.provider_subject.as_ref(),
        )
        .await?;

        self.commit_metadata_refresh(&MetadataRefreshPersistenceCommit {
            item: commit.item,
            raw_response: commit.raw_response,
            provider_mapping,
        })
        .await?;

        Ok(())
    }
}

#[async_trait]
impl<T> MetadataAttemptPort for T
where
    T: MetadataRepository,
{
    async fn record_metadata_attempt(&self, attempt: NewMetadataProviderAttempt) -> Result<()> {
        self.insert_metadata_provider_attempt(attempt).await
    }
}

async fn accepted_provider_mapping_commit<R>(
    repository: &R,
    item: &MediaItem,
    raw_response: &ProviderRawResponse,
    candidate_subject: Option<&MetadataCandidateSubject>,
) -> Result<MetadataRefreshProviderMappingCommit>
where
    R: ProviderMappingRepository,
{
    let provider = candidate_subject
        .map(|subject| subject.provider.clone())
        .unwrap_or_else(|| raw_response.provider.clone());
    let subject_kind = candidate_subject
        .map(|subject| subject.subject_kind.clone())
        .unwrap_or_else(|| provider_subject_kind_for_item(item.kind));
    let subject_key = candidate_subject
        .map(|subject| subject.subject_key.clone())
        .unwrap_or_else(|| raw_response.provider_key.clone());
    let title = candidate_subject
        .and_then(|subject| subject.title.clone())
        .or_else(|| Some(item.metadata.title.clone()));
    let release_year = candidate_subject
        .and_then(|subject| subject.release_year)
        .or_else(|| release_year(item.metadata.release_date.as_deref()).map(i32::from));
    let locale = candidate_subject.and_then(|subject| subject.locale.clone());
    let subject = match repository
        .find_provider_subject(&provider, &subject_kind, &subject_key)
        .await?
    {
        Some(existing) => ProviderSubject {
            title,
            release_year,
            locale,
            ..existing
        },
        None => ProviderSubject {
            id: ProviderSubjectId::new(),
            provider,
            subject_kind,
            subject_key,
            title,
            release_year,
            locale,
        },
    };
    let mapping_id = existing_mapping_id(repository, item.id, subject.id).await?;

    Ok(MetadataRefreshProviderMappingCommit {
        id: mapping_id,
        subject,
        confidence_milli: Some(1_000),
        source: MetadataSource::Provider(raw_response.provider.clone()),
    })
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

fn validate_refresh_profile(profile: &MetadataProfile) -> Result<()> {
    if profile.refresh_mode == MetadataRefreshMode::None {
        return Err(NakoError::Unsupported(
            "metadata refresh profile disables metadata refresh",
        ));
    }

    if profile.refresh_mode == MetadataRefreshMode::ValidationOnly {
        return Err(NakoError::Unsupported(
            "metadata refresh validation-only mode is not implemented yet",
        ));
    }

    if profile.metadata_providers.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "library metadata profile does not enable any metadata provider".to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod port_tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use crate::{MetadataCandidate, MetadataFetchRequest, MetadataFetchResult, MetadataLookup};
    use async_trait::async_trait;
    use nako_catalog::CatalogHydrationSummary;
    use nako_core::{
        CanonicalMetadata, ExternalId, ExternalProvider, JobId, MediaItem, MediaItemId, MediaKind,
        MetadataCandidateGraph, MetadataCandidateSource, MetadataField, MetadataFieldLock,
        MetadataProfile, MetadataSource, Result,
    };

    use super::*;

    #[derive(Clone, Debug)]
    struct WorkflowPort(Arc<FakeMetadataWorkflowPort>);

    impl WorkflowPort {
        fn new(inner: FakeMetadataWorkflowPort) -> Self {
            Self(Arc::new(inner))
        }

        fn inner(&self) -> &Arc<FakeMetadataWorkflowPort> {
            &self.0
        }
    }

    impl std::ops::Deref for WorkflowPort {
        type Target = Arc<FakeMetadataWorkflowPort>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    struct FakeMetadataWorkflowPort {
        refresh_snapshot: MetadataRefreshSnapshot,
        refresh_commit: Mutex<Option<MetadataRefreshCommit>>,
        hydration_requests: Mutex<Vec<(MediaItemId, MetadataSource)>>,
        load_refresh_calls: AtomicUsize,
        commit_refresh_calls: AtomicUsize,
        hydrate_catalog_calls: AtomicUsize,
    }

    #[async_trait]
    impl MetadataRefreshPort for WorkflowPort {
        async fn load_refresh_snapshot(
            &self,
            item_id: MediaItemId,
        ) -> Result<MetadataRefreshSnapshot> {
            assert_eq!(self.0.refresh_snapshot.item.id, item_id);
            self.0.load_refresh_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.0.refresh_snapshot.clone())
        }

        async fn commit_refresh(&self, commit: MetadataRefreshCommit) -> Result<()> {
            self.0.commit_refresh_calls.fetch_add(1, Ordering::SeqCst);
            *self.0.refresh_commit.lock().unwrap() = Some(commit);
            Ok(())
        }
    }

    #[async_trait]
    impl MetadataAttemptPort for WorkflowPort {
        async fn record_metadata_attempt(
            &self,
            _attempt: NewMetadataProviderAttempt,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl CatalogHydrationPort for WorkflowPort {
        async fn hydrate_catalog(
            &self,
            item_id: MediaItemId,
            source: MetadataSource,
        ) -> Result<CatalogHydrationSummary> {
            assert_eq!(self.0.refresh_snapshot.item.id, item_id);
            assert_eq!(source, MetadataSource::Provider(ExternalProvider::Tmdb));
            self.0.hydrate_catalog_calls.fetch_add(1, Ordering::SeqCst);
            self.0
                .hydration_requests
                .lock()
                .unwrap()
                .push((item_id, source));
            Ok(CatalogHydrationSummary {
                item_id,
                search_indexed: true,
                ..CatalogHydrationSummary::default()
            })
        }
    }

    #[derive(Debug)]
    struct FakeMetadataProvider {
        search: Vec<MetadataCandidate>,
        fetch: MetadataFetchResult,
        search_calls: Arc<AtomicUsize>,
        fetch_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl MetadataProvider for FakeMetadataProvider {
        fn provider(&self) -> ExternalProvider {
            ExternalProvider::Tmdb
        }

        fn provider_name(&self) -> &'static str {
            "fake"
        }

        async fn search(&self, _lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
            self.search_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.search.clone())
        }

        async fn fetch(&self, _request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
            self.fetch_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.fetch.clone())
        }
    }

    #[tokio::test]
    async fn refresh_service_uses_refresh_and_hydration_ports_without_sqlite() {
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Local Matrix".to_owned(),
                release_date: Some("1999".to_owned()),
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "603".to_owned(),
                }],
                ..CanonicalMetadata::default()
            },
        };
        let provider = FakeMetadataProvider {
            search: vec![MetadataCandidate {
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                score: 0.99,
                graph: MetadataCandidateGraph::from_canonical(
                    MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
                    MediaKind::Movie,
                    CanonicalMetadata {
                        title: "The Matrix".to_owned(),
                        release_date: Some("1999-03-31".to_owned()),
                        ..CanonicalMetadata::default()
                    },
                ),
            }],
            fetch: MetadataFetchResult {
                provider: ExternalProvider::Tmdb,
                provider_key: "603".to_owned(),
                graph: MetadataCandidateGraph::from_canonical(
                    MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
                    MediaKind::Movie,
                    CanonicalMetadata {
                        title: "The Matrix".to_owned(),
                        overview: Some("A hacker discovers reality.".to_owned()),
                        release_date: Some("1999-03-31".to_owned()),
                        ..CanonicalMetadata::default()
                    },
                ),
                raw_json: r#"{"id":603,"title":"The Matrix"}"#.to_owned(),
            },
            search_calls: Arc::new(AtomicUsize::new(0)),
            fetch_calls: Arc::new(AtomicUsize::new(0)),
        };
        let provider_search_calls = provider.search_calls.clone();
        let provider_fetch_calls = provider.fetch_calls.clone();
        let port = WorkflowPort::new(FakeMetadataWorkflowPort {
            refresh_snapshot: MetadataRefreshSnapshot {
                item: item.clone(),
                field_locks: vec![MetadataFieldLock {
                    item_id: item.id,
                    field: MetadataField::Title,
                    locked: true,
                    source: MetadataSource::User,
                }],
            },
            refresh_commit: Mutex::new(None),
            hydration_requests: Mutex::new(Vec::new()),
            load_refresh_calls: AtomicUsize::new(0),
            commit_refresh_calls: AtomicUsize::new(0),
            hydrate_catalog_calls: AtomicUsize::new(0),
        });
        let mut profile = MetadataProfile::default();
        profile.metadata_providers = vec![ExternalProvider::Tmdb];
        let service = MetadataRefreshService::new(provider, port.clone());

        let summary = service
            .refresh_item(MetadataRefreshRequest {
                job_id: JobId::new(),
                item_id: item.id,
                profile,
                force: false,
            })
            .await
            .unwrap();

        let refresh_commit = port.inner().refresh_commit.lock().unwrap().clone().unwrap();
        let hydration_requests = port.inner().hydration_requests.lock().unwrap().clone();

        assert_eq!(summary.item_id, item.id);
        assert_eq!(summary.provider, ExternalProvider::Tmdb);
        assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
        assert_eq!(refresh_commit.item.metadata.title, "Local Matrix");
        assert_eq!(refresh_commit.raw_response.provider_key, "603");
        assert_eq!(
            hydration_requests,
            vec![(item.id, MetadataSource::Provider(ExternalProvider::Tmdb))]
        );
        assert_eq!(port.inner().load_refresh_calls.load(Ordering::SeqCst), 1);
        assert_eq!(port.inner().commit_refresh_calls.load(Ordering::SeqCst), 1);
        assert_eq!(port.inner().hydrate_catalog_calls.load(Ordering::SeqCst), 1);
        assert_eq!(summary.attempted_providers.len(), 1);
        assert_eq!(provider_search_calls.load(Ordering::SeqCst), 0);
        assert_eq!(provider_fetch_calls.load(Ordering::SeqCst), 1);
    }
}
