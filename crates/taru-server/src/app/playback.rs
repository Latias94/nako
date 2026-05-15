use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures_util::StreamExt;
use serde::Serialize;
use taru_api::PlaybackDecisionResponse;
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, EventOutboxRepository, MediaProbeRepository,
    MediaSource, MediaSourceId, NewOutboxEvent, NewTranscodeSession, Result, StagingPurpose,
    TaruError, TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionRecord, TranscodeSessionRepository, TranscodeSessionState,
};
use taru_db::SqliteStore;
use taru_streaming::{
    ClientPlaybackCapabilities, DirectPlayRangeRequest, DirectPlayResponsePlan, PlaybackDecision,
    PlaybackMode, content_type_for_file_name, decide_playback, plan_direct_play_response,
};
use taru_transcode::{
    CancellationToken, FfmpegCommandBuilder, FfmpegHlsRunner, FfmpegOverwritePolicy,
    FfmpegRemuxRunner, HardwareAcceleration, HlsRequest, HlsRunOutcome, RemuxContainer,
    RemuxRequest, RemuxRunOutcome, RemuxRuntimeGuard, RemuxRuntimeLimits, TranscodeSessionManager,
};
use taru_vfs::{ByteRange, ReadStream, StageRequest, StorageBackend, StorageUri};
use tokio::sync::{Mutex, OwnedSemaphorePermit};
use tracing::{error, warn};

use crate::config::TaruServerConfig;

use super::{ManifestRecordingStorageBackend, TaruApp};

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

pub struct DirectPlaySourcePlan {
    pub source: MediaSource,
    pub body: DirectPlaySourceBody,
    pub response: DirectPlayResponsePlan,
}

impl fmt::Debug for DirectPlaySourcePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirectPlaySourcePlan")
            .field("source", &self.source)
            .field("body", &self.body)
            .field("response", &self.response)
            .finish()
    }
}

pub enum DirectPlaySourceBody {
    LocalPath(PathBuf),
    Stream(DirectPlayStreamBody),
    Empty,
}

pub struct DirectPlayStreamBody {
    pub stream: ReadStream,
    _permit: Option<OwnedSemaphorePermit>,
}

impl DirectPlayStreamBody {
    fn new(stream: ReadStream, permit: Option<OwnedSemaphorePermit>) -> Self {
        Self {
            stream,
            _permit: permit,
        }
    }

    pub(crate) fn unbudgeted(stream: ReadStream) -> Self {
        Self::new(stream, None)
    }

    pub fn into_read_stream(self) -> ReadStream {
        let Self {
            stream,
            _permit: permit,
        } = self;
        let ReadStream { uri, range, body } = stream;
        let body = match permit {
            Some(permit) => body
                .map(move |chunk| {
                    let _permit = &permit;
                    chunk
                })
                .boxed(),
            None => body,
        };

        ReadStream::new(uri, range, body)
    }
}

impl fmt::Debug for DirectPlaySourceBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalPath(path) => formatter.debug_tuple("LocalPath").field(path).finish(),
            Self::Stream(stream) => formatter.debug_tuple("Stream").field(stream).finish(),
            Self::Empty => formatter.write_str("Empty"),
        }
    }
}

impl fmt::Debug for DirectPlayStreamBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirectPlayStreamBody")
            .field("stream", &self.stream)
            .field("budgeted", &self._permit.is_some())
            .finish()
    }
}

impl TaruApp {
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
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let stream_permit = if should_budget_remote_stream(&uri) {
            Some(self.acquire_remote_stream_permit().await?)
        } else {
            None
        };
        let (response, body) =
            plan_direct_play_with_backend(&source, &uri, backend.as_ref(), range_request).await?;
        let body = match body {
            DirectPlaySourceBody::Stream(stream) => DirectPlaySourceBody::Stream(
                DirectPlayStreamBody::new(stream.stream, stream_permit),
            ),
            other => other,
        };

