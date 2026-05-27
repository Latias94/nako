use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use nako_api::public_client::{PlaybackDecisionResponse, playback_decision_response_to_dto};
use nako_core::{
    AuthenticatedPrincipal, EventOutboxRepository, LibraryAccessLevel, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, MediaSourceId, NakoError, NewOutboxEvent,
    NewPlaybackSession, NewTranscodeSession, OutboxEventRecord, PageRequest, PlaybackPermission,
    PlaybackPolicyRepository, PlaybackSessionHeartbeat, PlaybackSessionId,
    PlaybackSessionListFilter, PlaybackSessionMode, PlaybackSessionRecord,
    PlaybackSessionRepository, PlaybackSessionState, Result, StagingManifestRepository,
    TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionRepository, TranscodeSessionRuntimeMetrics, TranscodeSessionState, UserId,
    UserPrincipalId,
};
use nako_playback::{
    ClientPlaybackCapabilities, EffectivePlaybackPolicy, PlaybackDecision, PlaybackMode,
    PlaybackPlanner, PlaybackPlanningRequest, PlaybackProfile, PlaybackSelectionContext,
    PlaybackTarget, PlaybackTargetProfile,
};
use nako_streaming::{DirectPlayRangeRequest, DirectPlayResponsePlan};
use nako_transcode::{
    HardwareAccelerationPolicy, HardwareAccelerationReport, RemuxContainer,
    TranscodeOutputConstraints, TranscodePipelineReadiness, TranscodeRequestIdentity,
    TranscodeResourceBudget, TranscodeRuntimeInventory, TranscodeSourceIdentity,
};
use nako_vfs::StorageUri;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::NakoServerConfig;

use super::{
    playback_ticket::BrowserPlaybackTicketMode, runtime::RuntimeSupervisor,
    storage::StorageBackendRegistry,
};

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
use selection::{
    hls_pipeline_source_facts, hls_transcode_plan, playback_selection_context,
    remux_output_container,
};
pub(crate) use staging_policy::{HlsOutputLayout, HlsStagingPolicy, RemuxStagingPolicy};

#[async_trait]
pub(crate) trait PlaybackRuntimeStore: std::fmt::Debug + Send + Sync {
    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>>;

    async fn get_media_probe(&self, id: MediaSourceId) -> Result<Option<MediaProbeResult>>;

    async fn resolve_effective_playback_policy(
        &self,
        user_id: UserId,
        library_id: nako_core::LibraryId,
    ) -> Result<EffectivePlaybackPolicy>;

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

    async fn update_transcode_session_runtime_metrics(
        &self,
        id: TranscodeSessionId,
        metrics: TranscodeSessionRuntimeMetrics,
    ) -> Result<Option<TranscodeSessionRecord>>;

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
        + PlaybackPolicyRepository
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

    async fn resolve_effective_playback_policy(
        &self,
        user_id: UserId,
        library_id: nako_core::LibraryId,
    ) -> Result<EffectivePlaybackPolicy> {
        PlaybackPolicyRepository::resolve_effective_playback_policy(self, user_id, library_id).await
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

    async fn update_transcode_session_runtime_metrics(
        &self,
        id: TranscodeSessionId,
        metrics: TranscodeSessionRuntimeMetrics,
    ) -> Result<Option<TranscodeSessionRecord>> {
        TranscodeSessionRepository::update_transcode_session_runtime_metrics(self, id, metrics)
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
    pub response: DirectPlayResponsePlan,
}

#[derive(Clone, Debug)]
pub(crate) struct DirectPlaybackStreamRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub range_request: DirectPlayRangeRequest,
    pub client: ClientPlaybackCapabilities,
}

#[derive(Debug)]
pub(crate) struct DirectPlaybackStreamOutput {
    pub session: Option<PlaybackSessionRecord>,
    pub source_uri: String,
    pub body: DirectPlaySourceBody,
    pub response: DirectPlayResponsePlan,
}

#[derive(Clone, Debug)]
pub(crate) struct DirectPlaybackPreflightRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub range_request: DirectPlayRangeRequest,
    pub client: ClientPlaybackCapabilities,
}

#[derive(Clone, Debug)]
pub(crate) struct DirectPlaybackPreflightOutput {
    pub session: PlaybackSessionRecord,
    pub response: DirectPlayResponsePlan,
}

#[derive(Clone, Debug)]
pub(crate) struct RemuxPlaybackStreamRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub client: ClientPlaybackCapabilities,
    pub output_container: RemuxContainer,
    pub range_request: DirectPlayRangeRequest,
}

#[derive(Clone, Debug)]
pub(crate) struct RemuxPlaybackStreamOutput {
    pub session: PlaybackSessionRecord,
    pub output_path: PathBuf,
    pub response: DirectPlayResponsePlan,
}

#[derive(Clone, Debug)]
pub(crate) struct RemuxPlaybackPreflightRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub client: ClientPlaybackCapabilities,
    pub output_container: RemuxContainer,
}

#[derive(Clone, Debug)]
pub(crate) struct RemuxPlaybackPreflightOutput {
    pub session: PlaybackSessionRecord,
    pub response: DirectPlayResponsePlan,
}

#[derive(Clone, Debug)]
pub(crate) struct HlsPlaylistPlaybackRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub client: ClientPlaybackCapabilities,
}

