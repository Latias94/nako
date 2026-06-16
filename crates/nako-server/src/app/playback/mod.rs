use std::{collections::HashSet, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use nako_api::public_client::{PlaybackDecisionResponse, playback_decision_response_to_dto};
use nako_core::{
    AuthenticatedPrincipal, EventOutboxRepository, LibraryAccessLevel, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, MediaSourceId, MediaStreamInfo,
    MediaStreamKind, NakoError, NewOutboxEvent, NewPlaybackSession, NewTranscodeSession,
    OutboxEventRecord, PageRequest, PlaybackPermission, PlaybackPolicyRepository,
    PlaybackSessionHeartbeat, PlaybackSessionId, PlaybackSessionListFilter, PlaybackSessionMode,
    PlaybackSessionRecord, PlaybackSessionRepository, PlaybackSessionState, RendererSessionId,
    Result, StagingManifestRepository, TranscodeFailureCategory, TranscodeSessionId,
    TranscodeSessionKind, TranscodeSessionListFilter, TranscodeSessionRecord,
    TranscodeSessionRepository, TranscodeSessionRuntimeMetrics, TranscodeSessionState, UserId,
    UserPrincipalId,
};
use nako_playback::{
    ClientPlaybackCapabilities, EffectivePlaybackPolicy, PlaybackDecision, PlaybackPlanner,
    PlaybackPlanningRequest, PlaybackPreferenceContext, PlaybackSelectionContext, PlaybackTarget,
};
use nako_streaming::{DirectPlayRangeRequest, DirectPlayResponsePlan};
use nako_transcode::{HlsPlaybackGeneration, RemuxContainer};
use nako_vfs::{StorageBackend as _, StorageUri};
use serde::{Deserialize, Serialize};

use crate::config::NakoServerConfig;

use super::{
    playback_ticket::BrowserPlaybackTicketMode,
    renderer::RendererAppService,
    renderer_transport_ticket::RendererTransportTicketService,
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
mod hls_flow;
mod input;
mod paths;
mod playlist;
mod remux;
mod remux_flow;
mod renderer_flow;
mod resource;
mod runtime_session;
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
use hls_artifact::HlsArtifactService;
use input::FfmpegInputService;
#[cfg(test)]
pub(crate) use input::source_path_for_ffmpeg_with_backend;
use paths::{ensure_remux_output_parent, path_exists};
use remux::RemuxAppService;
pub(crate) use remux::RemuxRequestKey;
pub(crate) use resource::{
    PlaybackResourceAdmissionDecision, PlaybackResourceClass, PlaybackResourceDemand,
    PlaybackResourceEnforcement, PlaybackRuntimeAdmission, PlaybackRuntimeResourceClassPressure,
    PlaybackRuntimeResourcePressure,
};
#[cfg(test)]
pub(crate) use resource::{PlaybackResourceAdmissionStatus, PlaybackResourceCapacity};
use selection::playback_selection_context;
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

    async fn find_latest_playback_session_by_transcode_session(
        &self,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<Option<PlaybackSessionRecord>>;

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

    async fn find_latest_playback_session_by_transcode_session(
        &self,
        transcode_session_id: TranscodeSessionId,
    ) -> Result<Option<PlaybackSessionRecord>> {
        PlaybackSessionRepository::find_latest_playback_session_by_transcode_session(
            self,
            transcode_session_id,
        )
        .await
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaybackTraceContext {
    request_id: String,
}

impl PlaybackTraceContext {
    #[must_use]
    pub(crate) fn from_request_id(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
        }
    }

    #[must_use]
    pub(crate) fn request_id(&self) -> &str {
        &self.request_id
    }
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
    pub trace_context: Option<PlaybackTraceContext>,
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

#[derive(Clone, Debug)]
pub(crate) struct HlsSegmentPlaybackRequest {
    pub principal: AuthenticatedPrincipal,
    pub source_id: MediaSourceId,
    pub transcode_session_id: TranscodeSessionId,
    pub segment_name: String,
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
    pub trace_context: Option<PlaybackTraceContext>,
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
pub(crate) struct ResolveRendererTransportPlaybackRequest {
    pub token: String,
    pub renderer_session_id: RendererSessionId,
    pub playback_session_id: PlaybackSessionId,
    pub source_id: MediaSourceId,
    pub mode: PlaybackSessionMode,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedRendererTransportPlaybackContext {
    pub principal: AuthenticatedPrincipal,
    pub renderer_session_id: RendererSessionId,
    pub playback_session_id: PlaybackSessionId,
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

const ACTIVE_PLAYBACK_SESSION_STATES: [PlaybackSessionState; 3] = [
    PlaybackSessionState::Active,
    PlaybackSessionState::Paused,
    PlaybackSessionState::CancelRequested,
];

#[derive(Clone, Debug)]
pub(crate) struct PlaybackAppService {
    config: NakoServerConfig,
    runtime_store: Arc<dyn PlaybackRuntimeStore>,
    storage_backends: StorageBackendRegistry,
    runtime: RuntimeSupervisor,
    renderer: RendererAppService,
    renderer_transport_tickets: RendererTransportTicketService,
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
        renderer: RendererAppService,
        renderer_transport_tickets: RendererTransportTicketService,
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
            renderer,
            renderer_transport_tickets,
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
            .effective_playback_policy_for_playable_source(principal, &source)
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
            .effective_playback_policy_for_playable_source(&request.principal, &source)
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

    pub(super) fn client_capabilities_for_playback_session(
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
        renderer_flow::start_renderer_playback_session(self, request).await
    }

    pub(crate) async fn resolve_renderer_transport_playback_context(
        &self,
        request: ResolveRendererTransportPlaybackRequest,
    ) -> Result<ResolvedRendererTransportPlaybackContext> {
        renderer_flow::resolve_renderer_transport_playback_context(self, request).await
    }

    pub(crate) async fn get_playback_session(
        &self,
        session_id: PlaybackSessionId,
    ) -> Result<PlaybackSessionRecord> {
        PlaybackRuntimeStore::get_playback_session(self.runtime_store.as_ref(), session_id)
            .await?
            .ok_or_else(|| playback_session_not_found(session_id))
    }

    pub(crate) async fn get_playback_session_for_control(
        &self,
        principal: &AuthenticatedPrincipal,
        session_id: PlaybackSessionId,
    ) -> Result<PlaybackSessionRecord> {
        let session = self.get_playback_session(session_id).await?;
        self.ensure_playback_session_control_access(principal, &session)
            .await?;

        Ok(session)
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

    pub(crate) async fn record_playback_session_heartbeat_for_control(
        &self,
        principal: &AuthenticatedPrincipal,
        request: PlaybackSessionHeartbeatRequest,
    ) -> Result<PlaybackSessionRecord> {
        let session = self
            .get_playback_session_for_control(principal, request.session_id)
            .await?;

        self.record_playback_session_heartbeat(PlaybackSessionHeartbeatRequest {
            session_id: session.id,
            state: request.state,
            position_ms: request.position_ms,
            duration_ms: request.duration_ms,
        })
        .await
    }

    pub(crate) async fn cancel_playback_session_for_control(
        &self,
        principal: &AuthenticatedPrincipal,
        session_id: PlaybackSessionId,
    ) -> Result<PlaybackSessionRecord> {
        let session = self
            .get_playback_session_for_control(principal, session_id)
            .await?;

        self.cancel_playback_session(session.id).await
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
            Some(backend.try_acquire_stream_permit()?)
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
        self.ensure_source_play_access(&request.principal, request.source_id)
            .await?;
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
        self.ensure_source_play_access(&request.principal, request.source_id)
            .await?;
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
        self.ensure_source_play_access_for_source(&request.principal, &source)
            .await?;
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
        remux_flow::remux_playback_stream(self, request).await
    }

    pub(crate) async fn remux_playback_session_stream(
        &self,
        request: RemuxPlaybackSessionStreamRequest,
    ) -> Result<RemuxPlaybackStreamOutput> {
        remux_flow::remux_playback_session_stream(self, request).await
    }

    pub(crate) async fn remux_playback_preflight(
        &self,
        request: RemuxPlaybackPreflightRequest,
    ) -> Result<RemuxPlaybackPreflightOutput> {
        remux_flow::remux_playback_preflight(self, request).await
    }

    pub(crate) async fn hls_playlist_playback(
        &self,
        request: HlsPlaylistPlaybackRequest,
    ) -> Result<HlsPlaylistPlaybackOutput> {
        hls_flow::hls_playlist_playback(self, request).await
    }

    pub(crate) async fn hls_playlist_for_playback_session(
        &self,
        request: HlsPlaylistSessionRequest,
    ) -> Result<HlsPlaylistPlaybackOutput> {
        hls_flow::hls_playlist_for_playback_session(self, request).await
    }

    pub(crate) async fn remux_source(
        &self,
        request: RemuxSourceRequest,
    ) -> Result<RemuxSourceOutput> {
        remux_flow::remux_source(self, request).await
    }

    pub(crate) async fn hls_source(&self, request: HlsSourceRequest) -> Result<HlsSourceOutput> {
        self.hls_source_with_policy(request, None).await
    }

    async fn hls_source_with_policy(
        &self,
        request: HlsSourceRequest,
        effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    ) -> Result<HlsSourceOutput> {
        hls_flow::hls_source_with_policy(self, request, effective_policy).await
    }

    pub(crate) async fn hls_playlist(
        &self,
        request: HlsSourceRequest,
    ) -> Result<HlsPlaylistOutput> {
        self.hls_playlist_with_policy(request, None, None).await
    }

    async fn hls_playlist_with_policy(
        &self,
        request: HlsSourceRequest,
        effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
        trace_context: Option<PlaybackTraceContext>,
    ) -> Result<HlsPlaylistOutput> {
        hls_flow::hls_playlist_with_policy(self, request, effective_policy, trace_context).await
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

    pub(crate) async fn hls_segment_playback(
        &self,
        request: HlsSegmentPlaybackRequest,
    ) -> Result<HlsSegmentPlan> {
        self.ensure_source_play_access(&request.principal, request.source_id)
            .await?;
        self.plan_hls_segment(request.transcode_session_id, &request.segment_name)
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

    pub(super) async fn cancel_superseded_hls_playback_sessions(
        &self,
        source_id: MediaSourceId,
        replacement_transcode_session_id: TranscodeSessionId,
        replacement_playback_session_id: PlaybackSessionId,
    ) -> Result<()> {
        let superseded = self
            .active_hls_playback_sessions_for_source(source_id)
            .await?;
        let superseded_transcode_candidates = superseded
            .iter()
            .filter_map(|session| session.transcode_session_id)
            .filter(|id| *id != replacement_transcode_session_id)
            .collect();
        let superseded_transcode_ids = self
            .cancelled_or_cancelling_hls_transcode_ids(superseded_transcode_candidates)
            .await?;
        if superseded_transcode_ids.is_empty() {
            return Ok(());
        }

        let ended_at_ms = crate::app::current_time_ms()?;
        for session in superseded {
            if session.id == replacement_playback_session_id
                || session.mode != PlaybackSessionMode::Hls
            {
                continue;
            }
            let Some(transcode_session_id) = session.transcode_session_id else {
                continue;
            };
            if !superseded_transcode_ids.contains(&transcode_session_id) {
                continue;
            }

            let _ = PlaybackRuntimeStore::set_playback_session_state(
                self.runtime_store.as_ref(),
                session.id,
                PlaybackSessionState::Cancelled,
                Some(ended_at_ms),
            )
            .await?;
        }

        Ok(())
    }

    async fn active_hls_playback_sessions_for_source(
        &self,
        source_id: MediaSourceId,
    ) -> Result<Vec<PlaybackSessionRecord>> {
        let mut sessions = Vec::new();
        let mut seen = HashSet::new();

        for state in ACTIVE_PLAYBACK_SESSION_STATES {
            for session in PlaybackRuntimeStore::list_playback_sessions(
                self.runtime_store.as_ref(),
                PlaybackSessionListFilter {
                    principal_id: None,
                    source_id: Some(source_id),
                    state: Some(state),
                },
                PageRequest::new(PageRequest::MAX_LIMIT, 0),
            )
            .await?
            {
                if session.mode == PlaybackSessionMode::Hls && seen.insert(session.id) {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    async fn cancelled_or_cancelling_hls_transcode_ids(
        &self,
        transcode_session_ids: Vec<TranscodeSessionId>,
    ) -> Result<HashSet<TranscodeSessionId>> {
        let mut ids = HashSet::new();
        let mut seen = HashSet::new();

        for transcode_session_id in transcode_session_ids {
            if !seen.insert(transcode_session_id) {
                continue;
            }
            let transcode = self.get_transcode_session(transcode_session_id).await?;
            if transcode.kind == TranscodeSessionKind::HlsTranscode
                && matches!(
                    transcode.state,
                    TranscodeSessionState::CancelRequested | TranscodeSessionState::Cancelled
                )
            {
                ids.insert(transcode_session_id);
            }
        }

        Ok(ids)
    }

    #[cfg(test)]
    pub(super) async fn source_path_for_ffmpeg(&self, source: &MediaSource) -> Result<PathBuf> {
        let (uri, backend) = self.storage_backend_for_media_source(source).await?;
        self.input
            .source_path_for_ffmpeg(source, &uri, &backend)
            .await
    }

    #[cfg(test)]
    pub(super) async fn with_source_path_for_ffmpeg<T, Operation, OperationFuture>(
        &self,
        source: &MediaSource,
        operation: Operation,
    ) -> Result<T>
    where
        Operation: FnOnce(PathBuf) -> OperationFuture,
        OperationFuture: std::future::Future<Output = Result<T>>,
    {
        let (uri, backend) = self.storage_backend_for_media_source(source).await?;
        self.input
            .with_source_input(source, &uri, &backend, operation)
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

    async fn ensure_playback_session_control_access(
        &self,
        principal: &AuthenticatedPrincipal,
        session: &PlaybackSessionRecord,
    ) -> Result<()> {
        if session.principal_id != principal.principal_id {
            return Err(playback_session_not_found(session.id));
        }

        let Some(source) =
            PlaybackRuntimeStore::get_media_source(self.runtime_store.as_ref(), session.source_id)
                .await?
        else {
            return Err(playback_session_not_found(session.id));
        };

        if principal.is_administrator() {
            return Ok(());
        }

        let effective_policy = self
            .effective_playback_policy_for_source(principal, &source)
            .await?;
        if effective_policy.library_access.allows_play() {
            Ok(())
        } else {
            Err(playback_session_not_found(session.id))
        }
    }

    pub(super) async fn existing_playback_session_for_media_request(
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

    pub(super) async fn effective_playback_policy_for_playable_source_id(
        &self,
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
    ) -> Result<EffectivePlaybackPolicy> {
        let source = self.get_source_or_not_found(source_id).await?;
        self.effective_playback_policy_for_playable_source(principal, &source)
            .await
    }

    async fn effective_playback_policy_for_playable_source(
        &self,
        principal: &AuthenticatedPrincipal,
        source: &MediaSource,
    ) -> Result<EffectivePlaybackPolicy> {
        let effective_policy = self
            .effective_playback_policy_for_source(principal, source)
            .await?;
        if principal.is_administrator() || effective_policy.library_access.allows_play() {
            Ok(effective_policy)
        } else {
            Err(library_play_access_forbidden())
        }
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
            .effective_playback_policy_for_playable_source(principal, &source)
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

    async fn ensure_source_play_access(
        &self,
        principal: &AuthenticatedPrincipal,
        source_id: MediaSourceId,
    ) -> Result<()> {
        let source = self.get_source_or_not_found(source_id).await?;
        self.ensure_source_play_access_for_source(principal, &source)
            .await
    }

    async fn ensure_source_play_access_for_source(
        &self,
        principal: &AuthenticatedPrincipal,
        source: &MediaSource,
    ) -> Result<()> {
        let _ = self
            .effective_playback_policy_for_playable_source(principal, source)
            .await?;

        Ok(())
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

fn playback_session_not_found(session_id: PlaybackSessionId) -> NakoError {
    NakoError::NotFound {
        entity: "playback_session",
        id: session_id.to_string(),
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

fn playback_target_for_client(client: ClientPlaybackCapabilities) -> PlaybackTarget {
    PlaybackTarget::browser_with_capabilities("Public Client", client)
}

fn library_play_access_forbidden() -> NakoError {
    NakoError::Forbidden {
        message: "required Library Access level 'play' is not available".to_owned(),
    }
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use nako_playback::{
        PlaybackAudioCompatibilityReason, PlaybackAudioDownmixRequirement,
        PlaybackAudioNormalizationRequirement, PlaybackAudioOutputRequirement,
        PlaybackColorCompatibilityReason, PlaybackColorPipelineRequirement,
        PlaybackColorPipelineTarget, PlaybackHdrToneMappingRequirement,
    };
    use nako_transcode::{
        HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationReport,
        StaticHardwareAccelerationDetector, TranscodeAudioCompatibilityReason,
        TranscodeAudioCompatibilityReasons, TranscodeAudioDownmixRequirement,
        TranscodeAudioNormalizationRequirement, TranscodeAudioOutputRequirement,
        TranscodeColorPipelineRequirement, TranscodeColorPipelineTarget,
        TranscodeHdrToneMappingRequirement, TranscodeOutputConstraints,
    };

    use super::*;
    use crate::config::{MetadataConfig, PlaybackConfig, StagingConfig, TranscodeConfig};
    use crate::playback_mapping::{
        playback_audio_output_requirement_to_transcode,
        playback_color_pipeline_requirement_to_transcode,
    };

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
            vfs_cache_repair_automation:
                crate::config::VfsCacheRepairAutomationRuntimeConfig::default(),
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
    fn hls_service_execution_policy_preserves_audio_output_requirement() {
        let config = test_config(TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::None,
            hardware_fallback: HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 1,
            gpu_concurrency: 2,
        });
        let detector =
            StaticHardwareAccelerationDetector::new(HardwareAccelerationReport::with_available([
                HardwareAcceleration::None,
            ]));
        let service = HlsAppService::new_with_hardware_detector(&config, &detector).unwrap();
        let audio_output = TranscodeAudioOutputRequirement {
            source_channels: Some(6),
            max_supported_channels: Some(2),
            target_channels: Some(2),
            downmix: TranscodeAudioDownmixRequirement::Required,
            normalization: TranscodeAudioNormalizationRequirement::Requested,
            reasons: TranscodeAudioCompatibilityReasons {
                channel_limit_exceeded: true,
                downmix_required: true,
                normalization_requested: true,
            },
        };

        let policy = service
            .execution_policy_for_hls(
                nako_transcode::TranscodeTrackSelection::default(),
                TranscodeOutputConstraints::default(),
                audio_output,
                TranscodeColorPipelineRequirement::none(),
                None,
            )
            .unwrap();

        assert_eq!(policy.audio_output, audio_output);
    }

    #[test]
    fn hls_service_execution_policy_preserves_hdr_color_pipeline_requirement() {
        let config = test_config(TranscodeConfig {
            hardware_acceleration: HardwareAcceleration::Nvenc,
            hardware_fallback: HardwareAccelerationFallback::Cpu,
            cpu_concurrency: 1,
            gpu_concurrency: 2,
        });
        let detector =
            StaticHardwareAccelerationDetector::new(HardwareAccelerationReport::with_available([
                HardwareAcceleration::None,
                HardwareAcceleration::Nvenc,
            ]));
        let service = HlsAppService::new_with_hardware_detector(&config, &detector).unwrap();
        let color_pipeline = TranscodeColorPipelineRequirement::hdr_to_sdr_required();

        let policy = service
            .execution_policy_for_hls(
                nako_transcode::TranscodeTrackSelection::default(),
                TranscodeOutputConstraints::default(),
                TranscodeAudioOutputRequirement::none(),
                color_pipeline,
                None,
            )
            .unwrap();

        assert_eq!(policy.color_pipeline, color_pipeline);
        assert!(policy.acceleration.is_software_only());
    }

    #[test]
    fn hls_audio_output_requirement_mapping_preserves_playback_reasons() {
        let audio_output =
            playback_audio_output_requirement_to_transcode(&PlaybackAudioOutputRequirement {
                source_channels: Some(8),
                max_supported_channels: Some(2),
                target_channels: Some(2),
                downmix: PlaybackAudioDownmixRequirement::Required,
                normalization: PlaybackAudioNormalizationRequirement::Requested,
                reasons: vec![
                    PlaybackAudioCompatibilityReason::ChannelLimitExceeded,
                    PlaybackAudioCompatibilityReason::DownmixRequired,
                    PlaybackAudioCompatibilityReason::NormalizationRequested,
                ],
            });

        assert_eq!(audio_output.source_channels, Some(8));
        assert_eq!(audio_output.max_supported_channels, Some(2));
        assert_eq!(audio_output.target_channels, Some(2));
        assert_eq!(
            audio_output.downmix,
            TranscodeAudioDownmixRequirement::Required
        );
        assert_eq!(
            audio_output.normalization,
            TranscodeAudioNormalizationRequirement::Requested
        );
        assert!(
            audio_output
                .reasons
                .has(TranscodeAudioCompatibilityReason::ChannelLimitExceeded)
        );
        assert!(
            audio_output
                .reasons
                .has(TranscodeAudioCompatibilityReason::DownmixRequired)
        );
        assert!(
            audio_output
                .reasons
                .has(TranscodeAudioCompatibilityReason::NormalizationRequested)
        );
    }

    #[test]
    fn hls_audio_output_requirement_mapping_collapses_compatible_source_facts() {
        let audio_output =
            playback_audio_output_requirement_to_transcode(&PlaybackAudioOutputRequirement {
                source_channels: Some(2),
                max_supported_channels: None,
                target_channels: None,
                downmix: PlaybackAudioDownmixRequirement::None,
                normalization: PlaybackAudioNormalizationRequirement::None,
                reasons: Vec::new(),
            });

        assert_eq!(audio_output, TranscodeAudioOutputRequirement::none());
    }

    #[test]
    fn hls_color_pipeline_requirement_mapping_preserves_playback_reasons() {
        let color_pipeline =
            playback_color_pipeline_requirement_to_transcode(&PlaybackColorPipelineRequirement {
                source: None,
                target: PlaybackColorPipelineTarget::Sdr,
                tone_mapping: PlaybackHdrToneMappingRequirement::Required,
                reasons: vec![
                    PlaybackColorCompatibilityReason::SourceHdrDetected,
                    PlaybackColorCompatibilityReason::ClientHdrUnsupported,
                    PlaybackColorCompatibilityReason::ToneMappingRequired,
                ],
            });

        assert_eq!(color_pipeline.target, TranscodeColorPipelineTarget::Sdr);
        assert_eq!(
            color_pipeline.tone_mapping,
            TranscodeHdrToneMappingRequirement::Required
        );
        assert!(
            color_pipeline
                .reasons
                .has(nako_transcode::TranscodeColorCompatibilityReason::SourceHdrDetected)
        );
        assert!(
            color_pipeline
                .reasons
                .has(nako_transcode::TranscodeColorCompatibilityReason::ClientHdrUnsupported)
        );
        assert!(
            color_pipeline
                .reasons
                .has(nako_transcode::TranscodeColorCompatibilityReason::ToneMappingRequired)
        );
    }

    #[test]
    fn hls_color_pipeline_requirement_mapping_collapses_compatible_source_facts() {
        let color_pipeline =
            playback_color_pipeline_requirement_to_transcode(&PlaybackColorPipelineRequirement {
                source: None,
                target: PlaybackColorPipelineTarget::PreserveSource,
                tone_mapping: PlaybackHdrToneMappingRequirement::None,
                reasons: Vec::new(),
            });

        assert_eq!(color_pipeline, TranscodeColorPipelineRequirement::none());
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
                TranscodeAudioOutputRequirement::none(),
                TranscodeColorPipelineRequirement::none(),
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
