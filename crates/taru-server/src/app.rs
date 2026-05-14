use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Serialize;
use taru_api::{
    GenreItemsResponse, GenreListResponse, ImagesResponse, ItemCreditsResponse, ItemDetailResponse,
    ItemsResponse, LibraryListResponse, LibrarySourceResponse, LibrarySourcesResponse, PageInfo,
    PeopleResponse, PersonItemsResponse, PlaybackDecisionResponse, SearchItemHit, SearchResponse,
    TagItemsResponse, TagsResponse,
};
use taru_core::{
    CatalogRepository, ExternalProvider, GenreId, Job, JobId, JobKind, JobRepository, Library,
    LibraryId, LibraryRepository, MediaItemId, MediaProbeRepository, MediaRepository, MediaSource,
    MediaSourceId, MetadataProfile, NewJob, NewTranscodeSession, PageRequest, PersonId, Result,
    TagId, TaruError, TransactionManager, TranscodeFailureCategory, TranscodeSessionId,
    TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionRepository,
    TranscodeSessionState,
};
use taru_db::SqliteStore;
use taru_library::{
    LibraryIndexRequest, LibraryIndexService, LibraryIndexSummary, LibraryProbeOptions,
    LibraryProbeRequest, LibraryProbeService, LibraryProbeSummary,
};
use taru_media_probe::FfprobeMediaProbe;
use taru_metadata::{
    MetadataProviderRegistry, MetadataRefreshJobInput, MetadataRefreshRequest,
    MetadataRefreshSummary, MetadataStrategyExecutor, TmdbMetadataProvider, TmdbProviderConfig,
};
use taru_nfo::{
    MovieNfoCodec, NfoExportRequest, NfoExportSummary, NfoImportRequest, NfoImportSummary,
    NfoJobInput, NfoService,
};
use taru_search::{SearchIndex, SearchQuery};
use taru_streaming::{
    ClientPlaybackCapabilities, DirectPlayRangeRequest, DirectPlayResponsePlan, PlaybackDecision,
    PlaybackMode, content_type_for_file_name, decide_playback, plan_direct_play_response,
};
use taru_transcode::{
    CancellationToken, FfmpegCommandBuilder, FfmpegHlsRunner, FfmpegOverwritePolicy,
    FfmpegRemuxRunner, HlsRequest, HlsRunOutcome, RemuxContainer, RemuxRequest, RemuxRunOutcome,
    RemuxRuntimeGuard, RemuxRuntimeLimits, TranscodeSessionManager,
};
use taru_vfs::{LocalFsBackend, StorageBackend, StorageUri};
use tokio::sync::{Mutex, Semaphore};
use tracing::{Instrument, error, info, info_span, warn};

use crate::config::{TaruServerConfig, library_from_config};

#[derive(Clone, Debug)]
pub struct TaruApp {
    inner: Arc<TaruAppInner>,
}

#[derive(Debug)]
struct TaruAppInner {
    config: TaruServerConfig,
    store: SqliteStore,
    scan_permits: Arc<Semaphore>,
    metadata_permits: Arc<Semaphore>,
    nfo_permits: Arc<Semaphore>,
    remux: RemuxAppService,
    hls: HlsAppService,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScanCommandOutput {
    pub job: Job,
    pub index: LibraryIndexSummary,
    pub probe: LibraryProbeSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetadataRefreshCommandOutput {
    pub job: Job,
    pub refresh: MetadataRefreshSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct NfoImportCommandOutput {
    pub job: Job,
    pub import: NfoImportSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct NfoExportCommandOutput {
    pub job: Job,
    pub export: NfoExportSummary,
}

#[derive(Clone, Debug)]
pub struct DirectPlaySourcePlan {
    pub source: MediaSource,
    pub local_path: PathBuf,
    pub response: DirectPlayResponsePlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemuxSourceRequest {
    pub source_id: MediaSourceId,
    pub client: ClientPlaybackCapabilities,
    pub output_container: RemuxContainer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemuxSourceDisposition {
    Finished,
    ReusedExisting,
    Cancelled,
}

#[derive(Clone, Debug, Serialize)]
pub struct RemuxSourceOutput {
    pub source: MediaSource,
    pub decision: PlaybackDecision,
    pub output_path: PathBuf,
    pub output_container: RemuxContainer,
    pub disposition: RemuxSourceDisposition,
    pub session: Option<TranscodeSessionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HlsSourceRequest {
    pub source_id: MediaSourceId,
    pub client: ClientPlaybackCapabilities,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HlsSourceDisposition {
    Finished,
    ReusedExisting,
    Cancelled,
}

#[derive(Clone, Debug, Serialize)]
pub struct HlsSourceOutput {
    pub source: MediaSource,
    pub decision: PlaybackDecision,
    pub playlist_path: PathBuf,
    pub segment_dir: PathBuf,
    pub disposition: HlsSourceDisposition,
    pub session: TranscodeSessionRecord,
}

#[derive(Clone, Debug, Serialize)]
pub struct HlsPlaylistOutput {
    pub source: MediaSource,
    pub decision: PlaybackDecision,
    pub session: TranscodeSessionRecord,
    pub body: String,
}

#[derive(Clone, Debug)]
pub struct HlsSegmentPlan {
    pub path: PathBuf,
    pub content_type: &'static str,
}

#[derive(Clone, Debug)]
pub struct RemuxStagingPolicy {
    root: PathBuf,
}

impl RemuxStagingPolicy {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        if root.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "remux staging root cannot be empty".to_owned(),
            });
        }

        if root
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(TaruError::InvalidInput {
                message: "remux staging root must not contain relative path components".to_owned(),
            });
        }

        Ok(Self { root })
    }

    pub fn output_path(
        &self,
        source_id: MediaSourceId,
        container: RemuxContainer,
    ) -> Result<PathBuf> {
        let output = self
            .root
            .join(source_id.to_string())
            .join(format!("stream.{}", container.file_extension()));

        if !output.starts_with(&self.root) {
            return Err(TaruError::Storage {
                uri: self.root.display().to_string(),
                message: "remux staging output escaped the staging root".to_owned(),
            });
        }

        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct HlsStagingPolicy {
    root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HlsOutputLayout {
    pub output_dir: PathBuf,
    pub playlist_path: PathBuf,
    pub segment_pattern: PathBuf,
}

impl HlsStagingPolicy {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        if root.as_os_str().is_empty() {
            return Err(TaruError::InvalidInput {
                message: "hls staging root cannot be empty".to_owned(),
            });
        }

        if root
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(TaruError::InvalidInput {
                message: "hls staging root must not contain relative path components".to_owned(),
            });
        }

        Ok(Self { root })
    }

    pub fn single_variant_layout(&self, source_id: MediaSourceId) -> Result<HlsOutputLayout> {
        let output_dir = self.root.join(source_id.to_string()).join("single");
        let playlist_path = output_dir.join("playlist.m3u8");
        let segment_pattern = output_dir.join("segment_%05d.ts");

        for path in [&output_dir, &playlist_path, &segment_pattern] {
            if !path.starts_with(&self.root) {
                return Err(TaruError::Storage {
                    uri: self.root.display().to_string(),
                    message: "hls staging output escaped the staging root".to_owned(),
                });
            }
        }

        Ok(HlsOutputLayout {
            output_dir,
            playlist_path,
            segment_pattern,
        })
    }
}

#[derive(Clone, Debug)]
struct RemuxAppService {
    builder: FfmpegCommandBuilder,
    runner: FfmpegRemuxRunner,
    in_flight: Arc<Mutex<HashSet<RemuxRequestKey>>>,
}

impl RemuxAppService {
    fn new(config: &TaruServerConfig) -> Self {
        let guard = RemuxRuntimeGuard::new(RemuxRuntimeLimits {
            max_concurrent_sessions: config.remux_concurrency,
            timeout_ms: config.remux_timeout_ms,
        });

        Self {
            builder: FfmpegCommandBuilder::new(&config.ffmpeg_path),
            runner: FfmpegRemuxRunner::new(guard),
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    async fn run(
        &self,
        sessions: &SqliteStore,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        output_path: PathBuf,
        output_container: RemuxContainer,
    ) -> Result<RemuxSourceOutput> {
        let key = RemuxRequestKey {
            source_id: source.id,
            output_container,
        };

        match self.reserve(sessions, &key, &output_path).await? {
            RemuxRequestAdmission::ReuseExisting { session } => {
                return Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::ReusedExisting,
                    session,
                });
            }
            RemuxRequestAdmission::Run { session } => {
                let result = self
                    .run_reserved(
                        sessions,
                        session,
                        source,
                        decision,
                        input_path,
                        output_path,
                        output_container,
                    )
                    .await;
                self.release(&key).await;
                return result;
            }
        }
    }

    async fn reserve(
        &self,
        sessions: &SqliteStore,
        key: &RemuxRequestKey,
        output_path: &Path,
    ) -> Result<RemuxRequestAdmission> {
        let request_key = key.persisted_request_key();
        if let Some(active) = sessions
            .find_active_transcode_session(key.source_id, TranscodeSessionKind::Remux, &request_key)
            .await?
        {
            return Err(TaruError::Conflict {
                message: format!(
                    "remux request for source {} as {:?} is already in progress in session {}",
                    key.source_id, key.output_container, active.id
                ),
            });
        }

        let latest = sessions
            .find_latest_transcode_session(key.source_id, TranscodeSessionKind::Remux, &request_key)
            .await?;
        let output_exists = path_exists(output_path)?;

        if let Some(session) = latest.as_ref() {
            if session.state == TranscodeSessionState::Finished
                && session.output_path.as_path() == output_path
                && output_exists
            {
                return Ok(RemuxRequestAdmission::ReuseExisting {
                    session: Some(session.clone()),
                });
            }
        }

        if latest.is_none() && output_exists {
            return Ok(RemuxRequestAdmission::ReuseExisting { session: None });
        }

        {
            let mut in_flight = self.in_flight.lock().await;
            if !in_flight.insert(*key) {
                return Err(TaruError::Conflict {
                    message: format!(
                        "remux request for source {} as {:?} is already in progress",
                        key.source_id, key.output_container
                    ),
                });
            }
        }

        let session = sessions
            .create_transcode_session(NewTranscodeSession {
                id: TranscodeSessionId::new(),
                source_id: key.source_id,
                kind: TranscodeSessionKind::Remux,
                request_key,
                output_path: output_path.to_path_buf(),
                state: TranscodeSessionState::Planned,
            })
            .await;

        match session {
            Ok(session) => Ok(RemuxRequestAdmission::Run { session }),
            Err(error) => {
                self.release(key).await;
                Err(error)
            }
        }
    }

    async fn run_reserved(
        &self,
        sessions: &SqliteStore,
        persisted_session: TranscodeSessionRecord,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        output_path: PathBuf,
        output_container: RemuxContainer,
    ) -> Result<RemuxSourceOutput> {
        let session_id = persisted_session.id;

        if let Err(error) = ensure_remux_output_parent(&output_path).await {
            persist_session_failure(sessions, session_id, &error).await;
            return Err(error);
        }

        let mut manager = TranscodeSessionManager::new();
        if let Err(error) = manager.plan_remux_with_id(
            session_id,
            RemuxRequest {
                source_id: source.id,
                input_path,
                output_path: output_path.clone(),
                output_container,
                overwrite: FfmpegOverwritePolicy::Never,
            },
            &self.builder,
        ) {
            persist_session_failure(sessions, session_id, &error).await;
            return Err(error);
        }

        sessions
            .set_transcode_session_state(session_id, TranscodeSessionState::Running, None, None)
            .await?;

        let cancel = CancellationToken::new();

        let run_result = self
            .runner
            .run(&mut manager, session_id, cancel)
            .await
            .map_err(map_remux_runner_error);

        match run_result {
            Ok(RemuxRunOutcome::Finished { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Finished,
                        None,
                        None,
                    )
                    .await?;

                Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::Finished,
                    session: Some(session),
                })
            }
            Ok(RemuxRunOutcome::Cancelled { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Cancelled,
                        Some(TranscodeFailureCategory::Cancelled),
                        Some("remux session was cancelled".to_owned()),
                    )
                    .await?;

                Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::Cancelled,
                    session: Some(session),
                })
            }
            Err(error) => {
                persist_session_failure(sessions, session_id, &error).await;
                Err(error)
            }
        }
    }

