use nako_playback::{
    PlaybackAudioCompatibilityReason, PlaybackAudioDownmixRequirement,
    PlaybackAudioNormalizationRequirement, PlaybackAudioOutputRequirement,
    PlaybackColorCompatibilityReason, PlaybackColorPipelineRequirement,
    PlaybackColorPipelineTarget, PlaybackHdrToneMappingRequirement, PlaybackHlsOutputRequirement,
    PlaybackHlsSegmentContainer, PlaybackHlsVariantPolicy, PlaybackOutputConstraints,
    PlaybackRemuxContainer, PlaybackTrackSelection, PlaybackTranscodeContainer,
    PlaybackTranscodePlan,
};
use nako_transcode::{
    HlsOutputRequirement, HlsSegmentContainer, HlsVariantPolicy, OutputContainer, RemuxContainer,
    TranscodeAudioCompatibilityReasons, TranscodeAudioDownmixRequirement,
    TranscodeAudioNormalizationRequirement, TranscodeAudioOutputRequirement,
    TranscodeColorCompatibilityReasons, TranscodeColorPipelineRequirement,
    TranscodeColorPipelineTarget, TranscodeHdrToneMappingRequirement, TranscodeOutputConstraints,
    TranscodePlan, TranscodeTrackSelection,
};

pub(crate) const fn playback_remux_container_to_transcode(
    value: PlaybackRemuxContainer,
) -> RemuxContainer {
    match value {
        PlaybackRemuxContainer::Mp4 => RemuxContainer::Mp4,
        PlaybackRemuxContainer::Mkv => RemuxContainer::Mkv,
    }
}

pub(crate) const fn transcode_remux_container_to_playback(
    value: RemuxContainer,
) -> PlaybackRemuxContainer {
    match value {
        RemuxContainer::Mp4 => PlaybackRemuxContainer::Mp4,
        RemuxContainer::Mkv => PlaybackRemuxContainer::Mkv,
    }
}

pub(crate) const fn playback_transcode_container_to_transcode(
    value: PlaybackTranscodeContainer,
) -> OutputContainer {
    match value {
        PlaybackTranscodeContainer::Hls => OutputContainer::Hls,
        PlaybackTranscodeContainer::Mp4 => OutputContainer::Mp4,
        PlaybackTranscodeContainer::Mkv => OutputContainer::Mkv,
    }
}

pub(crate) const fn playback_hls_variant_policy_to_transcode(
    value: PlaybackHlsVariantPolicy,
) -> HlsVariantPolicy {
    match value {
        PlaybackHlsVariantPolicy::SingleVariant => HlsVariantPolicy::SingleVariant,
        PlaybackHlsVariantPolicy::Adaptive => HlsVariantPolicy::Adaptive,
    }
}

pub(crate) const fn playback_hls_segment_container_to_transcode(
    value: PlaybackHlsSegmentContainer,
) -> HlsSegmentContainer {
    match value {
        PlaybackHlsSegmentContainer::MpegTs => HlsSegmentContainer::MpegTs,
        PlaybackHlsSegmentContainer::Fmp4 => HlsSegmentContainer::Fmp4,
    }
}

pub(crate) const fn playback_hls_output_requirement_to_transcode(
    value: PlaybackHlsOutputRequirement,
) -> HlsOutputRequirement {
    HlsOutputRequirement {
        variant_policy: playback_hls_variant_policy_to_transcode(value.variant_policy),
        segment_container: playback_hls_segment_container_to_transcode(value.segment_container),
    }
}

pub(crate) const fn playback_track_selection_to_transcode(
    value: PlaybackTrackSelection,
) -> TranscodeTrackSelection {
    TranscodeTrackSelection {
        audio_stream: value.audio_stream,
        subtitle_stream: value.subtitle_stream,
    }
}

pub(crate) const fn playback_output_constraints_to_transcode(
    value: PlaybackOutputConstraints,
) -> TranscodeOutputConstraints {
    TranscodeOutputConstraints {
        max_video_bitrate: value.max_video_bitrate,
        max_width: value.max_width,
        max_height: value.max_height,
        prefer_hdr: value.prefer_hdr,
    }
}

pub(crate) fn playback_transcode_plan_to_transcode(value: &PlaybackTranscodePlan) -> TranscodePlan {
    TranscodePlan {
        input_locator: value.input_locator.clone(),
        output_container: playback_transcode_container_to_transcode(value.output_container),
        video_codec: value.video_codec.clone(),
        audio_codec: value.audio_codec.clone(),
    }
}

