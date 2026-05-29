use nako_core::{MediaProbeResult, MediaStreamInfo, MediaStreamKind, NakoError, Result};
use nako_playback::{
    PlaybackDecision, PlaybackRenditionPlan, PlaybackSelectionContext, PlaybackStorageContext,
    PlaybackTranscodeContainer,
};
use nako_transcode::{
    RemuxContainer, TranscodePipelineSourceFacts, TranscodePlan, TranscodeTrackSelection,
};
use nako_vfs::{StorageBackend, StorageCapabilities, StorageUri};

use crate::playback_mapping::{
    playback_remux_container_to_transcode, playback_track_selection_to_transcode,
    playback_transcode_plan_to_transcode,
};

use super::super::storage::LibraryStorageBackend;
use super::direct::should_budget_remote_stream;

pub(super) async fn playback_selection_context(
    uri: &StorageUri,
    backend: &LibraryStorageBackend,
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

pub(super) fn remux_output_container(decision: &PlaybackDecision) -> Result<RemuxContainer> {
    match &decision.rendition {
        PlaybackRenditionPlan::Remux(plan) => {
            Ok(playback_remux_container_to_transcode(plan.output_container))
        }
        _ => Err(NakoError::Unsupported(
            "remux app service requires a remux playback decision",
        )),
    }
}

pub(super) fn hls_transcode_plan(decision: &PlaybackDecision) -> Result<TranscodePlan> {
    match &decision.rendition {
        PlaybackRenditionPlan::Transcode(plan)
            if plan.plan.output_container == PlaybackTranscodeContainer::Hls =>
        {
            Ok(playback_transcode_plan_to_transcode(&plan.plan))
        }
        _ => Err(NakoError::Unsupported(
            "hls app service requires an hls transcode playback decision",
        )),
    }
}

pub(super) fn hls_transcode_track_selection(
    decision: &PlaybackDecision,
) -> Result<TranscodeTrackSelection> {
    match &decision.rendition {
        PlaybackRenditionPlan::Transcode(plan)
            if plan.plan.output_container == PlaybackTranscodeContainer::Hls =>
        {
            Ok(playback_track_selection_to_transcode(
                plan.requirement.track_selection,
            ))
        }
        _ => Err(NakoError::Unsupported(
            "hls app service requires an hls transcode playback decision",
        )),
    }
}

pub(super) fn hls_pipeline_source_facts(
    probe: Option<&MediaProbeResult>,
    track_selection: TranscodeTrackSelection,
) -> Option<TranscodePipelineSourceFacts> {
    Some(TranscodePipelineSourceFacts {
        video: selected_pipeline_stream(probe?, None, |stream| {
            matches!(stream.kind, MediaStreamKind::Video)
        }),
        audio: selected_pipeline_stream(probe?, track_selection.audio_stream, |stream| {
            matches!(stream.kind, MediaStreamKind::Audio)
        }),
        subtitle: selected_pipeline_stream(probe?, track_selection.subtitle_stream, |stream| {
            matches!(stream.kind, MediaStreamKind::Subtitle)
        }),
    })
}

fn selected_pipeline_stream(
    probe: &MediaProbeResult,
    requested_stream: Option<u32>,
    matches_kind: impl Fn(&MediaStreamInfo) -> bool,
) -> Option<MediaStreamInfo> {
    requested_stream
        .and_then(|index| {
            probe
                .streams
                .iter()
                .find(|stream| stream.index == index && matches_kind(stream))
        })
        .or_else(|| probe.streams.iter().find(|stream| matches_kind(stream)))
        .cloned()
}