    async fn release(&self, key: &RemuxRequestKey) {
        self.in_flight.lock().await.remove(key);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct RemuxRequestKey {
    source_id: MediaSourceId,
    output_container: RemuxContainer,
}

impl RemuxRequestKey {
    fn persisted_request_key(self) -> String {
        format!("remux:{}", self.output_container.file_extension())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RemuxRequestAdmission {
    Run {
        session: TranscodeSessionRecord,
    },
    ReuseExisting {
        session: Option<TranscodeSessionRecord>,
    },
}

#[derive(Clone, Debug)]
struct HlsAppService {
    builder: FfmpegCommandBuilder,
    runner: FfmpegHlsRunner,
    in_flight: Arc<Mutex<HashSet<HlsRequestKey>>>,
}

impl HlsAppService {
    fn new(config: &TaruServerConfig) -> Self {
        let guard = RemuxRuntimeGuard::new(RemuxRuntimeLimits {
            max_concurrent_sessions: config.remux_concurrency,
            timeout_ms: config.remux_timeout_ms,
        });

        Self {
            builder: FfmpegCommandBuilder::new(&config.ffmpeg_path),
            runner: FfmpegHlsRunner::new(guard),
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    async fn run(
        &self,
        sessions: &SqliteStore,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        layout: HlsOutputLayout,
    ) -> Result<HlsSourceOutput> {
        let key = HlsRequestKey {
            source_id: source.id,
        };

        match self.reserve(sessions, &key, &layout).await? {
            HlsRequestAdmission::ReuseExisting { session } => Ok(HlsSourceOutput {
                source,
                decision,
                playlist_path: layout.playlist_path,
                segment_dir: layout.output_dir,
                disposition: HlsSourceDisposition::ReusedExisting,
                session,
            }),
            HlsRequestAdmission::Run { session } => {
                let result = self
                    .run_reserved(sessions, session, source, decision, input_path, layout)
                    .await;
                self.release(&key).await;
                result
            }
        }
    }

    async fn reserve(
        &self,
        sessions: &SqliteStore,
        key: &HlsRequestKey,
        layout: &HlsOutputLayout,
    ) -> Result<HlsRequestAdmission> {
        let request_key = key.persisted_request_key();
        if let Some(active) = sessions
            .find_active_transcode_session(
                key.source_id,
                TranscodeSessionKind::HlsTranscode,
                &request_key,
            )
            .await?
        {
            return Err(TaruError::Conflict {
                message: format!(
                    "hls request for source {} is already in progress in session {}",
                    key.source_id, active.id
                ),
            });
        }

        let latest = sessions
            .find_latest_transcode_session(
                key.source_id,
                TranscodeSessionKind::HlsTranscode,
                &request_key,
            )
            .await?;
        let playlist_exists = path_exists(&layout.playlist_path)?;

        if let Some(session) = latest.as_ref() {
            if session.state == TranscodeSessionState::Finished
                && session.output_path == layout.playlist_path
                && playlist_exists
            {
                return Ok(HlsRequestAdmission::ReuseExisting {
                    session: session.clone(),
                });
            }
        }

        {
            let mut in_flight = self.in_flight.lock().await;
            if !in_flight.insert(*key) {
                return Err(TaruError::Conflict {
                    message: format!(
                        "hls request for source {} is already in progress",
                        key.source_id
                    ),
                });
            }
        }

        let session = sessions
            .create_transcode_session(NewTranscodeSession {
                id: TranscodeSessionId::new(),
                source_id: key.source_id,
                kind: TranscodeSessionKind::HlsTranscode,
                request_key,
                output_path: layout.playlist_path.clone(),
                state: TranscodeSessionState::Planned,
            })
            .await;

        match session {
            Ok(session) => Ok(HlsRequestAdmission::Run { session }),
            Err(error) => {
                self.release(key).await;
                Err(error)
            }
        }
    }

    async fn run_reserved(
        &self,
        sessions: &SqliteStore,
        persisted_session: TranscodeSessionRecord,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        layout: HlsOutputLayout,
    ) -> Result<HlsSourceOutput> {
        let session_id = persisted_session.id;
        let mut manager = TranscodeSessionManager::new();

        if let Err(error) = manager.plan_hls_with_id(
            session_id,
            HlsRequest {
                source_id: source.id,
                input_path,
                output_dir: layout.output_dir.clone(),
                playlist_path: layout.playlist_path.clone(),
                segment_pattern: layout.segment_pattern.clone(),
                segment_time_seconds: 6,
                overwrite: FfmpegOverwritePolicy::Allow,
            },
            &self.builder,
        ) {
            persist_session_failure(sessions, session_id, &error).await;
            return Err(error);
        }

        sessions
            .set_transcode_session_state(session_id, TranscodeSessionState::Running, None, None)
            .await?;

        let cancel = CancellationToken::new();
        let run_result = self
            .runner
            .run(&mut manager, session_id, cancel)
            .await
            .map_err(map_hls_runner_error);

        match run_result {
            Ok(HlsRunOutcome::Finished { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Finished,
                        None,
                        None,
                    )
                    .await?;

                Ok(HlsSourceOutput {
                    source,
                    decision,
                    playlist_path: layout.playlist_path,
                    segment_dir: layout.output_dir,
                    disposition: HlsSourceDisposition::Finished,
                    session,
                })
            }
            Ok(HlsRunOutcome::Cancelled { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Cancelled,
                        Some(TranscodeFailureCategory::Cancelled),
                        Some("hls session was cancelled".to_owned()),
                    )
                    .await?;

                Ok(HlsSourceOutput {
                    source,
                    decision,
                    playlist_path: layout.playlist_path,
                    segment_dir: layout.output_dir,
                    disposition: HlsSourceDisposition::Cancelled,
                    session,
                })
            }
            Err(error) => {
                persist_session_failure(sessions, session_id, &error).await;
                Err(error)
            }
        }
    }

    async fn release(&self, key: &HlsRequestKey) {
        self.in_flight.lock().await.remove(key);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct HlsRequestKey {
    source_id: MediaSourceId,
}

impl HlsRequestKey {
    fn persisted_request_key(self) -> String {
        "hls:single".to_owned()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum HlsRequestAdmission {
    Run { session: TranscodeSessionRecord },
    ReuseExisting { session: TranscodeSessionRecord },
}

impl TaruApp {
    pub async fn new(config: TaruServerConfig) -> Result<Self> {
        let store = SqliteStore::connect(&config.database_url).await?;
        Self::new_with_store(config, store).await
    }

    pub async fn new_with_store(config: TaruServerConfig, store: SqliteStore) -> Result<Self> {
        store.migrate().await?;
        let recovered_sessions = store
            .fail_stale_transcode_sessions(
                TranscodeFailureCategory::Stale,
                "session was active during server startup".to_owned(),
            )
            .await?;
        if recovered_sessions > 0 {
            warn!(
                recovered_sessions,
                "marked stale transcode sessions failed during startup"
            );
        }

        let app = Self {
            inner: Arc::new(TaruAppInner {
                scan_permits: Arc::new(Semaphore::new(config.scan_concurrency.max(1))),
                metadata_permits: Arc::new(Semaphore::new(config.metadata_concurrency.max(1))),
                nfo_permits: Arc::new(Semaphore::new(config.metadata_concurrency.max(1))),
                remux: RemuxAppService::new(&config),
                hls: HlsAppService::new(&config),
                config,
                store,
            }),
        };

        app.ensure_configured_library().await?;
        Ok(app)
    }

    #[must_use]
    pub fn config(&self) -> &TaruServerConfig {
        &self.inner.config
    }

    pub async fn list_libraries(&self, page: PageRequest) -> Result<LibraryListResponse> {
        let page = page.clamped();
        let libraries = self.inner.store.list_libraries(page).await?;

        Ok(LibraryListResponse {
            page: PageInfo::new(page, libraries.len()),
            libraries,
        })
    }

    pub async fn list_library_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<LibrarySourcesResponse> {
        let page = page.clamped();
        let library = self.get_library_or_not_found(library_id).await?;
        let sources = self
            .inner
            .store
            .list_media_sources(library.id, page)
            .await?;
        let mut output_sources = Vec::with_capacity(sources.len());

        for source in sources {
            let item = self.inner.store.get_media_item(source.item_id).await?;
            let probe = self.inner.store.get_media_probe(source.id).await?;
            output_sources.push(LibrarySourceResponse {
                source,
                item,
                probe,
            });
        }

        Ok(LibrarySourcesResponse {
            library,
            page: PageInfo::new(page, output_sources.len()),
            sources: output_sources,
        })
    }

    pub async fn list_items(&self, page: PageRequest) -> Result<ItemsResponse> {
        let page = page.clamped();
        let items = self.inner.store.list_media_items(page).await?;

        Ok(ItemsResponse {
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn get_item(&self, item_id: MediaItemId) -> Result<ItemDetailResponse> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let sources = self
            .inner
            .store
            .list_item_sources(item.id, PageRequest::first_page())
            .await?;
        let credits = self.inner.store.list_item_credits(item.id).await?;
        let genres = self.inner.store.list_item_genres(item.id).await?;
        let tags = self.inner.store.list_item_tags(item.id).await?;
        let collections = self.inner.store.list_item_collections(item.id).await?;
        let studios = self.inner.store.list_item_studios(item.id).await?;
        let images = self.inner.store.list_item_images(item.id).await?;

        Ok(ItemDetailResponse {
            item,
            sources,
            credits,
            genres,
            tags,
            collections,
            studios,
            images,
        })
    }

    pub async fn list_item_credits(&self, item_id: MediaItemId) -> Result<ItemCreditsResponse> {
        let item = self
            .inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let credits = self.inner.store.list_item_credits(item.id).await?;
        let mut people = Vec::with_capacity(credits.len());

        for credit in &credits {
            if let Some(person) = self.inner.store.get_person(credit.person_id).await? {
                people.push(person);
            }
        }

        Ok(ItemCreditsResponse {
            item_id: item.id,
            credits,
            people,
        })
    }

    pub async fn list_item_images(&self, item_id: MediaItemId) -> Result<ImagesResponse> {
        self.inner
            .store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let images = self.inner.store.list_item_images(item_id).await?;

        Ok(ImagesResponse { item_id, images })
    }

    pub async fn get_source_playback_decision(
        &self,
        source_id: MediaSourceId,
        client: ClientPlaybackCapabilities,
    ) -> Result<PlaybackDecisionResponse> {
        let source = self.get_source_or_not_found(source_id).await?;
        let probe = self.inner.store.get_media_probe(source.id).await?;
        let decision = decide_playback(&source, probe.as_ref(), &client);

        Ok(PlaybackDecisionResponse {
            source,
            probe,
            decision,
        })
    }

    pub async fn plan_direct_play(
        &self,
        source_id: MediaSourceId,
        range_request: DirectPlayRangeRequest,
    ) -> Result<DirectPlaySourcePlan> {
        let source = self.get_source_or_not_found(source_id).await?;
        let (local_path, total_len) = self.local_source_path_and_len(&source).await?;
        let content_type = content_type_for_file_name(&source.file_name).to_owned();
        let response = plan_direct_play_response(total_len, content_type, range_request);

        Ok(DirectPlaySourcePlan {
            source,
            local_path,
            response,
        })
    }

    pub async fn remux_source(&self, request: RemuxSourceRequest) -> Result<RemuxSourceOutput> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe = self.inner.store.get_media_probe(source.id).await?;
        let decision = decide_playback(&source, probe.as_ref(), &request.client);

        if decision.mode != PlaybackMode::Remux {
            return Err(TaruError::Unsupported(
                "remux app service requires a remux playback decision",
            ));
        }

        let (local_path, _total_len) = self.local_source_path_and_len(&source).await?;
        let staging = RemuxStagingPolicy::new(&self.config().remux_staging_root)?;
        let output_path = staging.output_path(source.id, request.output_container)?;

        self.inner
            .remux
            .run(
                &self.inner.store,
                source,
                decision,
                local_path,
                output_path,
                request.output_container,
            )
            .await
    }

    pub async fn hls_source(&self, request: HlsSourceRequest) -> Result<HlsSourceOutput> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe = self.inner.store.get_media_probe(source.id).await?;
        let decision = decide_playback(&source, probe.as_ref(), &request.client);
        let (local_path, _total_len) = self.local_source_path_and_len(&source).await?;
        let staging = HlsStagingPolicy::new(self.config().remux_staging_root.join("hls"))?;
        let layout = staging.single_variant_layout(source.id)?;

        self.inner
            .hls
            .run(&self.inner.store, source, decision, local_path, layout)
            .await
    }

    pub async fn hls_playlist(&self, request: HlsSourceRequest) -> Result<HlsPlaylistOutput> {
        let output = self.hls_source(request).await?;

        if output.disposition == HlsSourceDisposition::Cancelled {
            return Err(TaruError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message: "hls session was cancelled".to_owned(),
            });
        }

        let body = tokio::fs::read_to_string(&output.playlist_path)
            .await
            .map_err(|err| TaruError::Storage {
                uri: output.playlist_path.display().to_string(),
                message: format!("failed to read hls playlist: {err}"),
            })?;

        Ok(HlsPlaylistOutput {
            source: output.source,
            decision: output.decision,
            body: rewrite_hls_playlist(&body, output.session.id),
            session: output.session,
        })
    }

    pub async fn plan_hls_segment(
        &self,
        session_id: TranscodeSessionId,
        segment_name: &str,
    ) -> Result<HlsSegmentPlan> {
        validate_hls_segment_name(segment_name)?;
        let session = self.get_transcode_session(session_id).await?;

        if session.kind != TranscodeSessionKind::HlsTranscode {
            return Err(TaruError::InvalidInput {
                message: format!("session {session_id} is not an hls transcode session"),
            });
        }

        if session.state != TranscodeSessionState::Finished {
            return Err(TaruError::Conflict {
                message: format!(
                    "hls session {session_id} is not ready; current state is {:?}",
                    session.state
                ),
            });
        }

        let segment_dir = session
            .output_path
            .parent()
            .ok_or_else(|| TaruError::Storage {
                uri: session.output_path.display().to_string(),
                message: "hls playlist path does not have a parent directory".to_owned(),
            })?;
        let path = segment_dir.join(segment_name);

        if !path.starts_with(segment_dir) {
            return Err(TaruError::InvalidInput {
                message: "hls segment path escaped the session directory".to_owned(),
            });
        }

        if !path_exists(&path)? {
            return Err(TaruError::NotFound {
                entity: "hls_segment",
                id: segment_name.to_owned(),
            });
        }

        Ok(HlsSegmentPlan {
            path,
            content_type: "video/mp2t",
        })
    }

    pub async fn get_transcode_session(
        &self,
        session_id: TranscodeSessionId,
    ) -> Result<TranscodeSessionRecord> {
        self.inner
            .store
            .get_transcode_session(session_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })
    }

    async fn local_source_path_and_len(&self, source: &MediaSource) -> Result<(PathBuf, u64)> {
        let uri = StorageUri::parse(&source.locator)?;
        let backend = LocalFsBackend::new(&self.config().library.root)?;
        let metadata = backend.stat(&uri).await?;
        let virtual_file = backend.open_range(&uri, None).await?;
        let local_path = virtual_file.local_path_hint.ok_or_else(|| {
            TaruError::Unsupported("local playback operations currently require a local path hint")
        })?;
        let total_len = match metadata.len {
            Some(len) => len,
            None => tokio::fs::metadata(&local_path)
                .await
                .map_err(|err| TaruError::Storage {
                    uri: source.locator.clone(),
                    message: format!("failed to read direct play source length: {err}"),
                })?
                .len(),
        };

        Ok((local_path, total_len))
    }

    pub async fn list_people(&self, page: PageRequest) -> Result<PeopleResponse> {
        let page = page.clamped();
        let people = self.inner.store.list_people(page).await?;

        Ok(PeopleResponse {
            page: PageInfo::new(page, people.len()),
            people,
        })
    }

    pub async fn get_person(&self, person_id: PersonId) -> Result<taru_core::Person> {
        self.inner
            .store
            .get_person(person_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "person",
                id: person_id.to_string(),
            })
    }

    pub async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<PersonItemsResponse> {
        let page = page.clamped();
        let person = self.get_person(person_id).await?;
        let items = self.inner.store.list_person_items(person.id, page).await?;

        Ok(PersonItemsResponse {
            person,
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn list_tags(&self, page: PageRequest) -> Result<TagsResponse> {
        let page = page.clamped();
        let tags = self.inner.store.list_tags(page).await?;

        Ok(TagsResponse {
            page: PageInfo::new(page, tags.len()),
            tags,
        })
    }

    pub async fn list_tag_items(
        &self,
        tag_id: TagId,
        page: PageRequest,
    ) -> Result<TagItemsResponse> {
        let page = page.clamped();
        let tag = self
            .inner
            .store
            .get_tag(tag_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "tag",
                id: tag_id.to_string(),
            })?;
        let items = self.inner.store.list_tag_items(tag.id, page).await?;

        Ok(TagItemsResponse {
            tag,
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn list_genres(&self, page: PageRequest) -> Result<GenreListResponse> {
        let page = page.clamped();
        let genres = self.inner.store.list_genres(page).await?;

        Ok(GenreListResponse {
            page: PageInfo::new(page, genres.len()),
            genres,
        })
    }

    pub async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<GenreItemsResponse> {
        let page = page.clamped();
        let genre =
            self.inner
                .store
                .get_genre(genre_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "genre",
                    id: genre_id.to_string(),
                })?;
        let items = self.inner.store.list_genre_items(genre.id, page).await?;

        Ok(GenreItemsResponse {
            genre,
            page: PageInfo::new(page, items.len()),
            items,
        })
    }

    pub async fn search_items(
        &self,
        query: String,
        facets: Vec<String>,
        page: PageRequest,
    ) -> Result<SearchResponse> {
        let page = page.clamped();
        let hits = self
            .inner
            .store
            .search(SearchQuery {
                query,
                facets,
                limit: page.limit,
                offset: u32::try_from(page.offset).map_err(|err| TaruError::InvalidInput {
                    message: format!("search offset is too large: {err}"),
                })?,
            })
            .await?;
        let mut output_hits = Vec::with_capacity(hits.len());

        for hit in hits {
            let item = self
                .inner
                .store
                .get_media_item(hit.item_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "media_item",
                    id: hit.item_id.to_string(),
                })?;
            output_hits.push(SearchItemHit {
                item,
                score: hit.score,
            });
        }

        Ok(SearchResponse {
            page: PageInfo::new(page, output_hits.len()),
            hits: output_hits,
        })
    }

    pub async fn get_source_probe(
        &self,
        source_id: MediaSourceId,
    ) -> Result<taru_api::SourceProbeResponse> {
        let probe = self
            .inner
            .store
            .get_media_probe(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source_probe",
                id: source_id.to_string(),
            })?;

        Ok(taru_api::SourceProbeResponse { source_id, probe })
    }

    async fn get_source_or_not_found(&self, source_id: MediaSourceId) -> Result<MediaSource> {
        self.inner
            .store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })
    }

    pub async fn get_job(&self, job_id: JobId) -> Result<Job> {
        self.inner
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })
    }

    pub async fn enqueue_library_scan(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_library_scan_job(library_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_library_scan_job(job_id, library_id).await;
            }
            .instrument(info_span!(
                "library_scan_background_job",
                job_id = %job_id,
                library_id = %library_id,
                resource_class = "disk.scan"
            )),
        );

        Ok(job)
    }

    pub async fn scan_configured_library(&self) -> Result<ScanCommandOutput> {
        let library_id = self.config().library.id;
        let job = self.create_library_scan_job(library_id).await?;
        self.execute_library_scan_job(job.id, library_id).await
    }

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

    pub async fn enqueue_nfo_import(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_import_job(library_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_nfo_import_job(job_id, library_id).await;
            }
            .instrument(info_span!(
                "nfo_import_background_job",
                job_id = %job_id,
                library_id = %library_id,
                resource_class = "metadata.nfo.import"
            )),
        );

        Ok(job)
    }

    pub async fn enqueue_nfo_export(&self, library_id: LibraryId) -> Result<Job> {
        let job = self.create_nfo_export_job(library_id).await?;
        let job_id = job.id;
        let app = self.clone();

        tokio::spawn(
            async move {
                app.finish_nfo_export_job(job_id, library_id).await;
            }
            .instrument(info_span!(
                "nfo_export_background_job",
                job_id = %job_id,
                library_id = %library_id,
                resource_class = "metadata.nfo.export"
            )),
        );

        Ok(job)
    }

    pub async fn import_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        let job = self.create_nfo_import_job(library_id).await?;
        self.execute_nfo_import_job(job.id, library_id).await
    }

    pub async fn export_library_nfo(
        &self,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        let job = self.create_nfo_export_job(library_id).await?;
        self.execute_nfo_export_job(job.id, library_id).await
    }

    async fn ensure_configured_library(&self) -> Result<()> {
        let library = library_from_config(self.config());
        self.inner.store.upsert_library(&library).await?;
        Ok(())
    }

    async fn create_library_scan_job(&self, library_id: LibraryId) -> Result<Job> {
        self.configured_library_for(library_id)?;
        let input = LibraryScanJobInput {
            library_id,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                resource_class: "disk.scan".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn create_metadata_refresh_job(&self, item_id: MediaItemId) -> Result<Job> {
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

    async fn create_nfo_import_job(&self, library_id: LibraryId) -> Result<Job> {
        let library = self.configured_library_for(library_id)?;
        let input = NfoJobInput {
            library_id,
            policy: library.options.metadata_profile.local_metadata_policy,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize NFO import job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::NfoImport,
                resource_class: "metadata.nfo.import".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn create_nfo_export_job(&self, library_id: LibraryId) -> Result<Job> {
        let library = self.configured_library_for(library_id)?;
        let input = NfoJobInput {
            library_id,
            policy: library.options.metadata_profile.local_metadata_policy,
            force: false,
        };
        let input_json = serde_json::to_string(&input).map_err(|err| TaruError::InvalidInput {
            message: format!("failed to serialize NFO export job input: {err}"),
        })?;

        self.inner
            .store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::NfoExport,
                resource_class: "metadata.nfo.export".to_owned(),
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(input_json),
            })
            .await
    }

    async fn finish_library_scan_job(&self, job_id: JobId, library_id: LibraryId) {
        match self.execute_library_scan_job(job_id, library_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    status = ?output.job.status,
                    "library scan job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    library_id = %library_id,
                    error = %err,
                    "library scan job failed"
                );
            }
        }
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

    async fn finish_nfo_import_job(&self, job_id: JobId, library_id: LibraryId) {
        match self.execute_nfo_import_job(job_id, library_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    imported_items = output.import.imported_items,
                    status = ?output.job.status,
                    "NFO import job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    library_id = %library_id,
                    error = %err,
                    "NFO import job failed"
                );
            }
        }
    }

    async fn finish_nfo_export_job(&self, job_id: JobId, library_id: LibraryId) {
        match self.execute_nfo_export_job(job_id, library_id).await {
            Ok(output) => {
                info!(
                    job_id = %output.job.id,
                    library_id = %library_id,
                    exported_items = output.export.exported_items,
                    status = ?output.job.status,
                    "NFO export job completed"
                );
            }
            Err(err) => {
                error!(
                    job_id = %job_id,
                    library_id = %library_id,
                    error = %err,
                    "NFO export job failed"
                );
            }
        }
    }

    async fn execute_library_scan_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<ScanCommandOutput> {
        let permit = self
            .inner
            .scan_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("scan concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_library_scan(job_id, library_id).await {
            Ok((index, probe)) => {
                let output = ScanJobSummary {
                    index: index.clone(),
                    probe: probe.clone(),
                };
                let summary_json =
                    serde_json::to_string(&output).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(ScanCommandOutput { job, index, probe })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        library_id = %library_id,
                        error = %update_err,
                        "failed to persist failed job state"
                    );
                }

                Err(err)
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

    async fn execute_nfo_import_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoImportCommandOutput> {
        let permit = self
            .inner
            .nfo_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("NFO concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_nfo_import(job_id, library_id).await {
            Ok(import) => {
                let summary_json =
                    serde_json::to_string(&import).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize NFO import job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(NfoImportCommandOutput { job, import })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        library_id = %library_id,
                        error = %update_err,
                        "failed to persist failed NFO import job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn execute_nfo_export_job(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoExportCommandOutput> {
        let permit = self
            .inner
            .nfo_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| TaruError::InvalidInput {
                message: format!("NFO concurrency limiter is unavailable: {err}"),
            })?;
        let _permit = permit;

        self.inner.store.start_job(job_id).await?;

        match self.run_nfo_export(job_id, library_id).await {
            Ok(export) => {
                let summary_json =
                    serde_json::to_string(&export).map_err(|err| TaruError::InvalidInput {
                        message: format!("failed to serialize NFO export job summary: {err}"),
                    })?;
                let job = self
                    .inner
                    .store
                    .succeed_job(job_id, Some(summary_json))
                    .await?;

                Ok(NfoExportCommandOutput { job, export })
            }
            Err(err) => {
                if let Err(update_err) = self.inner.store.fail_job(job_id, err.to_string()).await {
                    warn!(
                        job_id = %job_id,
                        library_id = %library_id,
                        error = %update_err,
                        "failed to persist failed NFO export job state"
                    );
                }

                Err(err)
            }
        }
    }

    async fn run_library_scan(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<(LibraryIndexSummary, LibraryProbeSummary)> {
        let library = self.configured_library_for(library_id)?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            probe_concurrency = self.config().probe_concurrency.max(1),
            "starting library scan pipeline"
        );

        let index_backend = LocalFsBackend::new(&self.config().library.root)?;
        let scanner = taru_library::VfsLibraryScanner::new(index_backend);
        let index_service = LibraryIndexService::new(scanner, self.inner.store.clone());
        let index = index_service
            .index_library(LibraryIndexRequest {
                job_id,
                library,
                force: false,
            })
            .await?;

        let probe_backend = LocalFsBackend::new(&self.config().library.root)?;
        let probe = FfprobeMediaProbe::new(&self.config().ffprobe_path);
        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            probe,
            self.inner.store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: self.config().probe_concurrency.max(1),
            },
        );
        let probe = probe_service
            .probe_library(LibraryProbeRequest {
                job_id,
                library_id,
                force: false,
            })
            .await?;

        Ok((index, probe))
    }

    async fn run_metadata_refresh(
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

    async fn run_nfo_import(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoImportSummary> {
        let library = self.configured_library_for(library_id)?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            policy = ?library.options.metadata_profile.local_metadata_policy,
            "starting NFO import job"
        );

        let backend = LocalFsBackend::new(&self.config().library.root)?;
        let service = NfoService::new(backend, self.inner.store.clone(), MovieNfoCodec);

        service
            .import_library(NfoImportRequest {
                job_id,
                library_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: false,
            })
            .await
    }

    async fn run_nfo_export(
        &self,
        job_id: JobId,
        library_id: LibraryId,
    ) -> Result<NfoExportSummary> {
        let library = self.configured_library_for(library_id)?;
        info!(
            job_id = %job_id,
            library_id = %library_id,
            policy = ?library.options.metadata_profile.local_metadata_policy,
            "starting NFO export job"
        );

        let backend = LocalFsBackend::new(&self.config().library.root)?;
        let service = NfoService::new(backend, self.inner.store.clone(), MovieNfoCodec);

        service
            .export_library(NfoExportRequest {
                job_id,
                library_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: false,
            })
            .await
    }

    async fn library_for_item(&self, item_id: MediaItemId) -> Result<Library> {
        let configured = library_from_config(self.config());
        let mut offset = 0;

        loop {
            let sources = self
                .inner
                .store
                .list_media_sources(
                    configured.id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;

            if sources.iter().any(|source| source.item_id == item_id) {
                return Ok(configured);
            }

            if sources.len() < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(configured)
    }

    fn effective_metadata_profile(
        &self,
        library: &Library,
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

    fn configured_library_for(&self, library_id: LibraryId) -> Result<Library> {
        let library = library_from_config(self.config());

        if library.id == library_id {
            Ok(library)
        } else {
            Err(TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
        }
    }

    async fn get_library_or_not_found(&self, library_id: LibraryId) -> Result<Library> {
        self.inner
            .store
            .get_library(library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })
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

fn map_remux_runner_error(error: TaruError) -> TaruError {
    match error {
        TaruError::Provider { provider, message } if provider == "ffmpeg" => {
            let message = if message.to_ascii_lowercase().contains("timed out") {
                "remux runner timed out".to_owned()
            } else {
                "remux runner failed".to_owned()
            };

            TaruError::Provider {
                provider: "ffmpeg_remux".to_owned(),
                message,
            }
        }
        TaruError::Storage { uri, .. } => TaruError::Storage {
            uri,
            message: "remux staging operation failed".to_owned(),
        },
        TaruError::InvalidInput { message } => TaruError::InvalidInput {
            message: format!("invalid remux request: {message}"),
        },
        other => other,
    }
}

fn map_hls_runner_error(error: TaruError) -> TaruError {
    match error {
        TaruError::Provider { provider, message } if provider == "ffmpeg" => {
            let message = if message.to_ascii_lowercase().contains("timed out") {
                "hls runner timed out".to_owned()
            } else {
                "hls runner failed".to_owned()
            };

            TaruError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message,
            }
        }
        TaruError::Storage { uri, .. } => TaruError::Storage {
            uri,
            message: "hls staging operation failed".to_owned(),
        },
        TaruError::InvalidInput { message } => TaruError::InvalidInput {
            message: format!("invalid hls request: {message}"),
        },
        other => other,
    }
}

fn path_exists(path: &Path) -> Result<bool> {
    path.try_exists().map_err(|err| TaruError::Storage {
        uri: path.display().to_string(),
        message: format!("failed to check path: {err}"),
    })
}

fn rewrite_hls_playlist(body: &str, session_id: TranscodeSessionId) -> String {
    let mut rewritten = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                line.to_owned()
            } else {
                format!("/playback/sessions/{session_id}/hls/segments/{trimmed}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if body.ends_with('\n') {
        rewritten.push('\n');
    }

    rewritten
}

fn validate_hls_segment_name(segment_name: &str) -> Result<()> {
    if segment_name.is_empty()
        || segment_name.contains('/')
        || segment_name.contains('\\')
        || segment_name.contains("..")
    {
        return Err(TaruError::InvalidInput {
            message: "invalid hls segment name".to_owned(),
        });
    }

    let path = Path::new(segment_name);
    if !path
        .components()
        .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(TaruError::InvalidInput {
            message: "invalid hls segment name".to_owned(),
        });
    }

    Ok(())
}

async fn persist_session_failure(
    sessions: &SqliteStore,
    session_id: TranscodeSessionId,
    error: &TaruError,
) {
    let category = TranscodeFailureCategory::from_error(error);
    if let Err(update_error) = sessions
        .set_transcode_session_state(
            session_id,
            TranscodeSessionState::Failed,
            Some(category),
            Some(error.to_string()),
        )
        .await
    {
        error!(
            session_id = %session_id,
            error = %update_error,
            "failed to persist transcode session failure"
        );
    }
}

async fn ensure_remux_output_parent(output_path: &Path) -> Result<()> {
    let Some(parent) = output_path.parent() else {
        return Err(TaruError::Storage {
            uri: output_path.display().to_string(),
            message: "remux output path does not have a parent directory".to_owned(),
        });
    };

    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|err| TaruError::Storage {
            uri: parent.display().to_string(),
            message: format!("failed to create remux output directory: {err}"),
        })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TmdbProviderBuildError {
    Disabled(String),
    Unavailable(String),
}

#[derive(Clone, Debug, Serialize)]
struct ScanJobSummary {
    index: LibraryIndexSummary,
    probe: LibraryProbeSummary,
}

#[derive(Clone, Debug, Serialize)]
struct LibraryScanJobInput {
    library_id: LibraryId,
    force: bool,
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use taru_core::{
        CanonicalMetadata, JobKind, JobStatus, LibraryId, MediaItem, MediaItemId, MediaKind,
        MediaProbeRepository, MediaProbeResult, MediaRepository, MediaSource, MediaSourceId,
        MediaStreamInfo, MediaStreamKind, MetadataField, MetadataRepository, MetadataSource,
    };
    use taru_transcode::RemuxContainer;

    use super::*;
    use crate::config::{LocalLibraryConfig, MetadataConfig};

    #[tokio::test]
    async fn scan_configured_library_persists_job_success() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store).await.unwrap();

        let output = app.scan_configured_library().await.unwrap();
        let job = app.get_job(output.job.id).await.unwrap();

        assert_eq!(output.job.status, JobStatus::Succeeded);
        assert_eq!(job.status, JobStatus::Succeeded);
        assert_eq!(output.index.discovered_files, 0);
        assert_eq!(output.probe.total_sources, 0);
    }

    #[tokio::test]
    async fn metadata_refresh_job_input_does_not_include_secrets() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();

        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let input = job
            .input_json
            .as_ref()
            .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
            .unwrap();

        assert_eq!(job.kind, JobKind::MetadataRefresh);
        assert_eq!(job.resource_class, "metadata.tmdb");
        assert_eq!(job.library_id, Some(library_id));
        assert_eq!(
            input.get("item_id").and_then(serde_json::Value::as_str),
            Some(item.id.to_string().as_str())
        );
        assert_eq!(
            input.get("provider").and_then(serde_json::Value::as_str),
            Some("tmdb")
        );
        assert_eq!(
            input
                .get("refresh_mode")
                .and_then(serde_json::Value::as_str),
            Some("default")
        );
        assert!(input.get("access_token").is_none());
        assert!(input.get("api_key").is_none());
    }

    #[tokio::test]
    async fn metadata_refresh_job_records_disabled_profile_provider_for_executor() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();

        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

        assert_eq!(job.kind, JobKind::MetadataRefresh);
        assert_eq!(job.resource_class, "metadata.tmdb");
        let TaruError::Provider { provider, message } = err else {
            panic!("expected provider exhaustion error");
        };
        assert_eq!(provider, "metadata_strategy");
        assert!(message.contains("tmdb=skipped_disabled"));
        assert!(message.contains("disabled in config"));
    }

    #[tokio::test]
    async fn metadata_refresh_falls_back_from_unimplemented_bangumi_to_tmdb_unavailable() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Anime".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Anime,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Anime Movie".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();
        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let err = app.run_metadata_refresh(job.id, item.id).await.unwrap_err();

        assert_eq!(job.resource_class, "metadata.bangumi");
        let TaruError::Provider { provider, message } = err else {
            panic!("expected provider exhaustion error");
        };
        assert_eq!(provider, "metadata_strategy");
        assert!(message.contains("bangumi=not_implemented"));
        assert!(message.contains("tmdb=skipped_unavailable"));
        assert_eq!(app.get_job(job.id).await.unwrap().status, JobStatus::Queued);
    }

    #[tokio::test]
    async fn metadata_refresh_resolves_provider_order_from_library_profile() {
        let temp = tempfile::tempdir().unwrap();
        let library_id = LibraryId::new();
        let mut metadata = MetadataConfig::default();
        metadata.tmdb.enabled = true;
        metadata.tmdb.access_token_env = "TARU_TEST_MISSING_TMDB_TOKEN".to_owned();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata,
            library: LocalLibraryConfig {
                id: library_id,
                name: "Anime".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Anime,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Anime Movie".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_media_item(&item).await.unwrap();
        let job = app.create_metadata_refresh_job(item.id).await.unwrap();
        let input = job
            .input_json
            .as_ref()
            .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
            .unwrap();

        assert_eq!(job.resource_class, "metadata.bangumi");
        assert_eq!(
            input.get("provider").and_then(serde_json::Value::as_str),
            Some("bangumi")
        );
    }

    #[tokio::test]
    async fn nfo_import_job_imports_sidecar_and_persists_summary() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
</movie>
"#,
        )
        .unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: temp.path().join("taru-cache").join("remux"),
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: temp.path().to_path_buf(),
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "File Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///demo.mkv".to_owned(),
            file_name: "demo.mkv".to_owned(),
            size_bytes: Some(5),
            fingerprint: None,
        };
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();

