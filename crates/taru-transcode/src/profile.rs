use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{HardwareAcceleration, OutputContainer, RemuxContainer};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeProfileKind {
    Remux,
    HlsSingleVariant,
}

impl TranscodeProfileKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Remux => "remux",
            Self::HlsSingleVariant => "hls_single_variant",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscodeReusePolicy {
    #[default]
    FinishedOutput,
}

impl TranscodeReusePolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FinishedOutput => "finished_output",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeTrackSelection {
    pub audio_stream: Option<u32>,
    pub subtitle_stream: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RemuxTranscodeProfile {
    pub output_container: RemuxContainer,
    pub track_selection: TranscodeTrackSelection,
    pub remote_input: bool,
    pub playback_profile_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HlsTranscodeProfile {
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub hardware_acceleration: HardwareAcceleration,
    pub track_selection: TranscodeTrackSelection,
    pub max_video_bitrate: Option<u64>,
    pub prefer_hdr: Option<bool>,
    pub remote_input: bool,
    pub playback_profile_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeProfile {
    pub kind: TranscodeProfileKind,
    pub output_container: String,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub hardware_acceleration: HardwareAcceleration,
    pub track_selection: TranscodeTrackSelection,
    pub max_video_bitrate: Option<u64>,
    pub prefer_hdr: Option<bool>,
    pub remote_input: bool,
    pub reuse_policy: TranscodeReusePolicy,
    pub playback_profile_key: String,
}

impl TranscodeProfile {
    #[must_use]
    pub fn remux(profile: RemuxTranscodeProfile) -> Self {
        Self {
            kind: TranscodeProfileKind::Remux,
            output_container: profile.output_container.file_extension().to_owned(),
            video_codec: None,
            audio_codec: None,
            hardware_acceleration: HardwareAcceleration::None,
            track_selection: profile.track_selection,
            max_video_bitrate: None,
            prefer_hdr: None,
            remote_input: profile.remote_input,
            reuse_policy: TranscodeReusePolicy::FinishedOutput,
            playback_profile_key: profile.playback_profile_key,
        }
    }

    #[must_use]
    pub fn hls_single_variant(profile: HlsTranscodeProfile) -> Self {
        Self {
            kind: TranscodeProfileKind::HlsSingleVariant,
            output_container: OutputContainer::Hls.as_str().to_owned(),
            video_codec: normalized_optional(profile.video_codec),
            audio_codec: normalized_optional(profile.audio_codec),
            hardware_acceleration: profile.hardware_acceleration,
            track_selection: profile.track_selection,
            max_video_bitrate: profile.max_video_bitrate,
            prefer_hdr: profile.prefer_hdr,
            remote_input: profile.remote_input,
            reuse_policy: TranscodeReusePolicy::FinishedOutput,
            playback_profile_key: profile.playback_profile_key,
        }
    }

    #[must_use]
    pub fn identity(&self) -> TranscodeProfileIdentity {
        let request_key = self.persisted_request_key();
        let digest = Sha256::digest(request_key.as_bytes());
        let digest = lowercase_hex(&digest);

        TranscodeProfileIdentity {
            request_key,
            storage_slug: format!("{}-v1-{}", self.kind.as_str(), &digest[..16]),
        }
    }

    fn persisted_request_key(&self) -> String {
        format!(
            "transcode-profile:v1;kind={};container={};vcodec={};acodec={};hw={};audio={};subtitle={};max_video_bitrate={};prefer_hdr={};remote_input={};reuse={};playback={}",
            self.kind.as_str(),
            canonical_value(&self.output_container),
            optional_str(self.video_codec.as_deref()),
            optional_str(self.audio_codec.as_deref()),
            self.hardware_acceleration.as_str(),
            optional_u32(self.track_selection.audio_stream),
            optional_u32(self.track_selection.subtitle_stream),
            optional_u64(self.max_video_bitrate),
            optional_bool(self.prefer_hdr),
            self.remote_input,
            self.reuse_policy.as_str(),
            escaped_component(&self.playback_profile_key),
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TranscodeProfileIdentity {
    request_key: String,
    storage_slug: String,
}

impl TranscodeProfileIdentity {
    #[must_use]
    pub fn persisted_request_key(&self) -> &str {
        &self.request_key
    }

    #[must_use]
    pub fn storage_slug(&self) -> &str {
        &self.storage_slug
    }
}

fn optional_str(value: Option<&str>) -> String {
    value.map_or_else(|| "auto".to_owned(), canonical_value)
}

fn optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "default".to_owned(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}

fn optional_bool(value: Option<bool>) -> String {
    value.map_or_else(|| "auto".to_owned(), |value| value.to_string())
}

fn normalized_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| canonical_value(&value))
        .filter(|value| !value.is_empty())
}

fn canonical_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn escaped_component(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'%' => escaped.push_str("%25"),
            b';' => escaped.push_str("%3B"),
            b'=' => escaped.push_str("%3D"),
            _ => escaped.push(*byte as char),
        }
    }
    escaped
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
