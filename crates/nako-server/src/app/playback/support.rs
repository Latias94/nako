use nako_core::{
    MediaSource, MediaSourceId, NakoError, Result, TranscodeSessionId, TranscodeSessionRecord,
};
use nako_transcode::{
    HardwareAccelerationPolicy, HardwareAccelerationReport, TranscodePipelineReadiness,
    TranscodeResourceBudget, TranscodeRuntimeInventory,
};

use crate::config::NakoServerConfig;

use super::{
    PlaybackRuntimeAdmission, PlaybackRuntimeResourcePressure, PlaybackRuntimeStore,
    hls::HlsAppService,
};

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
    pub resource_pressure: PlaybackRuntimeResourcePressure,
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

pub(super) async fn support_evidence_context(
    store: &dyn PlaybackRuntimeStore,
    request: PlaybackSupportEvidenceRequest,
) -> Result<PlaybackSupportEvidenceContext> {
    let session = match request.session_id {
        Some(session_id) => Some(get_transcode_session_or_not_found(store, session_id).await?),
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
        Some(source_id) => Some(get_source_or_not_found(store, source_id).await?),
        None => None,
    };

    Ok(PlaybackSupportEvidenceContext { session, source })
}

pub(super) fn runtime_diagnostics(
    config: &NakoServerConfig,
    hls: &HlsAppService,
    resource_admission: &PlaybackRuntimeAdmission,
) -> PlaybackRuntimeDiagnostics {
    let hardware_policy = config.transcode.hardware_policy();
    let transcode_budget = config.transcode.resource_budget();

    PlaybackRuntimeDiagnostics {
        runtime_inventory: TranscodeRuntimeInventory::ffmpeg_cli(&hls.hardware_report),
        hardware_policy,
        hardware_report: hls.hardware_report.clone(),
        hls_pipeline_readiness: hls.pipeline_readiness(),
        transcode_budget,
        selected_hls_slots: hls.selected_hls_slots(transcode_budget),
        remux_concurrency: config.remux_concurrency.max(1),
        remux_timeout_ms: config.remux_timeout_ms.max(1),
        remote_stream_concurrency: config.playback.remote_stream_concurrency.max(1),
        remote_stage_concurrency: config.playback.remote_stage_concurrency.max(1),
        staging_max_bytes: config.staging.max_bytes,
        staging_retention_ms: config.staging.retention_ms,
        staging_cleanup_on_startup: config.staging.cleanup_on_startup,
        transcode_artifact_retention_ms: config.playback.transcode_artifact_retention_ms,
        transcode_artifact_cleanup_on_startup: config
            .playback
            .transcode_artifact_cleanup_on_startup,
        hls_segment_cleanup_enabled: config.playback.hls_segment_cleanup_enabled,
        hls_segment_keep_ms: config.playback.hls_segment_keep_ms,
        transcode_throttle_enabled: config.playback.transcode_throttle_enabled,
        transcode_throttle_delay_ms: config.playback.transcode_throttle_delay_ms,
        resource_pressure: resource_admission.resource_pressure(),
    }
}

async fn get_transcode_session_or_not_found(
    store: &dyn PlaybackRuntimeStore,
    session_id: TranscodeSessionId,
) -> Result<TranscodeSessionRecord> {
    PlaybackRuntimeStore::get_transcode_session(store, session_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "transcode_session",
            id: session_id.to_string(),
        })
}

async fn get_source_or_not_found(
    store: &dyn PlaybackRuntimeStore,
    source_id: MediaSourceId,
) -> Result<MediaSource> {
    PlaybackRuntimeStore::get_media_source(store, source_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "media_source",
            id: source_id.to_string(),
        })
}
