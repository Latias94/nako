use nako_playback::{
    PlaybackHlsOutputRequirement, PlaybackHlsSegmentContainer, PlaybackHlsVariantPolicy,
    PlaybackOutputConstraints, PlaybackRemuxContainer, PlaybackTrackSelection,
    PlaybackTranscodeContainer, PlaybackTranscodePlan,
};
use nako_transcode::{
    HlsOutputRequirement, HlsSegmentContainer, HlsVariantPolicy, OutputContainer, RemuxContainer,
    TranscodeOutputConstraints, TranscodePlan, TranscodeTrackSelection,
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
