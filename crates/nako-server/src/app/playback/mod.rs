use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use nako_api::public_client::{PlaybackDecisionResponse, playback_decision_response_to_dto};
use nako_core::{
    AuthenticatedPrincipal, EventOutboxRepository, LibraryAccessLevel, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, MediaSourceId, MediaStreamInfo,
    MediaStreamKind, NakoError, NewOutboxEvent, NewPlaybackSession, NewTranscodeSession,
    OutboxEventRecord, PageRequest, PlaybackPermission, PlaybackPolicyRepository,
    PlaybackSessionHeartbeat, PlaybackSessionId, PlaybackSessionListFilter, PlaybackSessionMode,
    PlaybackSessionRecord, PlaybackSessionRepository, PlaybackSessionState, Result,
    StagingManifestRepository, TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionListFilter, TranscodeSessionRecord, TranscodeSessionRepository,
    TranscodeSessionRuntimeMetrics, TranscodeSessionState, UserId, UserPrincipalId,
};
use nako_playback::{
    ClientPlaybackCapabilities, EffectivePlaybackPolicy, PlaybackDecision, PlaybackMode,
    PlaybackPlanner, PlaybackPlanningRequest, PlaybackPreferenceContext, PlaybackSelectionContext,
    PlaybackTarget, PlaybackTargetProfile, PlaybackTranscodeContainer,
};
use nako_streaming::{DirectPlayRangeRequest, DirectPlayResponsePlan};
use nako_transcode::{
    HlsAdaptiveLadderPlan, HlsAudioRendition, HlsMediaRenditionPlan, HlsPlaybackGeneration,
    HlsRequestVariantPlan, HlsVariantPolicy, PlaybackHlsProfileRequest,
    PlaybackRemuxProfileRequest, RemuxContainer, TranscodeOutputConstraints,
    TranscodePipelineSourceFacts, TranscodeRequestIdentity, TranscodeSourceIdentity,
    TranscodeSubtitleStrategy, TranscodeTrackSelection, build_playback_hls_profile,
    build_playback_remux_profile,
};
use nako_vfs::{StorageBackend as _, StorageUri};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::NakoServerConfig;
use crate::playback_mapping::{
    playback_hls_output_requirement_to_transcode, playback_output_constraints_to_transcode,
    playback_track_selection_to_transcode, transcode_remux_container_to_playback,
};

use super::{
    playback_ticket::BrowserPlaybackTicketMode,
    runtime::RuntimeSupervisor,
    storage::StorageBackendRegistry,
    subtitle_sidecar::{
        SUBTITLE_SIDECAR_MAX_BYTES, subtitle_content_type_for_extension,
        subtitle_sidecar_file_name_for_stream, subtitle_sidecar_uri_for_source,
    },
};

mod control;
mod direct;
mod events;
mod failure;
mod hls;
mod hls_artifact;
mod input;
mod paths;
mod playlist;
mod remux;
mod resource;
mod selection;
mod staging_policy;
mod support;

