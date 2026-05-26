use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use nako_api::public_client::{PlaybackDecisionResponse, playback_decision_response_to_dto};
use nako_core::{
    EventOutboxRepository, MediaProbeRepository, MediaProbeResult, MediaRepository, MediaSource,
    MediaSourceId, NakoError, NewOutboxEvent, NewPlaybackSession, NewTranscodeSession,
    OutboxEventRecord, PageRequest, PlaybackSessionHeartbeat, PlaybackSessionId,
    PlaybackSessionListFilter, PlaybackSessionMode, PlaybackSessionRecord,
    PlaybackSessionRepository, PlaybackSessionState, Result, StagingManifestRepository,
    TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionRepository, TranscodeSessionState, UserPrincipalId,
};
use nako_playback::{
    ClientPlaybackCapabilities, PlaybackDecision, PlaybackPlanner, PlaybackPlanningRequest,
    PlaybackProfile, PlaybackSelectionContext,
};
use nako_streaming::{DirectPlayRangeRequest, DirectPlayResponsePlan};
use nako_transcode::{
    HardwareAccelerationPolicy, HardwareAccelerationReport, HardwareAccelerationSelection,
    RemuxContainer, TranscodeRequestIdentity, TranscodeResourceBudget, TranscodeSourceIdentity,
};
use nako_vfs::StorageUri;
use serde::Serialize;
use tracing::warn;

use crate::config::NakoServerConfig;

use super::{runtime::RuntimeSupervisor, storage::StorageBackendRegistry};

mod control;
mod direct;
mod events;
mod failure;
mod hls;
mod input;
mod paths;
mod playlist;
mod remux;
mod selection;
mod staging_policy;

use control::PlaybackSessionCancellationRegistry;
pub(crate) use direct::{
    DirectPlaySourceBody, DirectPlaySourcePlan, DirectPlayStreamBody, plan_direct_play_with_backend,
};
use direct::{plan_direct_play_response_with_backend, should_budget_remote_stream};
use events::record_playback_session_finished_event;
use failure::{map_hls_runner_error, map_remux_runner_error, persist_session_failure};
use hls::HlsAppService;
use input::FfmpegInputService;
#[cfg(test)]
pub(crate) use input::source_path_for_ffmpeg_with_backend;
use paths::{ensure_remux_output_parent, path_exists};
use playlist::{rewrite_hls_playlist, validate_hls_segment_name};
use remux::RemuxAppService;
pub(crate) use remux::RemuxRequestKey;
use selection::{hls_transcode_plan, playback_selection_context, remux_output_container};
pub(crate) use staging_policy::{HlsOutputLayout, HlsStagingPolicy, RemuxStagingPolicy};

#[async_trait]
pub(crate) trait PlaybackRuntimeStore: std::fmt::Debug + Send + Sync {
    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>>;

    async fn get_media_probe(&self, id: MediaSourceId) -> Result<Option<MediaProbeResult>>;

    async fn create_playback_session(
        &self,
        session: NewPlaybackSession,
    ) -> Result<PlaybackSessionRecord>;

    async fn get_playback_session(
        &self,
        id: PlaybackSessionId,
    ) -> Result<Option<PlaybackSessionRecord>>;

