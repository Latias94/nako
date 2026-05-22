use std::{path::PathBuf, process::Stdio};

use async_trait::async_trait;
use nako_core::{MediaProbeResult, MediaStreamInfo, MediaStreamKind, NakoError, Result};
use nako_vfs::StorageUri;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeRequest {
    pub source: StorageUri,
    pub local_path_hint: Option<PathBuf>,
}

#[async_trait]
pub trait MediaProbe: Send + Sync {
    async fn probe(&self, request: MediaProbeRequest) -> Result<MediaProbeResult>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfprobeMediaProbe {
    ffprobe_path: PathBuf,
}

impl Default for FfprobeMediaProbe {
    fn default() -> Self {
        Self::new("ffprobe")
    }
}

impl FfprobeMediaProbe {
    pub fn new(ffprobe_path: impl Into<PathBuf>) -> Self {
        Self {
            ffprobe_path: ffprobe_path.into(),
        }
    }

    #[must_use]
    pub fn ffprobe_path(&self) -> &PathBuf {
        &self.ffprobe_path
    }

    async fn probe_path(&self, path: PathBuf) -> Result<MediaProbeResult> {
        let output = Command::new(&self.ffprobe_path)
            .arg("-v")
            .arg("error")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|err| NakoError::Provider {
                provider: "ffprobe".to_owned(),
                message: format!("failed to execute ffprobe: {err}"),
            })?;

        if !output.status.success() {
            return Err(NakoError::Provider {
                provider: "ffprobe".to_owned(),
                message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }

        parse_ffprobe_json(&output.stdout)
    }
}

#[async_trait]
impl MediaProbe for FfprobeMediaProbe {
    async fn probe(&self, request: MediaProbeRequest) -> Result<MediaProbeResult> {
        let Some(path) = request.local_path_hint else {
            return Err(NakoError::Unsupported(
                "ffprobe requires a local path hint for this source",
            ));
        };

        self.probe_path(path).await
    }
}

fn parse_ffprobe_json(bytes: &[u8]) -> Result<MediaProbeResult> {
    let output =
        serde_json::from_slice::<FfprobeOutput>(bytes).map_err(|err| NakoError::Provider {
            provider: "ffprobe".to_owned(),
            message: format!("failed to parse ffprobe JSON: {err}"),
        })?;

    Ok(MediaProbeResult {
        duration_ms: output
            .format
            .as_ref()
            .and_then(|format| parse_seconds_to_ms(format.duration.as_deref())),
        container: output
            .format
            .as_ref()
            .and_then(|format| format.format_name.clone()),
        bit_rate: output
            .format
            .as_ref()
            .and_then(|format| parse_u64(format.bit_rate.as_deref())),
        streams: output.streams.into_iter().map(stream_to_info).collect(),
    })
}

fn stream_to_info(stream: FfprobeStream) -> MediaStreamInfo {
    MediaStreamInfo {
        index: stream.index.unwrap_or_default(),
        kind: stream_kind(stream.codec_type.as_deref()),
        codec: stream.codec_name,
        language: stream.tags.and_then(|tags| tags.language),
        duration_ms: parse_seconds_to_ms(stream.duration.as_deref()),
        bit_rate: parse_u64(stream.bit_rate.as_deref()),
        width: stream.width,
        height: stream.height,
        channels: stream.channels,
        sample_rate: parse_u32(stream.sample_rate.as_deref()),
    }
}

fn stream_kind(value: Option<&str>) -> MediaStreamKind {
    match value {
        Some("video") => MediaStreamKind::Video,
        Some("audio") => MediaStreamKind::Audio,
        Some("subtitle") => MediaStreamKind::Subtitle,
        Some("data") => MediaStreamKind::Data,
        Some("attachment") => MediaStreamKind::Attachment,
        Some(other) => MediaStreamKind::Other(other.to_owned()),
        None => MediaStreamKind::Other("unknown".to_owned()),
    }
}

fn parse_seconds_to_ms(value: Option<&str>) -> Option<u64> {
    let value = value?.trim();
    let (seconds, fraction) = value.split_once('.').unwrap_or((value, ""));
    let seconds = seconds.parse::<u64>().ok()?;
    let fraction_ms = fraction
        .chars()
        .take(3)
        .collect::<String>()
        .parse::<u64>()
        .unwrap_or_default();
    let scale = 10_u64.saturating_pow(3_u32.saturating_sub(fraction.len().min(3) as u32));

    seconds
        .checked_mul(1_000)?
        .checked_add(fraction_ms.saturating_mul(scale))
}

fn parse_u64(value: Option<&str>) -> Option<u64> {
    value?.trim().parse().ok()
}

fn parse_u32(value: Option<&str>) -> Option<u32> {
    value?.trim().parse().ok()
}

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    index: Option<u32>,
    codec_type: Option<String>,
    codec_name: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    channels: Option<u32>,
    sample_rate: Option<String>,
    tags: Option<FfprobeTags>,
}

#[derive(Debug, Deserialize)]
struct FfprobeTags {
    language: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ffprobe_json_into_probe_result() {
        let json = br#"
        {
          "streams": [
            {
              "index": 0,
              "codec_name": "h264",
              "codec_type": "video",
              "width": 1920,
              "height": 1080,
              "duration": "120.250000",
              "bit_rate": "4000000",
              "tags": { "language": "und" }
            },
            {
              "index": 1,
              "codec_name": "aac",
              "codec_type": "audio",
              "sample_rate": "48000",
              "channels": 2,
              "duration": "120.240000",
              "bit_rate": "128000",
              "tags": { "language": "eng" }
            },
            {
              "index": 2,
              "codec_name": "subrip",
              "codec_type": "subtitle",
              "tags": { "language": "jpn" }
            }
          ],
          "format": {
            "format_name": "matroska,webm",
            "duration": "120.253000",
            "bit_rate": "4200000"
          }
        }
        "#;

        let result = parse_ffprobe_json(json).unwrap();

        assert_eq!(result.duration_ms, Some(120_253));
        assert_eq!(result.container, Some("matroska,webm".to_owned()));
        assert_eq!(result.bit_rate, Some(4_200_000));
        assert_eq!(result.streams.len(), 3);
        assert_eq!(result.streams[0].kind, MediaStreamKind::Video);
        assert_eq!(result.streams[0].codec, Some("h264".to_owned()));
        assert_eq!(result.streams[0].width, Some(1920));
        assert_eq!(result.streams[0].height, Some(1080));
        assert_eq!(result.streams[0].duration_ms, Some(120_250));
        assert_eq!(result.streams[1].kind, MediaStreamKind::Audio);
        assert_eq!(result.streams[1].sample_rate, Some(48_000));
        assert_eq!(result.streams[1].channels, Some(2));
        assert_eq!(result.streams[2].kind, MediaStreamKind::Subtitle);
        assert_eq!(result.streams[2].language, Some("jpn".to_owned()));
    }

    #[test]
    fn ffprobe_requires_local_path_hint() {
        let result = pollster::block_on(async {
            FfprobeMediaProbe::default()
                .probe(MediaProbeRequest {
                    source: StorageUri::from_parts("local", "movie.mkv").unwrap(),
                    local_path_hint: None,
                })
                .await
        });

        assert!(matches!(result, Err(NakoError::Unsupported(_))));
    }
}
