use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAcceleration {
    #[default]
    None,
    Vaapi,
    Nvenc,
    QuickSync,
}
