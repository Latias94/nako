use nako_client_protocol::PageInfo;
use nako_core::{
    LibraryId, MediaItemId, MediaSource, MediaSourceId, PlaybackSessionId, PlaybackSessionMode,
    PlaybackSessionRecord, PlaybackSessionState, TranscodeFailureCategory, TranscodeSessionId,
    TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionState,
};
use nako_transcode::{
    HardwareAcceleration, HardwareAccelerationPolicy, HardwareAccelerationReadiness,
    HardwareAccelerationReadinessReason, HardwareAccelerationReadinessStatus,
    HardwareAccelerationSelection,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::StorageBackendRuntimeStateScope;

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
    pub ffmpeg: AdminPlaybackFfmpegDiagnostics,
    pub hardware: AdminPlaybackHardwareDiagnostics,
    pub transcode: AdminPlaybackTranscodeBudgetDiagnostics,
    pub remux: AdminPlaybackRemuxRuntimeDiagnostics,
    pub remote_playback: AdminPlaybackRemoteBudgetDiagnostics,
    pub staging: AdminPlaybackStagingDiagnostics,
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
    pub ffmpeg: AdminPlaybackFfmpegDiagnostics,
    pub hardware: AdminPlaybackSupportHardwareEvidence,
    pub transcode: AdminPlaybackTranscodeBudgetDiagnostics,
    pub remux: AdminPlaybackRemuxRuntimeDiagnostics,
    pub remote_playback: AdminPlaybackRemoteBudgetDiagnostics,
    pub staging: AdminPlaybackStagingDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportHardwareEvidence {
    pub policy: HardwareAccelerationPolicy,
    pub selected_acceleration: HardwareAcceleration,
    pub fallback_used: bool,
    pub capability_count: u32,
    pub unavailable_capabilities: Vec<AdminPlaybackSupportHardwareCapabilityEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackSupportHardwareCapabilityEvidence {
    pub accelerator: HardwareAcceleration,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminPlaybackReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

impl From<HardwareAccelerationReadinessStatus> for AdminPlaybackReadinessStatus {
    fn from(status: HardwareAccelerationReadinessStatus) -> Self {
        match status {
            HardwareAccelerationReadinessStatus::Ready => Self::Ready,
            HardwareAccelerationReadinessStatus::Degraded => Self::Degraded,
            HardwareAccelerationReadinessStatus::Unavailable => Self::Unavailable,
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
    ProbeError,
    DeviceInitializationFailed,
    SmokeProbeFailed,
    SelectedAccelerationReady,
    CpuFallbackActive,
    TranscodeBudgetReady,
    TranscodeBudgetClamped,
    RemotePlaybackBudgetReady,
    RemotePlaybackBudgetClamped,
    StagingReady,
    StagingBudgetDisabled,
}

impl From<HardwareAccelerationReadinessReason> for AdminPlaybackReadinessReason {
    fn from(reason: HardwareAccelerationReadinessReason) -> Self {
        match reason {
            HardwareAccelerationReadinessReason::CpuRequested => Self::CpuRequested,
            HardwareAccelerationReadinessReason::RequestedAcceleratorReady => {
                Self::RequestedAcceleratorReady
            }
            HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFallbackToCpu => {
                Self::RequestedAcceleratorUnavailableFallbackToCpu
            }
            HardwareAccelerationReadinessReason::RequestedAcceleratorUnavailableFailPolicy => {
                Self::RequestedAcceleratorUnavailableFailPolicy
            }
            HardwareAccelerationReadinessReason::ProbeError => Self::ProbeError,
            HardwareAccelerationReadinessReason::DeviceInitializationFailed => {
                Self::DeviceInitializationFailed
            }
            HardwareAccelerationReadinessReason::SmokeProbeFailed => Self::SmokeProbeFailed,
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
    pub fn from_hardware(readiness: HardwareAccelerationReadiness) -> Self {
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
    Staging,
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
    pub fn from_hardware(readiness: HardwareAccelerationReadiness) -> Self {
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
    pub policy: HardwareAccelerationPolicy,
    pub selection: HardwareAccelerationSelection,
    pub capabilities: Vec<AdminPlaybackHardwareCapability>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackHardwareCapability {
    pub accelerator: HardwareAcceleration,
    pub available: bool,
    pub reason_code: AdminPlaybackHardwareCapabilityReason,
    pub encoder_discovery: AdminPlaybackHardwareEncoderDiscovery,
    pub device_initialization: AdminPlaybackHardwareDeviceInitialization,
    pub smoke_probe: AdminPlaybackHardwareSmokeProbe,
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
                    AdminPlaybackReadinessCheckName::TranscodeBudget,
                    AdminPlaybackReadinessReason::TranscodeBudgetReady,
                ),
            ]),
            ffmpeg: AdminPlaybackFfmpegDiagnostics {
                probe_status: AdminPlaybackRuntimeStatus::Degraded,
                has_probe_error: true,
                hardware_capability_count: 4,
                available_gpu_capabilities: 1,
            },
            hardware: AdminPlaybackHardwareDiagnostics {
                policy: HardwareAccelerationPolicy {
                    requested: HardwareAcceleration::Nvenc,
                    fallback: nako_transcode::HardwareAccelerationFallback::Cpu,
                },
                selection: HardwareAccelerationSelection {
                    acceleration: HardwareAcceleration::None,
                    fallback_used: true,
                    reason: "nvenc is unavailable; falling back to cpu".to_owned(),
                },
                capabilities: vec![AdminPlaybackHardwareCapability {
                    accelerator: HardwareAcceleration::Nvenc,
                    available: false,
                    reason_code: AdminPlaybackHardwareCapabilityReason::ProbeError,
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
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["readiness"]["status"], "degraded");
        assert_eq!(value["readiness"]["reason"], "probe_error");
        assert_eq!(value["readiness"]["checks"][0]["name"], "ffmpeg_probe");
        assert_eq!(value["readiness"]["checks"][0]["status"], "degraded");
        assert_eq!(value["readiness"]["checks"][1]["name"], "transcode_budget");
        assert_eq!(value["ffmpeg"]["probe_status"], "degraded");
        assert_eq!(value["hardware"]["policy"]["requested"], "nvenc");
        assert_eq!(value["hardware"]["selection"]["acceleration"], "none");
        assert_eq!(
            value["hardware"]["capabilities"][0]["reason_code"],
            "probe_error"
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
        assert!(!body.contains("ffmpeg_path"));
        assert!(!body.contains("remux_staging_root"));
        assert!(!body.contains("nako-cache"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("secret"));
        assert!(!body.contains("token"));
    }
}