        let output = app.import_library_nfo(library_id).await.unwrap();
        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();
        let job = app.get_job(output.job.id).await.unwrap();

        assert_eq!(output.job.kind, JobKind::NfoImport);
        assert_eq!(output.job.status, JobStatus::Succeeded);
        assert_eq!(output.import.imported_items, 1);
        assert_eq!(loaded.metadata.title, "NFO Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert!(locks.iter().any(|lock| {
            lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
        }));
        assert_eq!(job.status, JobStatus::Succeeded);
        assert!(job.summary_json.unwrap().contains("\"imported_items\":1"));
    }

    #[tokio::test]
    async fn remux_source_runs_runner_and_reuses_completed_output() {
        let script_root = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
        let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
        let request = RemuxSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
            output_container: RemuxContainer::Mp4,
        };

        let output = app.remux_source(request.clone()).await.unwrap();
        let session = output.session.as_ref().unwrap();

        assert_eq!(output.disposition, RemuxSourceDisposition::Finished);
        assert_eq!(session.state, TranscodeSessionState::Finished);
        assert!(
            output
                .output_path
                .starts_with(&app.config().remux_staging_root)
        );
        assert_eq!(fs::read_to_string(&output.output_path).unwrap(), "remuxed");
        assert_eq!(
            app.get_transcode_session(session.id).await.unwrap().state,
            TranscodeSessionState::Finished
        );
        assert_eq!(
            store
                .find_latest_transcode_session(
                    source.id,
                    TranscodeSessionKind::Remux,
                    &RemuxRequestKey {
                        source_id: source.id,
                        output_container: RemuxContainer::Mp4,
                    }
                    .persisted_request_key(),
                )
                .await
                .unwrap()
                .unwrap()
                .id,
            session.id
        );

        let reused = app.remux_source(request.clone()).await.unwrap();

        assert_eq!(reused.disposition, RemuxSourceDisposition::ReusedExisting);
        assert_eq!(reused.session.as_ref().unwrap().id, session.id);
        assert_eq!(reused.output_path, output.output_path);
        assert_eq!(fs::read_to_string(reused.output_path).unwrap(), "remuxed");

        let config = app.config().clone();
        drop(app);
        fs::remove_file(ffmpeg_path).unwrap();
        let restarted = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let restarted_reused = restarted.remux_source(request).await.unwrap();

        assert_eq!(
            restarted_reused.disposition,
            RemuxSourceDisposition::ReusedExisting
        );
        assert_eq!(restarted_reused.session.as_ref().unwrap().id, session.id);
    }

    #[tokio::test]
    async fn remux_source_rejects_persisted_active_duplicate() {
        let script_root = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
        let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
        let key = RemuxRequestKey {
            source_id: source.id,
            output_container: RemuxContainer::Mp4,
        };
        let staging = RemuxStagingPolicy::new(&app.config().remux_staging_root).unwrap();
        let active = store
            .create_transcode_session(NewTranscodeSession {
                id: TranscodeSessionId::new(),
                source_id: source.id,
                kind: TranscodeSessionKind::Remux,
                request_key: key.persisted_request_key(),
                output_path: staging.output_path(source.id, RemuxContainer::Mp4).unwrap(),
                state: TranscodeSessionState::Running,
            })
            .await
            .unwrap();

        let err = app
            .remux_source(RemuxSourceRequest {
                source_id: source.id,
                client: ClientPlaybackCapabilities::default(),
                output_container: RemuxContainer::Mp4,
            })
            .await
            .unwrap_err();

        let TaruError::Conflict { message } = err else {
            panic!("expected remux duplicate conflict");
        };
        assert!(message.contains("already in progress"));
        assert!(message.contains(&active.id.to_string()));
    }

    #[tokio::test]
    async fn remux_source_persists_runner_failure() {
        let script_root = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_failing_ffmpeg_script(script_root.path(), "failure");
        let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
        let request_key = RemuxRequestKey {
            source_id: source.id,
            output_container: RemuxContainer::Mp4,
        }
        .persisted_request_key();

        let err = app
            .remux_source(RemuxSourceRequest {
                source_id: source.id,
                client: ClientPlaybackCapabilities::default(),
                output_container: RemuxContainer::Mp4,
            })
            .await
            .unwrap_err();

        let TaruError::Provider { provider, message } = err else {
            panic!("expected remux provider failure");
        };
        assert_eq!(provider, "ffmpeg_remux");
        assert_eq!(message, "remux runner failed");

        let session = store
            .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(session.state, TranscodeSessionState::Failed);
        assert_eq!(
            session.failure_category,
            Some(TranscodeFailureCategory::Runner)
        );
        assert_eq!(
            session.failure_message.as_deref(),
            Some("external provider error from ffmpeg_remux: remux runner failed")
        );
    }

    #[tokio::test]
    async fn app_startup_marks_stale_transcode_sessions_failed() {
        let script_root = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_ffmpeg_script(script_root.path(), "success");
        let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
        let config = app.config().clone();
        let staging = RemuxStagingPolicy::new(&config.remux_staging_root).unwrap();
        let stale_id = TranscodeSessionId::new();

        store
            .create_transcode_session(NewTranscodeSession {
                id: stale_id,
                source_id: source.id,
                kind: TranscodeSessionKind::Remux,
                request_key: RemuxRequestKey {
                    source_id: source.id,
                    output_container: RemuxContainer::Mp4,
                }
                .persisted_request_key(),
                output_path: staging.output_path(source.id, RemuxContainer::Mp4).unwrap(),
                state: TranscodeSessionState::Running,
            })
            .await
            .unwrap();

        drop(app);
        let _restarted = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let stale = store
            .get_transcode_session(stale_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(stale.state, TranscodeSessionState::Failed);
        assert_eq!(
            stale.failure_category,
            Some(TranscodeFailureCategory::Stale)
        );
    }

    #[tokio::test]
    async fn hls_source_runs_runner_and_reuses_completed_session() {
        let script_root = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_success");
        let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path.clone()).await;
        let request = HlsSourceRequest {
            source_id: source.id,
            client: ClientPlaybackCapabilities::default(),
        };

        let output = app.hls_source(request.clone()).await.unwrap();
        let session_id = output.session.id;

        assert_eq!(output.disposition, HlsSourceDisposition::Finished);
        assert_eq!(output.session.kind, TranscodeSessionKind::HlsTranscode);
        assert_eq!(output.session.state, TranscodeSessionState::Finished);
        assert!(
            fs::read_to_string(&output.playlist_path)
                .unwrap()
                .contains("#EXTM3U")
        );
        assert_eq!(
            fs::read_to_string(output.segment_dir.join("segment_00000.ts")).unwrap(),
            "segment"
        );

        let playlist = app.hls_playlist(request.clone()).await.unwrap();
        assert!(playlist.body.contains(&format!(
            "/playback/sessions/{session_id}/hls/segments/segment_00000.ts"
        )));

        let segment = app
            .plan_hls_segment(session_id, "segment_00000.ts")
            .await
            .unwrap();
        assert_eq!(segment.content_type, "video/mp2t");
        assert!(segment.path.ends_with("segment_00000.ts"));
        assert!(
            app.plan_hls_segment(session_id, "../segment_00000.ts")
                .await
                .is_err()
        );

        fs::remove_file(ffmpeg_path).unwrap();
        let reused = app.hls_source(request.clone()).await.unwrap();
        assert_eq!(reused.disposition, HlsSourceDisposition::ReusedExisting);
        assert_eq!(reused.session.id, session_id);

        let config = app.config().clone();
        drop(app);
        let restarted = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let restarted_reused = restarted.hls_source(request).await.unwrap();

        assert_eq!(
            restarted_reused.disposition,
            HlsSourceDisposition::ReusedExisting
        );
        assert_eq!(restarted_reused.session.id, session_id);
    }

    #[tokio::test]
    async fn hls_source_rejects_persisted_active_duplicate() {
        let script_root = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_hls_ffmpeg_script(script_root.path(), "hls_success");
        let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;
        let staging = HlsStagingPolicy::new(app.config().remux_staging_root.join("hls")).unwrap();
        let layout = staging.single_variant_layout(source.id).unwrap();
        let active = store
            .create_transcode_session(NewTranscodeSession {
                id: TranscodeSessionId::new(),
                source_id: source.id,
                kind: TranscodeSessionKind::HlsTranscode,
                request_key: "hls:single".to_owned(),
                output_path: layout.playlist_path,
                state: TranscodeSessionState::Running,
            })
            .await
            .unwrap();

        let err = app
            .hls_source(HlsSourceRequest {
                source_id: source.id,
                client: ClientPlaybackCapabilities::default(),
            })
            .await
            .unwrap_err();

        let TaruError::Conflict { message } = err else {
            panic!("expected hls duplicate conflict");
        };
        assert!(message.contains("already in progress"));
        assert!(message.contains(&active.id.to_string()));
    }

    #[tokio::test]
    async fn hls_source_persists_runner_failure() {
        let script_root = tempfile::tempdir().unwrap();
        let ffmpeg_path = fake_failing_hls_ffmpeg_script(script_root.path(), "hls_failure");
        let (_temp, app, store, source) = remux_app_with_source(ffmpeg_path).await;

        let err = app
            .hls_source(HlsSourceRequest {
                source_id: source.id,
                client: ClientPlaybackCapabilities::default(),
            })
            .await
            .unwrap_err();

        let TaruError::Provider { provider, message } = err else {
            panic!("expected hls provider failure");
        };
        assert_eq!(provider, "ffmpeg_hls");
        assert_eq!(message, "hls runner failed");

        let session = store
            .find_latest_transcode_session(
                source.id,
                TranscodeSessionKind::HlsTranscode,
                "hls:single",
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(session.state, TranscodeSessionState::Failed);
        assert_eq!(
            session.failure_category,
            Some(TranscodeFailureCategory::Runner)
        );
        assert_eq!(
            session.failure_message.as_deref(),
            Some("external provider error from ffmpeg_hls: hls runner failed")
        );
    }

    #[test]
    fn remux_staging_policy_rejects_escaping_roots() {
        assert!(RemuxStagingPolicy::new(PathBuf::new()).is_err());
        assert!(RemuxStagingPolicy::new(PathBuf::from("cache/../outside")).is_err());

        let policy = RemuxStagingPolicy::new(PathBuf::from("cache/remux")).unwrap();
        let output = policy
            .output_path(MediaSourceId::new(), RemuxContainer::Mkv)
            .unwrap();

        assert!(output.starts_with(&policy.root));
        assert_eq!(
            output.extension().and_then(|value| value.to_str()),
            Some("mkv")
        );
    }

    #[test]
    fn hls_staging_policy_rejects_escaping_roots() {
        assert!(HlsStagingPolicy::new(PathBuf::new()).is_err());
        assert!(HlsStagingPolicy::new(PathBuf::from("cache/../outside")).is_err());

        let policy = HlsStagingPolicy::new(PathBuf::from("cache/hls")).unwrap();
        let layout = policy.single_variant_layout(MediaSourceId::new()).unwrap();

        assert!(layout.output_dir.starts_with(&policy.root));
        assert!(layout.playlist_path.starts_with(&policy.root));
        assert!(layout.segment_pattern.starts_with(&policy.root));
        assert_eq!(
            layout
                .playlist_path
                .file_name()
                .and_then(|value| value.to_str()),
            Some("playlist.m3u8")
        );
    }

    fn fake_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let content =
                "#!/bin/sh\nfor arg do out=\"$arg\"; done\nprintf remuxed > \"$out\"\nexit 0\n";
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("setlocal enabledelayedexpansion\r\n");
            content.push_str(":args\r\n");
            content.push_str("if \"%~1\"==\"\" goto run\r\n");
            content.push_str("set out=%~1\r\n");
            content.push_str("shift\r\n");
            content.push_str("goto args\r\n");
            content.push_str(":run\r\n");
            content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
            content.push_str("exit /b 0\r\n");
            fs::write(&path, content).unwrap();
            path
        }
    }

    fn fake_failing_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let content = "#!/bin/sh\necho remux failed >&2\nexit 7\n";
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("echo remux failed 1>&2\r\n");
            content.push_str("exit /b 7\r\n");
            fs::write(&path, content).unwrap();
            path
        }
    }

    fn fake_hls_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
        hls_ffmpeg_script(root, name, true)
    }

    fn fake_failing_hls_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
        hls_ffmpeg_script(root, name, false)
    }

    fn hls_ffmpeg_script(root: &Path, name: &str, success: bool) -> PathBuf {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let path = root.join(name);
            let mut content = String::from("#!/bin/sh\n");
            content.push_str("for arg do out=\"$arg\"; done\n");
            content.push_str("dir=$(dirname \"$out\")\n");
            content.push_str("mkdir -p \"$dir\"\n");
            if success {
                content.push_str("printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n");
                content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
                content.push_str("exit 0\n");
            } else {
                content.push_str("printf partial > \"$out\"\n");
                content.push_str("printf hls-failed >&2\n");
                content.push_str("exit 42\n");
            }
            fs::write(&path, content).unwrap();
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
            path
        }

        #[cfg(windows)]
        {
            let path = root.join(format!("{name}.cmd"));
            let mut content = String::from("@echo off\r\n");
            content.push_str("setlocal enabledelayedexpansion\r\n");
            content.push_str(":args\r\n");
            content.push_str("if \"%~1\"==\"\" goto run\r\n");
            content.push_str("set out=%~1\r\n");
            content.push_str("shift\r\n");
            content.push_str("goto args\r\n");
            content.push_str(":run\r\n");
            content.push_str("for %%I in (\"%out%\") do set dir=%%~dpI\r\n");
            content.push_str("if not exist \"%dir%\" mkdir \"%dir%\"\r\n");
            if success {
                content.push_str(">\"%out%\" echo #EXTM3U\r\n");
                content.push_str(">>\"%out%\" echo #EXTINF:1,\r\n");
                content.push_str(">>\"%out%\" echo segment_00000.ts\r\n");
                content.push_str(">>\"%out%\" echo #EXT-X-ENDLIST\r\n");
                content.push_str("<nul set /p dummy=segment>\"%dir%segment_00000.ts\"\r\n");
                content.push_str("exit /b 0\r\n");
            } else {
                content.push_str("<nul set /p dummy=partial>\"%out%\"\r\n");
                content.push_str("echo hls-failed 1>&2\r\n");
                content.push_str("exit /b 42\r\n");
            }
            fs::write(&path, content).unwrap();
            path
        }
    }

    async fn remux_app_with_source(
        ffmpeg_path: PathBuf,
    ) -> (tempfile::TempDir, TaruApp, SqliteStore, MediaSource) {
        let temp = tempfile::tempdir().unwrap();
        let library_root = temp.path().join("library");
        let staging_root = temp.path().join("cache").join("remux");
        fs::create_dir_all(&library_root).unwrap();
        fs::write(library_root.join("demo.mkv"), b"media").unwrap();
        let library_id = LibraryId::new();
        let config = TaruServerConfig {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path,
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            remux_timeout_ms: 30 * 60 * 1_000,
            remux_staging_root: staging_root,
            metadata: MetadataConfig::default(),
            library: LocalLibraryConfig {
                id: library_id,
                name: "Movies".to_owned(),
                root: library_root,
                preset: taru_core::LibraryPreset::Movies,
            },
        };
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let app = TaruApp::new_with_store(config, store.clone())
            .await
            .unwrap();
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///demo.mkv".to_owned(),
            file_name: "demo.mkv".to_owned(),
            size_bytes: Some(5),
            fingerprint: None,
        };

        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library_id, &source)
            .await
            .unwrap();
        store
            .upsert_media_probe(
                source.id,
                &MediaProbeResult {
                    duration_ms: Some(1_000),
                    container: Some("matroska,webm".to_owned()),
                    bit_rate: None,
                    streams: vec![
                        MediaStreamInfo {
                            index: 0,
                            kind: MediaStreamKind::Video,
                            codec: Some("h264".to_owned()),
                            language: None,
                            duration_ms: None,
                            bit_rate: None,
                            width: Some(1920),
                            height: Some(1080),
                            channels: None,
                            sample_rate: None,
                        },
                        MediaStreamInfo {
                            index: 1,
                            kind: MediaStreamKind::Audio,
                            codec: Some("aac".to_owned()),
                            language: None,
                            duration_ms: None,
                            bit_rate: None,
                            width: None,
                            height: None,
                            channels: Some(2),
                            sample_rate: Some(48_000),
                        },
                    ],
                },
            )
            .await
            .unwrap();

        (temp, app, store, source)
    }
}