#[derive(Clone, Debug)]
pub(crate) struct HlsPlaylistPlaybackOutput {
    pub session: PlaybackSessionRecord,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HlsSegmentPlaybackTarget {
    pub source_id: MediaSourceId,
    pub transcode_session_id: TranscodeSessionId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RendererPlaybackTransportPlan {
    pub mode: PlaybackSessionMode,
    pub remux_container: Option<RemuxContainer>,
    pub content_type: String,
    pub supports_range_requests: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct DirectPlaybackSessionStreamRequest {
    pub principal: AuthenticatedPrincipal,
    pub playback_session_id: PlaybackSessionId,
    pub source_id: MediaSourceId,
    pub range_request: DirectPlayRangeRequest,
}

#[derive(Clone, Debug)]
pub(crate) struct RemuxPlaybackSessionStreamRequest {
    pub principal: AuthenticatedPrincipal,
    pub playback_session_id: PlaybackSessionId,
    pub source_id: MediaSourceId,
    pub output_container: RemuxContainer,
    pub range_request: DirectPlayRangeRequest,
}

#[derive(Clone, Debug)]
pub(crate) struct HlsPlaylistSessionRequest {
    pub principal: AuthenticatedPrincipal,
    pub playback_session_id: PlaybackSessionId,
    pub source_id: MediaSourceId,
}

#[derive(Clone, Debug)]
pub(crate) struct StartPlaybackSessionRequest {
    pub principal_id: UserPrincipalId,
    pub source_id: MediaSourceId,
    pub mode: PlaybackSessionMode,
    pub client: Option<ClientPlaybackCapabilities>,
}

#[derive(Clone, Debug)]
pub(crate) struct StartRendererPlaybackSessionRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub target: PlaybackTarget,
}

#[derive(Clone, Debug)]
pub(crate) struct StartRendererPlaybackSessionOutput {
    pub session: PlaybackSessionRecord,
    pub transport: RendererPlaybackTransportPlan,
}

#[derive(Clone, Debug)]
pub(crate) struct BrowserPlaybackTicketValidationRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub mode: BrowserPlaybackTicketMode,
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
    pub runtime_inventory: TranscodeRuntimeInventory,
    pub hardware_policy: HardwareAccelerationPolicy,
    pub hardware_report: HardwareAccelerationReport,
    pub hls_pipeline_readiness: TranscodePipelineReadiness,
    pub transcode_budget: TranscodeResourceBudget,
    pub selected_hls_slots: usize,
    pub remux_concurrency: usize,
    pub remux_timeout_ms: u64,
    pub remote_stream_concurrency: usize,
    pub remote_stage_concurrency: usize,
    pub staging_max_bytes: u64,
    pub staging_retention_ms: u64,
    pub staging_cleanup_on_startup: bool,
    pub transcode_artifact_retention_ms: u64,
    pub transcode_artifact_cleanup_on_startup: bool,
    pub hls_segment_cleanup_enabled: bool,
    pub hls_segment_keep_ms: u64,
    pub transcode_throttle_enabled: bool,
    pub transcode_throttle_delay_ms: u64,
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
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
        client: ClientPlaybackCapabilities,
    ) -> Result<PlaybackDecisionResponse> {
        let source = self.get_source_or_not_found(source_id).await?;
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let context = self.playback_selection_context_for_source(&source).await?;
        let target = playback_target_for_client(client);
        let effective_policy = self
            .effective_playback_policy_for_source(principal, &source)
            .await?;
        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            target: &target,
            effective_policy: &effective_policy,
            context,
        });

        Ok(playback_decision_response_to_dto(
            source, probe, target, decision,
        ))
    }

    pub(crate) async fn validate_browser_playback_ticket_request(
        &self,
        request: BrowserPlaybackTicketValidationRequest,
    ) -> Result<()> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let effective_policy = self
            .effective_playback_policy_for_source(&request.principal, &source)
            .await?;
        let context = self.playback_selection_context_for_source(&source).await?;
        if context.storage.remote {
            ensure_playback_permission_allowed(
                &effective_policy,
                PlaybackPermission::RemotePlayback,
            )?;
        }

        match request.mode {
            BrowserPlaybackTicketMode::Direct => ensure_playback_permission_allowed(
                &effective_policy,
                PlaybackPermission::DirectPlay,
            ),
            BrowserPlaybackTicketMode::Remux => {
                ensure_playback_permission_allowed(&effective_policy, PlaybackPermission::Remux)
            }
            BrowserPlaybackTicketMode::Hls => {
                ensure_playback_permission_allowed(
                    &effective_policy,
                    PlaybackPermission::VideoTranscode,
                )?;
                ensure_playback_permission_allowed(
                    &effective_policy,
                    PlaybackPermission::AudioTranscode,
                )
            }
        }?;

        Ok(())
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

    pub(crate) async fn start_renderer_playback_session(
        &self,
        request: StartRendererPlaybackSessionRequest,
    ) -> Result<StartRendererPlaybackSessionOutput> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let context = self.playback_selection_context_for_source(&source).await?;
        let effective_policy = self
            .effective_playback_policy_for_source(&request.principal, &source)
            .await?;
        ensure_playback_permission_allowed(&effective_policy, PlaybackPermission::RemoteControl)?;

        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            target: &request.target,
            effective_policy: &effective_policy,
            context,
        });
        ensure_playback_decision_allowed(&decision)?;

        match decision.mode {
            PlaybackMode::DirectPlay => {
                let direct = decision.direct_play.as_ref().ok_or_else(|| {
                    NakoError::Unsupported("direct renderer decision did not include a direct plan")
                })?;
                let session = self
                    .start_playback_session(StartPlaybackSessionRequest {
                        principal_id: request.principal.principal_id,
                        source_id: request.source_id,
                        mode: PlaybackSessionMode::Direct,
                        client: Some(request.target.media_capabilities.clone()),
                    })
                    .await?;

                Ok(StartRendererPlaybackSessionOutput {
                    session,
                    transport: RendererPlaybackTransportPlan {
                        mode: PlaybackSessionMode::Direct,
                        remux_container: None,
                        content_type: direct.content_type.clone(),
                        supports_range_requests: direct.supports_range_requests,
                    },
                })
            }
            PlaybackMode::Remux => {
                let output_container = remux_output_container(&decision)?;
                let remux = self
                    .start_remux_source_with_policy(
                        RemuxSourceRequest {
                            source_id: request.source_id,
                            client: request.target.media_capabilities.clone(),
                            output_container,
                        },
                        effective_policy.clone(),
                    )
                    .await?;
                let session = self
                    .start_playback_session(StartPlaybackSessionRequest {
                        principal_id: request.principal.principal_id,
                        source_id: request.source_id,
                        mode: PlaybackSessionMode::Remux,
                        client: Some(request.target.media_capabilities.clone()),
                    })
                    .await?;
                self.link_playback_session_transcode(session.id, remux.session.id)
                    .await?;

                Ok(StartRendererPlaybackSessionOutput {
                    session,
                    transport: RendererPlaybackTransportPlan {
                        mode: PlaybackSessionMode::Remux,
                        remux_container: Some(output_container),
                        content_type: nako_streaming::content_type_for_file_name(&format!(
                            "stream.{}",
                            output_container.file_extension()
                        ))
                        .to_owned(),
                        supports_range_requests: true,
                    },
                })
            }
            PlaybackMode::Transcode => {
                let playlist = self
                    .hls_playlist_with_policy(
                        HlsSourceRequest {
                            source_id: request.source_id,
                            client: request.target.media_capabilities.clone(),
                        },
                        effective_policy.clone(),
                    )
                    .await?;
                let session = self
                    .start_playback_session(StartPlaybackSessionRequest {
                        principal_id: request.principal.principal_id,
                        source_id: request.source_id,
                        mode: PlaybackSessionMode::Hls,
                        client: Some(request.target.media_capabilities.clone()),
                    })
                    .await?;
                self.link_playback_session_transcode(session.id, playlist.session.id)
                    .await?;

                Ok(StartRendererPlaybackSessionOutput {
                    session,
                    transport: RendererPlaybackTransportPlan {
                        mode: PlaybackSessionMode::Hls,
                        remux_container: None,
                        content_type: "application/vnd.apple.mpegurl".to_owned(),
                        supports_range_requests: false,
                    },
                })
            }
            PlaybackMode::Denied => Err(playback_policy_forbidden(&decision)),
        }
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

    pub(crate) async fn direct_playback_stream(
        &self,
        request: DirectPlaybackStreamRequest,
    ) -> Result<DirectPlaybackStreamOutput> {
        self.ensure_direct_playback_allowed(&request.principal, request.source_id)
            .await?;
        let direct_play = self
            .plan_direct_play(request.source_id, request.range_request)
            .await?;
        let source_uri = direct_play.source.locator.clone();

        if direct_play.response.is_range_not_satisfiable() {
            return Ok(DirectPlaybackStreamOutput {
                session: None,
                source_uri,
                body: DirectPlaySourceBody::Empty,
                response: direct_play.response,
            });
        }

        let session = self
            .start_playback_session(StartPlaybackSessionRequest {
                principal_id: request.principal.principal_id,
                source_id: request.source_id,
                mode: PlaybackSessionMode::Direct,
                client: Some(request.client),
            })
            .await?;

        Ok(DirectPlaybackStreamOutput {
            session: Some(session),
            source_uri,
            body: direct_play.body,
            response: direct_play.response,
        })
    }

    pub(crate) async fn direct_playback_session_stream(
        &self,
        request: DirectPlaybackSessionStreamRequest,
    ) -> Result<DirectPlaybackStreamOutput> {
        let session = self
            .existing_playback_session_for_media_request(
                &request.principal,
                request.playback_session_id,
                request.source_id,
                PlaybackSessionMode::Direct,
            )
            .await?;
        let direct_play = self
            .plan_direct_play(request.source_id, request.range_request)
            .await?;
        let source_uri = direct_play.source.locator.clone();

        if direct_play.response.is_range_not_satisfiable() {
            return Ok(DirectPlaybackStreamOutput {
                session: Some(session),
                source_uri,
                body: DirectPlaySourceBody::Empty,
                response: direct_play.response,
            });
        }

        Ok(DirectPlaybackStreamOutput {
            session: Some(session),
            source_uri,
            body: direct_play.body,
            response: direct_play.response,
        })
    }

    pub(crate) async fn direct_playback_preflight(
        &self,
        request: DirectPlaybackPreflightRequest,
    ) -> Result<DirectPlaybackPreflightOutput> {
        self.ensure_direct_playback_allowed(&request.principal, request.source_id)
            .await?;
        let response = self
            .plan_direct_play_preflight(request.source_id, request.range_request)
            .await?;
        let session = self
            .start_playback_session(StartPlaybackSessionRequest {
                principal_id: request.principal.principal_id,
                source_id: request.source_id,
                mode: PlaybackSessionMode::Direct,
                client: Some(request.client),
            })
            .await?;

        Ok(DirectPlaybackPreflightOutput { session, response })
    }

    pub(crate) async fn remux_playback_stream(
        &self,
        request: RemuxPlaybackStreamRequest,
    ) -> Result<RemuxPlaybackStreamOutput> {
        let effective_policy = self
            .effective_playback_policy_for_source_id(&request.principal, request.source_id)
            .await?;
        let remux_request = RemuxSourceRequest {
            source_id: request.source_id,
            client: request.client.clone(),
            output_container: request.output_container,
        };
        let remux_start = self
            .start_remux_source_with_policy(remux_request, effective_policy)
            .await?;
        let playback_session = self
            .start_playback_session(StartPlaybackSessionRequest {
                principal_id: request.principal.principal_id,
                source_id: request.source_id,
                mode: PlaybackSessionMode::Remux,
                client: Some(request.client.clone()),
            })
            .await?;
        self.link_playback_session_transcode(playback_session.id, remux_start.session.id)
            .await?;
        let remux = self.wait_for_remux_start(remux_start).await?;

        if remux.disposition == RemuxSourceDisposition::Cancelled {
            return Err(NakoError::Provider {
                provider: "ffmpeg_remux".to_owned(),
                message: "remux session was cancelled".to_owned(),
            });
        }

        let total_len = tokio::fs::metadata(&remux.output_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    remux.output_path.display().to_string(),
                    format!("failed to read remux output length: {err}"),
                )
            })?
            .len();
        let response = nako_streaming::plan_direct_play_response(
            total_len,
            nako_streaming::content_type_for_file_name(&format!(
                "stream.{}",
                request.output_container.file_extension()
            )),
            request.range_request,
        );

        Ok(RemuxPlaybackStreamOutput {
            session: playback_session,
            output_path: remux.output_path,
            response,
        })
    }

    pub(crate) async fn remux_playback_session_stream(
        &self,
        request: RemuxPlaybackSessionStreamRequest,
    ) -> Result<RemuxPlaybackStreamOutput> {
        let playback_session = self
            .existing_playback_session_for_media_request(
                &request.principal,
                request.playback_session_id,
                request.source_id,
                PlaybackSessionMode::Remux,
            )
            .await?;
        let transcode_session_id =
            playback_session
                .transcode_session_id
                .ok_or_else(|| NakoError::Conflict {
                    message: format!(
                        "playback session {} does not have a remux artifact",
                        playback_session.id
                    ),
                })?;
        let transcode = self
            .wait_for_remux_transcode_output(transcode_session_id)
            .await?;
        if transcode.source_id != request.source_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "remux playback session {} source_id does not match transcode session {}",
                    playback_session.id, transcode.id
                ),
            });
        }

        let total_len = tokio::fs::metadata(&transcode.output_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    transcode.output_path.display().to_string(),
                    format!("failed to read remux output length: {err}"),
                )
            })?
            .len();
        let response = nako_streaming::plan_direct_play_response(
            total_len,
            nako_streaming::content_type_for_file_name(&format!(
                "stream.{}",
                request.output_container.file_extension()
            )),
            request.range_request,
        );

        Ok(RemuxPlaybackStreamOutput {
            session: playback_session,
            output_path: transcode.output_path,
            response,
        })
    }

    pub(crate) async fn remux_playback_preflight(
        &self,
        request: RemuxPlaybackPreflightRequest,
    ) -> Result<RemuxPlaybackPreflightOutput> {
        let effective_policy = self
            .effective_playback_policy_for_source_id(&request.principal, request.source_id)
            .await?;
        let remux = self
            .start_remux_source_with_policy(
                RemuxSourceRequest {
                    source_id: request.source_id,
                    client: request.client.clone(),
                    output_container: request.output_container,
                },
                effective_policy,
            )
            .await?;
        let playback_session = self
            .start_playback_session(StartPlaybackSessionRequest {
                principal_id: request.principal.principal_id,
                source_id: request.source_id,
                mode: PlaybackSessionMode::Remux,
                client: Some(request.client.clone()),
            })
            .await?;
        self.link_playback_session_transcode(playback_session.id, remux.session.id)
            .await?;

        let response = nako_streaming::plan_direct_play_response(
            0,
            nako_streaming::content_type_for_file_name(&format!(
                "stream.{}",
                request.output_container.file_extension()
            )),
            DirectPlayRangeRequest::None,
        );

        Ok(RemuxPlaybackPreflightOutput {
            session: playback_session,
            response,
        })
    }

    pub(crate) async fn hls_playlist_playback(
        &self,
        request: HlsPlaylistPlaybackRequest,
    ) -> Result<HlsPlaylistPlaybackOutput> {
        let effective_policy = self
            .effective_playback_policy_for_source_id(&request.principal, request.source_id)
            .await?;
        let playlist = self
            .hls_playlist_with_policy(
                HlsSourceRequest {
                    source_id: request.source_id,
                    client: request.client.clone(),
                },
                effective_policy,
            )
            .await?;
        let playback_session = self
            .start_playback_session(StartPlaybackSessionRequest {
                principal_id: request.principal.principal_id,
                source_id: request.source_id,
                mode: PlaybackSessionMode::Hls,
                client: Some(request.client.clone()),
            })
            .await?;
        self.link_playback_session_transcode(playback_session.id, playlist.session.id)
            .await?;
        let body =
            rewrite_hls_playlist_segments_for_playback_session(&playlist.body, playback_session.id);

        Ok(HlsPlaylistPlaybackOutput {
            session: playback_session,
            body,
        })
    }

    pub(crate) async fn hls_playlist_for_playback_session(
        &self,
        request: HlsPlaylistSessionRequest,
    ) -> Result<HlsPlaylistPlaybackOutput> {
        let playback_session = self
            .existing_playback_session_for_media_request(
                &request.principal,
                request.playback_session_id,
                request.source_id,
                PlaybackSessionMode::Hls,
            )
            .await?;
        let transcode_session_id =
            playback_session
                .transcode_session_id
                .ok_or_else(|| NakoError::Conflict {
                    message: format!(
                        "playback session {} does not have an hls artifact",
                        playback_session.id
                    ),
                })?;
        let transcode = self.get_transcode_session(transcode_session_id).await?;
        if transcode.kind != TranscodeSessionKind::HlsTranscode {
            return Err(NakoError::InvalidInput {
                message: format!("session {transcode_session_id} is not an hls transcode session"),
            });
        }
        if transcode.source_id != request.source_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "hls playback session {} source_id does not match transcode session {}",
                    playback_session.id, transcode.id
                ),
            });
        }
        if !hls_session_can_serve_artifacts(transcode.state) {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls session {transcode_session_id} is not ready; current state is {:?}",
                    transcode.state
                ),
            });
        }
        if !path_exists(&transcode.output_path)?
            && transcode.state == TranscodeSessionState::Running
        {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls playlist for session {transcode_session_id} is not ready; current state is {:?}",
                    transcode.state
                ),
            });
        }
        let body = tokio::fs::read_to_string(&transcode.output_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    transcode.output_path.display().to_string(),
                    format!("failed to read hls playlist: {err}"),
                )
            })?;
        let body = rewrite_hls_playlist_segments_for_playback_session(&body, playback_session.id);

        Ok(HlsPlaylistPlaybackOutput {
            session: playback_session,
            body,
        })
    }

    pub(crate) async fn remux_source(
        &self,
        request: RemuxSourceRequest,
    ) -> Result<RemuxSourceOutput> {
        let context = self.remux_source_context(&request, None).await?;
        self.run_remux_source_context(context).await
    }

    async fn start_remux_source_with_policy(
        &self,
        request: RemuxSourceRequest,
        effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    ) -> Result<RemuxSessionStart> {
        let effective_policy = effective_policy.into();
        let context = self
            .remux_source_context(&request, effective_policy)
            .await?;
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
        let task_effective_policy = effective_policy;
        self.runtime
            .spawn("playback_remux_start", "playback.remux", async move {
                if let Err(error) = task_app
                    .remux_source_with_policy(task_request, task_effective_policy)
                    .await
                {
                    warn!(error = %error, "background remux start failed");
                }
            });

        self.wait_for_started_remux_source_context(context).await
    }

    async fn remux_source_with_policy(
        &self,
        request: RemuxSourceRequest,
        effective_policy: Option<EffectivePlaybackPolicy>,
    ) -> Result<RemuxSourceOutput> {
        let context = self
            .remux_source_context(&request, effective_policy)
            .await?;
        self.run_remux_source_context(context).await
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

    async fn wait_for_remux_transcode_output(
        &self,
        session_id: TranscodeSessionId,
    ) -> Result<TranscodeSessionRecord> {
        let deadline = tokio::time::Instant::now()
            + std::time::Duration::from_millis(self.config.remux_timeout_ms.max(1));
        loop {
            let session = self.get_transcode_session(session_id).await?;
            if session.kind != TranscodeSessionKind::Remux {
                return Err(NakoError::InvalidInput {
                    message: format!("session {session_id} is not a remux session"),
                });
            }
            match session.state {
                TranscodeSessionState::Finished => {
                    if !path_exists(&session.output_path)? {
                        return Err(NakoError::storage_io(
                            session.output_path.display().to_string(),
                            "finished remux session output is missing",
                        ));
                    }

                    return Ok(session);
                }
                TranscodeSessionState::Cancelled => {
                    return Err(NakoError::Provider {
                        provider: "ffmpeg_remux".to_owned(),
                        message: "remux session was cancelled".to_owned(),
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
        effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    ) -> Result<RemuxSourceContext> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let mut context = playback_selection_context(&uri, backend.as_ref()).await;
        context.preferences.remux_output_container = Some(request.output_container);
        let target = playback_target_for_client(request.client.clone());
        let target_profile = PlaybackTargetProfile::from_target(&target, context.clone());
        let playback_profile = PlaybackProfile::from_target_profile(&target_profile);
        let effective_policy = effective_policy
            .into()
            .unwrap_or_else(|| default_playback_policy_for_source(&source));
        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            target: &target,
            effective_policy: &effective_policy,
            context,
        });
        ensure_playback_decision_allowed(&decision)?;
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
        self.hls_source_with_policy(request, None).await
    }

    async fn hls_source_with_policy(
        &self,
        request: HlsSourceRequest,
        effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    ) -> Result<HlsSourceOutput> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let mut context = playback_selection_context(&uri, backend.as_ref()).await;
        context.preferences.transcode_output_container = Some(nako_transcode::OutputContainer::Hls);
        let target = playback_target_for_client(request.client.clone());
        let target_profile = PlaybackTargetProfile::from_target(&target, context.clone());
        let playback_profile = PlaybackProfile::from_target_profile(&target_profile);
        let effective_policy = effective_policy
            .into()
            .unwrap_or_else(|| default_playback_policy_for_source(&source));
        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            target: &target,
            effective_policy: &effective_policy,
            context,
        });
        ensure_playback_decision_allowed(&decision)?;
        let transcode_plan = hls_transcode_plan(&decision)?;
        let track_selection = playback_profile.track_selection();
        let source_facts = hls_pipeline_source_facts(probe.as_ref(), track_selection);
        let execution_policy = self.hls.execution_policy_for_hls(
            track_selection,
            TranscodeOutputConstraints {
                max_video_bitrate: playback_profile.preferences.max_video_bitrate,
                prefer_hdr: playback_profile.preferences.prefer_hdr,
            },
            source_facts,
        )?;
        let hls_profile =
            playback_profile.try_hls_transcode_profile(transcode_plan, execution_policy)?;
        let execution_policy = hls_profile.execution_policy;
        let profile_identity = hls_profile.identity();
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
                execution_policy,
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
        self.hls_playlist_with_policy(request, None).await
    }

    async fn hls_playlist_with_policy(
        &self,
        request: HlsSourceRequest,
        effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    ) -> Result<HlsPlaylistOutput> {
        let output = self
            .hls_source_with_policy(request, effective_policy)
            .await?;

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

        if !hls_session_can_serve_artifacts(session.state) {
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

        cleanup_hls_segment_dir_if_enabled(&self.config.playback, segment_dir, segment_name)
            .await?;

        wait_for_hls_segment_if_configured(&self.config.playback, session.state, &path).await?;
        if !path_exists(&path)? {
            if session.state == TranscodeSessionState::Running {
                return Err(NakoError::Conflict {
                    message: format!(
                        "hls segment {segment_name} for session {session_id} is not ready; current state is {:?}",
                        session.state
                    ),
                });
            }

            return Err(NakoError::NotFound {
                entity: "hls_segment",
                id: segment_name.to_owned(),
            });
        }

        let total_len = tokio::fs::metadata(&path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    path.display().to_string(),
                    format!("failed to read hls segment length: {err}"),
                )
            })?
            .len();
        let response = nako_streaming::plan_direct_play_response(
            total_len,
            "video/mp2t",
            DirectPlayRangeRequest::None,
        );

        Ok(HlsSegmentPlan { path, response })
    }

    pub(crate) async fn hls_segment_playback_target(
        &self,
        session_id: PlaybackSessionId,
    ) -> Result<HlsSegmentPlaybackTarget> {
        let playback_session = self.get_playback_session(session_id).await?;
        let transcode_session_id =
            playback_session
                .transcode_session_id
                .ok_or_else(|| NakoError::Conflict {
                    message: format!("playback session {session_id} does not have an hls artifact"),
                })?;

        Ok(HlsSegmentPlaybackTarget {
            source_id: playback_session.source_id,
            transcode_session_id,
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
            runtime_inventory: TranscodeRuntimeInventory::ffmpeg_cli(&self.hls.hardware_report),
            hardware_policy,
            hardware_report: self.hls.hardware_report.clone(),
            hls_pipeline_readiness: self.hls.pipeline_readiness(),
            transcode_budget,
            selected_hls_slots: self.hls.selected_hls_slots(transcode_budget),
            remux_concurrency: self.config.remux_concurrency.max(1),
            remux_timeout_ms: self.config.remux_timeout_ms.max(1),
            remote_stream_concurrency: self.config.playback.remote_stream_concurrency.max(1),
            remote_stage_concurrency: self.config.playback.remote_stage_concurrency.max(1),
            staging_max_bytes: self.config.staging.max_bytes,
            staging_retention_ms: self.config.staging.retention_ms,
            staging_cleanup_on_startup: self.config.staging.cleanup_on_startup,
            transcode_artifact_retention_ms: self.config.playback.transcode_artifact_retention_ms,
            transcode_artifact_cleanup_on_startup: self
                .config
                .playback
                .transcode_artifact_cleanup_on_startup,
            hls_segment_cleanup_enabled: self.config.playback.hls_segment_cleanup_enabled,
            hls_segment_keep_ms: self.config.playback.hls_segment_keep_ms,
            transcode_throttle_enabled: self.config.playback.transcode_throttle_enabled,
            transcode_throttle_delay_ms: self.config.playback.transcode_throttle_delay_ms,
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

    async fn existing_playback_session_for_media_request(
        &self,
        principal: &AuthenticatedPrincipal,
        session_id: PlaybackSessionId,
        source_id: MediaSourceId,
        mode: PlaybackSessionMode,
    ) -> Result<PlaybackSessionRecord> {
        let session = self.get_playback_session(session_id).await?;
        if session.principal_id != principal.principal_id {
            return Err(NakoError::Forbidden {
                message: "playback session belongs to a different principal".to_owned(),
            });
        }
        if session.source_id != source_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "playback session {session_id} belongs to source {}, not {source_id}",
                    session.source_id
                ),
            });
        }
        if session.mode != mode {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "playback session {session_id} is {}, not {}",
                    session.mode.as_str(),
                    mode.as_str()
                ),
            });
        }
        if session.state.is_terminal() {
            return Err(NakoError::Conflict {
                message: format!(
                    "playback session {session_id} is terminal; current state is {}",
                    session.state.as_str()
                ),
            });
        }

        Ok(session)
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

    async fn effective_playback_policy_for_source_id(
        &self,
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
    ) -> Result<EffectivePlaybackPolicy> {
        let source = self.get_source_or_not_found(source_id).await?;
        self.effective_playback_policy_for_source(principal, &source)
            .await
    }

    async fn effective_playback_policy_for_source(
        &self,
        principal: &AuthenticatedPrincipal,
        source: &MediaSource,
    ) -> Result<EffectivePlaybackPolicy> {
        PlaybackRuntimeStore::resolve_effective_playback_policy(
            self.runtime_store.as_ref(),
            principal.user_id,
            source.library_id,
        )
        .await
    }

    async fn ensure_direct_playback_allowed(
        &self,
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
    ) -> Result<()> {
        let source = self.get_source_or_not_found(source_id).await?;
        let effective_policy = self
            .effective_playback_policy_for_source(principal, &source)
            .await?;
        let context = self.playback_selection_context_for_source(&source).await?;
        if context.storage.remote {
            ensure_playback_permission_allowed(
                &effective_policy,
                PlaybackPermission::RemotePlayback,
            )?;
        }
        ensure_playback_permission_allowed(&effective_policy, PlaybackPermission::DirectPlay)
    }
}

