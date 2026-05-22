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