pub(crate) fn playback_audio_output_requirement_to_transcode(
    requirement: &PlaybackAudioOutputRequirement,
) -> TranscodeAudioOutputRequirement {
    let downmix = playback_audio_downmix_requirement_to_transcode(requirement.downmix);
    let normalization =
        playback_audio_normalization_requirement_to_transcode(requirement.normalization);
    let reasons = playback_audio_compatibility_reasons_to_transcode(&requirement.reasons);
    if requirement.target_channels.is_none()
        && downmix == TranscodeAudioDownmixRequirement::None
        && normalization == TranscodeAudioNormalizationRequirement::None
        && !reasons.channel_limit_exceeded
        && !reasons.downmix_required
        && !reasons.normalization_requested
    {
        return TranscodeAudioOutputRequirement::none();
    }

    TranscodeAudioOutputRequirement {
        source_channels: requirement.source_channels,
        max_supported_channels: requirement.max_supported_channels,
        target_channels: requirement.target_channels,
        downmix,
        normalization,
        reasons,
    }
}

const fn playback_audio_downmix_requirement_to_transcode(
    requirement: PlaybackAudioDownmixRequirement,
) -> TranscodeAudioDownmixRequirement {
    match requirement {
        PlaybackAudioDownmixRequirement::None => TranscodeAudioDownmixRequirement::None,
        PlaybackAudioDownmixRequirement::Required => TranscodeAudioDownmixRequirement::Required,
    }
}

const fn playback_audio_normalization_requirement_to_transcode(
    requirement: PlaybackAudioNormalizationRequirement,
) -> TranscodeAudioNormalizationRequirement {
    match requirement {
        PlaybackAudioNormalizationRequirement::None => TranscodeAudioNormalizationRequirement::None,
        PlaybackAudioNormalizationRequirement::Requested => {
            TranscodeAudioNormalizationRequirement::Requested
        }
    }
}

fn playback_audio_compatibility_reasons_to_transcode(
    reasons: &[PlaybackAudioCompatibilityReason],
) -> TranscodeAudioCompatibilityReasons {
    TranscodeAudioCompatibilityReasons {
        channel_limit_exceeded: reasons
            .contains(&PlaybackAudioCompatibilityReason::ChannelLimitExceeded),
        downmix_required: reasons.contains(&PlaybackAudioCompatibilityReason::DownmixRequired),
        normalization_requested: reasons
            .contains(&PlaybackAudioCompatibilityReason::NormalizationRequested),
    }
}

pub(crate) fn playback_color_pipeline_requirement_to_transcode(
    requirement: &PlaybackColorPipelineRequirement,
) -> TranscodeColorPipelineRequirement {
    let target = playback_color_pipeline_target_to_transcode(requirement.target);
    let tone_mapping = playback_hdr_tone_mapping_requirement_to_transcode(requirement.tone_mapping);
    let reasons = playback_color_compatibility_reasons_to_transcode(&requirement.reasons);
    if target == TranscodeColorPipelineTarget::PreserveSource
        && tone_mapping == TranscodeHdrToneMappingRequirement::None
        && reasons == TranscodeColorCompatibilityReasons::none()
    {
        return TranscodeColorPipelineRequirement::none();
    }

    TranscodeColorPipelineRequirement {
        target,
        tone_mapping,
        reasons,
    }
}

const fn playback_color_pipeline_target_to_transcode(
    target: PlaybackColorPipelineTarget,
) -> TranscodeColorPipelineTarget {
    match target {
        PlaybackColorPipelineTarget::PreserveSource => TranscodeColorPipelineTarget::PreserveSource,
        PlaybackColorPipelineTarget::Sdr => TranscodeColorPipelineTarget::Sdr,
    }
}

const fn playback_hdr_tone_mapping_requirement_to_transcode(
    requirement: PlaybackHdrToneMappingRequirement,
) -> TranscodeHdrToneMappingRequirement {
    match requirement {
        PlaybackHdrToneMappingRequirement::None => TranscodeHdrToneMappingRequirement::None,
        PlaybackHdrToneMappingRequirement::Required => TranscodeHdrToneMappingRequirement::Required,
        PlaybackHdrToneMappingRequirement::DeferredUnsupported => {
            TranscodeHdrToneMappingRequirement::DeferredUnsupported
        }
    }
}

fn playback_color_compatibility_reasons_to_transcode(
    reasons: &[PlaybackColorCompatibilityReason],
) -> TranscodeColorCompatibilityReasons {
    TranscodeColorCompatibilityReasons {
        source_hdr_detected: reasons.contains(&PlaybackColorCompatibilityReason::SourceHdrDetected),
        client_hdr_unsupported: reasons
            .contains(&PlaybackColorCompatibilityReason::ClientHdrUnsupported),
        hdr_passthrough_supported: reasons
            .contains(&PlaybackColorCompatibilityReason::HdrPassthroughSupported),
        tone_mapping_required: reasons
            .contains(&PlaybackColorCompatibilityReason::ToneMappingRequired),
        unsupported_hdr_format_deferred: reasons
            .contains(&PlaybackColorCompatibilityReason::UnsupportedHdrFormatDeferred),
    }
}