fn rewrite_hls_playlist_segments_for_playback_session(
    body: &str,
    session_id: PlaybackSessionId,
) -> String {
    let mut rewritten = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return line.to_owned();
            }
            let Some(rest) = line.strip_prefix("/playback/sessions/") else {
                return format!("/playback/sessions/{session_id}/hls/segments/{trimmed}");
            };
            let Some((_old_session_id, segment_path)) = rest.split_once("/hls/segments/") else {
                return line.to_owned();
            };

            format!("/playback/sessions/{session_id}/hls/segments/{segment_path}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    if body.ends_with('\n') {
        rewritten.push('\n');
    }

    rewritten
}

fn hls_session_can_serve_artifacts(state: TranscodeSessionState) -> bool {
    matches!(
        state,
        TranscodeSessionState::Running | TranscodeSessionState::Finished
    )
}

async fn wait_for_hls_segment_if_configured(
    config: &crate::config::PlaybackConfig,
    state: TranscodeSessionState,
    path: &Path,
) -> Result<()> {
    if path_exists(path)?
        || state != TranscodeSessionState::Running
        || !config.transcode_throttle_enabled
    {
        return Ok(());
    }

    tokio::time::sleep(Duration::from_millis(config.transcode_throttle_delay_ms)).await;
    Ok(())
}

