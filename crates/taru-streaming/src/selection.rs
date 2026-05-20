use serde::{Deserialize, Serialize};
use taru_core::{LibraryId, MediaProbeResult, MediaSource, MediaSourceId, MediaStreamKind};
use taru_transcode::{
    HardwareAcceleration, HlsTranscodeProfile, OutputContainer, RemuxContainer,
    RemuxTranscodeProfile, TranscodePlan, TranscodeProfile, TranscodeTrackSelection,
};

use super::direct::content_type_for_file_name;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecision {
    pub mode: PlaybackMode,
    pub reason: String,
    pub selected_source: PlaybackSelectedSource,
    pub execution: PlaybackExecutionPlan,
    pub direct_play: Option<DirectPlayPlan>,
    pub transcode_plan: Option<TranscodePlan>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    DirectPlay,
    Remux,
    Transcode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectPlayPlan {
    pub source_id: MediaSourceId,
    pub content_type: String,
    pub supports_range_requests: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackSelectionRequest<'a> {
    pub source: &'a MediaSource,
    pub probe: Option<&'a MediaProbeResult>,
    pub client: &'a ClientPlaybackCapabilities,
    pub context: PlaybackSelectionContext,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSelectionContext {
    pub storage: PlaybackStorageContext,
    pub preferences: PlaybackPreferenceContext,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackStorageContext {
    pub remote: bool,
    pub range_readable: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackPreferenceContext {
    pub requested_audio_stream: Option<u32>,
    pub requested_subtitle_stream: Option<u32>,
    pub max_video_bitrate: Option<u64>,
    pub prefer_hdr: Option<bool>,
    pub remux_output_container: Option<RemuxContainer>,
    pub transcode_output_container: Option<OutputContainer>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackProfile {
    pub direct_play: bool,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
    pub storage: PlaybackStorageContext,
    pub preferences: PlaybackPreferenceContext,
}

impl PlaybackProfile {
    #[must_use]
    pub fn from_context(
        client: &ClientPlaybackCapabilities,
        context: PlaybackSelectionContext,
    ) -> Self {
        Self {
            direct_play: client.direct_play,
            containers: normalized_values(&client.containers),
            video_codecs: normalized_values(&client.video_codecs),
            audio_codecs: normalized_values(&client.audio_codecs),
            storage: context.storage,
            preferences: context.preferences,
        }
    }

    #[must_use]
    pub fn identity(&self) -> PlaybackProfileIdentity {
        PlaybackProfileIdentity {
            request_key: format!(
                "playback-profile:v1;direct={};containers={};vcodecs={};acodecs={};remote={};range={};audio={};subtitle={};max_video_bitrate={};prefer_hdr={};remux={};transcode={}",
                self.direct_play,
                list_key(&self.containers),
                list_key(&self.video_codecs),
                list_key(&self.audio_codecs),
                self.storage.remote,
                optional_bool(self.storage.range_readable),
                optional_u32(self.preferences.requested_audio_stream),
                optional_u32(self.preferences.requested_subtitle_stream),
                optional_u64(self.preferences.max_video_bitrate),
                optional_bool(self.preferences.prefer_hdr),
                self.preferences
                    .remux_output_container
                    .map_or("auto", RemuxContainer::file_extension),
                self.preferences
                    .transcode_output_container
                    .map_or("auto", OutputContainer::as_str),
            ),
        }
    }

    #[must_use]
    pub fn identity_key(&self) -> String {
        self.identity().persisted_request_key().to_owned()
    }

    #[must_use]
    pub fn track_selection(&self) -> TranscodeTrackSelection {
        TranscodeTrackSelection {
            audio_stream: self.preferences.requested_audio_stream,
            subtitle_stream: self.preferences.requested_subtitle_stream,
        }
    }

    #[must_use]
    pub fn remux_transcode_profile(&self, output_container: RemuxContainer) -> TranscodeProfile {
        TranscodeProfile::remux(RemuxTranscodeProfile {
            output_container,
            track_selection: self.track_selection(),
            remote_input: self.storage.remote,
            playback_profile_key: self.identity().persisted_request_key().to_owned(),
        })
    }

    #[must_use]
    pub fn hls_transcode_profile(
        &self,
        plan: &TranscodePlan,
        hardware_acceleration: HardwareAcceleration,
    ) -> TranscodeProfile {
        TranscodeProfile::hls_single_variant(HlsTranscodeProfile {
            video_codec: plan.video_codec.clone(),
            audio_codec: plan.audio_codec.clone(),
            hardware_acceleration,
            track_selection: self.track_selection(),
            max_video_bitrate: self.preferences.max_video_bitrate,
            prefer_hdr: self.preferences.prefer_hdr,
            remote_input: self.storage.remote,
            playback_profile_key: self.identity().persisted_request_key().to_owned(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackProfileIdentity {
    request_key: String,
}

impl PlaybackProfileIdentity {
    #[must_use]
    pub fn persisted_request_key(&self) -> &str {
        &self.request_key
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackSelectedSource {
    pub source_id: MediaSourceId,
    pub library_id: LibraryId,
    pub locator: String,
    pub file_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PlaybackExecutionPlan {
    DirectPlay(DirectPlayPlan),
    Remux(RemuxPlaybackPlan),
    Transcode(TranscodePlan),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxPlaybackPlan {
    pub source_id: MediaSourceId,
    pub input_locator: String,
    pub output_container: RemuxContainer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClientPlaybackCapabilities {
    pub direct_play: bool,
    pub containers: Vec<String>,
    pub video_codecs: Vec<String>,
    pub audio_codecs: Vec<String>,
}

impl Default for ClientPlaybackCapabilities {
    fn default() -> Self {
        Self {
            direct_play: true,
            containers: vec!["mp4".to_owned(), "m4v".to_owned(), "webm".to_owned()],
            video_codecs: vec!["h264".to_owned(), "hevc".to_owned(), "vp9".to_owned()],
            audio_codecs: vec!["aac".to_owned(), "mp3".to_owned(), "opus".to_owned()],
        }
    }
}

pub fn decide_playback(
    source: &MediaSource,
    probe: Option<&MediaProbeResult>,
    client: &ClientPlaybackCapabilities,
) -> PlaybackDecision {
    select_playback_source(PlaybackSelectionRequest {
        source,
        probe,
        client,
        context: PlaybackSelectionContext::default(),
    })
}

pub fn select_playback_source(request: PlaybackSelectionRequest<'_>) -> PlaybackDecision {
    let PlaybackSelectionRequest {
        source,
        probe,
        client,
        context,
    } = request;
    let content_type = content_type_for_file_name(&source.file_name).to_owned();
    let container = container_for_file_name(&source.file_name);
    let selected_source = PlaybackSelectedSource::from(source);

    if let Some(output_container) = context.preferences.transcode_output_container {
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            output_container,
            "playback request requires transcode output".to_owned(),
        );
    }

    if !client.direct_play {
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            OutputContainer::Hls,
            "client disabled direct play".to_owned(),
        );
    }

    let Some(container) = container else {
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            OutputContainer::Hls,
            "source container could not be inferred from file name".to_owned(),
        );
    };

    let container_allowed = client.containers.is_empty()
        || client
            .containers
            .iter()
            .any(|value| value.eq_ignore_ascii_case(container));

    if !container_allowed {
        let codecs_allowed = probe.is_some_and(|probe| codecs_are_supported(probe, client));

        let reason = format!("client does not advertise support for {container} container");
        return if codecs_allowed {
            remux_decision(
                selected_source,
                source.locator.clone(),
                context
                    .preferences
                    .remux_output_container
                    .unwrap_or(RemuxContainer::Mp4),
                reason,
            )
        } else {
            transcode_decision(
                selected_source,
                source.locator.clone(),
                OutputContainer::Hls,
                reason,
            )
        };
    }

    if probe.is_some_and(|probe| !codecs_are_supported(probe, client)) {
        return transcode_decision(
            selected_source,
            source.locator.clone(),
            OutputContainer::Hls,
            "source codecs are not compatible with client capabilities".to_owned(),
        );
    }

    let direct_play = DirectPlayPlan {
        source_id: source.id,
        content_type,
        supports_range_requests: context.storage.range_readable.unwrap_or(true),
    };
    direct_play_decision(
        selected_source,
        direct_play,
        "source container and codecs are compatible with client capabilities".to_owned(),
    )
}

impl From<&MediaSource> for PlaybackSelectedSource {
    fn from(source: &MediaSource) -> Self {
        Self {
            source_id: source.id,
            library_id: source.library_id,
            locator: source.locator.clone(),
            file_name: source.file_name.clone(),
        }
    }
}

fn direct_play_decision(
    selected_source: PlaybackSelectedSource,
    direct_play: DirectPlayPlan,
    reason: String,
) -> PlaybackDecision {
    PlaybackDecision {
        mode: PlaybackMode::DirectPlay,
        reason,
        selected_source,
        execution: PlaybackExecutionPlan::DirectPlay(direct_play.clone()),
        direct_play: Some(direct_play),
        transcode_plan: None,
    }
}

fn remux_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: RemuxContainer,
    reason: String,
) -> PlaybackDecision {
    PlaybackDecision {
        mode: PlaybackMode::Remux,
        reason,
        execution: PlaybackExecutionPlan::Remux(RemuxPlaybackPlan {
            source_id: selected_source.source_id,
            input_locator,
            output_container,
        }),
        selected_source,
        direct_play: None,
        transcode_plan: None,
    }
}

fn transcode_decision(
    selected_source: PlaybackSelectedSource,
    input_locator: String,
    output_container: OutputContainer,
    reason: String,
) -> PlaybackDecision {
    let transcode_plan = TranscodePlan {
        input_locator,
        output_container,
        video_codec: Some("h264".to_owned()),
        audio_codec: Some("aac".to_owned()),
        hardware_acceleration: HardwareAcceleration::None,
    };

    PlaybackDecision {
        mode: PlaybackMode::Transcode,
        reason,
        execution: PlaybackExecutionPlan::Transcode(transcode_plan.clone()),
        selected_source,
        direct_play: None,
        transcode_plan: Some(transcode_plan),
    }
}

fn codecs_are_supported(probe: &MediaProbeResult, client: &ClientPlaybackCapabilities) -> bool {
    probe.streams.iter().all(|stream| match stream.kind {
        MediaStreamKind::Video => codec_allowed(stream.codec.as_deref(), &client.video_codecs),
        MediaStreamKind::Audio => codec_allowed(stream.codec.as_deref(), &client.audio_codecs),
        MediaStreamKind::Subtitle | MediaStreamKind::Data | MediaStreamKind::Attachment => true,
        MediaStreamKind::Other(_) => true,
    })
}

fn codec_allowed(codec: Option<&str>, allowed: &[String]) -> bool {
    allowed.is_empty()
        || codec.is_none_or(|codec| {
            allowed
                .iter()
                .any(|value| value.eq_ignore_ascii_case(codec))
        })
}

fn container_for_file_name(file_name: &str) -> Option<&str> {
    match super::direct::extension(file_name)?.as_str() {
        "mp4" | "m4v" => Some("mp4"),
        "webm" => Some("webm"),
        "mkv" => Some("mkv"),
        "mov" => Some("mov"),
        "avi" => Some("avi"),
        "ts" | "m2ts" | "mts" => Some("mpegts"),
        _ => None,
    }
}

fn normalized_values(values: &[String]) -> Vec<String> {
    let mut values = values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn list_key(values: &[String]) -> String {
    if values.is_empty() {
        "any".to_owned()
    } else {
        values.join("|")
    }
}

fn optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "default".to_owned(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}

fn optional_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}
