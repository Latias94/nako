use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Serialize;
use taru_api::public_client::{PlaybackDecisionResponse, playback_decision_response_to_dto};
use taru_core::{
    DomainEventKind, DomainEventSubject, EventId, EventOutboxRepository, MediaProbeRepository,
    MediaRepository, MediaSource, MediaSourceId, NewOutboxEvent, Result, TaruError,
    TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind, TranscodeSessionListFilter,
    TranscodeSessionRecord, TranscodeSessionRepository, TranscodeSessionState,
};
use taru_db::TaruDatabase;
use taru_streaming::{
    ClientPlaybackCapabilities, DirectPlayRangeRequest, DirectPlayResponsePlan, PlaybackDecision,
    PlaybackExecutionPlan, PlaybackProfile, PlaybackSelectionContext, PlaybackSelectionRequest,
    PlaybackStorageContext, select_playback_source,
};
use taru_transcode::{
    HardwareAccelerationPolicy, HardwareAccelerationReport, HardwareAccelerationSelection,
    OutputContainer, RemuxContainer, TranscodePlan, TranscodeProfileIdentity,
    TranscodeResourceBudget,
};
use taru_vfs::{StorageBackend, StorageCapabilities, StorageUri};
use tracing::{error, warn};

use crate::config::TaruServerConfig;

use super::{runtime::RuntimeSupervisor, storage::StorageBackendRegistry};

mod control;
mod direct;
mod hls;
mod input;
mod remux;

use control::PlaybackSessionCancellationRegistry;
pub(crate) use direct::{
    DirectPlaySourceBody, DirectPlaySourcePlan, DirectPlayStreamBody, plan_direct_play_with_backend,
};
use direct::{plan_direct_play_response_with_backend, should_budget_remote_stream};
use hls::HlsAppService;
use input::FfmpegInputService;
#[cfg(test)]
pub(crate) use input::source_path_for_ffmpeg_with_backend;
use remux::RemuxAppService;
pub(crate) use remux::RemuxRequestKey;

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