async fn cleanup_hls_segment_dir_if_enabled(
    config: &crate::config::PlaybackConfig,
    segment_dir: &Path,
    requested_segment: &str,
) -> Result<()> {
    if !config.hls_segment_cleanup_enabled {
        return Ok(());
    }

    cleanup_hls_segment_dir_at(
        segment_dir,
        requested_segment,
        config.hls_segment_keep_ms,
        current_time_ms(),
    )
    .await
}

async fn cleanup_hls_segment_dir_at(
    segment_dir: &Path,
    requested_segment: &str,
    keep_ms: u64,
    now_ms: i64,
) -> Result<()> {
    let mut entries = match tokio::fs::read_dir(segment_dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(NakoError::storage_io(
                segment_dir.display().to_string(),
                format!("failed to read hls segment directory: {err}"),
            ));
        }
    };
    let keep_ms = i64::try_from(keep_ms).unwrap_or(i64::MAX);

    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(err) => {
                return Err(NakoError::storage_io(
                    segment_dir.display().to_string(),
                    format!("failed to iterate hls segment directory: {err}"),
                ));
            }
        };
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if file_name == requested_segment
            || path.extension().and_then(|value| value.to_str()) != Some("ts")
        {
            continue;
        }

        let metadata = match entry.metadata().await {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => {
                return Err(NakoError::storage_io(
                    path.display().to_string(),
                    format!("failed to read hls segment metadata: {err}"),
                ));
            }
        };
        if !metadata.is_file() {
            continue;
        }
        let Some(modified_ms) = metadata.modified().ok().and_then(system_time_ms) else {
            continue;
        };
        if now_ms.saturating_sub(modified_ms) < keep_ms {
            continue;
        }

        if let Err(err) = tokio::fs::remove_file(&path).await {
            if err.kind() == std::io::ErrorKind::NotFound {
                continue;
            }

            return Err(NakoError::storage_io(
                path.display().to_string(),
                format!("failed to remove stale hls segment: {err}"),
            ));
        }
    }

    Ok(())
}

