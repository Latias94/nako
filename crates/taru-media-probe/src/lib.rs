use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::Result;
use taru_vfs::StorageUri;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeRequest {
    pub source: StorageUri,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaProbeResult {
    pub duration_ms: Option<u64>,
    pub container: Option<String>,
    pub streams: Vec<MediaStreamInfo>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaStreamInfo {
    pub index: u32,
    pub kind: MediaStreamKind,
    pub codec: Option<String>,
    pub language: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaStreamKind {
    Video,
    Audio,
    Subtitle,
    Data,
}

#[async_trait]
pub trait MediaProbe: Send + Sync {
    async fn probe(&self, request: MediaProbeRequest) -> Result<MediaProbeResult>;
}