    async fn list_playback_sessions(
        &self,
        filter: PlaybackSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackSessionRecord>>;

    async fn link_playback_session_transcode(
        &self,
        id: PlaybackSessionId,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<PlaybackSessionRecord>;

    async fn record_playback_session_heartbeat(
        &self,
        heartbeat: PlaybackSessionHeartbeat,
    ) -> Result<Option<PlaybackSessionRecord>>;

    async fn set_playback_session_state(
        &self,
        id: PlaybackSessionId,
        state: PlaybackSessionState,
        ended_at_ms: Option<i64>,
    ) -> Result<Option<PlaybackSessionRecord>>;

    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord>;

    async fn get_transcode_session(
        &self,
        id: TranscodeSessionId,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn find_latest_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn find_active_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn set_transcode_session_state(
        &self,
        id: TranscodeSessionId,
        state: TranscodeSessionState,
        failure_category: Option<TranscodeFailureCategory>,
        failure_message: Option<String>,
    ) -> Result<TranscodeSessionRecord>;

    async fn request_transcode_session_cancellation(
        &self,
        id: TranscodeSessionId,
        failure_message: String,
    ) -> Result<Option<TranscodeSessionRecord>>;

    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord>;
}

#[async_trait]
impl<T> PlaybackRuntimeStore for T
where
    T: EventOutboxRepository
        + MediaProbeRepository
        + MediaRepository
        + PlaybackSessionRepository
        + TranscodeSessionRepository
        + std::fmt::Debug
        + Send
        + Sync,
{
    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>> {
        MediaRepository::get_media_source(self, id).await
    }

    async fn get_media_probe(&self, id: MediaSourceId) -> Result<Option<MediaProbeResult>> {
        MediaProbeRepository::get_media_probe(self, id).await
    }

    async fn create_playback_session(
        &self,
        session: NewPlaybackSession,
    ) -> Result<PlaybackSessionRecord> {
        PlaybackSessionRepository::create_playback_session(self, session).await
    }

    async fn get_playback_session(
        &self,
        id: PlaybackSessionId,
    ) -> Result<Option<PlaybackSessionRecord>> {
        PlaybackSessionRepository::get_playback_session(self, id).await
    }

    async fn list_playback_sessions(
        &self,
        filter: PlaybackSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackSessionRecord>> {
        PlaybackSessionRepository::list_playback_sessions(self, filter, page).await
    }

    async fn link_playback_session_transcode(
        &self,
        id: PlaybackSessionId,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<PlaybackSessionRecord> {
        PlaybackSessionRepository::link_playback_session_transcode(self, id, transcode_session_id)
            .await
    }

    async fn record_playback_session_heartbeat(
        &self,
        heartbeat: PlaybackSessionHeartbeat,
    ) -> Result<Option<PlaybackSessionRecord>> {
        PlaybackSessionRepository::record_playback_session_heartbeat(self, heartbeat).await
    }

    async fn set_playback_session_state(
        &self,
        id: PlaybackSessionId,
        state: PlaybackSessionState,
        ended_at_ms: Option<i64>,
    ) -> Result<Option<PlaybackSessionRecord>> {
        PlaybackSessionRepository::set_playback_session_state(self, id, state, ended_at_ms).await
    }

    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord> {
        TranscodeSessionRepository::create_transcode_session(self, session).await
    }

    async fn get_transcode_session(
        &self,
        id: TranscodeSessionId,
    ) -> Result<Option<TranscodeSessionRecord>> {
        TranscodeSessionRepository::get_transcode_session(self, id).await
    }

    async fn find_latest_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        TranscodeSessionRepository::find_latest_transcode_session(
            self,
            source_id,
            kind,
            request_key,
        )
        .await
    }

    async fn find_active_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        TranscodeSessionRepository::find_active_transcode_session(
            self,
            source_id,
            kind,
            request_key,
        )
        .await
    }

    async fn set_transcode_session_state(
        &self,
        id: TranscodeSessionId,
        state: TranscodeSessionState,
        failure_category: Option<TranscodeFailureCategory>,
        failure_message: Option<String>,
    ) -> Result<TranscodeSessionRecord> {
        TranscodeSessionRepository::set_transcode_session_state(
            self,
            id,
            state,
            failure_category,
            failure_message,
        )
        .await
    }

    async fn request_transcode_session_cancellation(
        &self,
        id: TranscodeSessionId,
        failure_message: String,
    ) -> Result<Option<TranscodeSessionRecord>> {
        TranscodeSessionRepository::request_transcode_session_cancellation(
            self,
            id,
            failure_message,
        )
        .await
    }

    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord> {
        EventOutboxRepository::enqueue_outbox_event(self, event).await
    }
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

#[derive(Clone, Debug)]
pub(crate) struct StartPlaybackSessionRequest {
    pub principal_id: UserPrincipalId,
    pub source_id: MediaSourceId,
    pub mode: PlaybackSessionMode,
    pub client: Option<ClientPlaybackCapabilities>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackSessionHeartbeatRequest {
    pub session_id: PlaybackSessionId,
    pub state: PlaybackSessionState,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackSupportEvidenceContext {
    pub session: Option<TranscodeSessionRecord>,
    pub source: Option<MediaSource>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackSupportEvidenceRequest {
    pub session_id: Option<TranscodeSessionId>,
    pub source_id: Option<MediaSourceId>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlaybackAppService {
    config: NakoServerConfig,
    runtime_store: Arc<dyn PlaybackRuntimeStore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
    input: FfmpegInputService,
    planner: PlaybackPlanner,
    cancellations: PlaybackSessionCancellationRegistry,
    remux: RemuxAppService,
    hls: HlsAppService,
}

impl PlaybackAppService {
    pub(super) fn new(
        config: NakoServerConfig,
        runtime_store: Arc<dyn PlaybackRuntimeStore>,
        staging_store: Arc<dyn StagingManifestRepository>,
        storage_backends: StorageBackendRegistry,
        runtime: RuntimeSupervisor,
    ) -> Result<Self> {
        let cancellations = PlaybackSessionCancellationRegistry::default();
        let input = FfmpegInputService::new(config.clone(), staging_store, runtime.clone());

        Ok(Self {
            input,
            planner: PlaybackPlanner::new(),
            remux: RemuxAppService::new(&config, cancellations.clone()),
            hls: HlsAppService::new(&config, cancellations.clone())?,
            config,
            runtime_store,
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
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let context = self.playback_selection_context_for_source(&source).await?;
        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            client: &client,
            context,
        });

        Ok(playback_decision_response_to_dto(source, probe, decision))
    }

    pub(crate) async fn start_playback_session(
        &self,
        request: StartPlaybackSessionRequest,
    ) -> Result<PlaybackSessionRecord> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let now_ms = crate::app::current_time_ms()?;
        let client_capabilities_json = request
            .client
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| NakoError::InvalidInput {
                message: format!("playback client capabilities could not be serialized: {err}"),
            })?;

        PlaybackRuntimeStore::create_playback_session(
            self.runtime_store.as_ref(),
            NewPlaybackSession {
                id: PlaybackSessionId::new(),
                principal_id: request.principal_id,
                source_id: source.id,
                item_id: source.item_id,
                mode: request.mode,
                state: PlaybackSessionState::Active,
                client_capabilities_json,
                started_at_ms: now_ms,
                updated_at_ms: now_ms,
            },
        )
        .await
    }

    pub(crate) async fn get_playback_session(
        &self,
        session_id: PlaybackSessionId,
    ) -> Result<PlaybackSessionRecord> {
        PlaybackRuntimeStore::get_playback_session(self.runtime_store.as_ref(), session_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "playback_session",
                id: session_id.to_string(),
            })
    }

    pub(crate) async fn list_playback_sessions(
        &self,
        filter: PlaybackSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackSessionRecord>> {
        PlaybackRuntimeStore::list_playback_sessions(self.runtime_store.as_ref(), filter, page)
            .await
    }

    pub(crate) async fn record_playback_session_heartbeat(
        &self,
        request: PlaybackSessionHeartbeatRequest,
    ) -> Result<PlaybackSessionRecord> {
        let now_ms = crate::app::current_time_ms()?;
        PlaybackRuntimeStore::record_playback_session_heartbeat(
            self.runtime_store.as_ref(),
            PlaybackSessionHeartbeat {
                id: request.session_id,
                state: request.state,
                position_ms: request.position_ms,
                duration_ms: request.duration_ms,
                heartbeat_at_ms: now_ms,
            },
        )
        .await?
        .ok_or_else(|| NakoError::Conflict {
            message: format!(
                "playback session {} is terminal or no longer accepts heartbeat updates",
                request.session_id
            ),
        })
    }

    pub(crate) async fn link_playback_session_transcode(
        &self,
        playback_session_id: PlaybackSessionId,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<PlaybackSessionRecord> {
        PlaybackRuntimeStore::link_playback_session_transcode(
            self.runtime_store.as_ref(),
            playback_session_id,
            transcode_session_id,
        )
        .await
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

    pub(crate) async fn start_remux_source(
        &self,
        request: RemuxSourceRequest,
    ) -> Result<RemuxSessionStart> {
        let context = self.remux_source_context(&request).await?;
        if let Some(active) = PlaybackRuntimeStore::find_active_transcode_session(
            self.runtime_store.as_ref(),
            context.source.id,
            TranscodeSessionKind::Remux,
            &context.request_key,
        )
        .await?
        {
            return Ok(context.session_start(active));
        }

        if let Some(latest) = PlaybackRuntimeStore::find_latest_transcode_session(
            self.runtime_store.as_ref(),
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

    pub(crate) async fn wait_for_remux_start(
        &self,
        start: RemuxSessionStart,
    ) -> Result<RemuxSourceOutput> {
        self.wait_for_remux_session_output(
            start.source,
            start.decision,
            start.output_path,
            start.output_container,
            start.session.id,
        )
        .await
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
                self.runtime_store.as_ref(),
                context.source,
                context.decision,
                input.path.clone(),
                context.output_path,
                context.output_container,
                context.request_identity,
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
            if let Some(active) = PlaybackRuntimeStore::find_active_transcode_session(
                self.runtime_store.as_ref(),
                context.source.id,
                TranscodeSessionKind::Remux,
                &context.request_key,
            )
            .await?
            {
                return Ok(context.session_start(active));
            }

            if let Some(latest) = PlaybackRuntimeStore::find_latest_transcode_session(
                self.runtime_store.as_ref(),
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
                return Err(NakoError::Conflict {
                    message: format!(
                        "remux request for source {} did not expose a playback session before timeout",
                        context.source.id
                    ),
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }

    async fn wait_for_remux_session_output(
        &self,
        source: MediaSource,
        decision: PlaybackDecision,
        output_path: PathBuf,
        output_container: RemuxContainer,
        session_id: TranscodeSessionId,
    ) -> Result<RemuxSourceOutput> {
        let deadline = tokio::time::Instant::now()
            + std::time::Duration::from_millis(self.config.remux_timeout_ms.max(1));
        loop {
            let session = self.get_transcode_session(session_id).await?;
            match session.state {
                TranscodeSessionState::Finished => {
                    if !path_exists(&output_path)? {
                        return Err(NakoError::storage_io(
                            output_path.display().to_string(),
                            "finished remux session output is missing",
                        ));
                    }

                    return Ok(RemuxSourceOutput {
                        source,
                        decision,
                        output_path,
                        output_container,
                        disposition: RemuxSourceDisposition::ReusedExisting,
                        session: Some(session),
                    });
                }
                TranscodeSessionState::Cancelled => {
                    return Ok(RemuxSourceOutput {
                        source,
                        decision,
                        output_path,
                        output_container,
                        disposition: RemuxSourceDisposition::Cancelled,
                        session: Some(session),
                    });
                }
                TranscodeSessionState::Failed => {
                    return Err(NakoError::Provider {
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
                return Err(NakoError::Provider {
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
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let mut context = playback_selection_context(&uri, backend.as_ref()).await;
        context.preferences.remux_output_container = Some(request.output_container);
        let playback_profile = PlaybackProfile::from_context(&request.client, context.clone());
        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            client: &request.client,
            context,
        });
        let output_container = remux_output_container(&decision)?;
        let profile_identity = playback_profile
            .try_remux_transcode_profile(output_container)?
            .identity();
        let request_identity =
            profile_identity.bind_source(&TranscodeSourceIdentity::from_media_source(&source));
        let staging = RemuxStagingPolicy::new(&self.config.remux_staging_root)?;
        let output_path = staging.output_path(source.id, &request_identity, output_container)?;
        let request_key = RemuxRequestKey {
            source_id: source.id,
            request_identity: request_identity.clone(),
        }
        .persisted_request_key();

        Ok(RemuxSourceContext {
            source,
            decision,
            uri,
            backend,
            output_path,
            output_container,
            request_identity,
            request_key,
        })
    }

    pub(crate) async fn hls_source(&self, request: HlsSourceRequest) -> Result<HlsSourceOutput> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let mut context = playback_selection_context(&uri, backend.as_ref()).await;
        context.preferences.transcode_output_container = Some(nako_transcode::OutputContainer::Hls);
        let playback_profile = PlaybackProfile::from_context(&request.client, context.clone());
        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            client: &request.client,
            context,
        });
        let transcode_plan = hls_transcode_plan(&decision)?;
        let profile_identity = playback_profile
            .try_hls_transcode_profile(transcode_plan, self.hls.hardware_selection.acceleration)?
            .identity();
        let request_identity =
            profile_identity.bind_source(&TranscodeSourceIdentity::from_media_source(&source));
        let input = self
            .input
            .source_input_for_ffmpeg(&source, &uri, &backend)
            .await?;
        let staging = HlsStagingPolicy::new(self.config.remux_staging_root.join("hls"))?;
        let layout = staging.single_variant_layout(source.id, &request_identity)?;
        let result = self
            .hls
            .run(
                self.runtime_store.as_ref(),
                source,
                decision,
                input.path.clone(),
                layout,
                request_identity,
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
            return Err(NakoError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message: "hls session was cancelled".to_owned(),
            });
        }

        let body = tokio::fs::read_to_string(&output.playlist_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
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
            return Err(NakoError::InvalidInput {
                message: format!("session {session_id} is not an hls transcode session"),
            });
        }

        if session.state != TranscodeSessionState::Finished {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls session {session_id} is not ready; current state is {:?}",
                    session.state
                ),
            });
        }

        let segment_dir = session.output_path.parent().ok_or_else(|| {
            NakoError::storage_security_violation(
                session.output_path.display().to_string(),
                "hls playlist path does not have a parent directory",
            )
        })?;
        let path = segment_dir.join(segment_name);

        if !path.starts_with(segment_dir) {
            return Err(NakoError::InvalidInput {
                message: "hls segment path escaped the session directory".to_owned(),
            });
        }

        if !path_exists(&path)? {
            return Err(NakoError::NotFound {
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
        PlaybackRuntimeStore::get_transcode_session(self.runtime_store.as_ref(), session_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "transcode_session",
                id: session_id.to_string(),
            })
    }

    pub(crate) async fn support_evidence_context(
        &self,
        request: PlaybackSupportEvidenceRequest,
    ) -> Result<PlaybackSupportEvidenceContext> {
        let session = match request.session_id {
            Some(session_id) => Some(self.get_transcode_session(session_id).await?),
            None => None,
        };
        if let (Some(session), Some(source_id)) = (&session, request.source_id) {
            if session.source_id != source_id {
                return Err(NakoError::InvalidInput {
                    message: format!(
                        "playback support evidence source_id {source_id} does not match session {} source_id {}",
                        session.id, session.source_id
                    ),
                });
            }
        }
        let source_id = session
            .as_ref()
            .map(|session| session.source_id)
            .or(request.source_id);
        let source = match source_id {
            Some(source_id) => Some(self.get_source_or_not_found(source_id).await?),
            None => None,
        };

        Ok(PlaybackSupportEvidenceContext { session, source })
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

    pub(crate) async fn cancel_playback_session(
        &self,
        session_id: PlaybackSessionId,
    ) -> Result<PlaybackSessionRecord> {
        let session = self.get_playback_session(session_id).await?;

        if session.state.is_terminal() {
            return Err(NakoError::Conflict {
                message: format!(
                    "playback session {session_id} is already terminal; current state is {}",
                    session.state.as_str()
                ),
            });
        }

        if let Some(transcode_session_id) = session.transcode_session_id {
            let transcode = self.get_transcode_session(transcode_session_id).await?;
            if transcode.state.is_active() {
                if !self.cancellations.cancel(transcode_session_id) {
                    return Err(NakoError::Conflict {
                        message: format!(
                            "playback session {session_id} is active but linked transcode session {transcode_session_id} is not running in this process"
                        ),
                    });
                }
                let _ = PlaybackRuntimeStore::request_transcode_session_cancellation(
                    self.runtime_store.as_ref(),
                    transcode_session_id,
                    "playback session cancellation requested".to_owned(),
                )
                .await?;
            }
        }

        PlaybackRuntimeStore::set_playback_session_state(
            self.runtime_store.as_ref(),
            session_id,
            PlaybackSessionState::Cancelled,
            Some(crate::app::current_time_ms()?),
        )
        .await?
        .ok_or_else(|| NakoError::Conflict {
            message: format!("playback session {session_id} is no longer active enough to cancel"),
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
        PlaybackRuntimeStore::get_media_source(self.runtime_store.as_ref(), source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
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
    request_identity: TranscodeRequestIdentity,
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use nako_transcode::{
        HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationReport,
        StaticHardwareAccelerationDetector,
    };

    use super::*;
    use crate::config::{MetadataConfig, PlaybackConfig, StagingConfig, TranscodeConfig};

    fn test_config(transcode: TranscodeConfig) -> NakoServerConfig {
        NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
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
