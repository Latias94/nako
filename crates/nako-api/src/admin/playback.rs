use nako_client_protocol::PageInfo;
use nako_core::{
    LibraryId, MediaItemId, MediaSource, MediaSourceId, PlaybackPermission, PlaybackSessionId,
    PlaybackSessionMode, PlaybackSessionRecord, PlaybackSessionState, PlaybackTargetKind,
    PlaybackTargetNetworkScope, PlaybackTargetTransportAuth, RendererControlCommand,
    RendererSessionId, RendererSessionRecord, RendererSessionState, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionRuntimeMetrics, TranscodeSessionState,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    AdminHardwareAcceleration, AdminHardwareAccelerationFallback, AdminHardwareAccelerationPolicy,
    StorageBackendRuntimeStateScope,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSessionListResponse {
    pub sessions: Vec<AdminPlaybackSessionListItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSessionListItem {
    pub id: PlaybackSessionId,
    pub source_id: MediaSourceId,
    pub item_id: MediaItemId,
    pub mode: PlaybackSessionMode,
    pub state: PlaybackSessionState,
    pub transcode_session_id: Option<TranscodeSessionId>,
    pub has_client_capabilities: bool,
    pub active: bool,
    pub terminal: bool,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub last_heartbeat_at_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminPlaybackSessionListItem {
    #[must_use]
    pub fn from_record(session: PlaybackSessionRecord) -> Self {
        Self {
            id: session.id,
            source_id: session.source_id,
            item_id: session.item_id,
            mode: session.mode,
            state: session.state,
            transcode_session_id: session.transcode_session_id,
            has_client_capabilities: session.client_capabilities_json.is_some(),
            active: session.state.is_active(),
            terminal: session.state.is_terminal(),
            started_at_ms: session.started_at_ms,
            ended_at_ms: session.ended_at_ms,
            last_heartbeat_at_ms: session.last_heartbeat_at_ms,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRuntimeDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub readiness: AdminPlaybackReadinessDiagnostics,
    pub policy: AdminPlaybackPolicyDiagnostics,
    pub ffmpeg: AdminPlaybackFfmpegDiagnostics,
    pub hardware: AdminPlaybackHardwareDiagnostics,
    pub transcode: AdminPlaybackTranscodeBudgetDiagnostics,
    pub remux: AdminPlaybackRemuxRuntimeDiagnostics,
    pub remote_playback: AdminPlaybackRemoteBudgetDiagnostics,
    pub staging: AdminPlaybackStagingDiagnostics,
    pub artifact_lifecycle: AdminPlaybackArtifactLifecycleDiagnostics,
    pub throttle: AdminPlaybackThrottleDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRendererRuntimeDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub readiness: AdminRendererReadinessDiagnostics,
    pub summary: AdminRendererSessionSummary,
    pub adapters: Vec<AdminRendererAdapterDiagnostics>,
    pub sessions: Vec<AdminRendererSessionDiagnostics>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRendererReadinessDiagnostics {
    pub status: AdminRendererReadinessStatus,
    pub reason: AdminRendererReadinessReason,
    pub checks: Vec<AdminRendererReadinessCheck>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererReadinessReason {
    Ready,
    RendererRepositoryReady,
    NakoRemoteClientAdapterReady,
    NakoRemoteClientCastSafeTransportReady,
    RendererRepositoryUnavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRendererReadinessCheck {
    pub name: AdminRendererReadinessCheckName,
    pub status: AdminRendererReadinessStatus,
    pub reason: AdminRendererReadinessReason,
}

impl AdminRendererReadinessCheck {
    #[must_use]
    pub const fn ready(
        name: AdminRendererReadinessCheckName,
        reason: AdminRendererReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminRendererReadinessStatus::Ready,
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererReadinessCheckName {
    RendererRepository,
    NakoRemoteClientAdapter,
    NakoRemoteClientCastSafeTransport,
}

impl AdminRendererReadinessDiagnostics {
    #[must_use]
    pub fn ready() -> Self {
        Self {
            status: AdminRendererReadinessStatus::Ready,
            reason: AdminRendererReadinessReason::RendererRepositoryReady,
            checks: vec![
                AdminRendererReadinessCheck::ready(
                    AdminRendererReadinessCheckName::RendererRepository,
                    AdminRendererReadinessReason::RendererRepositoryReady,
                ),
                AdminRendererReadinessCheck::ready(
                    AdminRendererReadinessCheckName::NakoRemoteClientAdapter,
                    AdminRendererReadinessReason::NakoRemoteClientAdapterReady,
                ),
                AdminRendererReadinessCheck::ready(
                    AdminRendererReadinessCheckName::NakoRemoteClientCastSafeTransport,
                    AdminRendererReadinessReason::NakoRemoteClientCastSafeTransportReady,
                ),
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRendererSessionSummary {
    pub returned_sessions: u32,
    pub online_sessions: u32,
    pub offline_sessions: u32,
    pub revoked_sessions: u32,
    pub expired_sessions: u32,
    pub active_playback_sessions: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRendererSessionDiagnostics {
    pub id: RendererSessionId,
    pub target_kind: PlaybackTargetKind,
    pub display_name: String,
    pub network_scope: PlaybackTargetNetworkScope,
    pub transport_auth: PlaybackTargetTransportAuth,
    pub state: RendererSessionState,
    pub active_playback_session_id: Option<PlaybackSessionId>,
    pub supported_commands: Vec<RendererControlCommand>,
    pub has_media_capabilities: bool,
    pub direct_play_supported: bool,
    pub expired: bool,
    pub last_seen_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminRendererSessionDiagnostics {
    #[must_use]
    pub fn from_record(session: RendererSessionRecord, now_ms: i64) -> Self {
        let direct_play_supported = session
            .media_capabilities_json
            .as_deref()
            .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
            .and_then(|value| {
                value
                    .get("direct_play")
                    .and_then(serde_json::Value::as_bool)
            })
            .unwrap_or(false);
        let expired = session
            .expires_at_ms
            .is_some_and(|expires_at_ms| expires_at_ms <= now_ms);

        Self {
            id: session.id,
            target_kind: session.target_kind,
            display_name: session.display_name,
            network_scope: session.network_scope,
            transport_auth: session.transport_auth,
            state: session.state,
            active_playback_session_id: session.active_playback_session_id,
            supported_commands: session.control_capabilities.commands,
            has_media_capabilities: session.media_capabilities_json.is_some(),
            direct_play_supported,
            expired,
            last_seen_at_ms: session.last_seen_at_ms,
            expires_at_ms: session.expires_at_ms,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRendererAdapterDiagnostics {
    pub adapter: AdminRendererAdapterKind,
    pub target_kind: PlaybackTargetKind,
    pub status: AdminRendererAdapterStatus,
    pub reason: AdminRendererAdapterReason,
    pub control_plane: AdminRendererControlPlane,
    pub discovery: AdminRendererDiscoveryMode,
    pub media_transport: AdminRendererMediaTransport,
    pub transport_auth: PlaybackTargetTransportAuth,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererAdapterKind {
    NakoRemoteClient,
    NakoRemoteClientCastSafeTransport,
    Chromecast,
    DlnaRenderer,
    Airplay,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererAdapterStatus {
    Ready,
    Planned,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererAdapterReason {
    NakoRemoteClientReady,
    CastSafeTransportReady,
    CastSafeTransportPending,
    ChromecastAdapterPlanned,
    DlnaAdapterPlanned,
    AirplayAdapterPlanned,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererControlPlane {
    PublicClientPolling,
    AdapterProcess,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererDiscoveryMode {
    ClientRegistration,
    LocalNetworkDiscovery,
    PlatformDiscovery,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRendererMediaTransport {
    AuthenticatedNakoClientStream,
    CastSafeUrl,
    NativeProtocolStream,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackPolicyDiagnostics {
    pub user_policy_rows_supported: bool,
    pub role_policy_rows_supported: bool,
    pub effective_resolution_supported: bool,
    pub library_access_required: bool,
    pub user_policy_overrides_role_policy: bool,
    pub role_policy_merge: AdminPlaybackPolicyRoleMergeStrategy,
    pub permissions: Vec<PlaybackPermission>,
}

impl AdminPlaybackPolicyDiagnostics {
    #[must_use]
    pub fn ready() -> Self {
        Self {
            user_policy_rows_supported: true,
            role_policy_rows_supported: true,
            effective_resolution_supported: true,
            library_access_required: true,
            user_policy_overrides_role_policy: true,
            role_policy_merge: AdminPlaybackPolicyRoleMergeStrategy::Restrictive,
            permissions: PlaybackPermission::ALL.to_vec(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackPolicyRoleMergeStrategy {
    Restrictive,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRuntimeSettingsPayload {
    pub hardware_acceleration: AdminHardwareAcceleration,
    pub hardware_fallback: AdminHardwareAccelerationFallback,
    pub cpu_concurrency: u32,
    pub gpu_concurrency: u32,
    pub remux_concurrency: u32,
    pub remux_timeout_ms: u64,
    pub remote_stream_concurrency: u32,
    pub remote_stage_concurrency: u32,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminUpdatePlaybackRuntimeSettingsRequest {
    pub settings: AdminPlaybackRuntimeSettingsPayload,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRuntimeSettingsResponse {
    pub admin_api_version: String,
    pub settings: AdminPlaybackRuntimeSettingsPayload,
    pub source: nako_core::AdminSettingsSource,
    pub effect: nako_core::AdminSettingsEffect,
    pub updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportEvidenceResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub subject: AdminPlaybackSupportSubject,
    pub session: Option<AdminPlaybackSupportSessionEvidence>,
    pub source: Option<AdminPlaybackSupportSourceEvidence>,
    pub runtime: AdminPlaybackSupportRuntimeEvidence,
    pub redaction: AdminPlaybackSupportRedactionEvidence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportSubject {
    pub session_id: Option<TranscodeSessionId>,
    pub source_id: Option<MediaSourceId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportSessionEvidence {
    pub id: TranscodeSessionId,
    pub source_id: MediaSourceId,
    pub kind: TranscodeSessionKind,
    pub state: TranscodeSessionState,
    pub failure_category: Option<TranscodeFailureCategory>,
    pub has_failure_message: bool,
    pub active: bool,
    pub terminal: bool,
    pub request_key_fingerprint: String,
    pub output_artifact_kind: AdminPlaybackOutputArtifactKind,
    pub runtime_metrics: TranscodeSessionRuntimeMetrics,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl AdminPlaybackSupportSessionEvidence {
    #[must_use]
    pub fn from_record(session: TranscodeSessionRecord) -> Self {
        Self {
            id: session.id,
            source_id: session.source_id,
            kind: session.kind,
            state: session.state,
            failure_category: session.failure_category,
            has_failure_message: session.failure_message.is_some(),
            active: session.state.is_active(),
            terminal: session.state.is_terminal(),
            request_key_fingerprint: stable_fingerprint(&session.request_key),
            output_artifact_kind: AdminPlaybackOutputArtifactKind::from_session_kind(session.kind),
            runtime_metrics: session.runtime_metrics,
            created_at: session.created_at,
            updated_at: session.updated_at,
            started_at: session.started_at,
            completed_at: session.completed_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportSourceEvidence {
    pub source_id: MediaSourceId,
    pub library_id: LibraryId,
    pub item_id: MediaItemId,
    pub source_scheme: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub has_fingerprint: bool,
}

impl AdminPlaybackSupportSourceEvidence {
    #[must_use]
    pub fn from_record(source: MediaSource) -> Self {
        Self {
            source_id: source.id,
            library_id: source.library_id,
            item_id: source.item_id,
            source_scheme: storage_scheme(&source.locator),
            file_name: source.file_name,
            size_bytes: source.size_bytes,
            has_fingerprint: source.fingerprint.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackOutputArtifactKind {
    RemuxFile,
    HlsPlaylist,
}

impl AdminPlaybackOutputArtifactKind {
    const fn from_session_kind(kind: TranscodeSessionKind) -> Self {
        match kind {
            TranscodeSessionKind::Remux => Self::RemuxFile,
            TranscodeSessionKind::HlsTranscode => Self::HlsPlaylist,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportRuntimeEvidence {
    pub readiness: AdminPlaybackReadinessDiagnostics,
    pub policy: AdminPlaybackPolicyDiagnostics,
    pub ffmpeg: AdminPlaybackFfmpegDiagnostics,
    pub hardware: AdminPlaybackSupportHardwareEvidence,
    pub transcode: AdminPlaybackTranscodeBudgetDiagnostics,
    pub remux: AdminPlaybackRemuxRuntimeDiagnostics,
    pub remote_playback: AdminPlaybackRemoteBudgetDiagnostics,
    pub staging: AdminPlaybackStagingDiagnostics,
    pub artifact_lifecycle: AdminPlaybackArtifactLifecycleDiagnostics,
    pub throttle: AdminPlaybackThrottleDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportHardwareEvidence {
    pub policy: AdminHardwareAccelerationPolicy,
    pub selected_acceleration: AdminHardwareAcceleration,
    pub fallback_used: bool,
    pub capability_count: u32,
    pub unavailable_capabilities: Vec<AdminPlaybackSupportHardwareCapabilityEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportHardwareCapabilityEvidence {
    pub accelerator: AdminHardwareAcceleration,
    pub reason_code: AdminPlaybackHardwareCapabilityReason,
    pub encoder_discovery_status: AdminPlaybackHardwareEncoderDiscoveryStatus,
    pub device_initialization_status: AdminPlaybackHardwareDeviceInitializationStatus,
    pub smoke_probe_status: AdminPlaybackHardwareSmokeProbeStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportRedactionEvidence {
    pub paths_redacted: bool,
    pub source_references_redacted: bool,
    pub ffmpeg_commands_redacted: bool,
    pub stderr_redacted: bool,
    pub credentials_redacted: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackReadinessDiagnostics {
    pub status: AdminPlaybackReadinessStatus,
    pub reason: AdminPlaybackReadinessReason,
    pub checks: Vec<AdminPlaybackReadinessCheck>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AdminTranscodePipelineReadiness {
    pub status: AdminTranscodePipelineReadinessStatus,
    pub reason: AdminTranscodePipelineReadinessReason,
    pub requested: AdminHardwareAcceleration,
    pub selected: AdminHardwareAcceleration,
    pub fallback_used: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminTranscodePipelineReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminTranscodePipelineReadinessReason {
    CpuRequested,
    RequestedPipelineReady,
    RequestedPipelineUnavailableFallbackToCpu,
    RequestedPipelineUnavailableFailPolicy,
    SoftwarePipelineUnavailable,
    CpuFallbackUnavailable,
    ProbeError,
    DeviceInitializationFailed,
    SmokeProbeFailed,
    SourceVideoCodecUnsupportedByRequestedPipeline,
    SourceVideoBitDepthUnsupportedByRequestedPipeline,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

impl From<AdminTranscodePipelineReadinessStatus> for AdminPlaybackReadinessStatus {
    fn from(status: AdminTranscodePipelineReadinessStatus) -> Self {
        match status {
            AdminTranscodePipelineReadinessStatus::Ready => Self::Ready,
            AdminTranscodePipelineReadinessStatus::Degraded => Self::Degraded,
            AdminTranscodePipelineReadinessStatus::Unavailable => Self::Unavailable,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackReadinessReason {
    Ready,
    FfmpegProbeReady,
    CpuRequested,
    RequestedAcceleratorReady,
    RequestedAcceleratorUnavailableFallbackToCpu,
    RequestedAcceleratorUnavailableFailPolicy,
    SoftwarePipelineUnavailable,
    CpuFallbackUnavailable,
    ProbeError,
    DeviceInitializationFailed,
    SmokeProbeFailed,
    SourceVideoCodecUnsupportedByRequestedPipeline,
    SourceVideoBitDepthUnsupportedByRequestedPipeline,
    SelectedAccelerationReady,
    CpuFallbackActive,
    TranscodeBudgetReady,
    TranscodeBudgetClamped,
    RemotePlaybackBudgetReady,
    RemotePlaybackBudgetClamped,
    StagingReady,
    StagingBudgetDisabled,
    ArtifactLifecycleReady,
    TranscodeThrottleReady,
    PlaybackPolicyReady,
}

impl From<AdminTranscodePipelineReadinessReason> for AdminPlaybackReadinessReason {
    fn from(reason: AdminTranscodePipelineReadinessReason) -> Self {
        match reason {
            AdminTranscodePipelineReadinessReason::CpuRequested => Self::CpuRequested,
            AdminTranscodePipelineReadinessReason::RequestedPipelineReady => {
                Self::RequestedAcceleratorReady
            }
            AdminTranscodePipelineReadinessReason::RequestedPipelineUnavailableFallbackToCpu => {
                Self::RequestedAcceleratorUnavailableFallbackToCpu
            }
            AdminTranscodePipelineReadinessReason::RequestedPipelineUnavailableFailPolicy => {
                Self::RequestedAcceleratorUnavailableFailPolicy
            }
            AdminTranscodePipelineReadinessReason::SoftwarePipelineUnavailable => {
                Self::SoftwarePipelineUnavailable
            }
            AdminTranscodePipelineReadinessReason::CpuFallbackUnavailable => {
                Self::CpuFallbackUnavailable
            }
            AdminTranscodePipelineReadinessReason::ProbeError => Self::ProbeError,
            AdminTranscodePipelineReadinessReason::DeviceInitializationFailed => {
                Self::DeviceInitializationFailed
            }
            AdminTranscodePipelineReadinessReason::SmokeProbeFailed => Self::SmokeProbeFailed,
            AdminTranscodePipelineReadinessReason::SourceVideoCodecUnsupportedByRequestedPipeline => {
                Self::SourceVideoCodecUnsupportedByRequestedPipeline
            }
            AdminTranscodePipelineReadinessReason::SourceVideoBitDepthUnsupportedByRequestedPipeline => {
                Self::SourceVideoBitDepthUnsupportedByRequestedPipeline
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackReadinessCheck {
    pub name: AdminPlaybackReadinessCheckName,
    pub status: AdminPlaybackReadinessStatus,
    pub reason: AdminPlaybackReadinessReason,
}

impl AdminPlaybackReadinessCheck {
    #[must_use]
    pub const fn ready(
        name: AdminPlaybackReadinessCheckName,
        reason: AdminPlaybackReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminPlaybackReadinessStatus::Ready,
            reason,
        }
    }

    #[must_use]
    pub const fn degraded(
        name: AdminPlaybackReadinessCheckName,
        reason: AdminPlaybackReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminPlaybackReadinessStatus::Degraded,
            reason,
        }
    }

    #[must_use]
    pub const fn unavailable(
        name: AdminPlaybackReadinessCheckName,
        reason: AdminPlaybackReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminPlaybackReadinessStatus::Unavailable,
            reason,
        }
    }

    #[must_use]
    pub fn from_hardware(readiness: AdminTranscodePipelineReadiness) -> Self {
        Self {
            name: AdminPlaybackReadinessCheckName::HardwareAcceleration,
            status: readiness.status.into(),
            reason: readiness.reason.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackReadinessCheckName {
    FfmpegProbe,
    HardwareAcceleration,
    SelectedFallback,
    TranscodeBudget,
    RemotePlaybackBudget,
    PlaybackPolicy,
    Staging,
    ArtifactLifecycle,
    TranscodeThrottle,
}

impl AdminPlaybackReadinessDiagnostics {
    #[must_use]
    pub fn from_checks(checks: Vec<AdminPlaybackReadinessCheck>) -> Self {
        let status = if checks
            .iter()
            .any(|check| check.status == AdminPlaybackReadinessStatus::Unavailable)
        {
            AdminPlaybackReadinessStatus::Unavailable
        } else if checks
            .iter()
            .any(|check| check.status == AdminPlaybackReadinessStatus::Degraded)
        {
            AdminPlaybackReadinessStatus::Degraded
        } else {
            AdminPlaybackReadinessStatus::Ready
        };
        let reason = checks
            .iter()
            .find(|check| check.status == status)
            .map_or(AdminPlaybackReadinessReason::Ready, |check| check.reason);

        Self {
            status,
            reason,
            checks,
        }
    }

    #[must_use]
    pub fn from_hardware(readiness: AdminTranscodePipelineReadiness) -> Self {
        Self::from_checks(vec![AdminPlaybackReadinessCheck::from_hardware(readiness)])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackFfmpegDiagnostics {
    pub probe_status: AdminPlaybackRuntimeStatus,
    pub has_probe_error: bool,
    pub hardware_capability_count: u32,
    pub available_gpu_capabilities: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackRuntimeStatus {
    Ready,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareDiagnostics {
    pub policy: AdminHardwareAccelerationPolicy,
    pub pipeline: AdminTranscodePipelineReadiness,
    pub capabilities: Vec<AdminPlaybackHardwareCapability>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareCapability {
    pub accelerator: AdminHardwareAcceleration,
    pub available: bool,
    pub reason_code: AdminPlaybackHardwareCapabilityReason,
    pub stage_capabilities: Vec<AdminPlaybackHardwareStageCapability>,
    pub encoder_discovery: AdminPlaybackHardwareEncoderDiscovery,
    pub device_initialization: AdminPlaybackHardwareDeviceInitialization,
    pub smoke_probe: AdminPlaybackHardwareSmokeProbe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareStageCapability {
    pub stage: AdminHardwarePipelineStage,
    pub available: bool,
    pub required: bool,
    pub feature: Option<String>,
    pub discovery_status: AdminPlaybackHardwareEncoderDiscoveryStatus,
    pub has_detail: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminHardwarePipelineStage {
    Decode,
    Filter,
    Encode,
    Hwaccel,
    BitstreamFilter,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareCapabilityReason {
    Available,
    EncoderNotListed,
    DeviceInitializationFailed,
    SmokeProbeFailed,
    ProbeError,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareSmokeProbe {
    pub status: AdminPlaybackHardwareSmokeProbeStatus,
    pub operator_check: String,
    pub has_detail: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareSmokeProbeStatus {
    NotRequired,
    NotRun,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareEncoderDiscovery {
    pub status: AdminPlaybackHardwareEncoderDiscoveryStatus,
    pub encoder: Option<String>,
    pub has_detail: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareEncoderDiscoveryStatus {
    NotRequired,
    Listed,
    Missing,
    ProbeError,
    Static,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareDeviceInitialization {
    pub status: AdminPlaybackHardwareDeviceInitializationStatus,
    pub operator_check: String,
    pub has_detail: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackHardwareDeviceInitializationStatus {
    NotRequired,
    NotRun,
    Passed,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackTranscodeBudgetDiagnostics {
    pub configured_cpu_slots: usize,
    pub configured_gpu_slots: usize,
    pub effective_cpu_slots: usize,
    pub effective_gpu_slots: usize,
    pub selected_hls_slots: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRemuxRuntimeDiagnostics {
    pub max_concurrent_sessions: usize,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackRemoteBudgetDiagnostics {
    pub backend_count: u32,
    pub stream_permits_available: usize,
    pub stream_permits_max: usize,
    pub stage_permits_available: usize,
    pub stage_permits_max: usize,
    pub state_scope: StorageBackendRuntimeStateScope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackStagingDiagnostics {
    pub max_bytes: u64,
    pub retention_ms: u64,
    pub cleanup_on_startup: bool,
    pub startup_deleted_records: u32,
    pub startup_deleted_files: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackArtifactLifecycleDiagnostics {
    pub transcode_artifact_retention_ms: u64,
    pub transcode_artifact_cleanup_on_startup: bool,
    pub hls_segment_cleanup_enabled: bool,
    pub hls_segment_keep_ms: u64,
    pub startup_examined_artifacts: u32,
    pub startup_deleted_artifacts: u32,
    pub startup_deleted_files: u32,
    pub startup_deleted_directories: u32,
    pub startup_deleted_bytes: u64,
    pub startup_skipped_security: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackThrottleDiagnostics {
    pub enabled: bool,
    pub delay_ms: u64,
}

fn stable_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());

    format!("sha256:{}", lowercase_hex(&digest[..16]))
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn storage_scheme(reference: &str) -> String {
    reference
        .split_once("://")
        .map_or("unknown", |(scheme, _path)| {
            if scheme.trim().is_empty() {
                "unknown"
            } else {
                scheme
            }
        })
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use crate::{admin::ADMIN_API_VERSION, public_client::API_VERSION};

    use super::*;

    #[test]
    fn admin_playback_session_list_item_summarizes_playback_not_transcode_artifacts() {
        let session = PlaybackSessionRecord {
            id: PlaybackSessionId::new(),
            source_id: MediaSourceId::new(),
            item_id: MediaItemId::new(),
            principal_id: nako_core::UserPrincipalId::local_admin(),
            mode: PlaybackSessionMode::Direct,
            state: PlaybackSessionState::Active,
            client_capabilities_json: Some(
                r#"{"direct_play":true,"container":["mp4"],"video_codec":["h264"]}"#.to_owned(),
            ),
            transcode_session_id: None,
            position_ms: Some(42_000),
            duration_ms: Some(600_000),
            last_heartbeat_at_ms: Some(1_779_814_401_000),
            started_at_ms: 1_779_814_400_000,
            ended_at_ms: None,
            created_at: "2026-05-18T00:00:00Z".to_owned(),
            updated_at: "2026-05-18T00:00:01Z".to_owned(),
        };
        let session_id = session.id;
        let source_id = session.source_id;

        let item = AdminPlaybackSessionListItem::from_record(session);
        let body = serde_json::to_string(&item).unwrap();

        assert_eq!(item.id, session_id);
        assert_eq!(item.source_id, source_id);
        assert_eq!(item.mode, PlaybackSessionMode::Direct);
        assert_eq!(item.state, PlaybackSessionState::Active);
        assert!(item.has_client_capabilities);
        assert!(item.active);
        assert!(!item.terminal);
        assert!(!body.contains("nako-cache"));
        assert!(!body.contains("playlist.m3u8"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("request_key"));
        assert!(!body.contains("ffmpeg failed while writing"));
    }

    #[test]
    fn admin_playback_support_evidence_redacts_session_secrets_but_keeps_support_facts() {
        let source_id = MediaSourceId::new();
        let request_key =
            "transcode-request:v1;source=source-revision:v1;digest=demo;profile=secret-profile"
                .to_owned();
        let session = TranscodeSessionRecord {
            id: TranscodeSessionId::new(),
            source_id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: request_key.clone(),
            output_path: "C:\\nako-cache\\hls\\secret\\playlist.m3u8".into(),
            state: TranscodeSessionState::Failed,
            failure_category: Some(TranscodeFailureCategory::Runner),
            failure_message: Some(
                "ffmpeg failed while writing C:\\nako-cache\\hls\\secret\\playlist.m3u8".to_owned(),
            ),
            runtime_metrics: Default::default(),
            created_at: "2026-05-18T00:00:00Z".to_owned(),
            updated_at: "2026-05-18T00:00:01Z".to_owned(),
            started_at: Some("2026-05-18T00:00:00Z".to_owned()),
            completed_at: Some("2026-05-18T00:00:01Z".to_owned()),
        };

        let evidence = AdminPlaybackSupportSessionEvidence::from_record(session);
        let body = serde_json::to_string(&evidence).unwrap();

        assert_eq!(evidence.source_id, source_id);
        assert_eq!(evidence.kind, TranscodeSessionKind::HlsTranscode);
        assert_eq!(
            evidence.failure_category,
            Some(TranscodeFailureCategory::Runner)
        );
        assert!(evidence.has_failure_message);
        assert_eq!(
            evidence.output_artifact_kind,
            AdminPlaybackOutputArtifactKind::HlsPlaylist
        );
        assert!(evidence.request_key_fingerprint.starts_with("sha256:"));
        assert_ne!(evidence.request_key_fingerprint, request_key);
        assert!(!body.contains("secret-profile"));
        assert!(!body.contains("transcode-request"));
        assert!(!body.contains("nako-cache"));
        assert!(!body.contains("playlist.m3u8"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("ffmpeg failed while writing"));
    }

    #[test]
    fn admin_playback_support_source_evidence_keeps_scheme_not_locator() {
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            locator: "webdav:///Movies/Private/Secret Demo.mkv?token=admin-token".to_owned(),
            file_name: "Secret Demo.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: Some("sha256:private-fingerprint".to_owned()),
        };

        let evidence = AdminPlaybackSupportSourceEvidence::from_record(source);
        let body = serde_json::to_string(&evidence).unwrap();

        assert_eq!(evidence.source_scheme, "webdav");
        assert_eq!(evidence.file_name, "Secret Demo.mkv");
        assert_eq!(evidence.size_bytes, Some(42));
        assert!(evidence.has_fingerprint);
        assert!(!body.contains("locator"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private-fingerprint"));
    }

    #[test]
    fn admin_playback_runtime_diagnostics_serializes_safe_summary_fields() {
        let response = AdminPlaybackRuntimeDiagnosticsResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            readiness: AdminPlaybackReadinessDiagnostics::from_checks(vec![
                AdminPlaybackReadinessCheck::degraded(
                    AdminPlaybackReadinessCheckName::FfmpegProbe,
                    AdminPlaybackReadinessReason::ProbeError,
                ),
                AdminPlaybackReadinessCheck::ready(
                    AdminPlaybackReadinessCheckName::PlaybackPolicy,
                    AdminPlaybackReadinessReason::PlaybackPolicyReady,
                ),
                AdminPlaybackReadinessCheck::ready(
                    AdminPlaybackReadinessCheckName::TranscodeBudget,
                    AdminPlaybackReadinessReason::TranscodeBudgetReady,
                ),
            ]),
            policy: AdminPlaybackPolicyDiagnostics::ready(),
            ffmpeg: AdminPlaybackFfmpegDiagnostics {
                probe_status: AdminPlaybackRuntimeStatus::Degraded,
                has_probe_error: true,
                hardware_capability_count: 6,
                available_gpu_capabilities: 1,
            },
            hardware: AdminPlaybackHardwareDiagnostics {
                policy: AdminHardwareAccelerationPolicy {
                    requested: AdminHardwareAcceleration::Nvenc,
                    fallback: AdminHardwareAccelerationFallback::Cpu,
                },
                pipeline: AdminTranscodePipelineReadiness {
                    status: AdminTranscodePipelineReadinessStatus::Degraded,
                    reason: AdminTranscodePipelineReadinessReason::ProbeError,
                    requested: AdminHardwareAcceleration::Nvenc,
                    selected: AdminHardwareAcceleration::None,
                    fallback_used: true,
                },
                capabilities: vec![AdminPlaybackHardwareCapability {
                    accelerator: AdminHardwareAcceleration::Nvenc,
                    available: false,
                    reason_code: AdminPlaybackHardwareCapabilityReason::ProbeError,
                    stage_capabilities: vec![AdminPlaybackHardwareStageCapability {
                        stage: AdminHardwarePipelineStage::Encode,
                        available: false,
                        required: true,
                        feature: None,
                        discovery_status: AdminPlaybackHardwareEncoderDiscoveryStatus::ProbeError,
                        has_detail: true,
                    }],
                    encoder_discovery: AdminPlaybackHardwareEncoderDiscovery {
                        status: AdminPlaybackHardwareEncoderDiscoveryStatus::ProbeError,
                        encoder: None,
                        has_detail: true,
                    },
                    device_initialization: AdminPlaybackHardwareDeviceInitialization {
                        status: AdminPlaybackHardwareDeviceInitializationStatus::NotRun,
                        operator_check: "Verify the NVIDIA driver and FFmpeg can initialize NVENC"
                            .to_owned(),
                        has_detail: false,
                    },
                    smoke_probe: AdminPlaybackHardwareSmokeProbe {
                        status: AdminPlaybackHardwareSmokeProbeStatus::NotRun,
                        operator_check: "Run an NVENC H.264 encode smoke test on the host"
                            .to_owned(),
                        has_detail: false,
                    },
                }],
            },
            transcode: AdminPlaybackTranscodeBudgetDiagnostics {
                configured_cpu_slots: 0,
                configured_gpu_slots: 2,
                effective_cpu_slots: 1,
                effective_gpu_slots: 2,
                selected_hls_slots: 1,
            },
            remux: AdminPlaybackRemuxRuntimeDiagnostics {
                max_concurrent_sessions: 1,
                timeout_ms: 30_000,
            },
            remote_playback: AdminPlaybackRemoteBudgetDiagnostics {
                backend_count: 1,
                stream_permits_available: 8,
                stream_permits_max: 8,
                stage_permits_available: 2,
                stage_permits_max: 2,
                state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
            },
            staging: AdminPlaybackStagingDiagnostics {
                max_bytes: 100,
                retention_ms: 200,
                cleanup_on_startup: true,
                startup_deleted_records: 1,
                startup_deleted_files: 1,
            },
            artifact_lifecycle: AdminPlaybackArtifactLifecycleDiagnostics {
                transcode_artifact_retention_ms: 300,
                transcode_artifact_cleanup_on_startup: true,
                hls_segment_cleanup_enabled: true,
                hls_segment_keep_ms: 60_000,
                startup_examined_artifacts: 3,
                startup_deleted_artifacts: 2,
                startup_deleted_files: 4,
                startup_deleted_directories: 1,
                startup_deleted_bytes: 128,
                startup_skipped_security: 0,
            },
            throttle: AdminPlaybackThrottleDiagnostics {
                enabled: true,
                delay_ms: 3_000,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["readiness"]["status"], "degraded");
        assert_eq!(value["readiness"]["reason"], "probe_error");
        assert_eq!(value["readiness"]["checks"][0]["name"], "ffmpeg_probe");
        assert_eq!(value["readiness"]["checks"][0]["status"], "degraded");
        assert_eq!(value["readiness"]["checks"][1]["name"], "playback_policy");
        assert_eq!(value["readiness"]["checks"][2]["name"], "transcode_budget");
        assert_eq!(value["policy"]["user_policy_rows_supported"], true);
        assert_eq!(value["policy"]["role_policy_merge"], "restrictive");
        assert_eq!(value["policy"]["permissions"][2], "remux");
        assert_eq!(value["ffmpeg"]["probe_status"], "degraded");
        assert_eq!(value["hardware"]["policy"]["requested"], "nvenc");
        assert_eq!(value["hardware"]["pipeline"]["selected"], "none");
        assert_eq!(
            value["hardware"]["capabilities"][0]["reason_code"],
            "probe_error"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["stage_capabilities"][0]["stage"],
            "encode"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["encoder_discovery"]["status"],
            "probe_error"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["device_initialization"]["status"],
            "not_run"
        );
        assert_eq!(
            value["hardware"]["capabilities"][0]["smoke_probe"]["status"],
            "not_run"
        );
        assert_eq!(value["remote_playback"]["state_scope"], "process_local");
        assert_eq!(
            value["artifact_lifecycle"]["transcode_artifact_retention_ms"],
            300
        );
        assert_eq!(value["throttle"]["delay_ms"], 3_000);
        assert!(!body.contains("ffmpeg_path"));
        assert!(!body.contains("remux_staging_root"));
        assert!(!body.contains("nako-cache"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("secret"));
        assert!(!body.contains("token"));
    }

    #[test]
    fn admin_renderer_runtime_diagnostics_serializes_adapter_state_without_secrets() {
        let response = AdminRendererRuntimeDiagnosticsResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            readiness: AdminRendererReadinessDiagnostics::ready(),
            summary: AdminRendererSessionSummary {
                returned_sessions: 1,
                online_sessions: 1,
                offline_sessions: 0,
                revoked_sessions: 0,
                expired_sessions: 0,
                active_playback_sessions: 0,
            },
            adapters: vec![
                AdminRendererAdapterDiagnostics {
                    adapter: AdminRendererAdapterKind::NakoRemoteClient,
                    target_kind: PlaybackTargetKind::NakoRemoteClient,
                    status: AdminRendererAdapterStatus::Ready,
                    reason: AdminRendererAdapterReason::NakoRemoteClientReady,
                    control_plane: AdminRendererControlPlane::PublicClientPolling,
                    discovery: AdminRendererDiscoveryMode::ClientRegistration,
                    media_transport: AdminRendererMediaTransport::AuthenticatedNakoClientStream,
                    transport_auth: PlaybackTargetTransportAuth::Bearer,
                },
                AdminRendererAdapterDiagnostics {
                    adapter: AdminRendererAdapterKind::NakoRemoteClientCastSafeTransport,
                    target_kind: PlaybackTargetKind::NakoRemoteClient,
                    status: AdminRendererAdapterStatus::Ready,
                    reason: AdminRendererAdapterReason::CastSafeTransportReady,
                    control_plane: AdminRendererControlPlane::PublicClientPolling,
                    discovery: AdminRendererDiscoveryMode::ClientRegistration,
                    media_transport: AdminRendererMediaTransport::CastSafeUrl,
                    transport_auth: PlaybackTargetTransportAuth::CastTicket,
                },
            ],
            sessions: vec![AdminRendererSessionDiagnostics {
                id: RendererSessionId::new(),
                target_kind: PlaybackTargetKind::NakoRemoteClient,
                display_name: "Living Room Desktop".to_owned(),
                network_scope: PlaybackTargetNetworkScope::Local,
                transport_auth: PlaybackTargetTransportAuth::Bearer,
                state: RendererSessionState::Online,
                active_playback_session_id: None,
                supported_commands: vec![
                    RendererControlCommand::Play,
                    RendererControlCommand::Seek,
                ],
                has_media_capabilities: true,
                direct_play_supported: true,
                expired: false,
                last_seen_at_ms: 1_779_814_400_000,
                expires_at_ms: Some(1_779_814_520_000),
                created_at: "2026-05-27T00:00:00Z".to_owned(),
                updated_at: "2026-05-27T00:00:01Z".to_owned(),
            }],
            page: PageInfo {
                limit: 50,
                offset: 0,
                returned: 1,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["readiness"]["status"], "ready");
        assert_eq!(
            value["adapters"][0]["media_transport"],
            "authenticated_nako_client_stream"
        );
        assert_eq!(value["adapters"][1]["status"], "ready");
        assert_eq!(value["adapters"][1]["reason"], "cast_safe_transport_ready");
        assert_eq!(
            value["sessions"][0]["supported_commands"],
            serde_json::json!(["play", "seek"])
        );
        assert_eq!(value["sessions"][0]["direct_play_supported"], true);
        for forbidden in [
            "principal",
            "payload_json",
            "media_capabilities_json",
            "source_locator",
            "local_path",
            "bearer_token",
            "access_token",
            "token_value",
        ] {
            assert!(
                !body.contains(forbidden),
                "renderer diagnostics leaked forbidden term: {forbidden}"
            );
        }
    }
}
