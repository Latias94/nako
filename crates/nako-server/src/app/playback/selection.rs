use nako_core::{
    MediaColorInfo, MediaHdrMetadata, MediaProbeResult, MediaSource, MediaStreamDisposition,
    MediaStreamInfo, MediaStreamTechnicalFacts, NakoError, Result,
};
use nako_playback::{
    PlaybackDecision, PlaybackRenditionPlan, PlaybackSelectionContext, PlaybackStorageContext,
    PlaybackTargetProfile, PlaybackTranscodeContainer, TranscodeRequirement,
    TranscodeRequirementStream,
};
use nako_transcode::{
    HardwareAccelerationPolicy, HlsPlaybackGeneration, HlsRuntimePlanRequest, RemuxContainer,
    TranscodePipelineSourceFacts, TranscodePlan, TranscodeTrackSelection,
};
use nako_vfs::{StorageBackend, StorageCapabilities, StorageUri};

use crate::playback_mapping::{
    playback_audio_output_requirement_to_transcode,
    playback_color_pipeline_requirement_to_transcode, playback_hls_output_requirement_to_transcode,
    playback_output_constraints_to_transcode, playback_remux_container_to_transcode,
    playback_subtitle_strategy_to_transcode, playback_track_selection_to_transcode,
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

pub(super) fn hls_runtime_plan_request(
    source: &MediaSource,
    decision: &PlaybackDecision,
    target_profile: &PlaybackTargetProfile,
    hardware_policy: HardwareAccelerationPolicy,
    playback_generation: HlsPlaybackGeneration,
    remote_input: bool,
    media_probe: Option<MediaProbeResult>,
) -> Result<HlsRuntimePlanRequest> {
    let plan = hls_transcode_plan(decision)?;
    let track_selection = hls_transcode_track_selection(decision)?;
    let transcode_requirement = hls_transcode_requirement(decision)?;
    let hls_output = transcode_requirement
        .hls_output
        .ok_or(NakoError::Unsupported(
            "hls app service requires an hls output requirement",
        ))?;

    Ok(HlsRuntimePlanRequest {
        source: source.clone(),
        plan,
        hardware_policy,
        track_selection,
        output_constraints: playback_output_constraints_to_transcode(
            transcode_requirement.output_constraints,
        ),
        audio_output: playback_audio_output_requirement_to_transcode(
            &transcode_requirement.audio_output,
        ),
        subtitle_strategy: playback_subtitle_strategy_to_transcode(
            transcode_requirement.subtitle_strategy,
        ),
        color_pipeline: playback_color_pipeline_requirement_to_transcode(
            &transcode_requirement.color_pipeline,
        ),
        hls_output: playback_hls_output_requirement_to_transcode(hls_output),
        source_facts: Some(hls_pipeline_source_facts_from_requirement(
            transcode_requirement,
        )),
        media_probe,
        playback_generation,
        remote_input,
        playback_profile_key: target_profile.identity_key(),
    })
}

fn hls_transcode_requirement(decision: &PlaybackDecision) -> Result<&TranscodeRequirement> {
    decision
        .transcode_requirement()
        .ok_or(NakoError::Unsupported(
            "hls app service requires an hls transcode playback decision",
        ))
}

fn hls_pipeline_source_facts_from_requirement(
    requirement: &TranscodeRequirement,
) -> TranscodePipelineSourceFacts {
    TranscodePipelineSourceFacts {
        video: requirement
            .selected_streams
            .video
            .as_ref()
            .map(media_stream_info_from_requirement),
        audio: requirement
            .selected_streams
            .audio
            .as_ref()
            .map(media_stream_info_from_requirement),
        subtitle: requirement
            .selected_streams
            .subtitle
            .as_ref()
            .map(media_stream_info_from_requirement),
    }
}

fn media_stream_info_from_requirement(stream: &TranscodeRequirementStream) -> MediaStreamInfo {
    MediaStreamInfo {
        index: stream.index,
        kind: stream.kind.clone(),
        codec: stream.codec.clone(),
        language: stream.language.clone(),
        duration_ms: stream.duration_ms,
        bit_rate: stream.bit_rate,
        width: stream.width,
        height: stream.height,
        channels: stream.channels,
        sample_rate: stream.sample_rate,
        technical: MediaStreamTechnicalFacts {
            codec_profile: stream.codec_profile.clone(),
            codec_level: stream.codec_level,
            pixel_format: stream.pixel_format.clone(),
            bits_per_raw_sample: stream.bits_per_raw_sample,
            bits_per_sample: stream.bits_per_sample,
            channel_layout: stream.channel_layout.clone(),
            color: MediaColorInfo {
                space: stream.color_space.clone(),
                transfer: stream.color_transfer.clone(),
                primaries: stream.color_primaries.clone(),
                ..MediaColorInfo::default()
            },
            hdr: MediaHdrMetadata {
                dynamic_range: stream.dynamic_range.clone(),
                mastering_display: stream.mastering_display,
                content_light_level: stream.content_light_level,
                dolby_vision: stream.dolby_vision,
                hdr10_plus: stream.hdr10_plus,
            },
            disposition: MediaStreamDisposition {
                default: stream.default,
                forced: stream.forced,
                ..MediaStreamDisposition::default()
            },
            ..MediaStreamTechnicalFacts::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use nako_core::{
        LibraryId, MediaItemId, MediaProbeResult, MediaSource, MediaSourceId, MediaStreamInfo,
        MediaStreamKind, MediaStreamTechnicalFacts,
    };
    use nako_playback::{
        ClientPlaybackCapabilities, PlaybackAudioOutputRequirement, PlaybackCapabilityEvaluation,
        PlaybackColorPipelineRequirement, PlaybackDecision, PlaybackDecisionReason,
        PlaybackDecisionReport, PlaybackHlsOutputRequirement, PlaybackHlsSegmentContainer,
        PlaybackHlsVariantPolicy, PlaybackMode, PlaybackOutputConstraints, PlaybackRenditionPlan,
        PlaybackSelectedSource, PlaybackSelectionContext, PlaybackSubtitleStrategy, PlaybackTarget,
        PlaybackTrackSelection, PlaybackTranscodeContainer, PlaybackTranscodePlan,
        TranscodeRenditionPlan, TranscodeRequirement, TranscodeRequirementStream,
        TranscodeRequirementStreams,
    };
    use nako_transcode::{
        HardwareAccelerationPolicy, HlsPlaybackGeneration, HlsSegmentContainer, HlsVariantPolicy,
        TranscodeSubtitleStrategy,
    };

    use super::hls_runtime_plan_request;

    #[test]
    fn hls_runtime_plan_request_uses_transcode_requirement_stream_facts() {
        let source_id = MediaSourceId::new();
        let library_id = LibraryId::new();
        let source = MediaSource {
            id: source_id,
            library_id,
            item_id: MediaItemId::new(),
            locator: "file:///media/movie.mkv".to_owned(),
            file_name: "movie.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: None,
        };
        let target = PlaybackTarget::browser_with_capabilities(
            "browser",
            ClientPlaybackCapabilities {
                supports_hdr: false,
                max_width: Some(1920),
                max_height: Some(1080),
                hls_variant_policy: PlaybackHlsVariantPolicy::SingleVariant,
                hls_segment_container: PlaybackHlsSegmentContainer::MpegTs,
                ..ClientPlaybackCapabilities::default()
            },
        );
        let target_profile = nako_playback::PlaybackTargetProfile::from_target(
            &target,
            PlaybackSelectionContext::default(),
        );
        let requirement = TranscodeRequirement {
            source_id,
            input_locator: source.locator.clone(),
            output_container: PlaybackTranscodeContainer::Hls,
            output_video_codec: Some("h264".to_owned()),
            output_audio_codec: Some("aac".to_owned()),
            track_selection: PlaybackTrackSelection {
                audio_stream: Some(2),
                subtitle_stream: Some(3),
            },
            output_constraints: PlaybackOutputConstraints {
                max_video_bitrate: Some(8_000_000),
                max_width: Some(1280),
                max_height: Some(720),
                prefer_hdr: Some(false),
            },
            color_pipeline: PlaybackColorPipelineRequirement::default(),
            audio_output: PlaybackAudioOutputRequirement::default(),
            hls_output: Some(PlaybackHlsOutputRequirement {
                variant_policy: PlaybackHlsVariantPolicy::Adaptive,
                segment_container: PlaybackHlsSegmentContainer::Fmp4,
            }),
            subtitle_strategy: PlaybackSubtitleStrategy::BurnInSelected,
            selected_streams: TranscodeRequirementStreams {
                video: Some(TranscodeRequirementStream {
                    index: 0,
                    kind: MediaStreamKind::Video,
                    codec: Some("hevc".to_owned()),
                    language: None,
                    duration_ms: Some(120_000),
                    bit_rate: Some(20_000_000),
                    width: Some(3840),
                    height: Some(2160),
                    channels: None,
                    sample_rate: None,
                    codec_profile: Some("main 10".to_owned()),
                    codec_level: Some(153),
                    pixel_format: Some("yuv420p10le".to_owned()),
                    bits_per_raw_sample: Some(10),
                    bits_per_sample: None,
                    dynamic_range: Some("hdr10".to_owned()),
                    color_space: Some("bt2020nc".to_owned()),
                    color_transfer: Some("smpte2084".to_owned()),
                    color_primaries: Some("bt2020".to_owned()),
                    mastering_display: true,
                    content_light_level: true,
                    dolby_vision: false,
                    hdr10_plus: false,
                    channel_layout: None,
                    forced: false,
                    default: true,
                }),
                audio: Some(TranscodeRequirementStream {
                    index: 2,
                    kind: MediaStreamKind::Audio,
                    codec: Some("eac3".to_owned()),
                    language: Some("eng".to_owned()),
                    duration_ms: Some(120_000),
                    bit_rate: Some(768_000),
                    width: None,
                    height: None,
                    channels: Some(6),
                    sample_rate: Some(48_000),
                    codec_profile: None,
                    codec_level: None,
                    pixel_format: None,
                    bits_per_raw_sample: None,
                    bits_per_sample: Some(24),
                    dynamic_range: None,
                    color_space: None,
                    color_transfer: None,
                    color_primaries: None,
                    mastering_display: false,
                    content_light_level: false,
                    dolby_vision: false,
                    hdr10_plus: false,
                    channel_layout: Some("5.1(side)".to_owned()),
                    forced: false,
                    default: true,
                }),
                subtitle: Some(TranscodeRequirementStream {
                    index: 3,
                    kind: MediaStreamKind::Subtitle,
                    codec: Some("subrip".to_owned()),
                    language: Some("jpn".to_owned()),
                    duration_ms: Some(120_000),
                    bit_rate: None,
                    width: None,
                    height: None,
                    channels: None,
                    sample_rate: None,
                    codec_profile: None,
                    codec_level: None,
                    pixel_format: None,
                    bits_per_raw_sample: None,
                    bits_per_sample: None,
                    dynamic_range: None,
                    color_space: None,
                    color_transfer: None,
                    color_primaries: None,
                    mastering_display: false,
                    content_light_level: false,
                    dolby_vision: false,
                    hdr10_plus: false,
                    channel_layout: None,
                    forced: false,
                    default: false,
                }),
            },
            reasons: Vec::new(),
        };
        let report = PlaybackDecisionReport {
            transcode: PlaybackCapabilityEvaluation::supported(),
            ..PlaybackDecisionReport::new(source_id, target_profile.identity_key())
                .with_selected_mode(PlaybackMode::Transcode)
        };
        let decision = PlaybackDecision {
            mode: PlaybackMode::Transcode,
            reason: PlaybackDecisionReason::RequestedTranscodeOutput,
            selected_source: PlaybackSelectedSource {
                source_id,
                library_id,
                locator: source.locator.clone(),
                file_name: source.file_name.clone(),
            },
            rendition: PlaybackRenditionPlan::Transcode(TranscodeRenditionPlan {
                plan: PlaybackTranscodePlan {
                    input_locator: source.locator.clone(),
                    output_container: PlaybackTranscodeContainer::Hls,
                    video_codec: Some("h264".to_owned()),
                    audio_codec: Some("aac".to_owned()),
                },
                requirement,
            }),
            report,
            denial: None,
        };
        let probe = MediaProbeResult {
            duration_ms: Some(120_000),
            container: Some("mkv".to_owned()),
            bit_rate: Some(21_000_000),
            streams: vec![MediaStreamInfo {
                index: 99,
                kind: MediaStreamKind::Video,
                codec: Some("mpeg2video".to_owned()),
                language: None,
                duration_ms: None,
                bit_rate: None,
                width: Some(720),
                height: Some(480),
                channels: None,
                sample_rate: None,
                technical: MediaStreamTechnicalFacts::default(),
            }],
        };

        let request = hls_runtime_plan_request(
            &source,
            &decision,
            &target_profile,
            HardwareAccelerationPolicy::default(),
            HlsPlaybackGeneration::from_start_position_ms(30_000),
            true,
            Some(probe),
        )
        .unwrap();

        assert_eq!(request.track_selection.audio_stream, Some(2));
        assert_eq!(request.track_selection.subtitle_stream, Some(3));
        assert_eq!(
            request.subtitle_strategy,
            TranscodeSubtitleStrategy::BurnInSelected
        );
        assert_eq!(request.output_constraints.max_width, Some(1280));
        assert_eq!(request.output_constraints.max_height, Some(720));
        assert_eq!(
            request.hls_output.variant_policy,
            HlsVariantPolicy::Adaptive
        );
        assert_eq!(
            request.hls_output.segment_container,
            HlsSegmentContainer::Fmp4
        );
        assert!(request.remote_input);
        assert_eq!(request.playback_generation.start_position_ms(), 30_000);
        let source_facts = request.source_facts.expect("source facts");
        let video = source_facts.video.expect("video stream facts");
        assert_eq!(video.index, 0);
        assert_eq!(video.codec.as_deref(), Some("hevc"));
        assert_eq!(video.technical.hdr.dynamic_range.as_deref(), Some("hdr10"));
        assert_eq!(video.technical.color.transfer.as_deref(), Some("smpte2084"));
        let audio = source_facts.audio.expect("audio stream facts");
        assert_eq!(audio.index, 2);
        assert_eq!(audio.channels, Some(6));
        assert_eq!(audio.technical.channel_layout.as_deref(), Some("5.1(side)"));
        let subtitle = source_facts.subtitle.expect("subtitle stream facts");
        assert_eq!(subtitle.index, 3);
        assert_eq!(subtitle.codec.as_deref(), Some("subrip"));
        assert_eq!(subtitle.language.as_deref(), Some("jpn"));
    }
}
