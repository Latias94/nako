use std::{path::PathBuf, process::Stdio};

use async_trait::async_trait;
use nako_core::{
    MediaColorInfo, MediaHdrMetadata, MediaProbeResult, MediaRational, MediaStreamDisposition,
    MediaStreamInfo, MediaStreamKind, MediaStreamTechnicalFacts, NakoError, Result,
};
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
    let hdr = hdr_metadata(&stream, &stream.side_data_list);
    let rotation_degrees = rotation_degrees(
        stream.tags.as_ref().and_then(|tags| tags.rotate.as_deref()),
        &stream.side_data_list,
    );
    let tags = stream.tags.unwrap_or_default();

    MediaStreamInfo {
        index: stream.index.unwrap_or_default(),
        kind: stream_kind(stream.codec_type.as_deref()),
        codec: stream.codec_name,
        language: tags.language,
        duration_ms: parse_seconds_to_ms(stream.duration.as_deref()),
        bit_rate: parse_u64(stream.bit_rate.as_deref()),
        width: stream.width,
        height: stream.height,
        channels: stream.channels,
        sample_rate: parse_u32(stream.sample_rate.as_deref()),
        technical: MediaStreamTechnicalFacts {
            codec_profile: stream.profile,
            codec_level: stream.level.and_then(|level| u32::try_from(level).ok()),
            codec_tag: stream.codec_tag_string.or(stream.codec_tag),
            pixel_format: stream.pix_fmt,
            bits_per_raw_sample: parse_u32(stream.bits_per_raw_sample.as_deref()),
            bits_per_sample: stream.bits_per_sample,
            average_frame_rate: parse_rational(stream.avg_frame_rate.as_deref()),
            nominal_frame_rate: parse_rational(stream.r_frame_rate.as_deref()),
            field_order: stream.field_order,
            rotation_degrees,
            channel_layout: stream.channel_layout,
            color: MediaColorInfo {
                range: stream.color_range,
                space: stream.color_space,
                transfer: stream.color_transfer,
                primaries: stream.color_primaries,
                chroma_location: stream.chroma_location,
            },
            hdr,
            disposition: stream.disposition.map(Into::into).unwrap_or_default(),
        },
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

fn parse_rational(value: Option<&str>) -> Option<MediaRational> {
    let value = value?.trim();
    let (numerator, denominator) = value.split_once('/')?;
    let numerator = numerator.trim().parse::<u32>().ok()?;
    let denominator = denominator.trim().parse::<u32>().ok()?;

    if numerator == 0 || denominator == 0 {
        return None;
    }

    Some(MediaRational {
        numerator,
        denominator,
    })
}

fn rotation_degrees(tag_rotate: Option<&str>, side_data: &[FfprobeSideData]) -> Option<i32> {
    tag_rotate
        .and_then(|value| value.trim().parse::<i32>().ok())
        .or_else(|| side_data.iter().find_map(|data| data.rotation))
}

fn hdr_metadata(stream: &FfprobeStream, side_data: &[FfprobeSideData]) -> MediaHdrMetadata {
    let mut hdr = MediaHdrMetadata {
        dynamic_range: dynamic_range(stream, side_data),
        ..MediaHdrMetadata::default()
    };

    for data in side_data {
        let Some(kind) = data.side_data_type.as_deref().map(str::to_ascii_lowercase) else {
            continue;
        };

        if kind.contains("mastering display") {
            hdr.mastering_display = true;
        }
        if kind.contains("content light") {
            hdr.content_light_level = true;
        }
        if kind.contains("dovi") || kind.contains("dolby vision") {
            hdr.dolby_vision = true;
        }
        if kind.contains("smpte2094-40") || kind.contains("hdr10+") {
            hdr.hdr10_plus = true;
        }
    }

    hdr
}

fn dynamic_range(stream: &FfprobeStream, side_data: &[FfprobeSideData]) -> Option<String> {
    if side_data.iter().any(|data| {
        data.side_data_type
            .as_deref()
            .is_some_and(|kind| kind.to_ascii_lowercase().contains("dovi"))
    }) {
        return Some("dolby_vision".to_owned());
    }

    if side_data.iter().any(|data| {
        data.side_data_type.as_deref().is_some_and(|kind| {
            let kind = kind.to_ascii_lowercase();
            kind.contains("smpte2094-40") || kind.contains("hdr10+")
        })
    }) {
        return Some("hdr10_plus".to_owned());
    }

    match stream.color_transfer.as_deref() {
        Some("smpte2084") => Some("hdr10".to_owned()),
        Some("arib-std-b67") => Some("hlg".to_owned()),
        _ => None,
    }
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
    profile: Option<String>,
    level: Option<i64>,
    codec_tag: Option<String>,
    codec_tag_string: Option<String>,
    duration: Option<String>,
    bit_rate: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    bits_per_raw_sample: Option<String>,
    bits_per_sample: Option<u32>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    field_order: Option<String>,
    color_range: Option<String>,
    color_space: Option<String>,
    color_transfer: Option<String>,
    color_primaries: Option<String>,
    chroma_location: Option<String>,
    channels: Option<u32>,
    sample_rate: Option<String>,
    channel_layout: Option<String>,
    tags: Option<FfprobeTags>,
    disposition: Option<FfprobeDisposition>,
    #[serde(default)]
    side_data_list: Vec<FfprobeSideData>,
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeTags {
    language: Option<String>,
    rotate: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeDisposition {
    #[serde(default)]
    default: u8,
    #[serde(default)]
    forced: u8,
    #[serde(default)]
    hearing_impaired: u8,
    #[serde(default)]
    visual_impaired: u8,
    #[serde(default)]
    commentary: u8,
    #[serde(default)]
    attached_pic: u8,
    #[serde(default)]
    captions: u8,
    #[serde(default)]
    descriptions: u8,
}

impl From<FfprobeDisposition> for MediaStreamDisposition {
    fn from(value: FfprobeDisposition) -> Self {
        Self {
            default: value.default != 0,
            forced: value.forced != 0,
            hearing_impaired: value.hearing_impaired != 0,
            visual_impaired: value.visual_impaired != 0,
            commentary: value.commentary != 0,
            attached_pic: value.attached_pic != 0,
            captions: value.captions != 0,
            descriptions: value.descriptions != 0,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FfprobeSideData {
    side_data_type: Option<String>,
    rotation: Option<i32>,
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
              "profile": "High",
              "level": 41,
              "pix_fmt": "yuv420p10le",
              "bits_per_raw_sample": "10",
              "avg_frame_rate": "24000/1001",
              "r_frame_rate": "24000/1001",
              "field_order": "progressive",
              "color_range": "tv",
              "color_space": "bt2020nc",
              "color_transfer": "smpte2084",
              "color_primaries": "bt2020",
              "chroma_location": "left",
              "width": 1920,
              "height": 1080,
              "duration": "120.250000",
              "bit_rate": "4000000",
              "disposition": { "default": 1, "forced": 0 },
              "tags": { "language": "und", "rotate": "90" },
              "side_data_list": [
                { "side_data_type": "Mastering display metadata" },
                { "side_data_type": "Content light level metadata" }
              ]
            },
            {
              "index": 1,
              "codec_name": "aac",
              "codec_type": "audio",
              "sample_rate": "48000",
              "channels": 2,
              "channel_layout": "stereo",
              "bits_per_sample": 16,
              "duration": "120.240000",
              "bit_rate": "128000",
              "tags": { "language": "eng" }
            },
            {
              "index": 2,
              "codec_name": "subrip",
              "codec_type": "subtitle",
              "disposition": { "forced": 1 },
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
        assert_eq!(
            result.streams[0].technical.codec_profile,
            Some("High".to_owned())
        );
        assert_eq!(result.streams[0].technical.codec_level, Some(41));
        assert_eq!(
            result.streams[0].technical.pixel_format,
            Some("yuv420p10le".to_owned())
        );
        assert_eq!(result.streams[0].technical.bits_per_raw_sample, Some(10));
        assert_eq!(
            result.streams[0].technical.average_frame_rate,
            Some(MediaRational {
                numerator: 24_000,
                denominator: 1_001,
            })
        );
        assert_eq!(
            result.streams[0].technical.color.transfer,
            Some("smpte2084".to_owned())
        );
        assert_eq!(
            result.streams[0].technical.hdr.dynamic_range,
            Some("hdr10".to_owned())
        );
        assert!(result.streams[0].technical.hdr.mastering_display);
        assert!(result.streams[0].technical.hdr.content_light_level);
        assert_eq!(result.streams[0].technical.rotation_degrees, Some(90));
        assert!(result.streams[0].technical.disposition.default);
        assert_eq!(result.streams[1].kind, MediaStreamKind::Audio);
        assert_eq!(result.streams[1].sample_rate, Some(48_000));
        assert_eq!(result.streams[1].channels, Some(2));
        assert_eq!(
            result.streams[1].technical.channel_layout,
            Some("stereo".to_owned())
        );
        assert_eq!(result.streams[1].technical.bits_per_sample, Some(16));
        assert_eq!(result.streams[2].kind, MediaStreamKind::Subtitle);
        assert_eq!(result.streams[2].language, Some("jpn".to_owned()));
        assert!(result.streams[2].technical.disposition.forced);
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
