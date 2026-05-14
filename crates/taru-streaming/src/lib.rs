use serde::{Deserialize, Serialize};
use taru_core::{MediaProbeResult, MediaSource, MediaSourceId, MediaStreamKind, Result, TaruError};
use taru_transcode::TranscodePlan;

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RequestedByteRange {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolvedByteRange {
    pub start: u64,
    pub end: u64,
}

impl ResolvedByteRange {
    #[must_use]
    pub const fn len(self) -> u64 {
        self.end - self.start + 1
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

pub fn parse_http_range_header(value: &str) -> Result<RequestedByteRange> {
    let Some(spec) = value.trim().strip_prefix("bytes=") else {
        return Err(TaruError::InvalidInput {
            message: "range header must use bytes unit".to_owned(),
        });
    };

    if spec.contains(',') {
        return Err(TaruError::InvalidInput {
            message: "multiple byte ranges are not supported".to_owned(),
        });
    }

    let Some((start, end)) = spec.split_once('-') else {
        return Err(TaruError::InvalidInput {
            message: "range header must include '-'".to_owned(),
        });
    };

    let start = parse_optional_u64(start.trim())?;
    let end = parse_optional_u64(end.trim())?;

    if start.is_none() && end.is_none() {
        return Err(TaruError::InvalidInput {
            message: "range header must include a start or suffix length".to_owned(),
        });
    }

    Ok(RequestedByteRange { start, end })
}

pub fn resolve_byte_range(
    requested: Option<RequestedByteRange>,
    total_len: u64,
) -> Result<Option<ResolvedByteRange>> {
    let Some(requested) = requested else {
        return Ok(None);
    };

    if total_len == 0 {
        return Err(TaruError::InvalidInput {
            message: "cannot satisfy range request for an empty source".to_owned(),
        });
    }

    let (start, end) = match (requested.start, requested.end) {
        (Some(start), Some(end)) => (start, end),
        (Some(start), None) => (start, total_len - 1),
        (None, Some(suffix_len)) => {
            if suffix_len == 0 {
                return Err(TaruError::InvalidInput {
                    message: "suffix byte range length must be greater than zero".to_owned(),
                });
            }

            let start = total_len.saturating_sub(suffix_len);
            (start, total_len - 1)
        }
        (None, None) => unreachable!("empty range is rejected by parser"),
    };

    if start > end || start >= total_len {
        return Err(TaruError::InvalidInput {
            message: format!("byte range {start}-{end} cannot be satisfied for length {total_len}"),
        });
    }

    Ok(Some(ResolvedByteRange {
        start,
        end: end.min(total_len - 1),
    }))
}

#[must_use]
pub fn content_type_for_file_name(file_name: &str) -> &'static str {
    match extension(file_name).as_deref().unwrap_or_default() {
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "ts" | "m2ts" | "mts" => "video/mp2t",
        _ => "application/octet-stream",
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
    match extension(file_name)?.as_str() {
        "mp4" | "m4v" => Some("mp4"),
        "webm" => Some("webm"),
        "mkv" => Some("mkv"),
        "mov" => Some("mov"),
        "avi" => Some("avi"),
        "ts" | "m2ts" | "mts" => Some("mpegts"),
        _ => None,
    }
}

fn extension(file_name: &str) -> Option<String> {
    file_name
        .rsplit_once('.')
        .map(|(_stem, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .map(str::to_ascii_lowercase)
}

fn parse_optional_u64(value: &str) -> Result<Option<u64>> {
    if value.is_empty() {
        return Ok(None);
    }

    value
        .parse::<u64>()
        .map(Some)
        .map_err(|err| TaruError::InvalidInput {
            message: format!("invalid byte range integer: {err}"),
        })
}

#[cfg(test)]
mod tests {
    use taru_core::{MediaSourceId, MediaStreamInfo};

    use super::*;

    #[test]
    fn direct_play_is_allowed_for_compatible_mp4() {
        let source = media_source("movie.mp4");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("mov,mp4,m4a,3gp,3g2,mj2".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };

        let decision = decide_playback(
            &source,
            Some(&probe),
            &ClientPlaybackCapabilities::default(),
        );

        assert_eq!(decision.mode, PlaybackMode::DirectPlay);
        assert_eq!(
            decision.direct_play.unwrap().content_type,
            "video/mp4".to_owned()
        );
    }

    #[test]
    fn unsupported_container_with_supported_codecs_requests_remux() {
        let source = media_source("movie.mkv");
        let probe = MediaProbeResult {
            duration_ms: Some(1_000),
            container: Some("matroska,webm".to_owned()),
            bit_rate: None,
            streams: vec![
                stream(MediaStreamKind::Video, Some("h264")),
                stream(MediaStreamKind::Audio, Some("aac")),
            ],
        };

        let decision = decide_playback(
            &source,
            Some(&probe),
            &ClientPlaybackCapabilities::default(),
        );

        assert_eq!(decision.mode, PlaybackMode::Remux);
    }

    #[test]
    fn range_parser_resolves_open_and_suffix_ranges() {
        let open = parse_http_range_header("bytes=2-").unwrap();
        let suffix = parse_http_range_header("bytes=-4").unwrap();

        assert_eq!(
            resolve_byte_range(Some(open), 10).unwrap(),
            Some(ResolvedByteRange { start: 2, end: 9 })
        );
        assert_eq!(
            resolve_byte_range(Some(suffix), 10).unwrap(),
            Some(ResolvedByteRange { start: 6, end: 9 })
        );
    }

    fn media_source(file_name: &str) -> MediaSource {
        MediaSource {
            id: MediaSourceId::new(),
            item_id: taru_core::MediaItemId::new(),
            locator: format!("local:///{file_name}"),
            file_name: file_name.to_owned(),
            size_bytes: Some(1_000),
            fingerprint: None,
        }
    }

    fn stream(kind: MediaStreamKind, codec: Option<&str>) -> MediaStreamInfo {
        MediaStreamInfo {
            index: 0,
            kind,
            codec: codec.map(ToOwned::to_owned),
            language: None,
            duration_ms: None,
            bit_rate: None,
            width: None,
            height: None,
            channels: None,
            sample_rate: None,
        }
    }
}
