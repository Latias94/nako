use serde::{Deserialize, Serialize};
use taru_core::{MediaProbeResult, MediaSource, MediaSourceId, MediaStreamKind};
use taru_transcode::TranscodePlan;

use super::direct::content_type_for_file_name;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackDecision {
    pub mode: PlaybackMode,
    pub reason: String,
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
    let content_type = content_type_for_file_name(&source.file_name).to_owned();
    let container = container_for_file_name(&source.file_name);

    if !client.direct_play {
        return PlaybackDecision {
            mode: PlaybackMode::Transcode,
            reason: "client disabled direct play".to_owned(),
            direct_play: None,
            transcode_plan: None,
        };
    }

    let Some(container) = container else {
        return PlaybackDecision {
            mode: PlaybackMode::Transcode,
            reason: "source container could not be inferred from file name".to_owned(),
            direct_play: None,
            transcode_plan: None,
        };
    };

    let container_allowed = client.containers.is_empty()
        || client
            .containers
            .iter()
            .any(|value| value.eq_ignore_ascii_case(container));

    if !container_allowed {
        let codecs_allowed = probe.is_some_and(|probe| codecs_are_supported(probe, client));

        return PlaybackDecision {
            mode: if codecs_allowed {
                PlaybackMode::Remux
            } else {
                PlaybackMode::Transcode
            },
            reason: format!("client does not advertise support for {container} container"),
            direct_play: None,
            transcode_plan: None,
        };
    }

    if probe.is_some_and(|probe| !codecs_are_supported(probe, client)) {
        return PlaybackDecision {
            mode: PlaybackMode::Transcode,
            reason: "source codecs are not compatible with client capabilities".to_owned(),
            direct_play: None,
            transcode_plan: None,
        };
    }

    PlaybackDecision {
        mode: PlaybackMode::DirectPlay,
        reason: "source container and codecs are compatible with client capabilities".to_owned(),
        direct_play: Some(DirectPlayPlan {
            source_id: source.id,
            content_type,
            supports_range_requests: true,
        }),
        transcode_plan: None,
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