#[derive(Clone, Debug, Serialize)]
pub struct RemuxSessionStart {
    pub source: MediaSource,
    pub decision: PlaybackDecision,
    pub output_path: PathBuf,
    pub output_container: RemuxContainer,
    pub session: TranscodeSessionRecord,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackRuntimeDiagnostics {
    pub hardware_policy: HardwareAccelerationPolicy,
    pub hardware_report: HardwareAccelerationReport,
    pub hardware_selection: HardwareAccelerationSelection,
    pub transcode_budget: TranscodeResourceBudget,
    pub selected_hls_slots: usize,
    pub remux_concurrency: usize,
    pub remux_timeout_ms: u64,
    pub remote_stream_concurrency: usize,
    pub remote_stage_concurrency: usize,
    pub staging_max_bytes: u64,
    pub staging_retention_ms: u64,
    pub staging_cleanup_on_startup: bool,
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
        profile_identity: &TranscodeProfileIdentity,
        container: RemuxContainer,
    ) -> Result<PathBuf> {
        let output = self
            .root
            .join(source_id.to_string())
            .join(profile_identity.storage_slug())
            .join(format!("stream.{}", container.file_extension()));

        if !output.starts_with(&self.root) {
            return Err(TaruError::storage_security_violation(
                self.root.display().to_string(),
                "remux staging output escaped the staging root",
            ));
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

    pub fn single_variant_layout(
        &self,
        source_id: MediaSourceId,
        profile_identity: &TranscodeProfileIdentity,
    ) -> Result<HlsOutputLayout> {
        let output_dir = self
            .root
            .join(source_id.to_string())
            .join(profile_identity.storage_slug());
        let playlist_path = output_dir.join("playlist.m3u8");
        let segment_pattern = output_dir.join("segment_%05d.ts");

        for path in [&output_dir, &playlist_path, &segment_pattern] {
            if !path.starts_with(&self.root) {
                return Err(TaruError::storage_security_violation(
                    self.root.display().to_string(),
                    "hls staging output escaped the staging root",
                ));
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
pub(crate) struct PlaybackAppService {
    config: TaruServerConfig,
    store: TaruDatabase,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
    input: FfmpegInputService,
    cancellations: PlaybackSessionCancellationRegistry,
    remux: RemuxAppService,
    hls: HlsAppService,
}

impl PlaybackAppService {
    pub(super) fn new(
        config: TaruServerConfig,
        store: TaruDatabase,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
    ) -> Result<Self> {
        let cancellations = PlaybackSessionCancellationRegistry::default();

        Ok(Self {
            input: FfmpegInputService::new(config.clone(), store.clone(), runtime.clone()),
            remux: RemuxAppService::new(&config, cancellations.clone()),
            hls: HlsAppService::new(&config, cancellations.clone())?,
            config,
            store,
            storage_backends,
            runtime,
            cancellations,
        })
    }

    pub(crate) async fn get_source_playback_decision(
        &self,
        source_id: MediaSourceId,
        client: ClientPlaybackCapabilities,
    ) -> Result<PlaybackDecisionResponse> {
        let source = self.get_source_or_not_found(source_id).await?;
        let probe = self.store.get_media_probe(source.id).await?;
        let context = self.playback_selection_context_for_source(&source).await?;
        let decision = select_playback_source(PlaybackSelectionRequest {
            source: &source,
            probe: probe.as_ref(),
            client: &client,
            context,
        });

        Ok(playback_decision_response_to_dto(source, probe, decision))
    }

    pub(crate) async fn plan_direct_play(
        &self,
        source_id: MediaSourceId,
        range_request: DirectPlayRangeRequest,
    ) -> Result<DirectPlaySourcePlan> {
        let source = self.get_source_or_not_found(source_id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let stream_permit = if should_budget_remote_stream(&uri) {
            Some(backend.acquire_stream_permit().await?)
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

    pub(crate) async fn plan_direct_play_preflight(
        &self,
        source_id: MediaSourceId,
        range_request: DirectPlayRangeRequest,
    ) -> Result<DirectPlayResponsePlan> {
        let source = self.get_source_or_not_found(source_id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;

        plan_direct_play_response_with_backend(&source, &uri, backend.as_ref(), range_request).await
    }

    pub(crate) async fn remux_source(
        &self,
        request: RemuxSourceRequest,
    ) -> Result<RemuxSourceOutput> {
        let context = self.remux_source_context(&request).await?;
        self.run_remux_source_context(context).await
    }

    pub(crate) async fn remux_source_or_wait(
        &self,
        request: RemuxSourceRequest,
    ) -> Result<RemuxSourceOutput> {
        let context = self.remux_source_context(&request).await?;
        if let Some(active) = self
            .store
            .find_active_transcode_session(
                context.source.id,
                TranscodeSessionKind::Remux,
                &context.request_key,
            )
            .await?
        {
            return self.wait_for_remux_source_context(context, active.id).await;
        }

        self.run_remux_source_context(context).await
    }

    pub(crate) async fn start_remux_source(
        &self,
        request: RemuxSourceRequest,
    ) -> Result<RemuxSessionStart> {
        let context = self.remux_source_context(&request).await?;
        if let Some(active) = self
            .store
            .find_active_transcode_session(
                context.source.id,
                TranscodeSessionKind::Remux,
                &context.request_key,
            )
            .await?
        {
            return Ok(context.session_start(active));
        }

        if let Some(latest) = self
            .store
            .find_latest_transcode_session(
                context.source.id,
                TranscodeSessionKind::Remux,
                &context.request_key,
            )
            .await?
        {
            if latest.state == TranscodeSessionState::Finished
                && latest.output_path == context.output_path
                && path_exists(&context.output_path)?
            {
                return Ok(context.session_start(latest));
            }
        }

        let task_app = self.clone();
        let task_request = request.clone();
        self.runtime
            .spawn("playback_remux_start", "playback.remux", async move {
                if let Err(error) = task_app.remux_source(task_request).await {
                    warn!(error = %error, "background remux start failed");
                }
            });

        self.wait_for_started_remux_source_context(context).await
    }

    async fn run_remux_source_context(
        &self,
        context: RemuxSourceContext,
    ) -> Result<RemuxSourceOutput> {
        let input = self
            .input
            .source_input_for_ffmpeg(&context.source, &context.uri, &context.backend)
            .await?;
        let result = self
            .remux
            .run(
                &self.store,
                context.source,
                context.decision,
                input.path.clone(),
                context.output_path,
                context.output_container,
                context.profile_identity,
            )
            .await;
        match result {
            Ok(output) => {
                self.input.release_source_input(input).await?;
                Ok(output)
            }
            Err(err) => {
                if let Err(release_err) = self.input.release_source_input(input).await {
                    warn!(
                        error = %release_err,
                        "failed to release remux staging lease after error"
                    );
                }
                Err(err)
            }
        }
    }

    async fn wait_for_started_remux_source_context(
        &self,
        context: RemuxSourceContext,
    ) -> Result<RemuxSessionStart> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(active) = self
                .store
                .find_active_transcode_session(
                    context.source.id,
                    TranscodeSessionKind::Remux,
                    &context.request_key,
                )
                .await?
            {
                return Ok(context.session_start(active));
            }

            if let Some(latest) = self
                .store
                .find_latest_transcode_session(
                    context.source.id,
                    TranscodeSessionKind::Remux,
                    &context.request_key,
                )
                .await?
            {
                if latest.state.is_terminal() {
                    return Ok(context.session_start(latest));
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(TaruError::Conflict {
                    message: format!(
                        "remux request for source {} did not expose a playback session before timeout",
                        context.source.id
                    ),
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }

    async fn wait_for_remux_source_context(
        &self,
        context: RemuxSourceContext,
        session_id: TranscodeSessionId,
    ) -> Result<RemuxSourceOutput> {
        let deadline = tokio::time::Instant::now()
            + std::time::Duration::from_millis(self.config.remux_timeout_ms.max(1));
        loop {
            let session = self.get_transcode_session(session_id).await?;
            match session.state {
                TranscodeSessionState::Finished => {
                    if !path_exists(&context.output_path)? {
                        return Err(TaruError::storage_io(
                            context.output_path.display().to_string(),
                            "finished remux session output is missing",
                        ));
                    }

                    return Ok(RemuxSourceOutput {
                        source: context.source,
                        decision: context.decision,
                        output_path: context.output_path,
                        output_container: context.output_container,
                        disposition: RemuxSourceDisposition::ReusedExisting,
                        session: Some(session),
                    });
                }
                TranscodeSessionState::Cancelled => {
                    return Ok(RemuxSourceOutput {
                        source: context.source,
                        decision: context.decision,
                        output_path: context.output_path,
                        output_container: context.output_container,
                        disposition: RemuxSourceDisposition::Cancelled,
                        session: Some(session),
                    });
                }
                TranscodeSessionState::Failed => {
                    return Err(TaruError::Provider {
                        provider: "ffmpeg_remux".to_owned(),
                        message: session
                            .failure_message
                            .unwrap_or_else(|| "remux runner failed".to_owned()),
                    });
                }
                TranscodeSessionState::Planned
                | TranscodeSessionState::Starting
                | TranscodeSessionState::Running
                | TranscodeSessionState::CancelRequested => {}
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(TaruError::Provider {
                    provider: "ffmpeg_remux".to_owned(),
                    message: format!("remux session {session_id} timed out while waiting"),
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    async fn remux_source_context(
        &self,
        request: &RemuxSourceRequest,
    ) -> Result<RemuxSourceContext> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe = self.store.get_media_probe(source.id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let mut context = playback_selection_context(&uri, backend.as_ref()).await;
        context.preferences.remux_output_container = Some(request.output_container);
        let playback_profile = PlaybackProfile::from_context(&request.client, context.clone());
        let decision = select_playback_source(PlaybackSelectionRequest {
            source: &source,
            probe: probe.as_ref(),
            client: &request.client,
            context,
        });
        let output_container = remux_output_container(&decision)?;
        let profile_identity = playback_profile
            .remux_transcode_profile(output_container)
            .identity();
        let staging = RemuxStagingPolicy::new(&self.config.remux_staging_root)?;
        let output_path = staging.output_path(source.id, &profile_identity, output_container)?;
        let request_key = RemuxRequestKey {
            source_id: source.id,
            profile_identity: profile_identity.clone(),
        }
        .persisted_request_key();

        Ok(RemuxSourceContext {
            source,
            decision,
            uri,
            backend,
            output_path,
            output_container,
            profile_identity,
            request_key,
        })
    }

    pub(crate) async fn hls_source(&self, request: HlsSourceRequest) -> Result<HlsSourceOutput> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe = self.store.get_media_probe(source.id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let mut context = playback_selection_context(&uri, backend.as_ref()).await;
        context.preferences.transcode_output_container = Some(OutputContainer::Hls);
        let playback_profile = PlaybackProfile::from_context(&request.client, context.clone());
        let decision = select_playback_source(PlaybackSelectionRequest {
            source: &source,
            probe: probe.as_ref(),
            client: &request.client,
            context,
        });
        let transcode_plan = hls_transcode_plan(&decision)?;
        let profile_identity = playback_profile
            .hls_transcode_profile(transcode_plan, self.hls.hardware_selection.acceleration)
            .identity();
        let input = self
            .input
            .source_input_for_ffmpeg(&source, &uri, &backend)
            .await?;
        let staging = HlsStagingPolicy::new(self.config.remux_staging_root.join("hls"))?;
        let layout = staging.single_variant_layout(source.id, &profile_identity)?;
        let result = self
            .hls
            .run(
                &self.store,
                source,
                decision,
                input.path.clone(),
                layout,
                profile_identity,
            )
            .await;
        match result {
            Ok(output) => {
                self.input.release_source_input(input).await?;
                Ok(output)
            }
            Err(err) => {
                if let Err(release_err) = self.input.release_source_input(input).await {
                    warn!(
                        error = %release_err,
                        "failed to release hls staging lease after error"
                    );
                }
                Err(err)
            }
        }
    }

    pub(crate) async fn hls_playlist(
        &self,
        request: HlsSourceRequest,
    ) -> Result<HlsPlaylistOutput> {
        let output = self.hls_source(request).await?;

        if output.disposition == HlsSourceDisposition::Cancelled {
            return Err(TaruError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message: "hls session was cancelled".to_owned(),
            });
        }

        let body = tokio::fs::read_to_string(&output.playlist_path)
            .await
            .map_err(|err| {
                TaruError::storage_io(
                    output.playlist_path.display().to_string(),
                    format!("failed to read hls playlist: {err}"),
                )
            })?;

        Ok(HlsPlaylistOutput {
            source: output.source,
            decision: output.decision,
            body: rewrite_hls_playlist(&body, output.session.id),
            session: output.session,
        })
    }

    pub(crate) async fn plan_hls_segment(
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

        let segment_dir = session.output_path.parent().ok_or_else(|| {
            TaruError::storage_security_violation(
                session.output_path.display().to_string(),
                "hls playlist path does not have a parent directory",
            )
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

    pub(crate) async fn get_transcode_session(
        &self,
        session_id: TranscodeSessionId,
    ) -> Result<TranscodeSessionRecord> {
        self.store
            .get_transcode_session(session_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })
    }

    pub(crate) async fn list_transcode_sessions(
        &self,
        filter: TranscodeSessionListFilter,
        page: taru_core::PageRequest,
    ) -> Result<Vec<TranscodeSessionRecord>> {
        self.store.list_transcode_sessions(filter, page).await
    }

    #[must_use]
    pub(crate) fn runtime_diagnostics(&self) -> PlaybackRuntimeDiagnostics {
        let hardware_policy = self.config.transcode.hardware_policy();
        let transcode_budget = self.config.transcode.resource_budget();

        PlaybackRuntimeDiagnostics {
            hardware_policy,
            hardware_report: self.hls.hardware_report.clone(),
            hardware_selection: self.hls.hardware_selection.clone(),
            transcode_budget,
            selected_hls_slots: transcode_budget
                .slots_for(self.hls.hardware_selection.acceleration),
            remux_concurrency: self.config.remux_concurrency.max(1),
            remux_timeout_ms: self.config.remux_timeout_ms.max(1),
            remote_stream_concurrency: self.config.playback.remote_stream_concurrency.max(1),
            remote_stage_concurrency: self.config.playback.remote_stage_concurrency.max(1),
            staging_max_bytes: self.config.staging.max_bytes,
            staging_retention_ms: self.config.staging.retention_ms,
            staging_cleanup_on_startup: self.config.staging.cleanup_on_startup,
        }
    }

    pub(crate) async fn cancel_transcode_session(
        &self,
        session_id: TranscodeSessionId,
    ) -> Result<TranscodeSessionRecord> {
        let session = self.get_transcode_session(session_id).await?;

        if session.state.is_terminal() {
            return Err(TaruError::Conflict {
                message: format!(
                    "playback session {session_id} is already terminal; current state is {}",
                    session.state.as_str()
                ),
            });
        }

        if !self.cancellations.cancel(session_id) {
            return Err(TaruError::Conflict {
                message: format!(
                    "playback session {session_id} is active but is not running in this process"
                ),
            });
        }

        self.store
            .request_transcode_session_cancellation(
                session_id,
                "playback session cancellation requested".to_owned(),
            )
            .await?
            .ok_or_else(|| TaruError::Conflict {
                message: format!(
                    "playback session {session_id} is no longer active enough to cancel"
                ),
            })
    }

    #[cfg(test)]
    pub(super) async fn source_path_for_ffmpeg(&self, source: &MediaSource) -> Result<PathBuf> {
        let (uri, backend) = self.storage_backend_for_media_source(source).await?;
        self.input
            .source_path_for_ffmpeg(source, &uri, &backend)
            .await
    }

    async fn get_source_or_not_found(&self, source_id: MediaSourceId) -> Result<MediaSource> {
        self.store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) async fn storage_backend_for_media_source(
        &self,
        source: &MediaSource,
    ) -> Result<(StorageUri, Arc<super::storage::LibraryStorageBackend>)> {
        self.storage_backends.backend_for_media_source(source).await
    }

    async fn playback_selection_context_for_source(
        &self,
        source: &MediaSource,
    ) -> Result<PlaybackSelectionContext> {
        let (uri, backend) = self.storage_backend_for_media_source(source).await?;
        Ok(playback_selection_context(&uri, backend.as_ref()).await)
    }
}

#[derive(Clone, Debug)]
struct RemuxSourceContext {
    source: MediaSource,
    decision: PlaybackDecision,
    uri: StorageUri,
    backend: Arc<super::storage::LibraryStorageBackend>,
    output_path: PathBuf,
    output_container: RemuxContainer,
    profile_identity: TranscodeProfileIdentity,
    request_key: String,
}

impl RemuxSourceContext {
    fn session_start(self, session: TranscodeSessionRecord) -> RemuxSessionStart {
        RemuxSessionStart {
            source: self.source,
            decision: self.decision,
            output_path: self.output_path,
            output_container: self.output_container,
            session,
        }
    }
}

async fn playback_selection_context(
    uri: &StorageUri,
    backend: &super::storage::LibraryStorageBackend,
) -> PlaybackSelectionContext {
    let capabilities = backend
        .stat(uri)
        .await
        .ok()
        .map(|metadata| metadata.capabilities);

    PlaybackSelectionContext {
        storage: PlaybackStorageContext {
            remote: should_budget_remote_stream(uri),
            range_readable: capabilities
                .map(|capabilities| capabilities.contains(StorageCapabilities::RANGE_READABLE)),
        },
        preferences: Default::default(),
    }
}

fn remux_output_container(decision: &PlaybackDecision) -> Result<RemuxContainer> {
    match &decision.execution {
        PlaybackExecutionPlan::Remux(plan) => Ok(plan.output_container),
        _ => Err(TaruError::Unsupported(
            "remux app service requires a remux playback decision",
        )),
    }
}

fn hls_transcode_plan(decision: &PlaybackDecision) -> Result<&TranscodePlan> {
    match &decision.execution {
        PlaybackExecutionPlan::Transcode(plan) if plan.output_container == OutputContainer::Hls => {
            Ok(plan)
        }
        _ => Err(TaruError::Unsupported(
            "hls app service requires an hls transcode playback decision",
        )),
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
        TaruError::Storage { uri, kind, .. } => {
            TaruError::storage(uri, kind, "remux staging operation failed")
        }
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
        TaruError::Storage { uri, kind, .. } => {
            TaruError::storage(uri, kind, "hls staging operation failed")
        }
        TaruError::InvalidInput { message } => TaruError::InvalidInput {
            message: format!("invalid hls request: {message}"),
        },
        other => other,
    }
}

fn path_exists(path: &Path) -> Result<bool> {
    path.try_exists().map_err(|err| {
        TaruError::storage_io(
            path.display().to_string(),
            format!("failed to check path: {err}"),
        )
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
    sessions: &TaruDatabase,
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
    store: &TaruDatabase,
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
        return Err(TaruError::storage_security_violation(
            output_path.display().to_string(),
            "remux output path does not have a parent directory",
        ));
    };

    tokio::fs::create_dir_all(parent).await.map_err(|err| {
        TaruError::storage_io(
            parent.display().to_string(),
            format!("failed to create remux output directory: {err}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use taru_transcode::{
        HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationReport,
        StaticHardwareAccelerationDetector,
    };

    use super::*;
    use crate::config::{MetadataConfig, PlaybackConfig, StagingConfig, TranscodeConfig};

    fn test_config(transcode: TranscodeConfig) -> TaruServerConfig {
        TaruServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            auth: crate::config::AuthConfig::disabled(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            remux_timeout_ms: 1_000,
            remux_staging_root: PathBuf::from("cache/remux"),
            metadata: MetadataConfig::default(),
            transcode,
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: Vec::new(),
        }
    }

    #[test]
    fn hls_service_uses_available_hardware_detector_selection() {
        let config = test_config(TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::Nvenc,
            hardware_fallback: HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 1,
            gpu_concurrency: 2,
        });
        let detector =
            StaticHardwareAccelerationDetector::new(HardwareAccelerationReport::with_available([
                HardwareAcceleration::Nvenc,
            ]));

        let service = HlsAppService::new_with_hardware_detector(&config, &detector).unwrap();

        assert_eq!(
            service.hardware_selection.acceleration,
            HardwareAcceleration::Nvenc
        );
        assert!(!service.hardware_selection.fallback_used);
    }
}
