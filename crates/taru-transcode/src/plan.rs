use serde::{Deserialize, Serialize};

use super::hardware::HardwareAcceleration;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodePlan {
    pub input_locator: String,
    pub output_container: OutputContainer,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub hardware_acceleration: HardwareAcceleration,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputContainer {
    Hls,
    Mp4,
    Mkv,
}

impl OutputContainer {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hls => "hls",
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}
