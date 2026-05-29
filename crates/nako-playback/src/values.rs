use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackRemuxContainer {
    Mp4,
    Mkv,
}

impl PlaybackRemuxContainer {
    #[must_use]
    pub const fn file_extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackTranscodeContainer {
    Hls,
    Mp4,
    Mkv,
}

impl PlaybackTranscodeContainer {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hls => "hls",
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackTranscodePlan {
    pub input_locator: String,
    pub output_container: PlaybackTranscodeContainer,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackTrackSelection {
    pub audio_stream: Option<u32>,
    pub subtitle_stream: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackOutputConstraints {
    pub max_video_bitrate: Option<u64>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub prefer_hdr: Option<bool>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackHlsVariantPolicy {
    #[default]
    SingleVariant,
    Adaptive,
}

impl PlaybackHlsVariantPolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SingleVariant => "single_variant",
            Self::Adaptive => "adaptive",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackHlsSegmentContainer {
    #[default]
    MpegTs,
    Fmp4,
}

impl PlaybackHlsSegmentContainer {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MpegTs => "mpeg_ts",
            Self::Fmp4 => "fmp4",
        }
    }

    #[must_use]
    pub const fn segment_extension(self) -> &'static str {
        match self {
            Self::MpegTs => "ts",
            Self::Fmp4 => "m4s",
        }
    }

    #[must_use]
    pub const fn segment_content_type(self) -> &'static str {
        match self {
            Self::MpegTs => "video/mp2t",
            Self::Fmp4 => "video/mp4",
        }
    }

    #[must_use]
    pub const fn init_segment_file_name(self) -> Option<&'static str> {
        match self {
            Self::MpegTs => None,
            Self::Fmp4 => Some("init.mp4"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackHlsOutputRequirement {
    pub variant_policy: PlaybackHlsVariantPolicy,
    pub segment_container: PlaybackHlsSegmentContainer,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackSubtitleStrategy {
    #[default]
    None,
    PreserveInContainer,
    OmitSelected,
    BurnInSelected,
    SidecarSelected,
}

impl PlaybackSubtitleStrategy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PreserveInContainer => "preserve_in_container",
            Self::OmitSelected => "omit_selected",
            Self::BurnInSelected => "burn_in_selected",
            Self::SidecarSelected => "sidecar_selected",
        }
    }
}