        Ok(DirectPlaySourcePlan {
            source,
            body,
            response,
        })
    }

    pub async fn plan_direct_play_preflight(
        &self,
        source_id: MediaSourceId,
        range_request: DirectPlayRangeRequest,
    ) -> Result<DirectPlayResponsePlan> {
        let source = self.get_source_or_not_found(source_id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;

        plan_direct_play_response_with_backend(&source, &uri, backend.as_ref(), range_request).await
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

        let local_path = self.source_path_for_ffmpeg(&source).await?;
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
        let local_path = self.source_path_for_ffmpeg(&source).await?;
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

    pub(super) async fn source_path_for_ffmpeg(&self, source: &MediaSource) -> Result<PathBuf> {
        let (uri, backend) = self.storage_backend_for_media_source(source).await?;
        let backend = ManifestRecordingStorageBackend::new(
            backend,
            self.inner.store.clone(),
            StagingPurpose::FfmpegInput,
            self.config().staging.max_bytes,
            self.config().staging.retention_ms,
            self.inner.remote_stage_permits.clone(),
            self.inner.remote_stage_budget_lock.clone(),
        );
        match local_source_path_and_len(source, &uri, &backend).await {
            Ok((path, _len)) => Ok(path),
            Err(TaruError::Unsupported(_)) => {
                let staged = backend
                    .stage(StageRequest::new(
                        uri.clone(),
                        self.config().remux_staging_root.join("inputs"),
                    ))
                    .await?;
                Ok(staged.path)
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct RemuxAppService {
    builder: FfmpegCommandBuilder,
    runner: FfmpegRemuxRunner,
    in_flight: Arc<Mutex<HashSet<RemuxRequestKey>>>,
}

impl RemuxAppService {
    pub(super) fn new(config: &TaruServerConfig) -> Self {
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
            RemuxRequestAdmission::ReuseExisting { session } => Ok(RemuxSourceOutput {
                source,
                decision,
                output_path,
                output_container,
                disposition: RemuxSourceDisposition::ReusedExisting,
                session,
            }),
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
                result
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
                record_playback_session_finished_event(sessions, &session).await;

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
pub(super) struct RemuxRequestKey {
    pub(super) source_id: MediaSourceId,
    pub(super) output_container: RemuxContainer,
}

impl RemuxRequestKey {
    pub(super) fn persisted_request_key(self) -> String {
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
pub(super) struct HlsAppService {
    builder: FfmpegCommandBuilder,
    runner: FfmpegHlsRunner,
    hardware_acceleration: HardwareAcceleration,
    in_flight: Arc<Mutex<HashSet<HlsRequestKey>>>,
}

impl HlsAppService {
    pub(super) fn new(config: &TaruServerConfig) -> Self {
        let hardware_policy = config.transcode.hardware_policy();
        let hardware_acceleration = hardware_policy.requested;
        let transcode_budget = config.transcode.resource_budget();
        let guard = RemuxRuntimeGuard::new(RemuxRuntimeLimits {
            max_concurrent_sessions: transcode_budget.slots_for(hardware_acceleration),
            timeout_ms: config.remux_timeout_ms,
        });

        Self {
            builder: FfmpegCommandBuilder::new(&config.ffmpeg_path),
            runner: FfmpegHlsRunner::new(guard),
            hardware_acceleration,
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
                hardware_acceleration: self.hardware_acceleration,
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
                record_playback_session_finished_event(sessions, &session).await;

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

pub(super) async fn plan_direct_play_with_backend(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
    range_request: DirectPlayRangeRequest,
) -> Result<(DirectPlayResponsePlan, DirectPlaySourceBody)> {
    let response =
        plan_direct_play_response_with_backend(source, uri, backend, range_request).await?;

    if response.is_range_not_satisfiable() {
        return Ok((response, DirectPlaySourceBody::Empty));
    }

    match local_source_path_and_len(source, uri, backend).await {
        Ok((local_path, _total_len)) => {
            return Ok((response, DirectPlaySourceBody::LocalPath(local_path)));
        }
        Err(TaruError::Unsupported(_)) => {}
        Err(err) => return Err(err),
    }

    let range = response.range.map(|range| ByteRange {
        offset: range.start,
        length: Some(range.len()),
    });
    let stream = backend.stream_range(uri, range).await?;

    Ok((
        response,
        DirectPlaySourceBody::Stream(DirectPlayStreamBody::unbudgeted(stream)),
    ))
}

async fn plan_direct_play_response_with_backend(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
    range_request: DirectPlayRangeRequest,
) -> Result<DirectPlayResponsePlan> {
    let metadata = backend.stat(uri).await?;
    let total_len = metadata.len.ok_or_else(|| TaruError::Storage {
        uri: source.locator.clone(),
        message: "direct play requires a known source length".to_owned(),
    })?;
    let content_type = content_type_for_file_name(&source.file_name).to_owned();

    Ok(plan_direct_play_response(
        total_len,
        content_type,
        range_request,
    ))
}

fn should_budget_remote_stream(uri: &StorageUri) -> bool {
    uri.scheme() != "local"
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

#[cfg(test)]
pub(super) async fn source_path_for_ffmpeg_with_backend(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
    staging_root: PathBuf,
) -> Result<PathBuf> {
    match local_source_path_and_len(source, uri, backend).await {
        Ok((path, _len)) => Ok(path),
        Err(TaruError::Unsupported(_)) => {
            let staged = backend
                .stage(StageRequest::new(uri.clone(), staging_root))
                .await?;
            Ok(staged.path)
        }
        Err(err) => Err(err),
    }
}

async fn local_source_path_and_len(
    source: &MediaSource,
    uri: &StorageUri,
    backend: &dyn StorageBackend,
) -> Result<(PathBuf, u64)> {
    let metadata = backend.stat(uri).await?;
    let virtual_file = backend.open_range(uri, None).await?;
    let local_path = virtual_file.local_path_hint.ok_or_else(|| {
        TaruError::Unsupported("local playback operations currently require a local path hint")
    })?;
    let total_len = match metadata.len {
        Some(len) => len,
        None => tokio::fs::metadata(&local_path)
            .await
            .map_err(|err| TaruError::Storage {
                uri: source.locator.clone(),
                message: format!("failed to read playback source length: {err}"),
            })?
            .len(),
    };

    Ok((local_path, total_len))
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

async fn record_playback_session_finished_event(
    store: &SqliteStore,
    session: &TranscodeSessionRecord,
) {
    let payload = serde_json::json!({
        "session_id": session.id,
        "source_id": session.source_id,
        "kind": session.kind,
        "request_key": &session.request_key,
        "state": session.state,
    });
    let idempotency_key = format!("playback.session_finished:{}", session.id);
    if let Err(err) = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::PlaybackSessionFinished,
            subject: DomainEventSubject::PlaybackSession(session.id),
            library_id: None,
            source_id: Some(session.source_id),
            idempotency_key: idempotency_key.clone(),
            payload_json: payload.to_string(),
        })
        .await
    {
        warn!(
            session_id = %session.id,
            idempotency_key,
            error = %err,
            "failed to persist playback session outbox event"
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