use control::PlaybackSessionCancellationRegistry;
pub(crate) use direct::{
    DirectPlaySourceBody, DirectPlaySourcePlan, DirectPlayStreamBody, plan_direct_play_with_backend,
};
use direct::{plan_direct_play_response_with_backend, should_budget_remote_stream};
use events::record_playback_session_finished_event;
use failure::{map_hls_runner_error, map_remux_runner_error, persist_session_failure};
use hls::HlsAppService;
use hls_artifact::{HlsArtifactService, hls_artifact_manifest_for_session};
#[cfg(test)]
pub(crate) use input::source_path_for_ffmpeg_with_backend;
use input::{FfmpegInputService, FfmpegSourceInput};
use paths::{ensure_remux_output_parent, path_exists};
use playlist::{HlsPlaylistSessionBinding, HlsPlaylistUrlDecoration, author_hls_session_playlist};
use remux::RemuxAppService;
pub(crate) use remux::RemuxRequestKey;
pub(crate) use resource::{
    PlaybackResourceAdmissionDecision, PlaybackResourceClass, PlaybackResourceDemand,
    PlaybackResourceEnforcement, PlaybackRuntimeAdmission, PlaybackRuntimeResourceClassPressure,
    PlaybackRuntimeResourcePressure,
};
#[cfg(test)]
pub(crate) use resource::{PlaybackResourceAdmissionStatus, PlaybackResourceCapacity};
use selection::{
    hls_pipeline_source_facts, hls_transcode_plan, playback_selection_context,
    remux_output_container,
};
pub(crate) use staging_policy::{HlsOutputLayout, HlsStagingPolicy, RemuxStagingPolicy};
pub(crate) use support::{
    PlaybackRuntimeDiagnostics, PlaybackSupportEvidenceContext, PlaybackSupportEvidenceRequest,
};

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

    async fn list_transcode_sessions(
        &self,
        filter: TranscodeSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<TranscodeSessionRecord>>;

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

    async fn list_transcode_sessions(
        &self,
        filter: TranscodeSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<TranscodeSessionRecord>> {
        TranscodeSessionRepository::list_transcode_sessions(self, filter, page).await
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
    pub preferences: PlaybackPreferenceContext,
    pub playback_generation: HlsPlaybackGeneration,
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
    pub preferences: PlaybackPreferenceContext,
    pub playback_generation: HlsPlaybackGeneration,
    pub transport_query: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct HlsPlaylistPlaybackOutput {
    pub session: PlaybackSessionRecord,
    pub body: String,
}

#[derive(Clone, Debug)]
pub(crate) struct SubtitlePlaybackRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub stream_index: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct SubtitlePlaybackOutput {
    pub content: String,
    pub content_type: &'static str,
    pub byte_len: u64,
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
    pub playback_generation: HlsPlaybackGeneration,
    pub transport_query: Option<String>,
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
    pub subtitle_stream_index: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackSessionHeartbeatRequest {
    pub session_id: PlaybackSessionId,
    pub state: PlaybackSessionState,
    pub position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlaybackAppService {
    config: NakoServerConfig,
    runtime_store: Arc<dyn PlaybackRuntimeStore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
    input: FfmpegInputService,
    planner: PlaybackPlanner,
    resource_admission: PlaybackRuntimeAdmission,
    cancellations: PlaybackSessionCancellationRegistry,
    remux: RemuxAppService,
    hls: HlsAppService,
    hls_artifacts: HlsArtifactService,
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
            resource_admission: PlaybackRuntimeAdmission::from_config(&config),
            remux: RemuxAppService::new(&config, cancellations.clone()),
            hls: HlsAppService::new(&config, cancellations.clone())?,
            hls_artifacts: HlsArtifactService::new(config.playback),
            config,
            runtime_store,
            storage_backends,
            runtime,
            cancellations,
        })
    }

    #[must_use]
    pub(crate) fn admit_playback_resource_demand(
        &self,
        demand: PlaybackResourceDemand,
    ) -> PlaybackResourceAdmissionDecision {
        self.resource_admission.decide(demand)
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
            BrowserPlaybackTicketMode::Subtitle => {
                ensure_playback_permission_allowed(
                    &effective_policy,
                    PlaybackPermission::MediaPlayback,
                )?;
                let stream_index =
                    request
                        .subtitle_stream_index
                        .ok_or_else(|| NakoError::InvalidInput {
                            message:
                                "subtitle browser playback ticket requires subtitle_stream_index"
                                    .to_owned(),
                        })?;
                let probe =
                    PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id)
                        .await?
                        .ok_or_else(|| NakoError::NotFound {
                            entity: "media_probe",
                            id: source.id.to_string(),
                        })?;
                let stream = subtitle_stream_for_probe(&probe, stream_index)?;
                let _ = subtitle_sidecar_file_name_for_stream(&source, stream)?;
                Ok(())
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

    fn client_capabilities_for_playback_session(
        session: &PlaybackSessionRecord,
    ) -> Result<ClientPlaybackCapabilities> {
        let Some(value) = session.client_capabilities_json.as_deref() else {
            return Ok(ClientPlaybackCapabilities::default());
        };

        serde_json::from_str(value).map_err(|err| NakoError::InvalidInput {
            message: format!(
                "playback session {} client capabilities could not be deserialized: {err}",
                session.id
            ),
        })
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
                let direct = decision.direct_play_plan().ok_or_else(|| {
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
                            preferences: PlaybackPreferenceContext::default(),
                            playback_generation: HlsPlaybackGeneration::default(),
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

    pub(crate) async fn direct_playback_session_preflight(
        &self,
        request: DirectPlaybackSessionStreamRequest,
    ) -> Result<DirectPlaybackPreflightOutput> {
        let session = self
            .existing_playback_session_for_media_request(
                &request.principal,
                request.playback_session_id,
                request.source_id,
                PlaybackSessionMode::Direct,
            )
            .await?;
        let response = self
            .plan_direct_play_preflight(request.source_id, request.range_request)
            .await?;

        Ok(DirectPlaybackPreflightOutput { session, response })
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

    pub(crate) async fn subtitle_playback(
        &self,
        request: SubtitlePlaybackRequest,
    ) -> Result<SubtitlePlaybackOutput> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        self.ensure_subtitle_playback_allowed_for_source(&request.principal, &source)
            .await?;
        let probe = PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_probe",
                id: source.id.to_string(),
            })?;
        let stream = subtitle_stream_for_probe(&probe, request.stream_index)?;
        let file_name = subtitle_sidecar_file_name_for_stream(&source, stream)?;
        let content_type =
            subtitle_content_type_for_extension(stream.codec.as_deref().unwrap_or_default())?;
        let (source_uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let sidecar_uri = subtitle_sidecar_uri_for_source(&source_uri, &file_name)?;
        let metadata = backend
            .stat(&sidecar_uri)
            .await
            .map_err(|err| redact_subtitle_sidecar_storage_error(err, source.id, stream.index))?;
        if metadata
            .len
            .is_some_and(|len| len > SUBTITLE_SIDECAR_MAX_BYTES)
        {
            return Err(NakoError::InvalidInput {
                message: "subtitle sidecar exceeds playback size limit".to_owned(),
            });
        }
        let content = backend
            .read_to_string(&sidecar_uri)
            .await
            .map_err(|err| redact_subtitle_sidecar_storage_error(err, source.id, stream.index))?;
        let byte_len = content.len() as u64;
        if byte_len > SUBTITLE_SIDECAR_MAX_BYTES {
            return Err(NakoError::InvalidInput {
                message: "subtitle sidecar exceeds playback size limit".to_owned(),
            });
        }

        Ok(SubtitlePlaybackOutput {
            content,
            content_type,
            byte_len,
        })
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
        let mut playback_session = self
            .existing_playback_session_for_media_request(
                &request.principal,
                request.playback_session_id,
                request.source_id,
                PlaybackSessionMode::Remux,
            )
            .await?;
        let transcode_session_id = match playback_session.transcode_session_id {
            Some(transcode_session_id) => transcode_session_id,
            None => {
                let client = Self::client_capabilities_for_playback_session(&playback_session)?;
                let effective_policy = self
                    .effective_playback_policy_for_source_id(&request.principal, request.source_id)
                    .await?;
                let remux_start = self
                    .start_remux_source_with_policy(
                        RemuxSourceRequest {
                            source_id: request.source_id,
                            client,
                            output_container: request.output_container,
                        },
                        effective_policy,
                    )
                    .await?;
                playback_session = self
                    .link_playback_session_transcode(playback_session.id, remux_start.session.id)
                    .await?;
                remux_start.session.id
            }
        };
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
                    preferences: request.preferences.clone(),
                    playback_generation: request.playback_generation,
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
        let playback_session = self
            .link_playback_session_transcode(playback_session.id, playlist.session.id)
            .await?;
        let body = self
            .hls_artifacts
            .read_playback_playlist(
                &playlist.session,
                playback_session.id,
                request.transport_query.as_deref(),
            )
            .await?;

        Ok(HlsPlaylistPlaybackOutput {
            session: playback_session,
            body,
        })
    }

    pub(crate) async fn hls_playlist_for_playback_session(
        &self,
        request: HlsPlaylistSessionRequest,
    ) -> Result<HlsPlaylistPlaybackOutput> {
        let mut playback_session = self
            .existing_playback_session_for_media_request(
                &request.principal,
                request.playback_session_id,
                request.source_id,
                PlaybackSessionMode::Hls,
            )
            .await?;
        if playback_session.transcode_session_id.is_none() {
            let client = Self::client_capabilities_for_playback_session(&playback_session)?;
            let effective_policy = self
                .effective_playback_policy_for_source_id(&request.principal, request.source_id)
                .await?;
            let playlist = self
                .hls_playlist_with_policy(
                    HlsSourceRequest {
                        source_id: request.source_id,
                        client,
                        preferences: PlaybackPreferenceContext::default(),
                        playback_generation: request.playback_generation,
                    },
                    effective_policy,
                )
                .await?;
            playback_session = self
                .link_playback_session_transcode(playback_session.id, playlist.session.id)
                .await?;
            let body = self
                .hls_artifacts
                .read_playback_playlist(
                    &playlist.session,
                    playback_session.id,
                    request.transport_query.as_deref(),
                )
                .await?;

            return Ok(HlsPlaylistPlaybackOutput {
                session: playback_session,
                body,
            });
        }

        let transcode_session_id = playback_session
            .transcode_session_id
            .expect("checked above");
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
        let body = self
            .hls_artifacts
            .read_playback_playlist(
                &transcode,
                playback_session.id,
                request.transport_query.as_deref(),
            )
            .await?;

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
        self.run_remux_source_context(context, None).await
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

        let resource_permit = self
            .resource_admission
            .try_acquire(&context.resource_demand())?;
        let task_app = self.clone();
        let task_request = request.clone();
        let task_effective_policy = effective_policy;
        self.runtime
            .spawn("playback_remux_start", "playback.remux", async move {
                if let Err(error) = task_app
                    .remux_source_with_policy(
                        task_request,
                        task_effective_policy,
                        Some(resource_permit),
                    )
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
        resource_permit: Option<resource::PlaybackResourcePermitSet>,
    ) -> Result<RemuxSourceOutput> {
        let context = self
            .remux_source_context(&request, effective_policy)
            .await?;
        self.run_remux_source_context(context, resource_permit)
            .await
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
        resource_permit: Option<resource::PlaybackResourcePermitSet>,
    ) -> Result<RemuxSourceOutput> {
        let resource_demand = context.resource_demand();
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
                &self.resource_admission,
                resource_demand,
                resource_permit,
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
        context.preferences.remux_output_container = Some(transcode_remux_container_to_playback(
            request.output_container,
        ));
        let target = playback_target_for_client(request.client.clone());
        let target_profile = PlaybackTargetProfile::from_target(&target, context.clone());
        let remote_input = target_profile.storage.remote;
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
        let profile_identity = build_playback_remux_profile(PlaybackRemuxProfileRequest {
            output_container,
            track_selection: playback_track_selection_to_transcode(
                target_profile.track_selection(),
            ),
            remote_input: target_profile.storage.remote,
            playback_profile_key: target_profile.identity_key(),
        })?
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
            remote_input,
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
        let context = self
            .hls_source_context(&request, effective_policy.into())
            .await?;
        self.run_hls_source_context(context).await
    }

    async fn hls_source_context(
        &self,
        request: &HlsSourceRequest,
        effective_policy: Option<EffectivePlaybackPolicy>,
    ) -> Result<HlsSourceContext> {
        let source = self.get_source_or_not_found(request.source_id).await?;
        let probe =
            PlaybackRuntimeStore::get_media_probe(self.runtime_store.as_ref(), source.id).await?;
        let (uri, backend) = self.storage_backend_for_media_source(&source).await?;
        let mut context = playback_selection_context(&uri, backend.as_ref()).await;
        context.preferences = request.preferences.clone();
        context.preferences.transcode_output_container = Some(PlaybackTranscodeContainer::Hls);
        let target = playback_target_for_client(request.client.clone());
        let target_profile = PlaybackTargetProfile::from_target(&target, context.clone());
        let remote_input = target_profile.storage.remote;
        let effective_policy =
            effective_policy.unwrap_or_else(|| default_playback_policy_for_source(&source));
        let decision = self.planner.plan(PlaybackPlanningRequest {
            source: &source,
            probe: probe.as_ref(),
            target: &target,
            effective_policy: &effective_policy,
            context,
        });
        ensure_playback_decision_allowed(&decision)?;
        let transcode_plan = hls_transcode_plan(&decision)?;
        let track_selection =
            playback_track_selection_to_transcode(target_profile.track_selection());
        let source_facts = hls_pipeline_source_facts(probe.as_ref(), track_selection);
        let mut execution_policy = self.hls.execution_policy_for_hls(
            track_selection,
            playback_output_constraints_to_transcode(target_profile.output_constraints()),
            source_facts.clone(),
        )?;
        let media_rendition_plan =
            hls_media_rendition_plan(probe.as_ref(), source_facts.as_ref(), track_selection)?;
        if media_rendition_plan.has_subtitles() {
            execution_policy.subtitle_strategy = TranscodeSubtitleStrategy::SidecarSelected;
        }
        let hls_profile = build_playback_hls_profile(PlaybackHlsProfileRequest {
            plan: transcode_plan.clone(),
            execution_policy,
            hls_output: playback_hls_output_requirement_to_transcode(
                target_profile.hls_output_requirement(),
            ),
            track_selection,
            remote_input: target_profile.storage.remote,
            playback_profile_key: target_profile.identity_key(),
        })?;
        let execution_policy = hls_profile.execution_policy;
        let hls_output =
            hls_profile
                .hls_output_requirement()
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "hls transcode profile did not carry HLS output requirements"
                        .to_owned(),
                })?;
        let adaptive_ladder_plan =
            (hls_output.variant_policy == HlsVariantPolicy::Adaptive).then(|| {
                HlsAdaptiveLadderPlan::from_source_facts(
                    source_facts.as_ref(),
                    execution_policy.output_constraints,
                )
            });
        let request_variant_plan =
            HlsRequestVariantPlan::new(adaptive_ladder_plan, media_rendition_plan)
                .with_playback_generation(request.playback_generation);
        let profile_identity = hls_profile.identity();
        let source_identity = TranscodeSourceIdentity::from_media_source(&source);
        let request_identity = if let Some(request_variant) = request_variant_plan.identity_key() {
            profile_identity.bind_source_with_request_variant(&source_identity, request_variant)
        } else {
            profile_identity.bind_source(&source_identity)
        };
        let staging = HlsStagingPolicy::new(self.config.remux_staging_root.join("hls"))?;
        let layout = staging.layout_for_output_with_request_variant_plan(
            source.id,
            &request_identity,
            hls_output,
            &request_variant_plan,
        )?;

        Ok(HlsSourceContext {
            source,
            decision,
            uri,
            backend,
            layout,
            track_selection: hls_profile.track_selection,
            execution_policy,
            playback_generation: request.playback_generation,
            request_identity: request_identity.clone(),
            request_key: request_identity.persisted_request_key().to_owned(),
            remote_input,
        })
    }

    async fn run_hls_source_context(&self, context: HlsSourceContext) -> Result<HlsSourceOutput> {
        let input = self
            .input
            .source_input_for_ffmpeg(&context.source, &context.uri, &context.backend)
            .await?;
        self.run_hls_source_context_with_input(context, input, None)
            .await
    }

    async fn run_hls_source_context_with_input(
        &self,
        context: HlsSourceContext,
        input: FfmpegSourceInput,
        resource_permit: Option<resource::PlaybackResourcePermitSet>,
    ) -> Result<HlsSourceOutput> {
        let resource_demand = context.resource_demand();
        let result = self
            .hls
            .run(
                self.runtime_store.as_ref(),
                context.source,
                context.decision,
                input.path.clone(),
                context.layout,
                context.track_selection,
                context.execution_policy,
                context.playback_generation,
                context.request_identity,
                &self.resource_admission,
                resource_demand,
                resource_permit,
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

    async fn start_hls_playlist_with_policy(
        &self,
        request: HlsSourceRequest,
        effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    ) -> Result<HlsPlaylistReadyOutput> {
        let effective_policy = effective_policy.into();
        let context = self
            .hls_source_context(&request, effective_policy.clone())
            .await?;

        if let Some(active) = PlaybackRuntimeStore::find_active_transcode_session(
            self.runtime_store.as_ref(),
            context.source.id,
            TranscodeSessionKind::HlsTranscode,
            &context.request_key,
        )
        .await?
        {
            return self
                .wait_for_hls_playlist_ready_context(context, active.id)
                .await;
        }

        if let Some(latest) = PlaybackRuntimeStore::find_latest_transcode_session(
            self.runtime_store.as_ref(),
            context.source.id,
            TranscodeSessionKind::HlsTranscode,
            &context.request_key,
        )
        .await?
        {
            if latest.state == TranscodeSessionState::Finished
                && latest.output_path == context.layout.playlist_path
                && path_exists(&context.layout.playlist_path)?
            {
                return Ok(context.playlist_ready(latest));
            }
        }

        let input = self
            .input
            .source_input_for_ffmpeg(&context.source, &context.uri, &context.backend)
            .await?;
        let resource_permit = match self
            .resource_admission
            .try_acquire(&context.resource_demand())
        {
            Ok(permit) => permit,
            Err(error) => {
                if let Err(release_error) = self.input.release_source_input(input).await {
                    warn!(
                        error = %release_error,
                        "failed to release HLS source input after playback resource admission rejection",
                    );
                }
                return Err(error);
            }
        };
        let wait_context = context.clone();
        let task_app = self.clone();
        self.runtime
            .spawn("playback_hls_start", "playback.hls", async move {
                if let Err(error) = task_app
                    .run_hls_source_context_with_input(context, input, Some(resource_permit))
                    .await
                {
                    warn!(error = %error, "background hls start failed");
                }
            });

        self.wait_for_hls_playlist_ready_context(wait_context, None)
            .await
    }

    async fn wait_for_hls_playlist_ready_context(
        &self,
        context: HlsSourceContext,
        preferred_session_id: impl Into<Option<TranscodeSessionId>>,
    ) -> Result<HlsPlaylistReadyOutput> {
        let preferred_session_id = preferred_session_id.into();
        let deadline = tokio::time::Instant::now()
            + std::time::Duration::from_millis(self.config.remux_timeout_ms.max(1));

        loop {
            let session = if let Some(session_id) = preferred_session_id {
                Some(self.get_transcode_session(session_id).await?)
            } else {
                PlaybackRuntimeStore::find_latest_transcode_session(
                    self.runtime_store.as_ref(),
                    context.source.id,
                    TranscodeSessionKind::HlsTranscode,
                    &context.request_key,
                )
                .await?
            };

            if let Some(session) = session {
                match session.state {
                    TranscodeSessionState::Finished => {
                        if !path_exists(&context.layout.playlist_path)? {
                            return Err(NakoError::storage_io(
                                context.layout.playlist_path.display().to_string(),
                                "finished hls session playlist is missing",
                            ));
                        }

                        return Ok(context.playlist_ready(session));
                    }
                    TranscodeSessionState::Running => {
                        if self.hls_artifacts.playlist_is_ready(&session).await? {
                            return Ok(context.playlist_ready(session));
                        }
                    }
                    TranscodeSessionState::Cancelled => {
                        return Err(NakoError::Provider {
                            provider: "ffmpeg_hls".to_owned(),
                            message: "hls session was cancelled".to_owned(),
                        });
                    }
                    TranscodeSessionState::Failed => {
                        return Err(NakoError::Provider {
                            provider: "ffmpeg_hls".to_owned(),
                            message: session
                                .failure_message
                                .unwrap_or_else(|| "hls runner failed".to_owned()),
                        });
                    }
                    TranscodeSessionState::Planned
                    | TranscodeSessionState::Starting
                    | TranscodeSessionState::CancelRequested => {}
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(NakoError::Conflict {
                    message: format!(
                        "hls playlist for source {} did not become ready before timeout",
                        context.source.id
                    ),
                });
            }

            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
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
            .start_hls_playlist_with_policy(request, effective_policy)
            .await?;

        let body = tokio::fs::read_to_string(&output.playlist_path)
            .await
            .map_err(|err| {
                NakoError::storage_io(
                    output.playlist_path.display().to_string(),
                    format!("failed to read hls playlist: {err}"),
                )
            })?;

        let manifest = hls_artifact_manifest_for_session(&output.session)?;
        let body = author_hls_session_playlist(
            &body,
            &manifest,
            HlsPlaylistSessionBinding::Transcode(output.session.id),
            HlsPlaylistUrlDecoration::none(),
        )?;

        Ok(HlsPlaylistOutput {
            source: output.source,
            decision: output.decision,
            body,
            session: output.session,
        })
    }

    pub(crate) async fn plan_hls_segment(
        &self,
        session_id: TranscodeSessionId,
        segment_name: &str,
    ) -> Result<HlsSegmentPlan> {
        let session = self.get_transcode_session(session_id).await?;
        self.hls_artifacts
            .plan_segment(&session, segment_name)
            .await
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
        support::support_evidence_context(self.runtime_store.as_ref(), request).await
    }

    #[must_use]
    pub(crate) fn runtime_diagnostics(&self) -> PlaybackRuntimeDiagnostics {
        support::runtime_diagnostics(&self.config, &self.hls, &self.resource_admission)
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

    async fn ensure_subtitle_playback_allowed_for_source(
        &self,
        principal: &AuthenticatedPrincipal,
        source: &MediaSource,
    ) -> Result<()> {
        let effective_policy = self
            .effective_playback_policy_for_source(principal, source)
            .await?;
        let context = self.playback_selection_context_for_source(source).await?;
        if context.storage.remote {
            ensure_playback_permission_allowed(
                &effective_policy,
                PlaybackPermission::RemotePlayback,
            )?;
        }
        ensure_playback_permission_allowed(&effective_policy, PlaybackPermission::MediaPlayback)
    }
}

fn subtitle_stream_for_probe(
    probe: &MediaProbeResult,
    stream_index: u32,
) -> Result<&MediaStreamInfo> {
    probe
        .streams
        .iter()
        .find(|stream| stream.index == stream_index && stream.kind == MediaStreamKind::Subtitle)
        .ok_or_else(|| NakoError::NotFound {
            entity: "subtitle_stream",
            id: stream_index.to_string(),
        })
}

fn redact_subtitle_sidecar_storage_error(
    error: NakoError,
    source_id: MediaSourceId,
    stream_index: u32,
) -> NakoError {
    match error {
        NakoError::NotFound { .. } => NakoError::NotFound {
            entity: "subtitle_sidecar",
            id: format!("{source_id}:{stream_index}"),
        },
        NakoError::Storage { kind, .. } => NakoError::Storage {
            uri: "subtitle_sidecar".to_owned(),
            kind,
            message: "subtitle sidecar storage operation failed".to_owned(),
        },
        NakoError::Database { message } => NakoError::Database { message },
        NakoError::InvalidInput { message } => NakoError::InvalidInput { message },
        NakoError::Conflict { message } => NakoError::Conflict { message },
        NakoError::Unauthorized { message } => NakoError::Unauthorized { message },
        NakoError::Forbidden { message } => NakoError::Forbidden { message },
        NakoError::Unsupported(message) => NakoError::Unsupported(message),
        NakoError::Provider { provider, message } => NakoError::Provider { provider, message },
    }
}

fn hls_media_rendition_plan(
    probe: Option<&MediaProbeResult>,
    source_facts: Option<&TranscodePipelineSourceFacts>,
    track_selection: TranscodeTrackSelection,
) -> Result<HlsMediaRenditionPlan> {
    HlsMediaRenditionPlan::selected_from_source_facts(source_facts, track_selection)?
        .with_audio_renditions(hls_audio_renditions_from_probe(probe, source_facts))
}

fn hls_audio_renditions_from_probe(
    probe: Option<&MediaProbeResult>,
    source_facts: Option<&TranscodePipelineSourceFacts>,
) -> Vec<HlsAudioRendition> {
    let Some(probe) = probe else {
        return Vec::new();
    };
    let audio_streams = probe
        .streams
        .iter()
        .filter(|stream| matches!(stream.kind, MediaStreamKind::Audio))
        .collect::<Vec<_>>();
    if audio_streams.len() < 2 {
        return Vec::new();
    }

    let default_stream_index = source_facts
        .and_then(|facts| facts.audio.as_ref())
        .map(|stream| stream.index)
        .unwrap_or(audio_streams[0].index);

    audio_streams
        .into_iter()
        .enumerate()
        .map(|(index, stream)| {
            HlsAudioRendition::new(
                index,
                stream.index,
                stream.language.clone(),
                stream.index == default_stream_index,
            )
        })
        .collect()
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
struct HlsSourceContext {
    source: MediaSource,
    decision: PlaybackDecision,
    uri: StorageUri,
    backend: Arc<super::storage::LibraryStorageBackend>,
    layout: HlsOutputLayout,
    track_selection: TranscodeTrackSelection,
    execution_policy: nako_transcode::TranscodeExecutionPolicy,
    playback_generation: HlsPlaybackGeneration,
    request_identity: TranscodeRequestIdentity,
    request_key: String,
    remote_input: bool,
}

impl HlsSourceContext {
    fn resource_demand(&self) -> PlaybackResourceDemand {
        PlaybackResourceDemand::hls(self.remote_input, self.execution_policy)
    }

    fn playlist_ready(self, session: TranscodeSessionRecord) -> HlsPlaylistReadyOutput {
        HlsPlaylistReadyOutput {
            source: self.source,
            decision: self.decision,
            playlist_path: self.layout.playlist_path,
            session,
        }
    }
}

#[derive(Clone, Debug)]
struct HlsPlaylistReadyOutput {
    source: MediaSource,
    decision: PlaybackDecision,
    playlist_path: PathBuf,
    session: TranscodeSessionRecord,
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
    remote_input: bool,
}

impl RemuxSourceContext {
    fn resource_demand(&self) -> PlaybackResourceDemand {
        PlaybackResourceDemand::remux(self.remote_input)
    }

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
