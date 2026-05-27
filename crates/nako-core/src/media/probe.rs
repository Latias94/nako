use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeResult {
    pub duration_ms: Option<u64>,
    pub container: Option<String>,
    pub bit_rate: Option<u64>,
    pub streams: Vec<MediaStreamInfo>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamInfo {
    pub index: u32,
    pub kind: MediaStreamKind,
    pub codec: Option<String>,
    pub language: Option<String>,
    pub duration_ms: Option<u64>,
    pub bit_rate: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub technical: MediaStreamTechnicalFacts,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaStreamKind {
    Video,
    Audio,
    Subtitle,
    Data,
    Attachment,
    Other(String),
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamTechnicalFacts {
    pub codec_profile: Option<String>,
    pub codec_level: Option<u32>,
    pub codec_tag: Option<String>,
    pub pixel_format: Option<String>,
    pub bits_per_raw_sample: Option<u32>,
    pub bits_per_sample: Option<u32>,
    pub average_frame_rate: Option<MediaRational>,
    pub nominal_frame_rate: Option<MediaRational>,
    pub field_order: Option<String>,
    pub rotation_degrees: Option<i32>,
    pub channel_layout: Option<String>,
    #[serde(default)]
    pub color: MediaColorInfo,
    #[serde(default)]
    pub hdr: MediaHdrMetadata,
    #[serde(default)]
    pub disposition: MediaStreamDisposition,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaRational {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaColorInfo {
    pub range: Option<String>,
    pub space: Option<String>,
    pub transfer: Option<String>,
    pub primaries: Option<String>,
    pub chroma_location: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaHdrMetadata {
    pub dynamic_range: Option<String>,
    pub mastering_display: bool,
    pub content_light_level: bool,
    pub dolby_vision: bool,
    pub hdr10_plus: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamDisposition {
    pub default: bool,
    pub forced: bool,
    pub hearing_impaired: bool,
    pub visual_impaired: bool,
    pub commentary: bool,
    pub attached_pic: bool,
    pub captions: bool,
    pub descriptions: bool,
}