fn current_time_ms() -> i64 {
    system_time_ms(SystemTime::now()).unwrap_or(i64::MAX)
}

fn system_time_ms(value: SystemTime) -> Option<i64> {
    let duration = value.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(duration.as_millis()).ok()
}

fn playback_target_for_client(client: ClientPlaybackCapabilities) -> PlaybackTarget {
    PlaybackTarget::browser_with_capabilities("Public Client", client)
}

fn default_playback_policy_for_source(source: &MediaSource) -> EffectivePlaybackPolicy {
    EffectivePlaybackPolicy::from_library_access(source.library_id, LibraryAccessLevel::Play)
}

fn ensure_playback_decision_allowed(decision: &PlaybackDecision) -> Result<()> {
    if decision.denial.is_some() {
        return Err(playback_policy_forbidden(decision));
    }

    Ok(())
}

fn ensure_playback_permission_allowed(
    policy: &EffectivePlaybackPolicy,
    permission: PlaybackPermission,
) -> Result<()> {
    let decision = policy.check(permission);
    if decision.allowed {
        return Ok(());
    }

    Err(NakoError::Forbidden {
        message: format!(
            "playback policy denied {}: {}",
            decision.permission.as_str(),
            decision.reason.as_str()
        ),
    })
}

fn playback_policy_forbidden(decision: &PlaybackDecision) -> NakoError {
    let Some(denial) = &decision.denial else {
        return NakoError::Forbidden {
            message: "playback policy denied playback".to_owned(),
        };
    };

    NakoError::Forbidden {
        message: format!(
            "playback policy denied {}: {}",
            denial.permission.as_str(),
            denial.reason.as_str()
        ),
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
    use std::time::Duration;

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

    #[tokio::test]
    async fn hls_segment_waits_once_for_running_segment_when_throttle_enabled() {
        let temp = tempfile::tempdir().unwrap();
        let segment_path = temp.path().join("segment_00000.ts");
        let writer_path = segment_path.clone();
        let writer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            tokio::fs::write(writer_path, b"segment").await.unwrap();
        });
        let config = PlaybackConfig {
            transcode_throttle_enabled: true,
            transcode_throttle_delay_ms: 50,
            ..PlaybackConfig::default()
        };

        wait_for_hls_segment_if_configured(&config, TranscodeSessionState::Running, &segment_path)
            .await
            .unwrap();

        assert!(path_exists(&segment_path).unwrap());
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn hls_segment_cleanup_removes_stale_siblings_and_keeps_requested() {
        let temp = tempfile::tempdir().unwrap();
        let segment_dir = temp.path();
        let requested = segment_dir.join("segment_00001.ts");
        let stale = segment_dir.join("segment_00000.ts");
        let playlist = segment_dir.join("playlist.m3u8");
        let subtitle = segment_dir.join("segment_00002.vtt");
        tokio::fs::write(&requested, b"requested").await.unwrap();
        tokio::fs::write(&stale, b"stale").await.unwrap();
        tokio::fs::write(&playlist, b"playlist").await.unwrap();
        tokio::fs::write(&subtitle, b"subtitle").await.unwrap();

        cleanup_hls_segment_dir_at(segment_dir, "segment_00001.ts", 60_000, i64::MAX / 2)
            .await
            .unwrap();

        assert!(path_exists(&requested).unwrap());
        assert!(!path_exists(&stale).unwrap());
        assert!(path_exists(&playlist).unwrap());
        assert!(path_exists(&subtitle).unwrap());
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
            service.pipeline_readiness().selected,
            HardwareAcceleration::Nvenc
        );
        assert!(!service.pipeline_readiness().fallback_used);
        assert_eq!(
            service.selected_hls_slots(config.transcode.resource_budget()),
            2
        );
    }

    #[test]
    fn hls_service_rejects_execution_policy_when_startup_pipeline_is_unavailable() {
        let config = test_config(TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::Nvenc,
            hardware_fallback: HardwareAccelerationFallback::Fail,
            cpu_concurrency: 1,
            gpu_concurrency: 2,
        });
        let detector =
            StaticHardwareAccelerationDetector::new(HardwareAccelerationReport::with_available([
                HardwareAcceleration::None,
            ]));
        let service = HlsAppService::new_with_hardware_detector(&config, &detector).unwrap();

        let err = service
            .execution_policy_for_hls(
                nako_transcode::TranscodeTrackSelection::default(),
                TranscodeOutputConstraints::default(),
                None,
            )
            .unwrap_err();

        assert_eq!(
            service.pipeline_readiness().status,
            nako_transcode::TranscodePipelineReadinessStatus::Unavailable
        );
        assert_eq!(
            service.selected_hls_slots(config.transcode.resource_budget()),
            0
        );
        assert!(err.to_string().contains("hardware pipeline"));
    }
}
